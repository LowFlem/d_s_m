// Types module for DSM Storage Node
//
// This module defines common types used throughout the DSM Storage Node.

use constant_time_eq;
use serde::{Deserialize, Serialize};
use serde_json::Value;
use std::collections::HashMap;
use std::sync::Arc;
use std::time::{SystemTime, UNIX_EPOCH};
use tokio::sync::RwLock;

pub mod policy_types;
pub mod state_types;

pub use policy_types::*;
pub use state_types::*;

pub mod storage_types;

/// Blinded state entry
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BlindedStateEntry {
    /// Blinded ID (deterministic hash of the entry content)
    pub blinded_id: String,

    /// Encrypted payload
    pub encrypted_payload: Vec<u8>,

    /// Timestamp (seconds since epoch)
    pub timestamp: u64,

    /// Time-to-live in seconds (0 = no expiration)
    pub ttl: u64,

    /// Geographic region
    pub region: String,

    /// Priority (higher = more important)
    pub priority: i32,

    /// Proof hash (cryptographic hash for verification)
    pub proof_hash: [u8; 32],

    /// Metadata (key-value pairs)
    pub metadata: HashMap<String, String>,
}

impl BlindedStateEntry {
    /// Validate the blinded state entry for correctness and security
    pub fn validate(&self) -> Result<(), String> {
        // Validate blinded_id
        if self.blinded_id.is_empty() {
            return Err("Blinded ID cannot be empty".to_string());
        }
        if self.blinded_id.len() > 512 {
            return Err("Blinded ID too long (max 512 characters)".to_string());
        }

        // Validate encrypted_payload
        if self.encrypted_payload.len() > 100_000_000 {
            // 100MB limit
            return Err("Encrypted payload too large (max 100MB)".to_string());
        }

        // Validate region
        if self.region.is_empty() {
            return Err("Region cannot be empty".to_string());
        }
        if self.region.len() > 64 {
            return Err("Region too long (max 64 characters)".to_string());
        }

        // Validate timestamp
        let current_time = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap_or_default()
            .as_secs();

        if self.timestamp > current_time + 300 {
            // Allow 5 minutes future time
            return Err("Timestamp too far in the future".to_string());
        }

        // Validate priority
        if self.priority < -1000 || self.priority > 1000 {
            return Err("Priority out of valid range (-1000 to 1000)".to_string());
        }

        // Validate metadata
        if self.metadata.len() > 100 {
            return Err("Too many metadata entries (max 100)".to_string());
        }

        for (key, value) in &self.metadata {
            if key.is_empty() {
                return Err("Metadata key cannot be empty".to_string());
            }
            if key.len() > 256 {
                return Err("Metadata key too long (max 256 characters)".to_string());
            }
            if value.len() > 1024 {
                return Err("Metadata value too long (max 1024 characters)".to_string());
            }
        }

        Ok(())
    }

    /// Check if the entry has expired based on TTL
    pub fn is_expired(&self) -> bool {
        if self.ttl == 0 {
            return false; // No expiration
        }

        let current_time = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap_or_default()
            .as_secs();

        current_time > self.timestamp + self.ttl
    }

    /// Get the estimated size of this entry in bytes
    pub fn estimated_size(&self) -> usize {
        // Calculate approximate memory usage
        let mut size = 0;
        size += self.blinded_id.len();
        size += self.encrypted_payload.len();
        size += 8; // timestamp
        size += 8; // ttl
        size += self.region.len();
        size += 4; // priority
        size += 32; // proof_hash

        // Metadata overhead
        for (key, value) in &self.metadata {
            size += key.len() + value.len() + 48; // String overhead
        }

        size + 64 // struct overhead
    }

    /// Generate a cryptographic hash of the entry content
    pub fn generate_content_hash(&self) -> [u8; 32] {
        let mut hasher = blake3::Hasher::new();
        hasher.update(self.blinded_id.as_bytes());
        hasher.update(&self.encrypted_payload);
        hasher.update(&self.timestamp.to_le_bytes());
        hasher.update(&self.ttl.to_le_bytes());
        hasher.update(self.region.as_bytes());
        hasher.update(&self.priority.to_le_bytes());

        // Add metadata in sorted order for deterministic hashing
        let mut sorted_metadata: Vec<_> = self.metadata.iter().collect();
        sorted_metadata.sort_by_key(|(k, _)| *k);
        for (key, value) in sorted_metadata {
            hasher.update(key.as_bytes());
            hasher.update(value.as_bytes());
        }

        let hash = hasher.finalize();
        let mut result = [0u8; 32];
        result.copy_from_slice(hash.as_bytes());
        result
    }

    /// Verify the proof hash matches the entry content
    pub fn verify_proof_hash(&self) -> bool {
        let computed_hash = self.generate_content_hash();
        // Use constant-time comparison to prevent timing attacks
        use constant_time_eq::constant_time_eq;
        constant_time_eq(&computed_hash, &self.proof_hash)
    }
}

/// Storage node information
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct StorageNode {
    /// Node ID
    pub id: String,

    /// Node name
    pub name: String,

    /// Node region
    pub region: String,

    /// Node public key
    pub public_key: String,

    /// Node endpoint
    pub endpoint: String,
}

/// Distribution node information
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DistributionNode {
    /// Node ID
    pub id: String,

    /// Node endpoint
    pub endpoint: String,

    /// Node public key
    pub public_key: String,

    /// Node region
    pub region: String,

    /// Connection status
    pub status: NodeStatus,

    /// Last seen timestamp
    pub last_seen: u64,

    /// Node capabilities
    pub capabilities: Vec<NodeCapability>,

    /// Stake amount
    pub stake: Option<u64>,
}

/// Node status
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, Hash)]
pub enum NodeStatus {
    /// Node is online and responding
    Online,

    /// Node is offline or not responding
    Offline,

    /// Node status is unknown
    Unknown,

    /// Node is pending verification
    Pending,

    /// Node is suspended
    Suspended,
}

/// Node capability
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum NodeCapability {
    /// Storage capability
    Storage,

    /// Distribution capability
    Distribution,

    /// Verification capability
    Verification,

    /// Staking capability
    Staking,

    /// Genesis capability
    Genesis,

    /// Checkpoint capability
    Checkpoint,

    /// Custom capability
    Custom(String),
}

/// Entry selector (for querying and filtering)
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EntrySelector {
    /// Blinded IDs to select
    pub blinded_ids: Option<Vec<String>>,

    /// Region to select from
    pub region: Option<String>,

    /// Minimum priority
    pub min_priority: Option<i32>,

    /// Maximum priority
    pub max_priority: Option<i32>,

    /// Minimum timestamp
    pub min_timestamp: Option<u64>,

    /// Maximum timestamp
    pub max_timestamp: Option<u64>,

    /// Include expired entries
    pub include_expired: bool,

    /// Metadata filters (key-value pairs that must match)
    pub metadata_filters: Option<HashMap<String, String>>,

    /// Limit results
    pub limit: Option<usize>,

    /// Offset results
    pub offset: Option<usize>,
}

/// API configuration settings from the config file
///
/// Controls the HTTP API behavior including binding address,
/// security features, and request limits.
#[derive(Debug, Deserialize, Clone)]
#[allow(dead_code)]
pub struct ApiConfig {
    /// IP address to bind the API server to
    pub bind_address: String,

    /// Port number for the API server
    pub port: u16,

    /// Whether to enable Cross-Origin Resource Sharing
    pub enable_cors: bool,

    /// Whether to enable rate limiting for API requests
    pub enable_rate_limits: bool,

    /// Maximum size of request bodies in bytes
    pub max_body_size: usize,
}

/// Node identity and metadata configuration
///
/// Defines the node's identity in the DSM network and
/// provides metadata about the node operator.
#[derive(Debug, Deserialize, Clone)]
#[allow(dead_code)]
pub struct NodeConfig {
    /// Unique identifier for this node in the network
    pub id: String,

    /// Human-readable name for the node
    pub name: String,

    /// Geographic region where the node is located
    pub region: String,

    /// Entity operating this node
    pub operator: String,

    /// Version string for this node
    pub version: String,

    /// Human-readable description of the node
    pub description: String,

    /// Public key for node identity verification
    pub public_key: String,

    /// Public endpoint where this node can be reached
    pub endpoint: String,
}

/// Storage engine configuration
///
/// Controls how data is stored, distributed, and managed
/// by this storage node.
#[derive(Debug, Deserialize, Clone)]
#[allow(dead_code)]
pub struct StorageConfig {
    /// Storage engine type ("sqlite", "memory", "epidemic")
    pub engine: String,

    /// Maximum storage capacity in bytes
    pub capacity: u64,

    /// Directory to store data files
    pub data_dir: String,

    /// Path to the database file (for sqlite engine)
    pub database_path: String,

    /// Strategy for assigning data to nodes
    /// Options: "DeterministicHashing", "RoundRobin", "LoadBalanced"
    pub assignment_strategy: String,

    /// Strategy for data replication across nodes
    /// Options: "FixedReplicas", "DynamicReplicas", "RegionAware"
    pub replication_strategy: String,

    /// Number of replicas to maintain for each data item
    pub replica_count: u8,

    /// Minimum number of different regions for replicas
    pub min_regions: u8,

    /// Default time-to-live for data in seconds (0 = no expiration)
    pub default_ttl: u64,

    /// Whether to enable automatic pruning of expired data
    pub enable_pruning: bool,

    /// Interval between pruning operations in seconds
    pub pruning_interval: u64,
}

/// Network configuration for P2P communication
///
/// Controls how the node communicates with other nodes
/// in the DSM network.
#[derive(Debug, Deserialize, Clone)]
#[allow(dead_code)]
pub struct NetworkConfig {
    /// IP address to listen on for P2P communication
    pub listen_addr: String,

    /// Public endpoint for other nodes to connect to this node
    pub public_endpoint: String,

    /// Port number for P2P communication
    pub port: u16,

    /// Maximum number of concurrent P2P connections
    pub max_connections: u16,

    /// Connection timeout in seconds
    pub connection_timeout: u16,

    /// Whether to enable automatic node discovery
    pub enable_discovery: bool,

    /// Interval between node discovery operations in seconds
    pub discovery_interval: u64,

    /// Maximum number of peer nodes to maintain
    pub max_peers: u16,
}

/// Cluster configuration for overlapping cluster topology
///
/// Controls cluster membership and gossip behavior for
/// the overlapping cluster architecture.
#[derive(Debug, Deserialize, Clone)]
pub struct ClusterConfig {
    /// Whether cluster management is enabled
    pub enabled: bool,

    /// List of clusters this node participates in
    pub clusters: Vec<String>,

    /// Overlap factor (how many clusters each node participates in)
    pub overlap_factor: u32,

    /// Target size for each cluster
    pub target_cluster_size: u32,

    /// Minimum size for each cluster
    pub min_cluster_size: u32,
}

/// Complete application configuration
///
/// Combines all configuration subsections into a single struct.
#[derive(Debug, Deserialize, Clone)]
pub struct AppConfig {
    /// API server configuration
    pub api: ApiConfig,

    /// Node identity and metadata
    pub node: NodeConfig,

    /// Storage engine configuration
    pub storage: StorageConfig,

    /// Network and P2P configuration
    pub network: NetworkConfig,

    /// Cluster management configuration (optional)
    pub cluster: Option<ClusterConfig>,
}

// Deref implementation for convenient access to storage config
impl std::ops::Deref for AppConfig {
    type Target = StorageConfig;

    fn deref(&self) -> &Self::Target {
        &self.storage
    }
}

/// MPC contribution request for genesis creation
#[derive(Deserialize)]
pub struct MpcContributionRequest {
    pub session_id: String,
    pub device_id: String,
    pub node_id: String,
    pub entropy_data: Vec<u8>,
    pub contribution_type: String,
    pub timestamp: u64,
}

/// MPC contribution response
#[derive(Serialize)]
pub struct MpcContributionResponse {
    pub success: bool,
    pub contribution: String,
    pub session_id: String,
    pub threshold_met: bool,
    pub accepted: bool,
    pub contributions_count: usize,
    pub threshold: usize,
    pub ready_for_processing: bool,
    pub message: String,
    pub timestamp: u64,
}

/// MPC Genesis request
#[derive(Deserialize)]
pub struct MpcGenesisRequest {
    pub device_id: String,
    pub threshold: Option<u8>,
}

/// MPC Genesis response
#[derive(Serialize)]
pub struct MpcGenesisResponse {
    pub success: bool,
    pub session_id: String,
    pub device_id: String,
    pub mnemonic: Option<String>,
    pub error: Option<String>,
}

/// Request for blind device ID creation
#[derive(Debug, Serialize, Deserialize)]
pub struct BlindDeviceIdRequest {
    /// Device ID requesting blind identity
    pub device_id: String,
    /// Required threshold for MPC
    pub threshold: usize,
    /// Request timestamp
    pub request_timestamp: u64,
}

/// Response for blind device ID creation
#[derive(Debug, Serialize, Deserialize)]
pub struct BlindDeviceIdResponse {
    /// Session ID for tracking
    pub session_id: String,
    /// Device ID
    pub device_id: String,
    /// Current session state
    pub state: String,
    /// Number of contributions received
    pub contributions_received: usize,
    /// Required threshold
    pub threshold: usize,
    /// Whether genesis creation is complete
    pub complete: bool,
    /// Timestamp
    pub timestamp: u64,
}

/// Request for contributing to blind device ID session
#[derive(Debug, Serialize, Deserialize)]
pub struct BlindContributionRequest {
    /// Session ID to contribute to
    pub session_id: String,
    /// Node ID making the contribution
    pub node_id: String,
    /// Entropy contribution data
    pub entropy_data: Vec<u8>,
    /// Optional cryptographic proof
    pub proof: Option<Vec<u8>>,
    /// Timestamp
    pub timestamp: u64,
}

/// Response for blind device ID contribution
#[derive(Debug, Serialize, Deserialize)]
pub struct BlindContributionResponse {
    /// Whether contribution was accepted
    pub success: bool,
    /// Session ID
    pub session_id: String,
    /// Whether threshold is met and genesis is ready
    pub threshold_met: bool,
    /// Error message if any
    pub error: Option<String>,
    /// Timestamp
    pub timestamp: u64,
}

/// Node discovery response
#[derive(Serialize)]
pub struct NodeDiscoveryResponse {
    pub nodes: Vec<String>,
    pub count: usize,
}

/// Node status response for the API
///
/// Contains information about the node's current state,
/// used by the status endpoint.
#[derive(Serialize)]
pub struct StatusResponse {
    /// Unique identifier for this node
    pub node_id: String,

    /// Current operational status
    pub status: String,

    /// Version string
    pub version: String,

    /// Time in seconds since the node started
    pub uptime: u64,

    /// Number of connected peer nodes
    pub peers: u16,

    /// Amount of storage used in bytes
    pub storage_used: u64,

    /// Total storage capacity in bytes
    pub storage_total: u64,

    /// Amount of tokens staked by this node (if any)
    pub staked_amount: Option<u64>,
}

/// Error response for the API
///
/// Used to return structured error information
/// when an API request fails.
#[derive(Serialize)]
#[allow(dead_code)]
pub struct ErrorResponse {
    /// Error message
    pub error: String,
}

/// Data response for the API
///
/// Used to return data items with their keys.
#[derive(Serialize, Deserialize)]
pub struct DataResponse {
    /// Key for the data item
    pub key: String,

    /// The data item's content as JSON
    pub data: Value,
}

/// MPC session state
pub struct MpcSession {
    pub session_id: String,
    pub device_id: String,
    pub threshold: u8,
    pub contributions: Vec<MpcContribution>,
    pub created_at: u64,
}

/// MPC contribution data
pub struct MpcContribution {
    pub party_id: String,
    pub contribution_data: Vec<u8>,
    pub timestamp: u64,
}

/// Application state shared across API handlers
///
/// Contains the core components and configuration needed by
/// the API handlers to process requests.
pub struct AppState {
    /// Application configuration
    pub config: AppConfig,

    /// Amount of tokens staked by this node (if any)
    pub staked_amount: Option<u64>,

    /// Storage engine implementation
    pub storage: Arc<dyn crate::storage::StorageEngine + Send + Sync>,

    /// Active MPC sessions
    pub mpc_sessions: Arc<RwLock<HashMap<String, MpcSession>>>,

    /// DSM Identity Manager for blind device ID creation
    pub identity_manager: Option<Arc<crate::identity::DsmIdentityManager>>,

    /// Comprehensive operation logger
    pub logger: Arc<crate::logging::StorageNodeLogger>,

    /// Cluster manager for overlapping cluster topology
    pub cluster_manager: Option<Arc<crate::cluster::ClusterManager>>,
}
