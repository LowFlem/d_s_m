use std::collections::HashMap;
use std::sync::{
    atomic::{AtomicBool, AtomicU64, Ordering},
    Arc,
};
use std::time::{Duration, SystemTime, UNIX_EPOCH};

use async_trait::async_trait;
use dashmap::DashMap;
use serde::{Deserialize, Serialize};
use tokio::sync::RwLock;

use crate::cluster::ClusterManager;
use crate::distribution::NetworkClientType;
use crate::error::{Result, StorageNodeError};
use crate::network::{NetworkClient, StateEntry};
use crate::storage::metrics::MetricsCollector;
use crate::storage::topology::NodeId;
use crate::storage::vector_clock::VectorClock;
use crate::storage::StorageEngine;
use crate::types::{BlindedStateEntry, StorageNode};

/// Helper to get current time in seconds
fn current_time_secs() -> u64 {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap_or_default()
        .as_secs()
}

/// Blake3 quantum-resistant hash function as specified in DSM whitepaper
fn blake3_hash(data: &[u8]) -> [u8; 32] {
    blake3::hash(data).into()
}

/// Health status for epidemic storage
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EpidemicStorageHealth {
    pub status: String,
    pub issues: Vec<String>,
    pub warnings: Vec<String>,
    pub total_operations: u64,
    pub failed_operations: u64,
    pub error_rate: f64,
    pub memory_bytes: usize,
    pub memory_utilization: f64,
    pub entry_count: usize,
    pub entry_utilization: f64,
    pub cluster_size: usize,
    pub gossip_rounds: u64,
}

/// DSM unilateral state entry with cryptographic verification
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct UnilateralEntry {
    entry_id: String,
    encrypted_payload: Vec<u8>,
    verification_hash: [u8; 32],
    timestamp: SystemTime,
    ttl_seconds: u64,
    region: String,
    priority: u8,
    size_bytes: usize,
    access_count: u64,
    last_accessed: SystemTime,
}

impl UnilateralEntry {
    fn new(
        entry_id: String,
        encrypted_payload: Vec<u8>,
        ttl_seconds: u64,
        region: String,
        priority: u8,
    ) -> Result<Self> {
        if entry_id.is_empty() {
            return Err(StorageNodeError::Validation(
                "Entry ID cannot be empty".into(),
            ));
        }
        if entry_id.len() > 512 {
            return Err(StorageNodeError::Validation(
                "Entry ID too long (max 512 chars)".into(),
            ));
        }
        if encrypted_payload.len() > 10_000_000 {
            return Err(StorageNodeError::Validation(
                "Payload too large (max 10MB)".into(),
            ));
        }
        if region.is_empty() {
            return Err(StorageNodeError::Validation(
                "Region cannot be empty".into(),
            ));
        }
        if region.len() > 64 {
            return Err(StorageNodeError::Validation(
                "Region too long (max 64 chars)".into(),
            ));
        }
        let size_bytes = entry_id.len() + encrypted_payload.len() + region.len() + 64;
        let verification_hash = blake3_hash(
            &[
                entry_id.as_bytes(),
                &encrypted_payload,
                &ttl_seconds.to_le_bytes(),
                region.as_bytes(),
                &[priority],
            ]
            .concat(),
        );
        let now = SystemTime::now();
        Ok(Self {
            entry_id,
            encrypted_payload,
            verification_hash,
            timestamp: now,
            ttl_seconds,
            region,
            priority,
            size_bytes,
            access_count: 0,
            last_accessed: now,
        })
    }

    /// Verify the cryptographic integrity of this entry
    pub fn verify(&self) -> bool {
        let expected_hash = blake3_hash(
            &[
                self.entry_id.as_bytes(),
                &self.encrypted_payload,
                &self.ttl_seconds.to_le_bytes(),
                self.region.as_bytes(),
                &[self.priority],
            ]
            .concat(),
        );
        self.verification_hash == expected_hash
    }

    fn is_expired(&self) -> bool {
        if self.ttl_seconds == 0 {
            return false;
        }
        self.timestamp
            .elapsed()
            .is_ok_and(|elapsed| elapsed.as_secs() > self.ttl_seconds)
    }

    /// Record an access to this entry for cache management
    pub fn record_access(&mut self) {
        self.access_count += 1;
        self.last_accessed = SystemTime::now();
    }

    fn idle_time_seconds(&self) -> u64 {
        self.last_accessed.elapsed().unwrap_or_default().as_secs()
    }

    fn priority_score(&self) -> u64 {
        let access_score = self.access_count.min(1000);
        let priority_score = (self.priority as u64) * 100;
        let recency_score = 3600_u64.saturating_sub(self.idle_time_seconds());
        access_score + priority_score + recency_score
    }
}

/// DSM deterministic storage assignment
#[derive(Clone, Debug)]
pub struct DeterministicAssignment {
    replication_factor: usize,
    assignment_threshold: u64,
}
impl DeterministicAssignment {
    fn new(replication_factor: usize, _min_geographic_regions: usize) -> Self {
        let assignment_threshold = u64::MAX / 3;
        Self {
            replication_factor,
            assignment_threshold,
        }
    }

    #[allow(dead_code)]
    fn is_responsible(&self, data_hash: &[u8; 32], node_id: &NodeId) -> bool {
        let combined = [data_hash.as_slice(), &node_id.0].concat();
        let assignment_hash = blake3_hash(&combined);
        let hash_value = u64::from_le_bytes([
            assignment_hash[0],
            assignment_hash[1],
            assignment_hash[2],
            assignment_hash[3],
            assignment_hash[4],
            assignment_hash[5],
            assignment_hash[6],
            assignment_hash[7],
        ]);
        hash_value < self.assignment_threshold
    }

    #[allow(dead_code)]
    fn get_responsible_nodes(
        &self,
        data_hash: &[u8; 32],
        all_nodes: &[StorageNode],
    ) -> Vec<String> {
        let responsible_nodes: Vec<String> = all_nodes
            .iter()
            .filter_map(|node| {
                if let Ok(node_id) = NodeId::from_string(&node.id) {
                    if self.is_responsible(data_hash, &node_id) {
                        Some(node.id.clone())
                    } else {
                        None
                    }
                } else {
                    None
                }
            })
            .take(self.replication_factor)
            .collect();
        if responsible_nodes.is_empty() && !all_nodes.is_empty() {
            all_nodes
                .iter()
                .take(self.replication_factor.min(all_nodes.len()))
                .map(|node| node.id.clone())
                .collect()
        } else {
            responsible_nodes
        }
    }
}

/// Configuration for the DSM Epidemic Storage Engine
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct EpidemicConfig {
    pub node_id: NodeId,
    pub gossip_interval_ms: u64,
    pub reconciliation_interval_ms: u64,
    pub topology_maintenance_interval_ms: u64,
    pub gossip_fanout: usize,
    pub max_reconciliation_diff: usize,
    pub replication_factor: usize,
    pub min_geographic_regions: usize,
    pub k_neighbors: usize,
    pub alpha: f64,
    pub max_long_links: usize,
    pub max_topology_connections: usize,
    pub topology_connection_timeout_ms: u64,
    pub default_ttl_seconds: u64,
    pub cleanup_interval_ms: u64,
    pub max_memory_bytes: usize,
    pub max_entries: usize,
    pub enable_eviction: bool,
    pub eviction_check_interval_ms: u64,
}

/// Parameters for gossip round execution
pub struct GossipRoundParams<'a> {
    pub cluster_nodes: &'a Arc<RwLock<Vec<StorageNode>>>,
    pub cluster_manager: &'a Arc<ClusterManager>,
    pub network_client: &'a Option<Arc<dyn NetworkClient>>,
    pub unilateral_store: &'a Arc<DashMap<String, UnilateralEntry>>,
    pub node_id_mapping: &'a Arc<DashMap<NodeId, String>>,
    pub reverse_node_mapping: &'a Arc<DashMap<String, NodeId>>,
    pub config: &'a EpidemicConfig,
    pub node_id: &'a str,
    pub crypto_node_id: &'a NodeId,
}
impl EpidemicConfig {
    pub fn validate(&self) -> Result<()> {
        if self.gossip_interval_ms == 0 {
            return Err(StorageNodeError::InvalidConfiguration(
                "Gossip interval cannot be zero".into(),
            ));
        }
        if self.gossip_fanout == 0 {
            return Err(StorageNodeError::InvalidConfiguration(
                "Gossip fanout cannot be zero".into(),
            ));
        }
        if self.gossip_fanout > 50 {
            return Err(StorageNodeError::InvalidConfiguration(
                "Gossip fanout too high (max 50)".into(),
            ));
        }
        if self.replication_factor == 0 {
            return Err(StorageNodeError::InvalidConfiguration(
                "Replication factor cannot be zero".into(),
            ));
        }
        if self.replication_factor > 10 {
            return Err(StorageNodeError::InvalidConfiguration(
                "Replication factor too high (max 10)".into(),
            ));
        }
        if self.alpha < 0.0 || self.alpha > 1.0 {
            return Err(StorageNodeError::InvalidConfiguration(
                "Alpha must be between 0.0 and 1.0".into(),
            ));
        }
        if self.max_memory_bytes == 0 {
            return Err(StorageNodeError::InvalidConfiguration(
                "Max memory bytes cannot be zero".into(),
            ));
        }
        if self.max_entries == 0 {
            return Err(StorageNodeError::InvalidConfiguration(
                "Max entries cannot be zero".into(),
            ));
        }
        Ok(())
    }

    pub fn default_for_testing() -> Self {
        Self {
            node_id: NodeId::generate(),
            gossip_interval_ms: 5000,
            reconciliation_interval_ms: 30000,
            topology_maintenance_interval_ms: 60000,
            gossip_fanout: 3,
            max_reconciliation_diff: 100,
            replication_factor: 3,
            min_geographic_regions: 2,
            k_neighbors: 4,
            alpha: 0.5,
            max_long_links: 15,
            max_topology_connections: 10,
            topology_connection_timeout_ms: 1000,
            default_ttl_seconds: 3600,
            cleanup_interval_ms: 60000,
            max_memory_bytes: 1024 * 1024 * 1024,
            max_entries: 1_000_000,
            enable_eviction: true,
            eviction_check_interval_ms: 30000,
        }
    }
}

/// Epidemic Storage Engine for DSM Storage Nodes
pub struct EpidemicStorageEngine {
    /// Local storage for unilateral entries
    pub unilateral_store: Arc<DashMap<String, UnilateralEntry>>,

    /// Configuration for epidemic storage
    pub config: EpidemicConfig,

    /// Network client for gossip protocol
    pub network_client: Option<Arc<dyn NetworkClient>>,

    /// Metrics collector
    pub metrics: Arc<MetricsCollector>,

    /// Node identifier (simple string ID for network operations)
    pub node_id: String,

    /// Cryptographic NodeId for epidemic protocol
    pub crypto_node_id: NodeId,

    /// Epidemic running flag
    pub epidemic_running: Arc<AtomicBool>,

    /// Epidemic round counter
    pub epidemic_rounds: Arc<AtomicU64>,

    /// Known cluster nodes (legacy - to be phased out)
    pub cluster_nodes: Arc<RwLock<Vec<StorageNode>>>,

    /// Cluster manager for overlapping cluster topology
    pub cluster_manager: Arc<ClusterManager>,

    /// Mapping from cryptographic NodeIds to network endpoints
    pub node_id_mapping: Arc<DashMap<NodeId, String>>,

    /// Reverse mapping from simple node IDs to cryptographic NodeIds
    pub reverse_node_mapping: Arc<DashMap<String, NodeId>>,

    /// Deterministic assignment for data placement
    pub assignment: DeterministicAssignment,
}

impl EpidemicStorageEngine {
    /// Create a new epidemic storage engine
    pub fn new(
        config: EpidemicConfig,
        network_client: NetworkClientType,
        metrics: Arc<MetricsCollector>,
        bootstrap_nodes: Vec<StorageNode>,
        node: StorageNode,
    ) -> Self {
        let assignment =
            DeterministicAssignment::new(config.replication_factor, config.min_geographic_regions);

        // Convert NetworkClientType to Option<Arc<dyn NetworkClient>>
        let network_client = match network_client {
            NetworkClientType::Http(client) => Some(client as Arc<dyn NetworkClient>),
            NetworkClientType::Mock(client) => Some(client as Arc<dyn NetworkClient>),
        };

        // Initialize cluster manager with legacy constructor for epidemic storage
        // Note: Epidemic storage will eventually be updated to use auto-discovery
        let cluster_manager = Arc::new(ClusterManager::new_legacy(node.id.clone()));

        // Prepare node endpoints for cluster initialization
        let mut node_endpoints = vec![(node.id.clone(), node.endpoint.clone())];
        for bootstrap_node in &bootstrap_nodes {
            node_endpoints.push((bootstrap_node.id.clone(), bootstrap_node.endpoint.clone()));
        }

        let storage_engine = Self {
            unilateral_store: Arc::new(DashMap::new()),
            config: config.clone(),
            network_client,
            metrics,
            node_id: node.id.clone(),
            crypto_node_id: config.node_id.clone(),
            epidemic_running: Arc::new(AtomicBool::new(false)),
            epidemic_rounds: Arc::new(AtomicU64::new(0)),
            cluster_nodes: Arc::new(RwLock::new(vec![])), // Legacy - will be phased out
            cluster_manager: cluster_manager.clone(),
            node_id_mapping: Arc::new(DashMap::new()),
            reverse_node_mapping: Arc::new(DashMap::new()),
            assignment,
        };

        // Note: Cluster topology initialization is now handled by auto-discovery
        // The epidemic storage will work with discovered peers automatically
        tracing::info!("EpidemicStorage initialized with auto-discovery support");

        // Initialize NodeId mappings for current node
        let current_crypto_id =
            NodeId::from_device_entropy(node.id.as_bytes(), "dsm_epidemic_storage");
        storage_engine
            .reverse_node_mapping
            .insert(node.id.clone(), current_crypto_id.clone());
        storage_engine
            .node_id_mapping
            .insert(current_crypto_id.clone(), node.id.clone());
        tracing::info!(
            "Initialized current node mapping: '{}' -> crypto NodeId '{}'",
            node.id,
            current_crypto_id
        );

        storage_engine
    }

    /// Update the network client for gossip protocol
    /// This can be used to change the network client after initialization if needed
    pub fn set_network_client(&mut self, client: Arc<dyn NetworkClient>) {
        self.network_client = Some(client);
    }

    /// Get current cluster nodes
    pub fn get_cluster_nodes(&self) -> Vec<StorageNode> {
        // Use blocking read since this is a non-async method
        tokio::task::block_in_place(|| {
            tokio::runtime::Handle::current()
                .block_on(async { self.cluster_nodes.read().await.clone() })
        })
    }

    /// Add a cluster node to the epidemic network
    pub fn add_cluster_node(&self, node: StorageNode) {
        let node_id = node.id.clone();
        let endpoint = node.endpoint.clone();
        tracing::info!("Adding cluster node: {} -> {}", node_id, endpoint);

        // Generate a cryptographic NodeId for this node if not already present
        let crypto_node_id = if let Some(existing_id) = self.reverse_node_mapping.get(&node_id) {
            existing_id.clone()
        } else {
            // Create a deterministic cryptographic NodeId from the simple node ID
            let crypto_id = NodeId::from_device_entropy(node_id.as_bytes(), "dsm_epidemic_storage");
            self.reverse_node_mapping
                .insert(node_id.clone(), crypto_id.clone());
            self.node_id_mapping
                .insert(crypto_id.clone(), node_id.clone());
            crypto_id
        };

        tracing::info!(
            "Mapped simple node ID '{}' to crypto NodeId '{}'",
            node_id,
            crypto_node_id
        );

        // Register the node with the network client first
        if let Some(ref client) = self.network_client {
            let client_clone = client.clone();
            let node_id_clone = node_id.clone();
            let endpoint_clone = endpoint.clone();
            tokio::spawn(async move {
                client_clone
                    .register_node(node_id_clone, endpoint_clone)
                    .await;
            });
        }

        // Add node to cluster nodes asynchronously
        let cluster_nodes = Arc::clone(&self.cluster_nodes);
        tokio::spawn(async move {
            let mut nodes = cluster_nodes.write().await;
            // Only add if not already present
            if !nodes.iter().any(|n| n.id == node_id) {
                nodes.push(node);
                tracing::info!("Successfully added cluster node: {}", node_id);
            } else {
                tracing::debug!("Cluster node {} already exists", node_id);
            }
        });
    }

    /// Start the epidemic protocol
    pub async fn start_epidemic_protocol(&self) -> Result<()> {
        if self.epidemic_running.swap(true, Ordering::SeqCst) {
            return Ok(()); // Already running
        }

        tracing::info!("Starting epidemic protocol for node: {}", self.node_id);

        // Start gossip protocol
        self.start_gossip_protocol().await?;

        // Start reconciliation protocol
        self.start_reconciliation_protocol().await?;

        // Start cleanup protocol
        self.start_cleanup_protocol().await?;

        Ok(())
    }

    /// Stop the epidemic protocol
    pub async fn stop_epidemic_protocol(&self) -> Result<()> {
        self.epidemic_running.store(false, Ordering::SeqCst);
        tracing::info!("Stopped epidemic protocol for node: {}", self.node_id);
        Ok(())
    }

    /// Start gossip protocol background task
    async fn start_gossip_protocol(&self) -> Result<()> {
        let gossip_interval = Duration::from_millis(self.config.gossip_interval_ms);
        let epidemic_running = Arc::clone(&self.epidemic_running);
        let epidemic_rounds = Arc::clone(&self.epidemic_rounds);
        let cluster_nodes = Arc::clone(&self.cluster_nodes);
        let cluster_manager = Arc::clone(&self.cluster_manager);
        let network_client = self.network_client.clone();
        let unilateral_store = Arc::clone(&self.unilateral_store);
        let node_id_mapping = Arc::clone(&self.node_id_mapping);
        let reverse_node_mapping = Arc::clone(&self.reverse_node_mapping);
        let config = self.config.clone();
        let node_id = self.node_id.clone();
        let crypto_node_id = self.crypto_node_id.clone();

        tokio::spawn(async move {
            let mut interval = tokio::time::interval(gossip_interval);

            while epidemic_running.load(Ordering::SeqCst) {
                interval.tick().await;

                // Execute gossip round
                if let Err(e) = Self::execute_gossip_round(GossipRoundParams {
                    cluster_nodes: &cluster_nodes,
                    cluster_manager: &cluster_manager,
                    network_client: &network_client,
                    unilateral_store: &unilateral_store,
                    node_id_mapping: &node_id_mapping,
                    reverse_node_mapping: &reverse_node_mapping,
                    config: &config,
                    node_id: &node_id,
                    crypto_node_id: &crypto_node_id,
                })
                .await
                {
                    tracing::warn!("Gossip round failed: {}", e);
                }

                epidemic_rounds.fetch_add(1, Ordering::SeqCst);
            }
        });

        Ok(())
    }

    /// Execute a single gossip round
    async fn execute_gossip_round(params: GossipRoundParams<'_>) -> Result<()> {
        // Use cluster manager to get gossip targets (excludes self automatically)
        let cluster_manager = &params.cluster_manager;
        let gossip_targets = cluster_manager.get_gossip_targets(None).await;

        if gossip_targets.is_empty() {
            return Ok(()); // No other nodes to gossip with
        }

        // Select random subset of nodes for gossip (fanout)
        let fanout = std::cmp::min(params.config.gossip_fanout, gossip_targets.len());
        let selected_nodes: Vec<_> = {
            use rand::seq::SliceRandom;
            let mut rng = rand::thread_rng();
            gossip_targets
                .choose_multiple(&mut rng, fanout)
                .cloned()
                .collect()
        };

        // Prepare gossip entries (latest entries up to limit)
        let gossip_entries: Vec<StateEntry> = params
            .unilateral_store
            .iter()
            .take(100) // Limit entries per gossip
            .map(|entry_ref| {
                let entry = entry_ref.value();
                StateEntry {
                    key: entry.entry_id.clone(),
                    value: entry.encrypted_payload.clone(),
                    timestamp: entry
                        .timestamp
                        .duration_since(UNIX_EPOCH)
                        .unwrap_or_default()
                        .as_secs(),
                    vector_clock: VectorClock::new(),
                    origin_node: params.node_id.to_string(),
                }
            })
            .collect();

        // Send entries to selected nodes using the proper NodeId mapping
        if let Some(client) = params.network_client {
            for node in selected_nodes {
                // Ensure the target node has a cryptographic NodeId mapping
                let target_crypto_node_id =
                    if let Some(existing_id) = params.reverse_node_mapping.get(&node.id) {
                        existing_id.clone()
                    } else {
                        // Create a deterministic cryptographic NodeId for this target node
                        let crypto_id =
                            NodeId::from_device_entropy(node.id.as_bytes(), "dsm_epidemic_storage");
                        params
                            .reverse_node_mapping
                            .insert(node.id.clone(), crypto_id.clone());
                        params
                            .node_id_mapping
                            .insert(crypto_id.clone(), node.id.clone());
                        tracing::info!(
                            "Created mapping for target node '{}' -> crypto NodeId '{}'",
                            node.id,
                            crypto_id
                        );
                        crypto_id
                    };

                tracing::debug!(
                    "Sending gossip entries from crypto NodeId '{}' to target '{}' (crypto NodeId '{}')",
                    params.crypto_node_id, node.id, target_crypto_node_id
                );

                // Send gossip entries using the simple node ID (which network client understands)
                if let Err(e) = client
                    .send_entries(node.id.clone(), gossip_entries.clone())
                    .await
                {
                    tracing::warn!("Failed to send gossip entries to node {}: {}", node.id, e);
                } else {
                    tracing::debug!(
                        "Successfully sent {} gossip entries to node {}",
                        gossip_entries.len(),
                        node.id
                    );
                }
            }
        }

        Ok(())
    }

    /// Start reconciliation protocol background task
    async fn start_reconciliation_protocol(&self) -> Result<()> {
        let reconciliation_interval = Duration::from_millis(self.config.reconciliation_interval_ms);
        let epidemic_running = Arc::clone(&self.epidemic_running);
        let cluster_nodes = Arc::clone(&self.cluster_nodes);
        let unilateral_store = Arc::clone(&self.unilateral_store);

        tokio::spawn(async move {
            let mut interval = tokio::time::interval(reconciliation_interval);

            while epidemic_running.load(Ordering::SeqCst) {
                interval.tick().await;

                // Execute anti-entropy reconciliation
                if let Err(e) =
                    Self::execute_reconciliation_round(&cluster_nodes, &unilateral_store).await
                {
                    tracing::warn!("Reconciliation round failed: {}", e);
                }
            }
        });

        Ok(())
    }

    /// Execute anti-entropy reconciliation
    async fn execute_reconciliation_round(
        cluster_nodes: &Arc<RwLock<Vec<StorageNode>>>,
        unilateral_store: &Arc<DashMap<String, UnilateralEntry>>,
    ) -> Result<()> {
        let nodes = cluster_nodes.read().await;
        if nodes.len() <= 1 {
            return Ok(());
        }

        // For each node, compare state digests and reconcile differences
        for node in nodes.iter() {
            // In a full implementation, this would:
            // 1. Request state digest from the node
            // 2. Compare with local state digest
            // 3. Request missing entries
            // 4. Send missing entries to the node

            tracing::debug!("Reconciling with node: {}", node.id);
        }

        // Clean up expired entries
        let expired_keys: Vec<String> = unilateral_store
            .iter()
            .filter_map(|entry_ref| {
                let entry = entry_ref.value();
                if entry.is_expired()
                    || (entry.ttl_seconds > 0
                        && entry.timestamp.elapsed().unwrap_or_default().as_secs()
                            > entry.ttl_seconds)
                {
                    Some(entry.entry_id.clone())
                } else {
                    None
                }
            })
            .collect();

        for key in expired_keys {
            unilateral_store.remove(&key);
        }

        Ok(())
    }

    /// Start cleanup protocol background task
    async fn start_cleanup_protocol(&self) -> Result<()> {
        let cleanup_interval = Duration::from_millis(self.config.cleanup_interval_ms);
        let epidemic_running = Arc::clone(&self.epidemic_running);
        let unilateral_store = Arc::clone(&self.unilateral_store);
        let config = self.config.clone();

        tokio::spawn(async move {
            let mut interval = tokio::time::interval(cleanup_interval);

            while epidemic_running.load(Ordering::SeqCst) {
                interval.tick().await;

                // Execute cleanup
                Self::execute_cleanup_round(&unilateral_store, &config).await;
            }
        });

        Ok(())
    }

    /// Execute cleanup round
    async fn execute_cleanup_round(
        unilateral_store: &Arc<DashMap<String, UnilateralEntry>>,
        config: &EpidemicConfig,
    ) {
        let current_size = unilateral_store.len();
        let current_memory = unilateral_store
            .iter()
            .map(|entry_ref| entry_ref.value().size_bytes)
            .sum::<usize>();

        // Check if eviction is needed
        if !config.enable_eviction {
            return;
        }

        let needs_eviction =
            current_size > config.max_entries || current_memory > config.max_memory_bytes;

        if needs_eviction {
            tracing::info!(
                "Starting eviction: {} entries, {} bytes",
                current_size,
                current_memory
            );

            // Collect entries with priority scores
            let mut entries_with_scores: Vec<(String, u64)> = unilateral_store
                .iter()
                .map(|entry_ref| {
                    let entry = entry_ref.value();
                    (entry.entry_id.clone(), entry.priority_score())
                })
                .collect();

            // Sort by priority (lowest first for eviction)
            entries_with_scores.sort_by_key(|(_, score)| *score);

            // Evict bottom 10% of entries
            let eviction_count = std::cmp::max(1, current_size / 10);
            for (key, _) in entries_with_scores.iter().take(eviction_count) {
                unilateral_store.remove(key);
            }

            tracing::info!("Evicted {} entries", eviction_count);
        }
    }

    /// Get epidemic storage health status
    pub async fn get_health(&self) -> EpidemicStorageHealth {
        let cluster_size = self.cluster_nodes.read().await.len();
        let entry_count = self.unilateral_store.len();
        let memory_bytes = self
            .unilateral_store
            .iter()
            .map(|entry_ref| entry_ref.value().size_bytes)
            .sum();

        let gossip_rounds = self.epidemic_rounds.load(Ordering::SeqCst);

        // Calculate utilization metrics
        let memory_utilization = if self.config.max_memory_bytes > 0 {
            (memory_bytes as f64) / (self.config.max_memory_bytes as f64)
        } else {
            0.0
        };

        let entry_utilization = if self.config.max_entries > 0 {
            (entry_count as f64) / (self.config.max_entries as f64)
        } else {
            0.0
        };

        // Determine health status
        let status = if memory_utilization > 0.9 || entry_utilization > 0.9 {
            "critical".to_string()
        } else if memory_utilization > 0.7 || entry_utilization > 0.7 {
            "warning".to_string()
        } else {
            "healthy".to_string()
        };

        let mut issues = Vec::new();
        let mut warnings = Vec::new();

        if memory_utilization > 0.9 {
            issues.push("Memory utilization critical".to_string());
        } else if memory_utilization > 0.7 {
            warnings.push("Memory utilization high".to_string());
        }

        if entry_utilization > 0.9 {
            issues.push("Entry count critical".to_string());
        } else if entry_utilization > 0.7 {
            warnings.push("Entry count high".to_string());
        }

        if cluster_size <= 1 {
            warnings.push("Single node cluster".to_string());
        }

        EpidemicStorageHealth {
            status,
            issues,
            warnings,
            total_operations: gossip_rounds, // Using gossip rounds as operation count
            failed_operations: 0,            // Would need to track actual failures
            error_rate: 0.0,                 // Would need to calculate from failure metrics
            memory_bytes,
            memory_utilization,
            entry_count,
            entry_utilization,
            cluster_size,
            gossip_rounds,
        }
    }

    /// Merge gossip entries from another node
    pub async fn merge_gossip_entries(&self, entries: Vec<StateEntry>) -> Result<()> {
        for state_entry in entries {
            // Convert StateEntry to BlindedStateEntry
            let blinded_entry = BlindedStateEntry {
                blinded_id: state_entry.key,
                encrypted_payload: state_entry.value,
                timestamp: state_entry.timestamp,
                ttl: self.config.default_ttl_seconds,
                region: "default".to_string(), // Default region
                priority: 0,                   // Default priority
                proof_hash: [0u8; 32],         // Default proof hash
                metadata: std::collections::HashMap::new(),
            };
            // Store the gossip entry
            self.store(blinded_entry).await?;
        }
        Ok(())
    }
}

#[async_trait]
impl super::StorageEngine for EpidemicStorageEngine {
    async fn store(
        &self,
        entry: BlindedStateEntry,
    ) -> Result<crate::types::storage_types::StorageResponse> {
        // Convert to unilateral entry
        let unilateral_entry = UnilateralEntry::new(
            entry.blinded_id.clone(),
            entry.encrypted_payload,
            entry.ttl,
            entry.region.clone(),
            entry.priority.try_into().unwrap_or(0),
        )?;

        // Verify entry integrity before storing
        if !unilateral_entry.verify() {
            return Err(StorageNodeError::Validation(
                "Entry failed integrity verification".into(),
            ));
        }

        self.unilateral_store
            .insert(entry.blinded_id.clone(), unilateral_entry);
        Ok(crate::types::storage_types::StorageResponse {
            blinded_id: entry.blinded_id,
            timestamp: current_time_secs(),
            status: "success".to_string(),
            message: None,
        })
    }

    async fn retrieve(&self, blinded_id: &str) -> Result<Option<BlindedStateEntry>> {
        if let Some(mut entry) = self.unilateral_store.get_mut(blinded_id) {
            // Record access for cache management
            entry.record_access();

            Ok(Some(BlindedStateEntry {
                blinded_id: entry.entry_id.clone(),
                encrypted_payload: entry.encrypted_payload.clone(),
                timestamp: entry
                    .timestamp
                    .duration_since(UNIX_EPOCH)
                    .unwrap_or_default()
                    .as_secs(),
                ttl: entry.ttl_seconds,
                region: entry.region.clone(),
                priority: entry.priority as i32,
                proof_hash: entry.verification_hash,
                metadata: HashMap::new(),
            }))
        } else {
            Ok(None)
        }
    }

    async fn delete(&self, blinded_id: &str) -> Result<bool> {
        Ok(self.unilateral_store.remove(blinded_id).is_some())
    }

    async fn exists(&self, blinded_id: &str) -> Result<bool> {
        Ok(self.unilateral_store.contains_key(blinded_id))
    }

    async fn list(&self, limit: Option<usize>, offset: Option<usize>) -> Result<Vec<String>> {
        let keys: Vec<String> = self
            .unilateral_store
            .iter()
            .map(|entry| entry.key().clone())
            .collect();
        let offset = offset.unwrap_or(0);
        let end = if let Some(limit) = limit {
            std::cmp::min(offset + limit, keys.len())
        } else {
            keys.len()
        };
        Ok(keys.into_iter().skip(offset).take(end - offset).collect())
    }

    async fn get_stats(&self) -> Result<super::StorageStats> {
        let now = current_time_secs();
        let mut total_entries = 0;
        let mut total_bytes = 0;
        let mut total_expired = 0;
        let mut oldest_timestamp = u64::MAX;
        let mut newest_timestamp = 0;
        let mut regions = std::collections::HashSet::new();

        // Iterate through all entries to collect comprehensive stats
        for entry_ref in self.unilateral_store.iter() {
            let entry = entry_ref.value();
            total_entries += 1;
            total_bytes += entry.size_bytes;

            // Track expired entries
            if entry.is_expired() {
                total_expired += 1;
            }

            // Track timestamp range
            let entry_timestamp = entry
                .timestamp
                .duration_since(UNIX_EPOCH)
                .unwrap_or_default()
                .as_secs();
            oldest_timestamp = oldest_timestamp.min(entry_timestamp);
            newest_timestamp = newest_timestamp.max(entry_timestamp);

            // Track unique regions
            regions.insert(entry.region.clone());
        }

        Ok(super::StorageStats {
            total_entries,
            total_bytes,
            total_expired,
            oldest_entry: if total_entries > 0 {
                Some(oldest_timestamp)
            } else {
                None
            },
            newest_entry: if total_entries > 0 {
                Some(newest_timestamp)
            } else {
                None
            },
            average_entry_size: if total_entries > 0 {
                total_bytes / total_entries
            } else {
                0
            },
            total_regions: regions.len(),
            last_updated: now,
        })
    }

    fn as_any(&self) -> &dyn std::any::Any {
        self
    }
}

impl EpidemicStorageEngine {
    /// Dump registered nodes for debugging
    pub async fn dump_registered_nodes(&self) -> Result<()> {
        let cluster_nodes = self.cluster_nodes.read().await;
        tracing::info!("Registered nodes ({}): ", cluster_nodes.len());
        for node in cluster_nodes.iter() {
            tracing::info!("  - Node: {} -> {}", node.id, node.endpoint);
        }
        Ok(())
    }
}
