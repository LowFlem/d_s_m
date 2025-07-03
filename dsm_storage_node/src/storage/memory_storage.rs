use crate::error::{Result, StorageNodeError};
use crate::types::storage_types::{StorageResponse, StorageStats};
use crate::types::BlindedStateEntry;
use async_trait::async_trait;
use dashmap::DashMap;
use std::fs;
use std::path::PathBuf;
use std::sync::atomic::{AtomicU64, Ordering};
use std::sync::Arc;
use std::time::{SystemTime, UNIX_EPOCH};
use tokio::sync::RwLock;
use tracing::{debug, warn};

const DEFAULT_MAX_MEMORY_BYTES: usize = 1024 * 1024 * 1024; // 1GB
const DEFAULT_MAX_ENTRIES: usize = 1_000_000;
const PERSISTENCE_INTERVAL_SECS: u64 = 300; // 5 minutes

#[derive(Clone, Debug)]
pub struct MemoryStorageConfig {
    pub max_memory_bytes: usize,
    pub max_entries: usize,
    pub persistence_path: Option<PathBuf>,
    pub eviction_policy: EvictionPolicy,
    #[allow(dead_code)]
    pub(crate) db_path: String,
    #[allow(dead_code)]
    pub(crate) compression: Option<String>,
}

#[derive(Clone, Debug)]
pub enum EvictionPolicy {
    LRU,
    LFU,
    FIFO,
}

pub struct MemoryStorage {
    entries: Arc<DashMap<String, BlindedStateEntry>>,
    total_bytes: Arc<AtomicU64>,
    config: MemoryStorageConfig,
    persistence_path: Option<PathBuf>,
    access_counts: Arc<DashMap<String, u64>>,
    last_accessed: Arc<DashMap<String, u64>>,
    insertion_order: Arc<RwLock<Vec<String>>>,
}

impl Clone for MemoryStorage {
    fn clone(&self) -> Self {
        Self {
            entries: self.entries.clone(),
            total_bytes: self.total_bytes.clone(),
            config: self.config.clone(),
            persistence_path: self.persistence_path.clone(),
            access_counts: self.access_counts.clone(),
            last_accessed: self.last_accessed.clone(),
            insertion_order: self.insertion_order.clone(),
        }
    }
}

impl Default for MemoryStorageConfig {
    fn default() -> Self {
        Self {
            max_memory_bytes: DEFAULT_MAX_MEMORY_BYTES,
            max_entries: DEFAULT_MAX_ENTRIES,
            persistence_path: None,
            eviction_policy: EvictionPolicy::LRU,
            db_path: "default_db_path".to_string(),
            compression: Some("lz4".to_string()),
        }
    }
}

impl MemoryStorage {
    pub fn new(config: MemoryStorageConfig) -> Self {
        let instance = Self {
            entries: Arc::new(DashMap::new()),
            total_bytes: Arc::new(AtomicU64::new(0)),
            config: config.clone(),
            persistence_path: config.persistence_path.clone(),
            access_counts: Arc::new(DashMap::new()),
            last_accessed: Arc::new(DashMap::new()),
            insertion_order: Arc::new(RwLock::new(Vec::new())),
        };

        // Load persisted data if available
        if let Some(path) = &config.persistence_path {
            // Use a separate block to handle synchronous file operations
            let deserialize_result = (|| {
                let data = fs::read(path)?;
                bincode::deserialize::<Vec<(String, BlindedStateEntry)>>(&data)
                    .map_err(|e| std::io::Error::new(std::io::ErrorKind::InvalidData, e))
            })();

            if let Ok(entries) = deserialize_result {
                let mut insertion_order = Vec::new();
                for (id, entry) in entries {
                    instance.entries.insert(id.clone(), entry);
                    insertion_order.push(id);
                }
                // Update insertion order after collecting all IDs
                if let Ok(mut order) = instance.insertion_order.try_write() {
                    order.extend(insertion_order);
                } else {
                    warn!("Failed to acquire write lock for insertion order");
                }
            }
        }

        // Start persistence task if path configured
        if config.persistence_path.is_some() {
            let storage = instance.clone();
            tokio::spawn(async move {
                let mut interval = tokio::time::interval(tokio::time::Duration::from_secs(
                    PERSISTENCE_INTERVAL_SECS,
                ));
                loop {
                    interval.tick().await;
                    if let Err(e) = storage.persist().await {
                        warn!("Failed to persist storage: {}", e);
                    }
                }
            });
        }

        instance
    }

    async fn persist(&self) -> Result<()> {
        if let Some(path) = &self.persistence_path {
            let entries: Vec<_> = self
                .entries
                .iter()
                .map(|e| (e.key().clone(), e.value().clone()))
                .collect();
            let data = bincode::serialize(&entries)
                .map_err(|e| StorageNodeError::Serialization(e.to_string()))?;

            // Use tokio's async file I/O instead of std's blocking I/O
            tokio::fs::write(path, data).await.map_err(|e| {
                StorageNodeError::Storage(format!("Failed to persist storage: {e}"))
            })?;
        }
        Ok(())
    }

    async fn evict(&self) -> Result<()> {
        let current_size = self.total_bytes.load(Ordering::Relaxed) as usize;
        if current_size <= self.config.max_memory_bytes
            && self.entries.len() <= self.config.max_entries
        {
            return Ok(());
        }

        let to_evict = match self.config.eviction_policy {
            EvictionPolicy::LRU => {
                let now = SystemTime::now()
                    .duration_since(UNIX_EPOCH)
                    .unwrap()
                    .as_secs();
                let mut candidates: Vec<_> = self
                    .last_accessed
                    .iter()
                    .map(|entry| (entry.key().clone(), now - *entry.value()))
                    .collect();
                candidates.sort_by_key(|(_id, age)| *age);
                candidates
                    .into_iter()
                    .map(|(id, _)| id)
                    .take(100)
                    .collect::<Vec<_>>()
            }
            EvictionPolicy::LFU => {
                let mut candidates: Vec<_> = self
                    .access_counts
                    .iter()
                    .map(|entry| (entry.key().clone(), *entry.value()))
                    .collect();
                candidates.sort_by_key(|(_id, count)| *count);
                candidates.into_iter().map(|(id, _)| id).take(100).collect()
            }
            EvictionPolicy::FIFO => {
                let insertion_order = self.insertion_order.read().await;
                insertion_order.iter().take(100).cloned().collect()
            }
        };

        for id in to_evict {
            if let Some(entry) = self.entries.remove(&id) {
                let entry_size = bincode::serialize(&entry)
                    .map_err(|e| StorageNodeError::Serialization(e.to_string()))?
                    .len() as u64;
                self.total_bytes.fetch_sub(entry_size, Ordering::Relaxed);
                self.access_counts.remove(&id);
                self.last_accessed.remove(&id);
            }
        }

        Ok(())
    }

    fn update_access_metrics(&self, id: &str) {
        let now = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap()
            .as_secs();
        self.last_accessed.insert(id.to_string(), now);
        self.access_counts
            .entry(id.to_string())
            .and_modify(|count| *count += 1)
            .or_insert(1);
    }
}

#[async_trait]
impl super::StorageEngine for MemoryStorage {
    /// Store a policy in memory storage
    async fn store_policy(&self, entry: &crate::policy::PolicyStorageEntry) -> Result<bool> {
        debug!("Storing policy with ID: {}", entry.id);

        // Create a blinded state entry to store the policy
        let policy_key = format!("policy:{}", entry.id);
        let policy_data = serde_json::to_vec(entry)
            .map_err(|e| StorageNodeError::Serialization(e.to_string()))?;

        let blinded_entry = BlindedStateEntry {
            blinded_id: policy_key,
            encrypted_payload: policy_data,
            timestamp: entry.timestamp,
            ttl: 0, // Policies don't expire
            region: "default".to_string(),
            priority: 1,
            proof_hash: {
                let mut hasher = blake3::Hasher::new();
                hasher.update(&entry.data);
                let hash = hasher.finalize();
                let mut hash_bytes = [0u8; 32];
                hash_bytes.copy_from_slice(hash.as_bytes());
                hash_bytes
            },
            metadata: entry.metadata.clone(),
        };

        // Store using the regular store method
        self.store(blinded_entry).await?;

        debug!("Policy {} stored successfully in memory", entry.id);
        Ok(true)
    }

    /// Retrieve a policy from memory storage
    async fn get_policy(
        &self,
        policy_id: &str,
    ) -> Result<Option<crate::policy::PolicyStorageEntry>> {
        debug!("Retrieving policy with ID: {}", policy_id);

        let policy_key = format!("policy:{policy_id}");

        if let Some(blinded_entry) = self.retrieve(&policy_key).await? {
            let policy_entry = serde_json::from_slice::<crate::policy::PolicyStorageEntry>(
                &blinded_entry.encrypted_payload,
            )
            .map_err(|e| {
                StorageNodeError::Serialization(format!("Failed to deserialize policy: {e}"))
            })?;

            debug!("Policy {} retrieved successfully from memory", policy_id);
            Ok(Some(policy_entry))
        } else {
            debug!("Policy {} not found in memory", policy_id);
            Ok(None)
        }
    }

    /// List all policies in memory storage
    async fn list_policies(&self) -> Result<Vec<crate::policy::PolicyStorageEntry>> {
        debug!("Listing all policies from memory");

        let mut policies = Vec::new();

        // Iterate through all entries and find policy entries
        for entry in self.entries.iter() {
            if entry.key().starts_with("policy:") {
                if let Some(blinded_entry) = self.retrieve(entry.key()).await? {
                    match serde_json::from_slice::<crate::policy::PolicyStorageEntry>(
                        &blinded_entry.encrypted_payload,
                    ) {
                        Ok(policy_entry) => {
                            policies.push(policy_entry);
                        }
                        Err(e) => {
                            warn!("Failed to deserialize policy {}: {}", entry.key(), e);
                        }
                    }
                }
            }
        }

        // Sort by timestamp (newest first)
        policies.sort_by(|a, b| b.timestamp.cmp(&a.timestamp));

        debug!("Listed {} policies from memory", policies.len());
        Ok(policies)
    }

    /// Remove a policy from memory storage
    async fn remove_policy(&self, policy_id: &str) -> Result<bool> {
        debug!("Removing policy with ID: {}", policy_id);

        let policy_key = format!("policy:{policy_id}");
        let deleted = self.delete(&policy_key).await?;

        debug!("Policy {} deleted from memory: {}", policy_id, deleted);
        Ok(deleted)
    }
    async fn store(&self, entry: BlindedStateEntry) -> Result<StorageResponse> {
        let entry_size = bincode::serialize(&entry)
            .map_err(|e| StorageNodeError::Serialization(e.to_string()))?
            .len() as u64;

        // Check size limits and evict if needed
        self.evict().await?;

        let blinded_id = entry.blinded_id.clone();
        self.entries.insert(blinded_id.clone(), entry);
        self.total_bytes.fetch_add(entry_size, Ordering::Relaxed);
        self.insertion_order.write().await.push(blinded_id.clone());

        Ok(StorageResponse {
            blinded_id,
            timestamp: SystemTime::now()
                .duration_since(UNIX_EPOCH)
                .unwrap_or_else(|_| std::time::Duration::from_secs(0))
                .as_secs(),
            status: "success".to_string(),
            message: Some("Entry stored successfully".to_string()),
        })
    }

    async fn retrieve(&self, blinded_id: &str) -> Result<Option<BlindedStateEntry>> {
        if let Some(entry) = self.entries.get(blinded_id) {
            self.update_access_metrics(blinded_id);
            Ok(Some(entry.clone()))
        } else {
            Ok(None)
        }
    }

    async fn delete(&self, blinded_id: &str) -> Result<bool> {
        let existed = self.entries.remove(blinded_id).is_some();
        if existed {
            self.access_counts.remove(blinded_id);
            self.last_accessed.remove(blinded_id);
        }
        Ok(existed)
    }

    async fn exists(&self, blinded_id: &str) -> Result<bool> {
        Ok(self.entries.contains_key(blinded_id))
    }

    async fn list(&self, limit: Option<usize>, offset: Option<usize>) -> Result<Vec<String>> {
        let offset = offset.unwrap_or(0);
        let limit = limit.unwrap_or(100);

        let mut entries: Vec<String> = self
            .entries
            .iter()
            .map(|entry| entry.key().clone())
            .skip(offset)
            .take(limit)
            .collect();

        entries.sort();
        Ok(entries)
    }

    async fn get_stats(&self) -> Result<StorageStats> {
        Ok(StorageStats {
            total_entries: self.entries.len(),
            total_bytes: self.total_bytes.load(Ordering::Relaxed) as usize,
            total_expired: 0,   // Memory storage doesn't track expiration
            oldest_entry: None, // Memory storage doesn't track entry age
            newest_entry: None,
            average_entry_size: if !self.entries.is_empty() {
                self.total_bytes.load(Ordering::Relaxed) as usize / self.entries.len()
            } else {
                0
            },
            total_regions: 1, // Memory storage is a single region
            last_updated: SystemTime::now()
                .duration_since(UNIX_EPOCH)
                .unwrap_or_else(|_| std::time::Duration::from_secs(0))
                .as_secs(),
        })
    }
    fn as_any(&self) -> &dyn std::any::Any {
        self
    }
}

/// Detailed metrics for memory storage
#[derive(Debug, Clone)]
pub struct MemoryStorageMetrics {
    pub total_entries: usize,
    pub total_bytes: u64,
    pub total_operations: u64,
    pub failed_operations: u64,
    pub eviction_count: u64,
    pub cache_hits: u64,
    pub cache_misses: u64,
    pub hit_rate: f64,
    pub avg_entry_age_seconds: u64,
    pub memory_utilization: f64,
    pub entry_utilization: f64,
}

/// Health status for memory storage
#[derive(Debug, Clone)]
pub struct MemoryStorageHealth {
    pub status: String,
    pub issues: Vec<String>,
    pub warnings: Vec<String>,
    pub metrics: MemoryStorageMetrics,
}
