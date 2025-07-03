//! # Storage Module for DSM Storage Node
//!
//! This module provides the core storage functionality for the DSM Storage Node,
//! implementing various storage backends and distributed storage strategies.
//!
//! ## Key Features
//!
//! * Multiple storage backend implementations (Memory, SQL, Epidemic)
//! * Distributed storage with customizable replication strategies
//! * Data partitioning and routing across multiple nodes
//! * Epidemic protocols for eventual consistency
//! * Vector clocks for conflict resolution
//! * Metrics collection and health monitoring
//!
//! ## Architecture
//!
//! The storage module is built around the `StorageEngine` trait, which defines
//! the core interface for all storage backends. Various implementations provide
//! different trade-offs in terms of persistence, performance, and distribution:
//!
//! * `MemoryStorage`: In-memory storage with optional persistence
//! * `SqlStorage`: SQLite-based persistent storage
//! * `DistributedStorage`: Distributes data across multiple nodes
//! * `EpidemicStorageEngine`: Eventually consistent distributed storage
//!
//! ## Usage
//!
//! Storage engines are typically created through the `StorageFactory`, which
//! handles configuration and initialization details:
//!
//! ```rust,no_run
//! use dsm_storage_node::storage::{StorageConfig, StorageFactory};
//!
//! // Create configuration
//! let config = StorageConfig {
//!     database_path: "data/storage.db".to_string(),
//!     default_ttl: 0, // No expiration
//!     enable_pruning: true,
//!     pruning_interval: 3600,
//! };
//!
//! // Create factory and storage engine
//! let factory = StorageFactory::new(config);
//! let storage = factory.create_sql_storage().unwrap();
//!
//! // Use the storage engine
//! // storage.store(...).await?;
//! ```

use crate::error::Result;
use crate::policy::PolicyStorageEntry;
use crate::storage::topology::NodeId;
use crate::types::storage_types::StorageStats;
use crate::types::BlindedStateEntry;
use async_trait::async_trait;
use std::path::PathBuf;
use std::sync::Arc;
use tracing::{error, info};

// Module declarations
/// Deterministic storage assignment and consistency enforcement
pub mod deterministic_assignment;
/// Data digest generation for content addressing
pub mod digest;
/// Distributed storage across multiple nodes
pub mod distributed_storage;
/// Epidemic protocol-based eventually consistent storage
pub mod epidemic_storage;
/// Health checking and monitoring
pub mod health;
/// In-memory storage implementation
pub mod memory_storage;
/// Storage metrics collection
pub mod metrics;
/// Data partitioning strategies
pub mod partition;
/// Data reconciliation between nodes
pub mod reconciliation;
/// Request routing algorithms
pub mod routing;
/// SQLite-based persistent storage
pub mod sql_storage;
/// Background maintenance tasks
pub mod tasks;
/// Network topology management
pub mod topology;
/// Vector clocks for causality tracking
pub mod vector_clock;

// Re-exports for convenience
pub use digest::DigestGenerator;
pub use distributed_storage::DistributedStorage;
pub use epidemic_storage::{EpidemicConfig, EpidemicStorageEngine};
pub use memory_storage::{EvictionPolicy, MemoryStorage, MemoryStorageConfig};
pub use sql_storage::SqlStorage;

/// Core interface for all storage engines in the DSM Storage Node.
///
/// This trait defines the essential operations that any storage implementation
/// must provide, regardless of whether it's a local or distributed storage.
/// All methods are asynchronous to allow for efficient I/O operations and
/// network communication.
///
/// # Examples
///
/// ```rust,no_run
/// use dsm_storage_node::storage::{StorageEngine, MemoryStorage, MemoryStorageConfig};
/// use dsm_storage_node::types::BlindedStateEntry;
/// use std::sync::Arc;
///
/// # async fn example() -> Result<(), Box<dyn std::error::Error>> {
/// // Create a storage engine
/// let config = MemoryStorageConfig::default();
/// let storage: Arc<dyn StorageEngine + Send + Sync> = Arc::new(MemoryStorage::new(config));
///
/// // Store a blinded state entry
/// let entry = BlindedStateEntry {
///     blinded_id: "test_id".to_string(),
///     encrypted_payload: vec![1,2,3],
///     timestamp: chrono::Utc::now().timestamp() as u64,
///     ttl: 3600,
///     region: "test".to_string(),
///     priority: 0,
///     proof_hash: [0;32],
///     metadata: std::collections::HashMap::new(),
/// };
/// storage.store(entry.clone()).await?;
///
/// // Retrieve the entry
/// let retrieved = storage.retrieve(&entry.blinded_id).await?;
/// # Ok(())
/// # }
/// ```
#[async_trait]
pub trait StorageEngine: Send + Sync {
    async fn store(
        &self,
        entry: BlindedStateEntry,
    ) -> Result<crate::types::storage_types::StorageResponse>;

    async fn retrieve(&self, blinded_id: &str) -> Result<Option<BlindedStateEntry>>;

    async fn delete(&self, blinded_id: &str) -> Result<bool>;

    async fn exists(&self, blinded_id: &str) -> Result<bool>;

    async fn list(&self, limit: Option<usize>, offset: Option<usize>) -> Result<Vec<String>>;

    /// Store a policy in the storage engine
    async fn store_policy(&self, _entry: &PolicyStorageEntry) -> Result<bool> {
        // Default implementation returns error
        Err(crate::error::StorageNodeError::NotImplemented(
            "Policy storage not implemented for this engine".to_string(),
        ))
    }

    /// Retrieve a policy from the storage engine
    async fn get_policy(&self, _policy_id: &str) -> Result<Option<PolicyStorageEntry>> {
        // Default implementation returns None
        Ok(None)
    }

    /// List all policies in the storage engine
    async fn list_policies(&self) -> Result<Vec<PolicyStorageEntry>> {
        // Default implementation returns empty list
        Ok(Vec::new())
    }

    /// Remove a policy from the storage engine
    async fn remove_policy(&self, _policy_id: &str) -> Result<bool> {
        // Default implementation returns error
        Err(crate::error::StorageNodeError::NotImplemented(
            "Policy removal not implemented for this engine".to_string(),
        ))
    }

    async fn get_stats(&self) -> Result<StorageStats>;

    /// Get known cluster nodes (for distributed storage engines)
    fn get_cluster_nodes(&self) -> Vec<crate::types::StorageNode> {
        // Default implementation returns empty list for non-distributed engines
        Vec::new()
    }

    /// Support for downcasting to concrete types
    fn as_any(&self) -> &dyn std::any::Any;
}

/// Configuration for storage engines.
///
/// This struct provides common configuration parameters used by
/// various storage engine implementations.
#[derive(Clone, Debug)]
pub struct StorageConfig {
    /// Path to the database file (for SQL storage)
    pub database_path: String,

    /// Default time-to-live for entries in seconds
    /// A value of 0 means no expiration (entries live forever)
    pub default_ttl: u64,

    /// Whether to enable automatic pruning of expired entries
    pub enable_pruning: bool,

    /// Interval for pruning expired entries in seconds
    pub pruning_interval: u64,
}

/// Factory for creating different storage engine implementations.
///
/// This factory simplifies the creation of storage engines by handling
/// the complexity of configuration and initialization. It supports creating
/// various types of storage engines with appropriate default settings.
pub struct StorageFactory {
    /// Storage configuration
    config: StorageConfig,
}

impl StorageFactory {
    /// Create a new storage factory with the given configuration.
    pub fn new(config: StorageConfig) -> Self {
        Self { config }
    }

    pub fn create_memory_storage(&self) -> Result<Arc<dyn StorageEngine + Send + Sync>> {
        let config = MemoryStorageConfig {
            max_memory_bytes: 1024 * 1024 * 1024,
            max_entries: 1_000_000,
            persistence_path: Some(PathBuf::from(format!(
                "{}.memdb",
                self.config.database_path
            ))),
            eviction_policy: EvictionPolicy::LRU,
            db_path: self.config.database_path.clone(),
            compression: Some("lz4".to_string()),
        };
        Ok(Arc::new(MemoryStorage::new(config)))
    }

    pub fn create_sql_storage(&self) -> Result<Arc<dyn StorageEngine + Send + Sync>> {
        let storage = SqlStorage::new(&self.config.database_path)?;
        Ok(Arc::new(storage))
    }

    pub fn create_distributed_storage(
        &self,
        local_storage: Arc<dyn StorageEngine + Send + Sync>,
        node_id: String,
        peers: Vec<crate::types::StorageNode>,
        replication_factor: usize,
        max_hops: usize,
    ) -> Result<Arc<dyn StorageEngine + Send + Sync>> {
        let storage = distributed_storage::DistributedStorage::new(
            local_storage.clone(),
            node_id.clone(),
            peers.clone(),
            replication_factor,
            max_hops,
        )?;
        Ok(Arc::new(storage))
    }

    pub async fn create_epidemic_storage(
        &self,
        node_id: String,
        node: crate::types::StorageNode,
        bootstrap_nodes: Vec<crate::types::StorageNode>,
        _cluster_manager: Option<Arc<crate::cluster::ClusterManager>>,
    ) -> Result<Arc<dyn StorageEngine + Send + Sync>> {
        let config = epidemic_storage::EpidemicConfig {
            node_id: NodeId::from_string(&node_id).unwrap(),
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
            max_memory_bytes: 1024 * 1024 * 1024, // 1GB
            max_entries: 1000000,
            enable_eviction: true,
            eviction_check_interval_ms: 60000,
            default_ttl_seconds: 3600,
            cleanup_interval_ms: 60000,
        };

        let network_client = crate::distribution::NetworkClientType::Http(Arc::new(
            crate::network::HttpNetworkClient::new(
                node_id.clone(),
                crate::network::NetworkClientConfig::default(),
            )?,
        ));
        let metrics = Arc::new(crate::storage::metrics::MetricsCollector::new(
            crate::storage::metrics::MetricsCollectorConfig::default(),
        ));

        let storage = epidemic_storage::EpidemicStorageEngine::new(
            config,
            network_client,
            metrics,
            bootstrap_nodes,
            node,
        );

        // Wait a moment for cluster initialization before starting epidemic protocol
        info!("Waiting for cluster initialization to complete...");
        tokio::time::sleep(std::time::Duration::from_secs(2)).await;

        // Start the epidemic protocol after cluster is ready
        info!("Starting epidemic protocol...");
        if let Err(e) = storage.start_epidemic_protocol().await {
            error!("Failed to start epidemic protocol: {}", e);
            // Continue anyway - the storage engine can still function
        } else {
            info!("Epidemic protocol started successfully");
        }

        Ok(Arc::new(storage))
    }
}
/// High-level storage provider that manages multiple storage engines.
pub struct StorageProvider {
    primary: Arc<dyn StorageEngine + Send + Sync>,
    backup: Option<Arc<dyn StorageEngine + Send + Sync>>,
    node: crate::types::StorageNode,
    default_ttl: u64,
    default_region: String,
}

impl StorageProvider {
    pub fn new(
        primary: Arc<dyn StorageEngine + Send + Sync>,
        backup: Option<Arc<dyn StorageEngine + Send + Sync>>,
        node: crate::types::StorageNode,
        default_ttl: u64,
        default_region: String,
    ) -> Self {
        Self {
            primary,
            backup,
            node,
            default_ttl,
            default_region,
        }
    }

    pub fn get_primary(&self) -> Arc<dyn StorageEngine + Send + Sync> {
        self.primary.clone()
    }

    pub fn get_backup(&self) -> Option<Arc<dyn StorageEngine + Send + Sync>> {
        self.backup.clone()
    }

    pub fn get_node(&self) -> &crate::types::StorageNode {
        &self.node
    }

    pub fn get_default_ttl(&self) -> u64 {
        self.default_ttl
    }

    pub fn get_default_region(&self) -> &str {
        &self.default_region
    }
}
