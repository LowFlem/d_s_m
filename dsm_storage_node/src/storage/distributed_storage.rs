// Distributed storage implementation for DSM Storage Node
//
// This is a distributed storage implementation based on the epidemic
// distribution protocol described in Section 16.4 of the whitepaper.

use crate::error::Result;
use crate::types::storage_types::{StorageResponse, StorageStats};
use crate::types::BlindedStateEntry;
use crate::types::StorageNode;
use async_trait::async_trait;
use std::collections::HashMap;
use std::sync::{Arc, Mutex};
use std::time::{Duration, SystemTime, UNIX_EPOCH};
use tokio::sync::RwLock;
use tracing::info;

/// Calculate hash distance for consistent hashing
fn calculate_hash_distance(hash1: &blake3::Hash, hash2: &blake3::Hash) -> u64 {
    let bytes1 = hash1.as_bytes();
    let bytes2 = hash2.as_bytes();

    // Calculate XOR distance for the first 8 bytes
    let mut distance = 0u64;
    for i in 0..8 {
        distance ^= (bytes1[i] as u64) << (i * 8);
        distance ^= (bytes2[i] as u64) << (i * 8);
    }

    distance
}

/// Storage node configuration
#[derive(Debug, Clone)]
pub struct DistributedNodeConfig {
    /// Node ID
    pub id: String,
    /// Node name
    pub name: String,
    /// Node region
    pub region: String,
    /// Endpoint URL
    pub endpoint: String,
    /// Replication factor
    pub replication_factor: u8,
    /// Minimum geographic regions
    pub min_regions: u8,
    /// Bootstrap nodes
    pub bootstrap_nodes: Vec<String>,
    /// Synchronization interval
    pub sync_interval: Duration,
}

/// Distributed storage engine
#[derive(Clone)]
#[allow(dead_code)]
pub struct DistributedStorage {
    /// Local storage engine
    local_storage: Arc<dyn super::StorageEngine + Send + Sync>,
    /// Node configuration
    config: DistributedNodeConfig,
    /// Known nodes
    nodes: Arc<RwLock<HashMap<String, StorageNode>>>,
    /// Distribution cache (pending entries to distribute)
    distribution_cache: Arc<Mutex<HashMap<String, BlindedStateEntry>>>,
    /// Synchronization state
    sync_state: Arc<RwLock<HashMap<String, u64>>>,
}

impl DistributedStorage {
    /// Create a new distributed storage engine
    pub fn new(
        local_storage: Arc<dyn super::StorageEngine + Send + Sync>,
        node_id: String,
        storage_nodes: Vec<StorageNode>,
        replication_factor: usize,
        _max_hops: usize,
    ) -> Result<Self> {
        info!(
            "Creating new distributed storage engine with {} storage nodes",
            storage_nodes.len()
        );

        // Create default config
        let config = DistributedNodeConfig {
            id: node_id,
            name: format!("dsm-node-{}", uuid::Uuid::new_v4()),
            region: "global".to_string(),
            endpoint: "http://localhost:3000".to_string(),
            replication_factor: replication_factor as u8,
            min_regions: 1,
            bootstrap_nodes: Vec::new(),
            sync_interval: Duration::from_secs(60),
        };

        // Initialize nodes map
        let mut nodes_map = HashMap::new();
        for node in storage_nodes {
            nodes_map.insert(node.id.clone(), node);
        }

        let instance = Self {
            local_storage,
            config,
            nodes: Arc::new(RwLock::new(nodes_map)),
            distribution_cache: Arc::new(Mutex::new(HashMap::new())),
            sync_state: Arc::new(RwLock::new(HashMap::new())),
        };

        // Start background synchronization task
        let sync_instance = instance.clone();
        tokio::spawn(async move {
            let mut interval = tokio::time::interval(sync_instance.config.sync_interval);
            loop {
                interval.tick().await;
                if let Err(e) = sync_instance.synchronize_all_nodes().await {
                    tracing::warn!("Background sync failed: {}", e);
                }
            }
        });

        Ok(instance)
    }

    /// Start the sync task
    pub async fn start_sync_task(&self) -> Result<()> {
        // Periodic synchronization with other nodes
        let mut interval = tokio::time::interval(self.config.sync_interval);

        loop {
            interval.tick().await;
            if let Err(e) = self.synchronize_all_nodes().await {
                tracing::warn!("Sync task error: {}", e);
            }
        }
    }

    /// Determine responsible nodes for an entry
    #[allow(dead_code)]
    async fn determine_responsible_nodes(&self, blinded_id: &str) -> Vec<StorageNode> {
        let nodes = self.nodes.read().await;

        // Implement deterministic node selection using consistent hashing
        let data_hash = blake3::hash(blinded_id.as_bytes());
        let mut sorted_nodes: Vec<_> = nodes.values().cloned().collect();

        // Sort nodes by hash distance from data hash for consistent assignment
        sorted_nodes.sort_by_key(|node| {
            let node_hash = blake3::hash(format!("{}{}", blinded_id, node.id).as_bytes());

            calculate_hash_distance(&data_hash, &node_hash)
        });

        // Return top N nodes based on replication factor
        sorted_nodes
            .into_iter()
            .take(self.config.replication_factor as usize)
            .collect()
    }

    /// Distribute an entry to responsible nodes
    #[allow(dead_code)]
    async fn distribute_entry(&self, entry: BlindedStateEntry) -> Result<()> {
        let responsible_nodes = self.determine_responsible_nodes(&entry.blinded_id).await;

        // Distribute to responsible nodes in parallel
        let tasks: Vec<_> = responsible_nodes
            .into_iter()
            .filter(|node| node.id != self.config.id)
            .map(|node| {
                let entry_clone = entry.clone();
                async move { self.replicate_to_node(&node, &entry_clone).await }
            })
            .collect();

        // Wait for all replications to complete
        let results = futures::future::join_all(tasks).await;

        // Check if at least one replication succeeded
        let successful = results.iter().any(|r| r.is_ok());
        if !successful {
            return Err(crate::error::StorageNodeError::Distribution(
                "Failed to replicate to any responsible nodes".to_string(),
            ));
        }

        Ok(())
    }

    /// Replicate an entry to a node
    #[allow(dead_code)]
    async fn replicate_to_node(&self, node: &StorageNode, entry: &BlindedStateEntry) -> Result<()> {
        // Create HTTP request to replicate data to peer node
        let client = reqwest::Client::new();
        let endpoint = format!("{}/data", node.endpoint);

        let response = client
            .post(&endpoint)
            .header("Content-Type", "application/octet-stream")
            .header("X-Blinded-ID", &entry.blinded_id)
            .header("X-TTL", entry.ttl.to_string())
            .header("X-Priority", entry.priority.to_string())
            .body(entry.encrypted_payload.clone())
            .send()
            .await
            .map_err(|e| crate::error::StorageNodeError::Network(e.to_string()))?;

        if response.status().is_success() {
            tracing::debug!(
                "Successfully replicated {} to {}",
                entry.blinded_id,
                node.id
            );
            Ok(())
        } else {
            Err(crate::error::StorageNodeError::Network(format!(
                "Replication failed with status: {}",
                response.status()
            )))
        }
    }

    /// Retrieve an entry from a node
    #[allow(dead_code)]
    async fn retrieve_from_node(
        &self,
        node: &StorageNode,
        blinded_id: &str,
    ) -> Result<Option<BlindedStateEntry>> {
        let client = reqwest::Client::new();
        let endpoint = format!("{}/data/{}", node.endpoint, blinded_id);

        let response = client
            .get(&endpoint)
            .send()
            .await
            .map_err(|e| crate::error::StorageNodeError::Network(e.to_string()))?;

        if response.status() == 404 {
            return Ok(None);
        }

        if !response.status().is_success() {
            return Err(crate::error::StorageNodeError::Network(format!(
                "Retrieval failed with status: {}",
                response.status()
            )));
        }

        let encrypted_payload = response
            .bytes()
            .await
            .map_err(|e| crate::error::StorageNodeError::Network(e.to_string()))?
            .to_vec();

        // Reconstruct BlindedStateEntry
        let entry = BlindedStateEntry {
            blinded_id: blinded_id.to_string(),
            encrypted_payload,
            ttl: 0, // Will be updated from headers if available
            region: "unknown".to_string(),
            priority: 0,
            proof_hash: [0; 32], // Will be calculated
            metadata: std::collections::HashMap::new(),
            timestamp: std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .unwrap_or_default()
                .as_secs(),
        };

        Ok(Some(entry))
    }

    /// Synchronize with another node
    #[allow(dead_code)]
    async fn synchronize_with_node(&self, node: &StorageNode) -> Result<()> {
        // Get local entries list
        let local_entries = self.local_storage.list(None, None).await?;

        // Get remote entries list
        let client = reqwest::Client::new();
        let endpoint = format!("{}/data", node.endpoint);

        let response = client
            .get(&endpoint)
            .send()
            .await
            .map_err(|e| crate::error::StorageNodeError::Network(e.to_string()))?;

        if !response.status().is_success() {
            return Err(crate::error::StorageNodeError::Network(format!(
                "Failed to get remote entries: {}",
                response.status()
            )));
        }

        let remote_entries: Vec<String> = response
            .json()
            .await
            .map_err(|e| crate::error::StorageNodeError::Network(e.to_string()))?;

        // Find entries missing locally
        let missing_locally: Vec<_> = remote_entries
            .iter()
            .filter(|id| !local_entries.contains(id))
            .cloned()
            .collect();

        let missing_count = missing_locally.len();

        // Retrieve missing entries
        for blinded_id in missing_locally {
            if let Ok(Some(entry)) = self.retrieve_from_node(node, &blinded_id).await {
                let _ = self.local_storage.store(entry).await;
            }
        }

        tracing::debug!(
            "Synchronized with node {}: retrieved {} entries",
            node.id,
            missing_count
        );
        Ok(())
    }

    /// Synchronize with all known nodes
    async fn synchronize_all_nodes(&self) -> Result<()> {
        let nodes = self.nodes.read().await;
        let node_list: Vec<_> = nodes.values().cloned().collect();
        drop(nodes); // Release the read lock

        for node in node_list {
            if node.id != self.config.id {
                if let Err(e) = self.synchronize_with_node(&node).await {
                    tracing::warn!("Failed to sync with node {}: {}", node.id, e);
                }
            }
        }

        Ok(())
    }

    /// Get remote node statistics
    async fn get_remote_stats(&self, node: &StorageNode) -> Result<StorageStats> {
        let client = reqwest::Client::new();
        let endpoint = format!("{}/stats", node.endpoint);

        let response = client
            .get(&endpoint)
            .send()
            .await
            .map_err(|e| crate::error::StorageNodeError::Network(e.to_string()))?;

        if !response.status().is_success() {
            return Err(crate::error::StorageNodeError::Network(format!(
                "Failed to get remote stats: {}",
                response.status()
            )));
        }

        let stats: StorageStats = response
            .json()
            .await
            .map_err(|e| crate::error::StorageNodeError::Network(e.to_string()))?;

        Ok(stats)
    }
}

#[async_trait]
impl super::StorageEngine for DistributedStorage {
    /// Store a blinded state entry
    async fn store(&self, entry: BlindedStateEntry) -> Result<StorageResponse> {
        let blinded_id = entry.blinded_id.clone();

        // Store locally first
        self.local_storage.store(entry.clone()).await?;

        // Add to distribution cache
        {
            let mut cache = self.distribution_cache.lock().unwrap();
            cache.insert(blinded_id.clone(), entry);
        }

        Ok(StorageResponse {
            blinded_id,
            timestamp: SystemTime::now()
                .duration_since(UNIX_EPOCH)
                .unwrap_or_else(|_| Duration::from_secs(0))
                .as_secs(),
            status: "success".to_string(),
            message: Some("Entry stored successfully".to_string()),
        })
    }

    /// Retrieve a blinded state entry by its ID
    async fn retrieve(&self, blinded_id: &str) -> Result<Option<BlindedStateEntry>> {
        // Try local storage first
        if let Some(entry) = self.local_storage.retrieve(blinded_id).await? {
            return Ok(Some(entry));
        }

        // Try to retrieve from other nodes if not found locally
        let responsible_nodes = self.determine_responsible_nodes(blinded_id).await;
        for node in responsible_nodes {
            if node.id != self.config.id {
                if let Ok(Some(entry)) = self.retrieve_from_node(&node, blinded_id).await {
                    // Store locally for future access
                    let _ = self.local_storage.store(entry.clone()).await;
                    return Ok(Some(entry));
                }
            }
        }

        Ok(None)
    }

    /// Delete a blinded state entry by its ID
    async fn delete(&self, blinded_id: &str) -> Result<bool> {
        // Delete locally
        let local_result = self.local_storage.delete(blinded_id).await?;

        // Propagate deletion to other nodes
        let responsible_nodes = self.determine_responsible_nodes(blinded_id).await;
        for node in responsible_nodes {
            if node.id != self.config.id {
                let client = reqwest::Client::new();
                let endpoint = format!("{}/data/{}", node.endpoint, blinded_id);
                let _ = client.delete(&endpoint).send().await;
            }
        }

        Ok(local_result)
    }

    /// Check if a blinded state entry exists
    async fn exists(&self, blinded_id: &str) -> Result<bool> {
        // Check local storage first
        if self.local_storage.exists(blinded_id).await? {
            return Ok(true);
        }

        // Check other nodes if not found locally
        let responsible_nodes = self.determine_responsible_nodes(blinded_id).await;
        for node in responsible_nodes {
            if node.id != self.config.id {
                if let Ok(Some(_)) = self.retrieve_from_node(&node, blinded_id).await {
                    return Ok(true);
                }
            }
        }

        Ok(false)
    }

    /// List blinded state entry IDs with optional pagination
    async fn list(&self, limit: Option<usize>, offset: Option<usize>) -> Result<Vec<String>> {
        // List from local storage only
        self.local_storage.list(limit, offset).await
    }

    /// Get storage statistics
    async fn get_stats(&self) -> Result<StorageStats> {
        // Get local storage stats
        let stats = self.local_storage.get_stats().await?;

        // Aggregate stats from other nodes
        let mut total_stats = stats;
        let nodes = self.nodes.read().await;

        for node in nodes.values() {
            if node.id != self.config.id {
                if let Ok(remote_stats) = self.get_remote_stats(node).await {
                    total_stats.total_entries += remote_stats.total_entries;
                    total_stats.total_bytes += remote_stats.total_bytes;
                    if let Some(oldest) = remote_stats.oldest_entry {
                        total_stats.oldest_entry = Some(
                            total_stats
                                .oldest_entry
                                .map_or(oldest, |local| local.min(oldest)),
                        );
                    }
                    if let Some(newest) = remote_stats.newest_entry {
                        total_stats.newest_entry = Some(
                            total_stats
                                .newest_entry
                                .map_or(newest, |local| local.max(newest)),
                        );
                    }
                }
            }
        }

        Ok(total_stats)
    }

    fn as_any(&self) -> &dyn std::any::Any {
        self
    }
}
