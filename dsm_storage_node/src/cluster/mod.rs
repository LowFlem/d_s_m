// DSM Storage Node Cluster Manager
// Implements overlapping cluster topology for fault tolerance and scalability

use crate::auto_network::{auto_cluster::AutoClusterManager, AutoNetworkConfig, DiscoveredNode};
use serde::{Deserialize, Serialize};
use std::collections::{HashMap, HashSet};
use std::sync::Arc;
use tokio::sync::RwLock;

/// Cluster identifier
pub type ClusterId = String;

/// Node identifier  
pub type NodeId = String;

/// Geographic region identifier
pub type RegionId = String;

/// Cluster configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ClusterConfig {
    /// Cluster identifier
    pub id: ClusterId,

    /// Target cluster size
    pub target_size: usize,

    /// Minimum cluster size for operation
    pub min_size: usize,

    /// Geographic region preference
    pub preferred_region: Option<RegionId>,

    /// Cluster type (primary, secondary, etc.)
    pub cluster_type: ClusterType,

    /// Overlap factor (how many clusters each node should participate in)
    pub overlap_factor: usize,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum ClusterType {
    Primary,   // Main MPC clusters
    Secondary, // Backup clusters
    Gossip,    // Gossip-only clusters
    Bridge,    // Inter-cluster routing
}

/// Node information in cluster context
#[derive(Debug, Clone, Serialize, Deserialize, Hash, Eq, PartialEq)]
pub struct ClusterNode {
    pub id: NodeId,
    pub endpoint: String,
    pub region: Option<RegionId>,
    pub capabilities: NodeCapabilities,
    pub status: NodeStatus,
    pub join_time: chrono::DateTime<chrono::Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize, Hash, Eq, PartialEq)]
pub struct NodeCapabilities {
    pub mpc_enabled: bool,
    pub storage_capacity: u64,
    pub bandwidth_mbps: u32,
    pub cpu_cores: u32,
}

#[derive(Debug, Clone, Serialize, Deserialize, Hash, Eq, PartialEq)]
pub enum NodeStatus {
    Active,
    Inactive,
    Suspected,
    Failed,
}

/// Cluster membership information
#[derive(Debug, Clone)]
pub struct ClusterMembership {
    /// Clusters this node is a member of
    pub clusters: HashSet<ClusterId>,

    /// Nodes in each cluster this node participates in
    pub cluster_members: HashMap<ClusterId, HashSet<NodeId>>,

    /// Overlap connections (nodes that are in multiple clusters with this node)
    pub overlap_connections: HashSet<NodeId>,
}

/// Main cluster manager
pub struct ClusterManager {
    /// This node's ID
    node_id: NodeId,

    /// Auto cluster manager for dynamic discovery
    auto_cluster: Arc<AutoClusterManager>,

    /// Cluster configurations
    cluster_configs: Arc<RwLock<HashMap<ClusterId, ClusterConfig>>>,

    /// Active clusters and their members
    clusters: Arc<RwLock<HashMap<ClusterId, HashSet<ClusterNode>>>>,

    /// This node's cluster membership
    membership: Arc<RwLock<ClusterMembership>>,

    /// All known nodes in the network (from discovery)
    known_nodes: Arc<RwLock<HashMap<NodeId, ClusterNode>>>,

    /// Cluster health status
    cluster_health: Arc<RwLock<HashMap<ClusterId, f64>>>,
}

impl ClusterManager {
    /// Create new cluster manager with auto-discovery
    pub async fn new(
        node_id: NodeId,
        config: AutoNetworkConfig,
        local_node: DiscoveredNode,
        discovered_nodes: Arc<RwLock<HashMap<String, DiscoveredNode>>>,
    ) -> Self {
        let auto_cluster = Arc::new(AutoClusterManager::new(
            config,
            local_node,
            discovered_nodes,
        ));

        Self {
            node_id,
            auto_cluster,
            cluster_configs: Arc::new(RwLock::new(HashMap::new())),
            clusters: Arc::new(RwLock::new(HashMap::new())),
            membership: Arc::new(RwLock::new(ClusterMembership {
                clusters: HashSet::new(),
                cluster_members: HashMap::new(),
                overlap_connections: HashSet::new(),
            })),
            known_nodes: Arc::new(RwLock::new(HashMap::new())),
            cluster_health: Arc::new(RwLock::new(HashMap::new())),
        }
    }

    /// Create new cluster manager (legacy constructor for backwards compatibility)
    pub fn new_legacy(node_id: NodeId) -> Self {
        // Create minimal configuration for legacy mode
        let config = AutoNetworkConfig::default();
        let local_node = DiscoveredNode {
            node_id: node_id.clone(),
            name: format!("legacy-{node_id}"),
            ip: "127.0.0.1".parse().unwrap(),
            port: 8080,
            service_type: "dsm-storage".to_string(),
            properties: HashMap::new(),
            discovered_at: std::time::SystemTime::now(),
            last_seen: std::time::SystemTime::now(),
            capabilities: vec!["mpc".to_string(), "storage".to_string()],
        };
        let discovered_nodes = Arc::new(RwLock::new(HashMap::new()));

        let auto_cluster = Arc::new(AutoClusterManager::new(
            config,
            local_node,
            discovered_nodes,
        ));

        Self {
            node_id,
            auto_cluster,
            cluster_configs: Arc::new(RwLock::new(HashMap::new())),
            clusters: Arc::new(RwLock::new(HashMap::new())),
            membership: Arc::new(RwLock::new(ClusterMembership {
                clusters: HashSet::new(),
                cluster_members: HashMap::new(),
                overlap_connections: HashSet::new(),
            })),
            known_nodes: Arc::new(RwLock::new(HashMap::new())),
            cluster_health: Arc::new(RwLock::new(HashMap::new())),
        }
    }

    /// Start the auto-discovery and dynamic cluster formation
    pub async fn start_discovery(&self) -> Result<(), ClusterError> {
        self.auto_cluster.start().await.map_err(|e| {
            ClusterError::Configuration(format!("Failed to start auto-discovery: {e}"))
        })?;

        // Start periodic sync with auto-discovery
        self.start_discovery_sync().await;

        Ok(())
    }

    /// Sync discovered peers into cluster management
    async fn start_discovery_sync(&self) {
        let auto_cluster = self.auto_cluster.clone();
        let known_nodes = self.known_nodes.clone();
        let clusters = self.clusters.clone();

        tokio::spawn(async move {
            let mut interval = tokio::time::interval(std::time::Duration::from_secs(30));

            loop {
                interval.tick().await;

                // Get discovered peers from auto-cluster
                let gossip_peers = auto_cluster.get_gossip_peers().await;

                // Convert to ClusterNode format and update known_nodes
                let mut nodes = known_nodes.write().await;
                for peer in gossip_peers {
                    let cluster_node = ClusterNode {
                        id: peer.node_id.clone(),
                        endpoint: peer.endpoint(),
                        region: peer.properties.get("region").cloned(),
                        capabilities: NodeCapabilities {
                            mpc_enabled: peer.capabilities.contains(&"mpc".to_string()),
                            storage_capacity: peer
                                .properties
                                .get("storage_capacity")
                                .and_then(|s| s.parse().ok())
                                .unwrap_or(10_737_418_240),
                            bandwidth_mbps: peer
                                .properties
                                .get("bandwidth_mbps")
                                .and_then(|s| s.parse().ok())
                                .unwrap_or(100),
                            cpu_cores: peer
                                .properties
                                .get("cpu_cores")
                                .and_then(|s| s.parse().ok())
                                .unwrap_or(4),
                        },
                        status: NodeStatus::Active,
                        join_time: peer.discovered_at.into(),
                    };
                    nodes.insert(peer.node_id.clone(), cluster_node);
                }

                // Form dynamic clusters based on discovered topology
                let topology = auto_cluster.get_topology().await;
                let mut cluster_map = clusters.write().await;
                cluster_map.clear();

                for cluster in topology.clusters {
                    let cluster_nodes: HashSet<ClusterNode> = cluster
                        .nodes
                        .iter()
                        .filter_map(|node_id| nodes.get(node_id).cloned())
                        .collect();

                    if !cluster_nodes.is_empty() {
                        cluster_map.insert(cluster.id, cluster_nodes);
                    }
                }
            }
        });
    }

    /// Update this node's cluster membership
    #[allow(dead_code)]
    async fn update_membership(&self) {
        let clusters = self.clusters.read().await;
        let mut membership = self.membership.write().await;

        membership.clusters.clear();
        membership.cluster_members.clear();
        membership.overlap_connections.clear();

        // Find clusters this node belongs to
        for (cluster_id, cluster_nodes) in clusters.iter() {
            if cluster_nodes.iter().any(|node| node.id == self.node_id) {
                membership.clusters.insert(cluster_id.clone());

                let member_ids: HashSet<NodeId> = cluster_nodes
                    .iter()
                    .filter(|node| node.id != self.node_id) // Exclude self
                    .map(|node| node.id.clone())
                    .collect();

                membership
                    .cluster_members
                    .insert(cluster_id.clone(), member_ids);
            }
        }

        // Find overlap connections (nodes in multiple clusters with this node)
        let my_clusters: Vec<_> = membership.clusters.iter().cloned().collect();
        let mut overlap_connections = HashSet::new();

        for cluster_id in &my_clusters {
            if let Some(members) = membership.cluster_members.get(cluster_id) {
                for member_id in members {
                    // Check if this member is in other clusters with us
                    for other_cluster_id in &my_clusters {
                        if other_cluster_id != cluster_id {
                            if let Some(other_members) =
                                membership.cluster_members.get(other_cluster_id)
                            {
                                if other_members.contains(member_id) {
                                    overlap_connections.insert(member_id.clone());
                                }
                            }
                        }
                    }
                }
            }
        }

        membership.overlap_connections = overlap_connections;
    }

    /// Get nodes to gossip with using auto-discovery
    pub async fn get_gossip_targets(&self, cluster_id: Option<ClusterId>) -> Vec<ClusterNode> {
        // First try auto-discovery peers
        let discovered_peers = self.auto_cluster.get_gossip_peers().await;

        let mut targets: Vec<ClusterNode> = discovered_peers
            .into_iter()
            .filter(|peer| peer.node_id != self.node_id)
            .map(|peer| ClusterNode {
                id: peer.node_id.clone(),
                endpoint: peer.endpoint(),
                region: peer.properties.get("region").cloned(),
                capabilities: NodeCapabilities {
                    mpc_enabled: peer.capabilities.contains(&"mpc".to_string()),
                    storage_capacity: 10_737_418_240,
                    bandwidth_mbps: 100,
                    cpu_cores: 4,
                },
                status: NodeStatus::Active,
                join_time: peer.discovered_at.into(),
            })
            .collect();

        // If we have specific cluster requirements, filter by cluster
        if let Some(cluster_id) = cluster_id {
            let membership = self.membership.read().await;
            let known_nodes = self.known_nodes.read().await;

            // Add cluster-specific members
            if let Some(members) = membership.cluster_members.get(&cluster_id) {
                for member_id in members {
                    if let Some(node) = known_nodes.get(member_id) {
                        if node.status == NodeStatus::Active
                            && !targets.iter().any(|t| t.id == node.id)
                        {
                            targets.push(node.clone());
                        }
                    }
                }
            }
        }

        targets
    }

    /// Get available MPC cluster for genesis creation using auto-discovery
    pub async fn get_mpc_cluster(&self) -> Option<Vec<ClusterNode>> {
        // First check auto-discovered peers
        let discovered_peers = self.auto_cluster.get_gossip_peers().await;

        let mpc_peers: Vec<ClusterNode> = discovered_peers
            .into_iter()
            .filter(|peer| {
                peer.capabilities.contains(&"mpc".to_string()) && peer.node_id != self.node_id
            })
            .map(|peer| ClusterNode {
                id: peer.node_id.clone(),
                endpoint: peer.endpoint(),
                region: peer.properties.get("region").cloned(),
                capabilities: NodeCapabilities {
                    mpc_enabled: true,
                    storage_capacity: 10_737_418_240,
                    bandwidth_mbps: 100,
                    cpu_cores: 4,
                },
                status: NodeStatus::Active,
                join_time: peer.discovered_at.into(),
            })
            .collect();

        if mpc_peers.len() >= 2 {
            // Need at least 2 other peers for MPC (3 total including this node)
            tracing::info!(
                "Found {} MPC-capable peers via auto-discovery",
                mpc_peers.len()
            );
            return Some(mpc_peers);
        }

        // Fallback to configured clusters
        let clusters = self.clusters.read().await;
        let cluster_health = self.cluster_health.read().await;

        for (cluster_id, nodes) in clusters.iter() {
            let config = self.cluster_configs.read().await;
            if let Some(cluster_config) = config.get(cluster_id) {
                if matches!(cluster_config.cluster_type, ClusterType::Primary) {
                    let active_nodes: Vec<_> = nodes
                        .iter()
                        .filter(|node| {
                            node.status == NodeStatus::Active && node.capabilities.mpc_enabled
                        })
                        .cloned()
                        .collect();

                    if active_nodes.len() >= cluster_config.min_size {
                        let health = cluster_health.get(cluster_id).copied().unwrap_or(1.0);
                        if health > 0.7 {
                            return Some(active_nodes);
                        }
                    }
                }
            }
        }

        None
    }

    /// Check if node is an overlap node (participates in multiple clusters)
    pub async fn is_overlap_node(&self) -> bool {
        let membership = self.membership.read().await;
        membership.clusters.len() > 1
    }

    /// Get clusters this node participates in
    pub async fn get_my_clusters(&self) -> Vec<ClusterId> {
        let membership = self.membership.read().await;
        membership.clusters.iter().cloned().collect()
    }
}

#[derive(Debug, thiserror::Error)]
pub enum ClusterError {
    #[error("Cluster not found: {0}")]
    ClusterNotFound(ClusterId),

    #[error("Node not found: {0}")]
    NodeNotFound(NodeId),

    #[error("Insufficient cluster size: {current} < {required}")]
    InsufficientSize { current: usize, required: usize },

    #[error("Configuration error: {0}")]
    Configuration(String),
}
