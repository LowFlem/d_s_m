/// Mobile Discovery API Module
///
/// Provides a REST API for mobile devices to automatically discover DSM storage nodes
/// on the local network. This eliminates the need for manual IP configuration in
/// mobile applications.
use super::DiscoveredNode;
use axum::{
    extract::{Query, State},
    http::StatusCode,
    response::Json,
    routing::get,
    Router,
};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::sync::Arc;
use std::time::{Duration, SystemTime};
use tokio::sync::RwLock;
use tracing::{debug, error, info, warn};

/// Mobile discovery API server
pub struct MobileDiscoveryApi {
    port: u16,
    discovered_nodes: Arc<RwLock<HashMap<String, DiscoveredNode>>>,
    server_handle: Arc<RwLock<Option<tokio::task::JoinHandle<()>>>>,
}

impl MobileDiscoveryApi {
    /// Create a new mobile discovery API
    pub async fn new(
        port: u16,
        discovered_nodes: Arc<RwLock<HashMap<String, DiscoveredNode>>>,
    ) -> Result<Self, Box<dyn std::error::Error + Send + Sync>> {
        Ok(Self {
            port,
            discovered_nodes,
            server_handle: Arc::new(RwLock::new(None)),
        })
    }

    /// Start the mobile discovery API server
    pub async fn start(&self) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
        info!("Starting mobile discovery API on port {}", self.port);

        let app = create_mobile_api_router(self.discovered_nodes.clone());

        let addr = format!("0.0.0.0:{}", self.port);
        let listener = tokio::net::TcpListener::bind(&addr).await?;

        info!("Mobile discovery API listening on {}", addr);

        // Start server in background
        let server_task = tokio::spawn(async move {
            if let Err(e) = axum::Server::from_tcp(listener.into_std().unwrap())
                .unwrap()
                .serve(app.into_make_service())
                .await
            {
                error!("Mobile discovery API server error: {}", e);
            }
        });

        *self.server_handle.write().await = Some(server_task);
        Ok(())
    }

    /// Stop the mobile discovery API server
    pub async fn stop(&self) {
        info!("Stopping mobile discovery API");

        if let Some(handle) = self.server_handle.write().await.take() {
            handle.abort();
            let _ = handle.await;
        }

        info!("Mobile discovery API stopped");
    }
}

/// Create the router for mobile discovery API
fn create_mobile_api_router(
    discovered_nodes: Arc<RwLock<HashMap<String, DiscoveredNode>>>,
) -> Router {
    Router::new()
        .route("/api/discover", get(discover_nodes_handler))
        .route("/api/nodes", get(list_nodes_handler))
        .route("/api/config", get(mobile_config_handler))
        .route("/api/health", get(health_check_handler))
        .route("/api/network-status", get(network_status_handler))
        .with_state(discovered_nodes)
}

/// Query parameters for node discovery
#[derive(Debug, Deserialize)]
struct DiscoverQuery {
    /// Filter by service type
    service_type: Option<String>,
    /// Include inactive nodes
    include_inactive: Option<bool>,
    /// Maximum age in seconds
    max_age: Option<u64>,
}

/// Response for node discovery
#[derive(Debug, Serialize, Deserialize)]
struct DiscoverResponse {
    /// Number of nodes found
    count: usize,
    /// List of discovered nodes
    nodes: Vec<MobileNodeInfo>,
    /// Discovery metadata
    metadata: DiscoveryMetadata,
}

/// Mobile-friendly node information
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MobileNodeInfo {
    /// Node ID
    pub node_id: String,
    /// Human-readable name
    pub name: String,
    /// Node endpoint URL
    pub endpoint: String,
    /// Service type
    pub service_type: String,
    /// Node capabilities
    pub capabilities: Vec<String>,
    /// Last seen timestamp (Unix epoch)
    pub last_seen: u64,
    /// Connection status
    pub status: String,
    /// Additional properties
    pub properties: HashMap<String, String>,
}

/// Discovery metadata
#[derive(Debug, Serialize, Deserialize)]
struct DiscoveryMetadata {
    /// Discovery timestamp
    pub timestamp: u64,
    /// Total nodes in database
    pub total_nodes: usize,
    /// Filtered nodes count
    pub filtered_count: usize,
    /// Discovery server version
    pub version: String,
}

/// Handler for node discovery endpoint
async fn discover_nodes_handler(
    Query(query): Query<DiscoverQuery>,
    State(discovered_nodes): State<Arc<RwLock<HashMap<String, DiscoveredNode>>>>,
) -> Result<Json<DiscoverResponse>, StatusCode> {
    debug!("Mobile discovery request: {:?}", query);

    let nodes = discovered_nodes.read().await;
    let now = SystemTime::now();

    // Filter nodes based on query parameters
    let max_age = Duration::from_secs(query.max_age.unwrap_or(300)); // Default 5 minutes
    let include_inactive = query.include_inactive.unwrap_or(false);

    let filtered_nodes: Vec<MobileNodeInfo> = nodes
        .values()
        .filter(|node| {
            // Filter by service type if specified
            if let Some(ref service_type) = query.service_type {
                if &node.service_type != service_type {
                    return false;
                }
            }

            // Filter by freshness unless including inactive
            if !include_inactive && !node.is_fresh(max_age) {
                return false;
            }

            true
        })
        .map(|node| convert_to_mobile_info(node, &now))
        .collect();

    let response = DiscoverResponse {
        count: filtered_nodes.len(),
        nodes: filtered_nodes.clone(),
        metadata: DiscoveryMetadata {
            timestamp: now
                .duration_since(SystemTime::UNIX_EPOCH)
                .unwrap_or_default()
                .as_secs(),
            total_nodes: nodes.len(),
            filtered_count: filtered_nodes.len(),
            version: "1.0.0".to_string(),
        },
    };

    info!("Mobile discovery: returned {} nodes", response.count);
    Ok(Json(response))
}

/// Handler for listing all nodes
async fn list_nodes_handler(
    State(discovered_nodes): State<Arc<RwLock<HashMap<String, DiscoveredNode>>>>,
) -> Result<Json<Vec<MobileNodeInfo>>, StatusCode> {
    let nodes = discovered_nodes.read().await;
    let now = SystemTime::now();

    let mobile_nodes: Vec<MobileNodeInfo> = nodes
        .values()
        .map(|node| convert_to_mobile_info(node, &now))
        .collect();

    debug!("Listed {} nodes for mobile client", mobile_nodes.len());
    Ok(Json(mobile_nodes))
}

/// Handler for mobile configuration generation
async fn mobile_config_handler(
    State(discovered_nodes): State<Arc<RwLock<HashMap<String, DiscoveredNode>>>>,
) -> Result<Json<serde_json::Value>, StatusCode> {
    let nodes = discovered_nodes.read().await;
    let max_age = Duration::from_secs(300); // 5 minutes

    // Get active storage nodes
    let storage_nodes: Vec<&DiscoveredNode> = nodes
        .values()
        .filter(|node| node.service_type == "dsm-storage" && node.is_fresh(max_age))
        .collect();

    if storage_nodes.is_empty() {
        warn!("No active storage nodes found for mobile config");
        return Err(StatusCode::SERVICE_UNAVAILABLE);
    }

    let mobile_config = generate_mobile_config(&storage_nodes).await;

    info!(
        "Generated mobile config with {} storage nodes",
        storage_nodes.len()
    );
    Ok(Json(mobile_config))
}

/// Handler for health check
async fn health_check_handler() -> Result<Json<serde_json::Value>, StatusCode> {
    let health = serde_json::json!({
        "status": "healthy",
        "service": "mobile-discovery-api",
        "timestamp": SystemTime::now()
            .duration_since(SystemTime::UNIX_EPOCH)
            .unwrap_or_default()
            .as_secs(),
        "version": "1.0.0"
    });

    Ok(Json(health))
}

/// Handler for network status
async fn network_status_handler(
    State(discovered_nodes): State<Arc<RwLock<HashMap<String, DiscoveredNode>>>>,
) -> Result<Json<serde_json::Value>, StatusCode> {
    let nodes = discovered_nodes.read().await;
    let now = SystemTime::now();
    let max_age = Duration::from_secs(300);

    let total_nodes = nodes.len();
    let active_nodes = nodes.values().filter(|n| n.is_fresh(max_age)).count();
    let storage_nodes = nodes
        .values()
        .filter(|n| n.service_type == "dsm-storage")
        .count();

    let network_status = serde_json::json!({
        "network": {
            "total_nodes": total_nodes,
            "active_nodes": active_nodes,
            "storage_nodes": storage_nodes,
            "inactive_nodes": total_nodes - active_nodes
        },
        "health": {
            "network_healthy": active_nodes > 0,
            "sufficient_storage": storage_nodes >= 1,
            "cluster_ready": storage_nodes >= 3
        },
        "timestamp": now.duration_since(SystemTime::UNIX_EPOCH)
            .unwrap_or_default()
            .as_secs()
    });

    Ok(Json(network_status))
}

/// Convert DiscoveredNode to MobileNodeInfo
fn convert_to_mobile_info(node: &DiscoveredNode, _now: &SystemTime) -> MobileNodeInfo {
    let status = if node.is_fresh(Duration::from_secs(300)) {
        "active"
    } else {
        "inactive"
    };

    MobileNodeInfo {
        node_id: node.node_id.clone(),
        name: node.name.clone(),
        endpoint: node.endpoint(),
        service_type: node.service_type.clone(),
        capabilities: node.capabilities.clone(),
        last_seen: node
            .last_seen
            .duration_since(SystemTime::UNIX_EPOCH)
            .unwrap_or_default()
            .as_secs(),
        status: status.to_string(),
        properties: node.properties.clone(),
    }
}

/// Generate mobile configuration from discovered nodes
async fn generate_mobile_config(storage_nodes: &[&DiscoveredNode]) -> serde_json::Value {
    let endpoints: Vec<String> = storage_nodes.iter().map(|n| n.endpoint()).collect();
    let bootstrap_node = endpoints.first().cloned().unwrap_or_default();

    serde_json::json!({
        "development": {
            "discovery_nodes": endpoints,
            "connection_timeout": "30000",
            "bootstrap_node": bootstrap_node,
            "auto_discovered": true
        },
        "storage": {
            "cache_dir": "/data/data/com.dsm.wallet/files/cache",
            "local_storage_path": "/data/data/com.dsm.wallet/files/storage",
            "identity_file": "/data/data/com.dsm.wallet/files/identity.key",
            "create_identity_if_missing": true
        },
        "testing": {
            "replication_factor": determine_replication_factor(storage_nodes.len()),
            "min_sync_consistency": 0.95,
            "mpc_threshold": determine_mpc_threshold(storage_nodes.len()).to_string(),
            "max_discovery_attempts": 3,
            "enable_quantum_resistance": true,
            "epidemic_protocol_enabled": storage_nodes.len() > 3,
            "bilateral_sync_interval": 30000,
            "node_health_check_interval": 60000
        },
        "mpc": {
            "enabled": "true",
            "threshold": determine_mpc_threshold(storage_nodes.len()).to_string(),
            "max_participants": storage_nodes.len().to_string(),
            "session_timeout": "300",
            "enable_blind_signatures": "true",
            "dbrw_enabled": "true",
            "bootstrap_node": bootstrap_node,
            "nodes": endpoints
        },
        "security": {
            "encryption_key": "auto_generated_key",
            "signing_key": "auto_generated_key",
            "identity_file": "/data/data/com.dsm.wallet/files/identity.key",
            "create_identity_if_missing": true,
            "enable_encryption": true,
            "enable_signing": true
        },
        "network": {
            "auto_discovery": true,
            "discovery_api_enabled": true,
            "discovery_refresh_interval": 60,
            "fallback_nodes": endpoints
        },
        "auto_config": {
            "generated_at": chrono::Utc::now().to_rfc3339(),
            "source": "auto-discovery",
            "node_count": storage_nodes.len(),
            "version": "1.0.0"
        }
    })
}

/// Determine replication factor based on node count
fn determine_replication_factor(node_count: usize) -> u32 {
    match node_count {
        1 => 1,
        2..=3 => 2,
        4..=6 => 3,
        _ => (node_count / 2).min(5) as u32,
    }
}

/// Determine MPC threshold based on node count
fn determine_mpc_threshold(node_count: usize) -> u32 {
    match node_count {
        1 => 1,
        2..=3 => 2,
        4..=6 => 3,
        _ => ((node_count * 2) / 3).max(3) as u32,
    }
}

/// Mobile discovery client for testing
pub struct MobileDiscoveryClient {
    base_url: String,
    client: reqwest::Client,
}

impl MobileDiscoveryClient {
    /// Create a new mobile discovery client
    pub fn new(discovery_api_url: &str) -> Self {
        Self {
            base_url: discovery_api_url.to_string(),
            client: reqwest::Client::builder()
                .timeout(Duration::from_secs(10))
                .build()
                .unwrap(),
        }
    }

    /// Discover storage nodes
    pub async fn discover_storage_nodes(
        &self,
    ) -> Result<Vec<MobileNodeInfo>, Box<dyn std::error::Error + Send + Sync>> {
        let url = format!("{}/api/discover?service_type=dsm-storage", self.base_url);

        let response: DiscoverResponse = self.client.get(&url).send().await?.json().await?;

        Ok(response.nodes)
    }

    /// Get mobile configuration
    pub async fn get_mobile_config(
        &self,
    ) -> Result<serde_json::Value, Box<dyn std::error::Error + Send + Sync>> {
        let url = format!("{}/api/config", self.base_url);

        let config = self.client.get(&url).send().await?.json().await?;

        Ok(config)
    }

    /// Check network status
    pub async fn get_network_status(
        &self,
    ) -> Result<serde_json::Value, Box<dyn std::error::Error + Send + Sync>> {
        let url = format!("{}/api/network-status", self.base_url);

        let status = self.client.get(&url).send().await?.json().await?;

        Ok(status)
    }

    /// Health check
    pub async fn health_check(&self) -> Result<bool, Box<dyn std::error::Error + Send + Sync>> {
        let url = format!("{}/api/health", self.base_url);

        let response = self.client.get(&url).send().await?;

        Ok(response.status().is_success())
    }
}

/// Utility functions for mobile discovery
pub mod mobile_utils {
    use super::*;

    /// Find the best storage node for a mobile client
    pub fn find_best_node(nodes: &[MobileNodeInfo]) -> Option<&MobileNodeInfo> {
        nodes
            .iter()
            .filter(|n| n.status == "active" && n.service_type == "dsm-storage")
            .max_by_key(|n| n.last_seen)
    }

    /// Create a balanced endpoint list for load distribution
    pub fn create_balanced_endpoints(nodes: &[MobileNodeInfo]) -> Vec<String> {
        let mut active_nodes: Vec<_> = nodes
            .iter()
            .filter(|n| n.status == "active" && n.service_type == "dsm-storage")
            .collect();

        // Sort by last seen (most recent first)
        active_nodes.sort_by_key(|n| std::cmp::Reverse(n.last_seen));

        // Return endpoints in order
        active_nodes.iter().map(|n| n.endpoint.clone()).collect()
    }

    /// Check if the network has sufficient nodes for MPC
    pub fn is_mpc_ready(nodes: &[MobileNodeInfo]) -> bool {
        let active_storage_nodes = nodes
            .iter()
            .filter(|n| n.status == "active" && n.service_type == "dsm-storage")
            .count();

        active_storage_nodes >= 1 // At least 1 node for MPC (can be adjusted)
    }

    /// Generate connection priorities for mobile clients
    pub fn generate_connection_priorities(
        nodes: &[MobileNodeInfo],
        prefer_local: bool,
    ) -> Vec<(String, u32)> {
        let mut priorities = Vec::new();

        for node in nodes {
            if node.status != "active" || node.service_type != "dsm-storage" {
                continue;
            }

            let mut priority = 100u32; // Base priority

            // Prefer more recently seen nodes
            let age_seconds = SystemTime::now()
                .duration_since(SystemTime::UNIX_EPOCH)
                .unwrap_or_default()
                .as_secs()
                .saturating_sub(node.last_seen);

            if age_seconds < 60 {
                priority += 50; // Very recent
            } else if age_seconds < 300 {
                priority += 25; // Recent
            }

            // Prefer nodes with more capabilities
            priority += node.capabilities.len() as u32 * 10;

            // Local network preference (simple heuristic)
            if prefer_local && (node.endpoint.contains("192.168.") || node.endpoint.contains("10."))
            {
                priority += 30;
            }

            priorities.push((node.endpoint.clone(), priority));
        }

        // Sort by priority (highest first)
        priorities.sort_by_key(|(_, p)| std::cmp::Reverse(*p));
        priorities
    }
}
