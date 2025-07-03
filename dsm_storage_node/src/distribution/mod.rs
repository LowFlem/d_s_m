//! # Distribution Module for DSM Storage Node
//!
//! This module implements distributed storage management across multiple storage nodes,
//! providing data replication, sharding, and consistency mechanisms as specified in
//! the DSM whitepaper.
//!
//! ## Key Features
//!
//! * **Data Sharding**: Distributes data across multiple nodes based on configurable strategies
//! * **Replication**: Maintains multiple copies of data for availability and durability
//! * **Consistency**: Provides configurable consistency levels (eventual, strong)
//! * **Load Balancing**: Distributes requests across available nodes
//! * **Geographic Distribution**: Places replicas across different regions for resilience
//! * **Dynamic Rebalancing**: Automatically redistributes data when nodes join/leave
//!
//! ## Architecture
//!
//! The distribution layer operates above the storage engines and coordinates data placement
//! and retrieval across the cluster. It implements multiple distribution strategies:
//!
//! * **Hash-based sharding**: Uses consistent hashing for deterministic placement
//! * **Geographic sharding**: Distributes data based on geographical regions
//! * **Load-aware placement**: Considers node capacity and current load
//! * **Adaptive replication**: Adjusts replication factor based on data importance

use crate::error::{Result, StorageNodeError};
use crate::network::NetworkClient;
/// Distribution manager for coordinating data placement
use crate::network::{HttpNetworkClient, MockNetworkClient, NodeStatus, StateEntry};
use crate::storage::StorageEngine;
use crate::types::{BlindedStateEntry, StorageNode};

use async_trait::async_trait;
use blake3;
use serde::{Deserialize, Serialize};
use std::collections::{HashMap, HashSet};
use std::sync::Arc;
use std::time::{SystemTime, UNIX_EPOCH};
use tokio::sync::RwLock;
use tracing::{debug, error, info, warn};

/// Configuration for data distribution
#[derive(Debug, Clone)]
pub struct DistributionConfig {
    /// Default replication factor
    pub default_replication_factor: usize,
    /// Minimum number of replicas
    pub min_replicas: usize,
    /// Maximum number of replicas
    pub max_replicas: usize,
    /// Minimum number of geographic regions for replicas
    pub min_regions: usize,
    /// Consistency level (Eventual, Strong)
    pub consistency_level: ConsistencyLevel,
    /// Strategy for data placement
    pub placement_strategy: PlacementStrategy,
    /// Maximum time to wait for replication (seconds)
    pub replication_timeout_sec: u64,
    /// Whether to enable automatic rebalancing
    pub enable_rebalancing: bool,
    /// Interval for rebalancing operations (seconds)
    pub rebalancing_interval_sec: u64,
    /// Load threshold for triggering rebalancing (0.0-1.0)
    pub rebalancing_threshold: f64,
}

impl Default for DistributionConfig {
    fn default() -> Self {
        Self {
            default_replication_factor: 3,
            min_replicas: 2,
            max_replicas: 5,
            min_regions: 2,
            consistency_level: ConsistencyLevel::Eventual,
            placement_strategy: PlacementStrategy::ConsistentHashing,
            replication_timeout_sec: 30,
            enable_rebalancing: true,
            rebalancing_interval_sec: 3600, // 1 hour
            rebalancing_threshold: 0.8,     // 80% load threshold
        }
    }
}

/// Consistency levels for distributed operations
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum ConsistencyLevel {
    /// Eventual consistency - faster but may temporarily return stale data
    Eventual,
    /// Strong consistency - slower but guarantees latest data
    Strong,
    /// Quorum consistency - requires majority of replicas to agree
    Quorum,
}

/// Strategies for data placement across nodes
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum PlacementStrategy {
    /// Consistent hashing for even distribution
    ConsistentHashing,
    /// Geographic distribution prioritizing region diversity
    Geographic,
    /// Load-aware placement considering node capacity
    LoadAware,
    /// Random placement for testing
    Random,
}

/// Information about data placement
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PlacementInfo {
    /// Primary node storing the data
    pub primary_node: String,
    /// Replica nodes storing copies
    pub replica_nodes: Vec<String>,
    /// Placement timestamp
    pub placed_at: u64,
    /// Placement strategy used
    pub strategy: PlacementStrategy,
    /// Replication factor achieved
    pub replication_factor: usize,
    /// Number of regions covered
    pub regions_covered: usize,
}

/// Node capacity and load information
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NodeMetrics {
    /// Node identifier
    pub node_id: String,
    /// Total storage capacity in bytes
    pub total_capacity: u64,
    /// Used storage in bytes
    pub used_capacity: u64,
    /// Current load as percentage (0.0-1.0)
    pub load_percentage: f64,
    /// Number of operations per second
    pub operations_per_sec: f64,
    /// Network latency to this node (milliseconds)
    pub latency_ms: u64,
    /// Whether the node is healthy and available
    pub is_healthy: bool,
    /// Last updated timestamp
    pub last_updated: u64,
}

/// Enum to hold different network client implementations
#[derive(Clone)]
pub enum NetworkClientType {
    Http(Arc<HttpNetworkClient>),
    Mock(Arc<MockNetworkClient>),
}

#[async_trait]
impl NetworkClient for NetworkClientType {
    async fn health_check(&self, node_id: &str) -> Result<bool> {
        match self {
            NetworkClientType::Http(client) => client.health_check(node_id).await,
            NetworkClientType::Mock(client) => client.health_check(node_id).await,
        }
    }

    async fn forward_get(&self, node_id: String, key: String) -> Result<Option<Vec<u8>>> {
        match self {
            NetworkClientType::Http(client) => client.forward_get(node_id, key).await,
            NetworkClientType::Mock(client) => client.forward_get(node_id, key).await,
        }
    }

    async fn forward_put(&self, node_id: String, key: String, value: Vec<u8>) -> Result<()> {
        match self {
            NetworkClientType::Http(client) => client.forward_put(node_id, key, value).await,
            NetworkClientType::Mock(client) => client.forward_put(node_id, key, value).await,
        }
    }

    async fn forward_delete(&self, node_id: String, key: String) -> Result<()> {
        match self {
            NetworkClientType::Http(client) => client.forward_delete(node_id, key).await,
            NetworkClientType::Mock(client) => client.forward_delete(node_id, key).await,
        }
    }

    async fn send_entries(&self, node_id: String, entries: Vec<StateEntry>) -> Result<()> {
        match self {
            NetworkClientType::Http(client) => client.send_entries(node_id, entries).await,
            NetworkClientType::Mock(client) => client.send_entries(node_id, entries).await,
        }
    }

    async fn request_entries(&self, node_id: String, keys: Vec<String>) -> Result<Vec<StateEntry>> {
        match self {
            NetworkClientType::Http(client) => client.request_entries(node_id, keys).await,
            NetworkClientType::Mock(client) => client.request_entries(node_id, keys).await,
        }
    }

    async fn get_node_status(&self, node_id: &str) -> Result<NodeStatus> {
        match self {
            NetworkClientType::Http(client) => client.get_node_status(node_id).await,
            NetworkClientType::Mock(client) => client.get_node_status(node_id).await,
        }
    }

    async fn join_cluster(
        &self,
        bootstrap_nodes: Vec<String>,
        node_endpoint: String,
    ) -> Result<Vec<StorageNode>> {
        match self {
            NetworkClientType::Http(client) => {
                client.join_cluster(bootstrap_nodes, node_endpoint).await
            }
            NetworkClientType::Mock(client) => {
                client.join_cluster(bootstrap_nodes, node_endpoint).await
            }
        }
    }

    async fn register_node(&self, node_id: String, endpoint: String) {
        match self {
            NetworkClientType::Http(client) => client.register_node(node_id, endpoint).await,
            NetworkClientType::Mock(client) => client.register_node(node_id, endpoint).await,
        }
    }

    fn send_message(
        &self,
        device_id: std::net::SocketAddr,
        message_id: [u8; 32],
        data: Vec<u8>,
        ttl: u8,
    ) -> Result<()> {
        match self {
            NetworkClientType::Http(client) => {
                client.send_message(device_id, message_id, data, ttl)
            }
            NetworkClientType::Mock(client) => {
                client.send_message(device_id, message_id, data, ttl)
            }
        }
    }

    fn find_nodes(&self, target: &crate::storage::topology::NodeId) -> Result<()> {
        match self {
            NetworkClientType::Http(client) => client.find_nodes(target),
            NetworkClientType::Mock(client) => client.find_nodes(target),
        }
    }

    fn find_nodes_in_region(&self, region: u8) -> Result<()> {
        match self {
            NetworkClientType::Http(client) => client.find_nodes_in_region(region),
            NetworkClientType::Mock(client) => client.find_nodes_in_region(region),
        }
    }

    fn get_metrics(&self) -> crate::network::NetworkMetricsSnapshot {
        match self {
            NetworkClientType::Http(client) => client.get_metrics(),
            NetworkClientType::Mock(client) => client.get_metrics(),
        }
    }

    fn get_connection_status(&self) -> crate::network::ConnectionPoolStatus {
        match self {
            NetworkClientType::Http(client) => client.get_connection_status(),
            NetworkClientType::Mock(client) => client.get_connection_status(),
        }
    }
}

pub struct DistributionManager {
    /// Configuration for distribution behavior
    config: DistributionConfig,
    /// Network client for communicating with other nodes
    network_client: NetworkClientType,
    /// Local storage engine
    local_storage: Arc<dyn StorageEngine + Send + Sync>,
    /// Information about known nodes in the cluster
    cluster_nodes: Arc<RwLock<HashMap<String, StorageNode>>>,
    /// Metrics for each node
    node_metrics: Arc<RwLock<HashMap<String, NodeMetrics>>>,
    /// Placement information for stored data
    placement_registry: Arc<RwLock<HashMap<String, PlacementInfo>>>,
    /// This node's identifier
    local_node_id: String,
}

impl DistributionManager {
    /// Create a new distribution manager
    pub fn new(
        config: DistributionConfig,
        network_client: NetworkClientType,
        local_storage: Arc<dyn StorageEngine + Send + Sync>,
        local_node_id: String,
    ) -> Self {
        Self {
            config,
            network_client,
            local_storage,
            cluster_nodes: Arc::new(RwLock::new(HashMap::new())),
            node_metrics: Arc::new(RwLock::new(HashMap::new())),
            placement_registry: Arc::new(RwLock::new(HashMap::new())),
            local_node_id,
        }
    }

    /// Initialize the distribution manager
    pub async fn initialize(&mut self, initial_nodes: Vec<StorageNode>) -> Result<()> {
        info!(
            "Initializing distribution manager with {} nodes",
            initial_nodes.len()
        );

        // Add initial nodes to the cluster
        {
            let mut nodes = self.cluster_nodes.write().await;
            for node in initial_nodes {
                nodes.insert(node.id.clone(), node);
            }
        }

        // Start periodic tasks
        self.start_periodic_tasks().await;

        // Update node metrics
        self.update_all_node_metrics().await?;

        info!("Distribution manager initialized successfully");
        Ok(())
    }

    /// Add a node to the cluster
    pub async fn add_node(&self, node: StorageNode) -> Result<()> {
        info!("Adding node {} to cluster", node.id);

        {
            let mut nodes = self.cluster_nodes.write().await;
            nodes.insert(node.id.clone(), node.clone());
        }

        // Update metrics for the new node
        self.update_node_metrics(&node.id).await?;

        // Trigger rebalancing if enabled
        if self.config.enable_rebalancing {
            self.trigger_rebalancing().await?;
        }

        Ok(())
    }

    /// Remove a node from the cluster
    pub async fn remove_node(&self, node_id: &str) -> Result<()> {
        info!("Removing node {} from cluster", node_id);

        {
            let mut nodes = self.cluster_nodes.write().await;
            nodes.remove(node_id);
        }

        {
            let mut metrics = self.node_metrics.write().await;
            metrics.remove(node_id);
        }

        // Handle data migration from the removed node
        self.handle_node_removal(node_id).await?;

        Ok(())
    }

    /// Store data with distribution
    pub async fn store_distributed(&self, entry: BlindedStateEntry) -> Result<PlacementInfo> {
        debug!("Storing data with distribution: {}", entry.blinded_id);

        // Determine replication factor based on entry metadata
        let replication_factor = self.determine_replication_factor(&entry);

        // Select nodes for placement
        let selected_nodes = self
            .select_nodes_for_placement(&entry.blinded_id, replication_factor)
            .await?;

        if selected_nodes.is_empty() {
            return Err(StorageNodeError::Distribution(
                "No nodes available for placement".to_string(),
            ));
        }

        // Store on the primary node (first in the list)
        let primary_node = &selected_nodes[0];
        let replica_nodes = selected_nodes[1..].to_vec();

        // Store locally if this is the primary node
        if primary_node == &self.local_node_id {
            self.local_storage.store(entry.clone()).await.map_err(|e| {
                StorageNodeError::Distribution(format!("Failed to store locally: {e}"))
            })?;
        } else {
            // Forward to primary node
            let node_info = self.get_node_info(primary_node).await?;
            self.network_client
                .forward_put(
                    node_info.id.clone(),
                    entry.blinded_id.clone(),
                    entry.encrypted_payload.clone(),
                )
                .await
                .map_err(|e| {
                    StorageNodeError::Distribution(format!("Failed to forward to primary: {e}"))
                })?;
        }

        // Replicate to replica nodes
        let mut successful_replicas = Vec::new();
        for replica_node in &replica_nodes {
            if replica_node == &self.local_node_id {
                // Store locally
                match self.local_storage.store(entry.clone()).await {
                    Ok(_) => successful_replicas.push(replica_node.clone()),
                    Err(e) => warn!("Failed to store replica locally: {}", e),
                }
            } else {
                // Forward to replica node
                match self.get_node_info(replica_node).await {
                    Ok(node_info) => {
                        match self
                            .network_client
                            .forward_put(
                                node_info.id.clone(),
                                entry.blinded_id.clone(),
                                entry.encrypted_payload.clone(),
                            )
                            .await
                        {
                            Ok(_) => successful_replicas.push(replica_node.clone()),
                            Err(e) => warn!("Failed to replicate to {}: {}", replica_node, e),
                        }
                    }
                    Err(e) => warn!(
                        "Failed to get info for replica node {}: {}",
                        replica_node, e
                    ),
                }
            }
        }

        // Count regions covered
        let regions_covered = self.count_regions_covered(&selected_nodes).await;

        let replication_factor = successful_replicas.len() + 1; // +1 for primary

        // Create placement info
        let placement_info = PlacementInfo {
            primary_node: primary_node.clone(),
            replica_nodes: successful_replicas.clone(),
            placed_at: SystemTime::now()
                .duration_since(UNIX_EPOCH)
                .unwrap_or_default()
                .as_secs(),
            strategy: self.config.placement_strategy,
            replication_factor,
            regions_covered,
        };

        // Store placement info
        {
            let mut registry = self.placement_registry.write().await;
            registry.insert(entry.blinded_id.clone(), placement_info.clone());
        }

        debug!(
            "Successfully placed data {} on {} nodes (primary: {}, replicas: {:?})",
            entry.blinded_id, placement_info.replication_factor, primary_node, successful_replicas
        );

        Ok(placement_info)
    }

    /// Retrieve data with distribution
    pub async fn retrieve_distributed(
        &self,
        blinded_id: &str,
    ) -> Result<Option<BlindedStateEntry>> {
        debug!("Retrieving distributed data: {}", blinded_id);

        // Check local storage first
        if let Ok(Some(entry)) = self.local_storage.retrieve(blinded_id).await {
            debug!("Found data locally: {}", blinded_id);
            return Ok(Some(entry));
        }

        // Get placement info
        let placement_info = {
            let registry = self.placement_registry.read().await;
            registry.get(blinded_id).cloned()
        };

        if let Some(placement) = placement_info {
            // Try primary node first
            if placement.primary_node != self.local_node_id {
                match self
                    .retrieve_from_node(&placement.primary_node, blinded_id)
                    .await
                {
                    Ok(Some(entry)) => {
                        debug!(
                            "Retrieved data from primary node: {}",
                            placement.primary_node
                        );
                        return Ok(Some(entry));
                    }
                    Ok(None) => {
                        debug!("Data not found on primary node: {}", placement.primary_node);
                    }
                    Err(e) => {
                        warn!(
                            "Failed to retrieve from primary node {}: {}",
                            placement.primary_node, e
                        );
                    }
                }
            }

            // Try replica nodes
            for replica_node in &placement.replica_nodes {
                if replica_node != &self.local_node_id {
                    match self.retrieve_from_node(replica_node, blinded_id).await {
                        Ok(Some(entry)) => {
                            debug!("Retrieved data from replica node: {}", replica_node);
                            return Ok(Some(entry));
                        }
                        Ok(None) => {
                            debug!("Data not found on replica node: {}", replica_node);
                        }
                        Err(e) => {
                            warn!(
                                "Failed to retrieve from replica node {}: {}",
                                replica_node, e
                            );
                        }
                    }
                }
            }
        } else {
            // No placement info - try all available nodes
            debug!(
                "No placement info found, searching all nodes for: {}",
                blinded_id
            );

            let nodes = self.cluster_nodes.read().await;
            for (node_id, _) in nodes.iter() {
                if node_id != &self.local_node_id {
                    match self.retrieve_from_node(node_id, blinded_id).await {
                        Ok(Some(entry)) => {
                            debug!("Found data on node: {}", node_id);
                            return Ok(Some(entry));
                        }
                        Ok(None) => continue,
                        Err(e) => {
                            warn!("Failed to query node {}: {}", node_id, e);
                            continue;
                        }
                    }
                }
            }
        }

        debug!("Data not found on any node: {}", blinded_id);
        Ok(None)
    }

    /// Delete data with distribution
    pub async fn delete_distributed(&self, blinded_id: &str) -> Result<bool> {
        debug!("Deleting distributed data: {}", blinded_id);

        let mut deleted_anywhere = false;

        // Delete locally
        if let Ok(deleted) = self.local_storage.delete(blinded_id).await {
            if deleted {
                deleted_anywhere = true;
                debug!("Deleted data locally: {}", blinded_id);
            }
        }

        // Get placement info
        let placement_info = {
            let registry = self.placement_registry.read().await;
            registry.get(blinded_id).cloned()
        };

        if let Some(placement) = placement_info {
            // Delete from primary node
            if placement.primary_node != self.local_node_id {
                match self
                    .delete_from_node(&placement.primary_node, blinded_id)
                    .await
                {
                    Ok(deleted) => {
                        if deleted {
                            deleted_anywhere = true;
                            debug!("Deleted data from primary node: {}", placement.primary_node);
                        }
                    }
                    Err(e) => {
                        warn!(
                            "Failed to delete from primary node {}: {}",
                            placement.primary_node, e
                        );
                    }
                }
            }

            // Delete from replica nodes
            for replica_node in &placement.replica_nodes {
                if replica_node != &self.local_node_id {
                    match self.delete_from_node(replica_node, blinded_id).await {
                        Ok(deleted) => {
                            if deleted {
                                deleted_anywhere = true;
                                debug!("Deleted data from replica node: {}", replica_node);
                            }
                        }
                        Err(e) => {
                            warn!("Failed to delete from replica node {}: {}", replica_node, e);
                        }
                    }
                }
            }

            // Remove placement info
            {
                let mut registry = self.placement_registry.write().await;
                registry.remove(blinded_id);
            }
        } else {
            // No placement info - try all available nodes
            let nodes = self.cluster_nodes.read().await;
            for (node_id, _) in nodes.iter() {
                if node_id != &self.local_node_id {
                    match self.delete_from_node(node_id, blinded_id).await {
                        Ok(deleted) => {
                            if deleted {
                                deleted_anywhere = true;
                                debug!("Deleted data from node: {}", node_id);
                            }
                        }
                        Err(e) => {
                            warn!("Failed to delete from node {}: {}", node_id, e);
                        }
                    }
                }
            }
        }

        Ok(deleted_anywhere)
    }

    /// Get distribution statistics
    pub async fn get_distribution_stats(&self) -> Result<DistributionStats> {
        let placement_count = self.placement_registry.read().await.len();
        let cluster_size = self.cluster_nodes.read().await.len();

        let metrics = self.node_metrics.read().await;
        let healthy_nodes = metrics.values().filter(|m| m.is_healthy).count();

        let total_capacity: u64 = metrics.values().map(|m| m.total_capacity).sum();
        let used_capacity: u64 = metrics.values().map(|m| m.used_capacity).sum();

        let avg_load = if !metrics.is_empty() {
            metrics.values().map(|m| m.load_percentage).sum::<f64>() / metrics.len() as f64
        } else {
            0.0
        };

        Ok(DistributionStats {
            cluster_size,
            healthy_nodes,
            total_entries: placement_count,
            total_capacity,
            used_capacity,
            average_load: avg_load,
            replication_factor: self.config.default_replication_factor,
        })
    }

    // Helper methods

    /// Select nodes for data placement
    async fn select_nodes_for_placement(
        &self,
        blinded_id: &str,
        replication_factor: usize,
    ) -> Result<Vec<String>> {
        let nodes = self.cluster_nodes.read().await;
        let metrics = self.node_metrics.read().await;

        // Filter healthy nodes
        let healthy_nodes: Vec<_> = nodes
            .iter()
            .filter(|(node_id, _)| metrics.get(*node_id).map(|m| m.is_healthy).unwrap_or(false))
            .collect();

        if healthy_nodes.is_empty() {
            return Err(StorageNodeError::Distribution(
                "No healthy nodes available".to_string(),
            ));
        }

        let selected = match self.config.placement_strategy {
            PlacementStrategy::ConsistentHashing => {
                self.select_nodes_consistent_hashing(blinded_id, &healthy_nodes, replication_factor)
            }
            PlacementStrategy::Geographic => {
                self.select_nodes_geographic(&healthy_nodes, replication_factor)
            }
            PlacementStrategy::LoadAware => {
                self.select_nodes_load_aware(&healthy_nodes, &metrics, replication_factor)
            }
            PlacementStrategy::Random => {
                self.select_nodes_random(&healthy_nodes, replication_factor)
            }
        };

        Ok(selected)
    }

    /// Select nodes using consistent hashing
    fn select_nodes_consistent_hashing(
        &self,
        blinded_id: &str,
        healthy_nodes: &[(&String, &StorageNode)],
        replication_factor: usize,
    ) -> Vec<String> {
        let mut node_hashes: Vec<_> = healthy_nodes
            .iter()
            .map(|(node_id, _)| {
                let hash = self.hash_for_placement(blinded_id, node_id);
                (hash, (*node_id).clone())
            })
            .collect();

        // Sort by hash value
        node_hashes.sort_by_key(|(hash, _)| *hash);

        // Take the required number of nodes
        node_hashes
            .into_iter()
            .take(replication_factor.min(healthy_nodes.len()))
            .map(|(_, node_id)| node_id)
            .collect()
    }

    /// Select nodes for geographic distribution
    fn select_nodes_geographic(
        &self,
        healthy_nodes: &[(&String, &StorageNode)],
        replication_factor: usize,
    ) -> Vec<String> {
        let mut selected = Vec::new();
        let mut used_regions = HashSet::new();

        // First pass: select one node from each region
        for (node_id, node) in healthy_nodes {
            if !used_regions.contains(&node.region) && selected.len() < replication_factor {
                selected.push((*node_id).clone());
                used_regions.insert(node.region.clone());
            }
        }

        // Second pass: fill remaining slots with any healthy nodes
        for (node_id, _) in healthy_nodes {
            if !selected.contains(*node_id) && selected.len() < replication_factor {
                selected.push((*node_id).clone());
            }
        }

        selected
    }

    /// Select nodes based on load
    fn select_nodes_load_aware(
        &self,
        healthy_nodes: &[(&String, &StorageNode)],
        metrics: &HashMap<String, NodeMetrics>,
        replication_factor: usize,
    ) -> Vec<String> {
        let mut node_loads: Vec<_> = healthy_nodes
            .iter()
            .filter_map(|(node_id, _)| {
                metrics
                    .get(*node_id)
                    .map(|m| (m.load_percentage, (*node_id).clone()))
            })
            .collect();

        // Sort by load (ascending - prefer less loaded nodes)
        node_loads.sort_by(|a, b| a.0.partial_cmp(&b.0).unwrap_or(std::cmp::Ordering::Equal));

        node_loads
            .into_iter()
            .take(replication_factor.min(healthy_nodes.len()))
            .map(|(_, node_id)| node_id)
            .collect()
    }

    /// Select nodes randomly
    fn select_nodes_random(
        &self,
        healthy_nodes: &[(&String, &StorageNode)],
        replication_factor: usize,
    ) -> Vec<String> {
        use rand::seq::SliceRandom;
        use rand::thread_rng;

        let mut node_ids: Vec<_> = healthy_nodes.iter().map(|(id, _)| (*id).clone()).collect();
        node_ids.shuffle(&mut thread_rng());

        node_ids
            .into_iter()
            .take(replication_factor.min(healthy_nodes.len()))
            .collect()
    }

    /// Calculate hash for consistent placement
    fn hash_for_placement(&self, blinded_id: &str, node_id: &str) -> u64 {
        let mut hasher = blake3::Hasher::new();
        hasher.update(blinded_id.as_bytes());
        hasher.update(node_id.as_bytes());
        let hash = hasher.finalize();

        // Convert first 8 bytes to u64
        let bytes = hash.as_bytes();
        u64::from_be_bytes([
            bytes[0], bytes[1], bytes[2], bytes[3], bytes[4], bytes[5], bytes[6], bytes[7],
        ])
    }

    /// Determine replication factor for an entry
    fn determine_replication_factor(&self, entry: &BlindedStateEntry) -> usize {
        // Use metadata to determine importance
        let priority = entry.priority;

        if priority >= 10 {
            self.config.max_replicas.min(5) // High priority
        } else if priority >= 5 {
            self.config.default_replication_factor.min(4) // Medium priority
        } else {
            self.config.min_replicas.max(2) // Low priority
        }
    }

    /// Count regions covered by selected nodes
    async fn count_regions_covered(&self, node_ids: &[String]) -> usize {
        let nodes = self.cluster_nodes.read().await;
        let regions: HashSet<_> = node_ids
            .iter()
            .filter_map(|id| nodes.get(id).map(|node| &node.region))
            .collect();
        regions.len()
    }

    /// Get node information
    async fn get_node_info(&self, node_id: &str) -> Result<StorageNode> {
        let nodes = self.cluster_nodes.read().await;
        nodes
            .get(node_id)
            .cloned()
            .ok_or_else(|| StorageNodeError::NotFound(format!("Node not found: {node_id}")))
    }

    /// Retrieve data from a specific node
    async fn retrieve_from_node(
        &self,
        node_id: &str,
        blinded_id: &str,
    ) -> Result<Option<BlindedStateEntry>> {
        match self
            .network_client
            .forward_get(node_id.to_string(), blinded_id.to_string())
            .await
        {
            Ok(Some(data)) => {
                // Deserialize the data back to BlindedStateEntry
                match bincode::deserialize::<BlindedStateEntry>(&data) {
                    Ok(entry) => Ok(Some(entry)),
                    Err(_) => {
                        // If deserialization fails, create a basic entry
                        let entry = BlindedStateEntry {
                            blinded_id: blinded_id.to_string(),
                            encrypted_payload: data,
                            timestamp: SystemTime::now()
                                .duration_since(UNIX_EPOCH)
                                .unwrap_or_default()
                                .as_secs(),
                            ttl: 0,
                            region: "unknown".to_string(),
                            priority: 0,
                            proof_hash: [0u8; 32],
                            metadata: HashMap::new(),
                        };
                        Ok(Some(entry))
                    }
                }
            }
            Ok(None) => Ok(None),
            Err(e) => Err(e),
        }
    }

    /// Delete data from a specific node
    async fn delete_from_node(&self, node_id: &str, blinded_id: &str) -> Result<bool> {
        match self
            .network_client
            .forward_delete(node_id.to_string(), blinded_id.to_string())
            .await
        {
            Ok(_) => Ok(true),
            Err(_) => Ok(false),
        }
    }

    /// Update metrics for all nodes
    async fn update_all_node_metrics(&self) -> Result<()> {
        let nodes = self.cluster_nodes.read().await.clone();

        for (node_id, _) in nodes {
            if let Err(e) = self.update_node_metrics(&node_id).await {
                warn!("Failed to update metrics for node {}: {}", node_id, e);
            }
        }

        Ok(())
    }

    /// Update metrics for a specific node
    async fn update_node_metrics(&self, node_id: &str) -> Result<()> {
        if node_id == self.local_node_id {
            // Update local metrics
            let stats = self.local_storage.get_stats().await?;

            let metrics = NodeMetrics {
                node_id: node_id.to_string(),
                total_capacity: 1024 * 1024 * 1024 * 1024, // 1TB default
                used_capacity: stats.total_bytes as u64,
                load_percentage: (stats.total_bytes as f64) / (1024.0 * 1024.0 * 1024.0 * 1024.0),
                operations_per_sec: 100.0, // Placeholder
                latency_ms: 0,
                is_healthy: true,
                last_updated: SystemTime::now()
                    .duration_since(UNIX_EPOCH)
                    .unwrap_or_default()
                    .as_secs(),
            };

            let mut node_metrics = self.node_metrics.write().await;
            node_metrics.insert(node_id.to_string(), metrics);
        } else {
            // Query remote node for metrics
            match self.network_client.health_check(node_id).await {
                Ok(is_healthy) => {
                    let metrics = NodeMetrics {
                        node_id: node_id.to_string(),
                        total_capacity: 1024 * 1024 * 1024 * 1024, // 1TB default
                        used_capacity: 0,                          // Would be fetched from remote
                        load_percentage: 0.5,                      // Placeholder
                        operations_per_sec: 100.0,                 // Placeholder
                        latency_ms: 50,                            // Placeholder
                        is_healthy,
                        last_updated: SystemTime::now()
                            .duration_since(UNIX_EPOCH)
                            .unwrap_or_default()
                            .as_secs(),
                    };

                    let mut node_metrics = self.node_metrics.write().await;
                    node_metrics.insert(node_id.to_string(), metrics);
                }
                Err(e) => {
                    warn!("Health check failed for node {}: {}", node_id, e);

                    // Mark as unhealthy
                    let mut node_metrics = self.node_metrics.write().await;
                    if let Some(metrics) = node_metrics.get_mut(node_id) {
                        metrics.is_healthy = false;
                        metrics.last_updated = SystemTime::now()
                            .duration_since(UNIX_EPOCH)
                            .unwrap_or_default()
                            .as_secs();
                    }
                }
            }
        }

        Ok(())
    }

    /// Start periodic background tasks
    async fn start_periodic_tasks(&self) {
        // Metrics update task
        let metrics_manager = self.clone();
        tokio::spawn(async move {
            let mut interval = tokio::time::interval(tokio::time::Duration::from_secs(60)); // 1 minute

            loop {
                interval.tick().await;
                if let Err(e) = metrics_manager.update_all_node_metrics().await {
                    error!("Failed to update node metrics: {}", e);
                }
            }
        });

        // Rebalancing task
        if self.config.enable_rebalancing {
            let rebalancing_manager = self.clone();
            tokio::spawn(async move {
                let mut interval = tokio::time::interval(tokio::time::Duration::from_secs(
                    rebalancing_manager.config.rebalancing_interval_sec,
                ));

                loop {
                    interval.tick().await;
                    if let Err(e) = rebalancing_manager.check_and_rebalance().await {
                        error!("Failed to perform rebalancing: {}", e);
                    }
                }
            });
        }
    }

    /// Trigger rebalancing operation
    async fn trigger_rebalancing(&self) -> Result<()> {
        info!("Triggering data rebalancing");
        self.check_and_rebalance().await
    }

    /// Check if rebalancing is needed and perform it
    async fn check_and_rebalance(&self) -> Result<()> {
        let metrics = self.node_metrics.read().await;

        // Check if any node is over the threshold
        let needs_rebalancing = metrics
            .values()
            .any(|m| m.is_healthy && m.load_percentage > self.config.rebalancing_threshold);

        if needs_rebalancing {
            info!("Node load threshold exceeded, starting rebalancing");
            self.perform_rebalancing().await?;
        }

        Ok(())
    }

    /// Perform actual rebalancing operation
    async fn perform_rebalancing(&self) -> Result<()> {
        // This is a simplified rebalancing implementation
        // In production, this would be more sophisticated

        let metrics = self.node_metrics.read().await;

        // Find overloaded and underloaded nodes
        let overloaded: Vec<_> = metrics
            .iter()
            .filter(|(_, m)| m.is_healthy && m.load_percentage > self.config.rebalancing_threshold)
            .map(|(id, _)| id.clone())
            .collect();

        let underloaded: Vec<_> = metrics
            .iter()
            .filter(|(_, m)| m.is_healthy && m.load_percentage < 0.3) // 30% threshold
            .map(|(id, _)| id.clone())
            .collect();

        if overloaded.is_empty() || underloaded.is_empty() {
            return Ok(());
        }

        info!(
            "Rebalancing {} overloaded nodes to {} underloaded nodes",
            overloaded.len(),
            underloaded.len()
        );

        // This would implement the actual data migration logic
        // For now, we just log the operation
        Ok(())
    }

    /// Handle removal of a node from the cluster
    async fn handle_node_removal(&self, removed_node_id: &str) -> Result<()> {
        info!("Handling removal of node: {}", removed_node_id);

        // Find all data that was stored on the removed node
        let placement_registry = self.placement_registry.read().await;
        let affected_entries: Vec<_> = placement_registry
            .iter()
            .filter(|(_, placement)| {
                placement.primary_node == removed_node_id
                    || placement
                        .replica_nodes
                        .contains(&removed_node_id.to_string())
            })
            .map(|(id, placement)| (id.clone(), placement.clone()))
            .collect();

        drop(placement_registry);

        info!(
            "Found {} entries affected by node removal",
            affected_entries.len()
        );

        // For each affected entry, ensure we still have enough replicas
        for (blinded_id, mut placement_info) in affected_entries {
            let target_replicas = self.config.default_replication_factor;
            let current_replicas = if placement_info.primary_node == removed_node_id {
                placement_info.replica_nodes.len()
            } else {
                placement_info.replica_nodes.len() + 1 // +1 for primary
            };

            if current_replicas < target_replicas {
                // Need to create more replicas
                let needed_replicas = target_replicas - current_replicas;

                match self
                    .create_additional_replicas(&blinded_id, needed_replicas)
                    .await
                {
                    Ok(new_replicas) => {
                        info!(
                            "Created {} additional replicas for {}: {:?}",
                            new_replicas.len(),
                            blinded_id,
                            new_replicas
                        );

                        // Update placement info
                        if placement_info.primary_node == removed_node_id
                            && !placement_info.replica_nodes.is_empty()
                        {
                            // Promote first replica to primary
                            placement_info.primary_node = placement_info.replica_nodes.remove(0);
                        }

                        placement_info.replica_nodes.extend(new_replicas);
                        placement_info
                            .replica_nodes
                            .retain(|id| id != removed_node_id);

                        // Update registry
                        let mut registry = self.placement_registry.write().await;
                        registry.insert(blinded_id, placement_info);
                    }
                    Err(e) => {
                        error!(
                            "Failed to create additional replicas for {}: {}",
                            blinded_id, e
                        );
                    }
                }
            }
        }

        Ok(())
    }

    /// Create additional replicas for data
    async fn create_additional_replicas(
        &self,
        blinded_id: &str,
        count: usize,
    ) -> Result<Vec<String>> {
        // First, try to retrieve the data
        let entry = match self.retrieve_distributed(blinded_id).await? {
            Some(entry) => entry,
            None => {
                return Err(StorageNodeError::NotFound(format!(
                    "Data not found for replication: {blinded_id}"
                )));
            }
        };

        // Select new nodes for replicas
        let new_nodes = self.select_nodes_for_placement(blinded_id, count).await?;
        let mut successful_replicas = Vec::new();

        // Create replicas on selected nodes
        for node_id in new_nodes {
            if node_id == self.local_node_id {
                // Store locally
                match self.local_storage.store(entry.clone()).await {
                    Ok(_) => successful_replicas.push(node_id),
                    Err(e) => warn!("Failed to store replica locally: {}", e),
                }
            } else {
                // Forward to remote node
                match self
                    .network_client
                    .forward_put(
                        node_id.clone(),
                        entry.blinded_id.clone(),
                        entry.encrypted_payload.clone(),
                    )
                    .await
                {
                    Ok(_) => successful_replicas.push(node_id),
                    Err(e) => warn!("Failed to create replica on {}: {}", node_id, e),
                }
            }
        }

        Ok(successful_replicas)
    }

    /// Clone method for async tasks
    fn clone(&self) -> Self {
        Self {
            config: self.config.clone(),
            network_client: self.network_client.clone(),
            local_storage: self.local_storage.clone(),
            cluster_nodes: self.cluster_nodes.clone(),
            node_metrics: self.node_metrics.clone(),
            placement_registry: self.placement_registry.clone(),
            local_node_id: self.local_node_id.clone(),
        }
    }
}

/// Statistics about the distribution system
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DistributionStats {
    /// Total number of nodes in the cluster
    pub cluster_size: usize,
    /// Number of healthy nodes
    pub healthy_nodes: usize,
    /// Total number of distributed entries
    pub total_entries: usize,
    /// Total storage capacity across all nodes
    pub total_capacity: u64,
    /// Used storage capacity across all nodes
    pub used_capacity: u64,
    /// Average load across all nodes
    pub average_load: f64,
    /// Current replication factor
    pub replication_factor: usize,
}

/// Trait for distributed storage operations
#[async_trait]
pub trait DistributedStorage: Send + Sync {
    /// Store data with distribution
    async fn store_distributed(&self, entry: BlindedStateEntry) -> Result<PlacementInfo>;

    /// Retrieve data with distribution
    async fn retrieve_distributed(&self, blinded_id: &str) -> Result<Option<BlindedStateEntry>>;

    /// Delete data with distribution
    async fn delete_distributed(&self, blinded_id: &str) -> Result<bool>;

    /// Get distribution statistics
    async fn get_distribution_stats(&self) -> Result<DistributionStats>;
}

#[async_trait]
impl DistributedStorage for DistributionManager {
    async fn store_distributed(&self, entry: BlindedStateEntry) -> Result<PlacementInfo> {
        self.store_distributed(entry).await
    }

    async fn retrieve_distributed(&self, blinded_id: &str) -> Result<Option<BlindedStateEntry>> {
        self.retrieve_distributed(blinded_id).await
    }

    async fn delete_distributed(&self, blinded_id: &str) -> Result<bool> {
        self.delete_distributed(blinded_id).await
    }

    async fn get_distribution_stats(&self) -> Result<DistributionStats> {
        self.get_distribution_stats().await
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::network::MockNetworkClient;
    use crate::storage::memory_storage::{MemoryStorage, MemoryStorageConfig};

    fn create_test_manager() -> DistributionManager {
        let config = DistributionConfig::default();
        let network_client = NetworkClientType::Mock(Arc::new(MockNetworkClient::new()));
        let storage = Arc::new(MemoryStorage::new(MemoryStorageConfig::default()));

        DistributionManager::new(config, network_client, storage, "test-node".to_string())
    }

    #[tokio::test]
    async fn test_node_management() {
        let manager = create_test_manager();

        let node = StorageNode {
            id: "node1".to_string(),
            name: "Test Node 1".to_string(),
            region: "us-east-1".to_string(),
            public_key: "test-key".to_string(),
            endpoint: "http://localhost:8080".to_string(),
        };

        // Test adding node
        assert!(manager.add_node(node.clone()).await.is_ok());

        // Test removing node
        assert!(manager.remove_node(&node.id).await.is_ok());
    }

    #[tokio::test]
    async fn test_consistent_hashing() {
        let manager = create_test_manager();

        let node1 = StorageNode {
            id: "node1".to_string(),
            name: "Node 1".to_string(),
            region: "us-east-1".to_string(),
            public_key: "key1".to_string(),
            endpoint: "http://localhost:8081".to_string(),
        };
        let node2 = StorageNode {
            id: "node2".to_string(),
            name: "Node 2".to_string(),
            region: "us-west-1".to_string(),
            public_key: "key2".to_string(),
            endpoint: "http://localhost:8082".to_string(),
        };

        let node1_id = "node1".to_string();
        let node2_id = "node2".to_string();

        let nodes = vec![(&node1_id, &node1), (&node2_id, &node2)];

        let selected = manager.select_nodes_consistent_hashing("test-key", &nodes, 2);
        assert_eq!(selected.len(), 2);
    }
}
