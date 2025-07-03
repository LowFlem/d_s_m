/// Auto Cluster Management Module
///
/// Automatically forms and manages gossip networks between DSM storage nodes.
/// This module handles cluster topology optimization, peer selection, and
/// automatic network healing.
use super::{AutoNetworkConfig, DiscoveredNode};
use serde::{Deserialize, Serialize};
use std::collections::{HashMap, HashSet};
use std::sync::Arc;
use std::time::{Duration, SystemTime};
use tokio::sync::RwLock;
use tracing::{debug, error, info};

/// Type alias for complex return type of multi-cluster topology creation
type ClusterTopologyResult = Result<
    (
        Vec<Cluster>,
        HashMap<String, Vec<String>>,
        HashMap<String, NodeRole>,
    ),
    Box<dyn std::error::Error + Send + Sync>,
>;

/// Auto cluster manager for DSM nodes
pub struct AutoClusterManager {
    config: AutoNetworkConfig,
    local_node: DiscoveredNode,
    discovered_nodes: Arc<RwLock<HashMap<String, DiscoveredNode>>>,
    cluster_topology: Arc<RwLock<ClusterTopology>>,
    running: Arc<RwLock<bool>>,
}

/// Cluster topology information
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ClusterTopology {
    /// Clusters in the network
    pub clusters: Vec<Cluster>,
    /// Peer connections
    pub connections: HashMap<String, Vec<String>>,
    /// Node roles
    pub node_roles: HashMap<String, NodeRole>,
    /// Last topology update
    pub last_updated: SystemTime,
    /// Topology version for conflict resolution
    pub version: u64,
}

/// Individual cluster information
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Cluster {
    /// Cluster identifier
    pub id: String,
    /// Nodes in this cluster
    pub nodes: Vec<String>,
    /// Cluster center (representative node)
    pub center: String,
    /// Cluster formation timestamp
    pub created_at: SystemTime,
    /// Health status
    pub healthy: bool,
}

/// Node roles in the cluster
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum NodeRole {
    /// Seed node (cluster initiator)
    Seed,
    /// Regular cluster member
    Member,
    /// Bridge node (connects multiple clusters)
    Bridge,
    /// Standby node (not in active cluster)
    Standby,
}

impl AutoClusterManager {
    /// Create a new auto cluster manager
    pub fn new(
        config: AutoNetworkConfig,
        local_node: DiscoveredNode,
        discovered_nodes: Arc<RwLock<HashMap<String, DiscoveredNode>>>,
    ) -> Self {
        let initial_topology = ClusterTopology {
            clusters: vec![],
            connections: HashMap::new(),
            node_roles: HashMap::new(),
            last_updated: SystemTime::now(),
            version: 0,
        };

        Self {
            config,
            local_node,
            discovered_nodes,
            cluster_topology: Arc::new(RwLock::new(initial_topology)),
            running: Arc::new(RwLock::new(false)),
        }
    }

    /// Start the auto cluster management
    pub async fn start(&self) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
        info!(
            "Starting auto cluster management for node: {}",
            self.local_node.node_id
        );

        *self.running.write().await = true;

        // Start cluster formation process
        self.start_cluster_management_loop().await;

        // Start topology maintenance
        self.start_topology_maintenance().await;

        info!("Auto cluster management started");
        Ok(())
    }

    /// Stop auto cluster management
    pub async fn stop(&self) {
        info!("Stopping auto cluster management");
        *self.running.write().await = false;
        info!("Auto cluster management stopped");
    }

    /// Get current cluster topology
    pub async fn get_topology(&self) -> ClusterTopology {
        self.cluster_topology.read().await.clone()
    }

    /// Get recommended gossip peers for this node
    pub async fn get_gossip_peers(&self) -> Vec<DiscoveredNode> {
        let nodes = self.discovered_nodes.read().await;
        let topology = self.cluster_topology.read().await;

        let mut peers = Vec::new();

        // Get peers from our cluster
        if let Some(connections) = topology.connections.get(&self.local_node.node_id) {
            for peer_id in connections {
                if let Some(node) = nodes.get(peer_id) {
                    if node.is_fresh(Duration::from_secs(self.config.node_expiry)) {
                        peers.push(node.clone());
                    }
                }
            }
        }

        // If we don't have enough peers, add some from other clusters
        if peers.len() < 3 {
            let all_active: Vec<_> = nodes
                .values()
                .filter(|n| {
                    n.node_id != self.local_node.node_id
                        && n.is_fresh(Duration::from_secs(self.config.node_expiry))
                })
                .cloned()
                .collect();

            for node in all_active.into_iter().take(5 - peers.len()) {
                if !peers.iter().any(|p| p.node_id == node.node_id) {
                    peers.push(node);
                }
            }
        }

        debug!(
            "Selected {} gossip peers for node {}",
            peers.len(),
            self.local_node.node_id
        );
        peers
    }

    /// Start the cluster management loop
    async fn start_cluster_management_loop(&self) {
        let discovered_nodes = self.discovered_nodes.clone();
        let cluster_topology = self.cluster_topology.clone();
        let running = self.running.clone();
        let local_node_id = self.local_node.node_id.clone();
        let config = self.config.clone();

        tokio::spawn(async move {
            let mut interval = tokio::time::interval(Duration::from_secs(30)); // Check every 30 seconds

            loop {
                interval.tick().await;

                if !*running.read().await {
                    break;
                }

                // Update cluster topology
                if let Err(e) = Self::update_cluster_topology(
                    &discovered_nodes,
                    &cluster_topology,
                    &local_node_id,
                    &config,
                )
                .await
                {
                    error!("Failed to update cluster topology: {}", e);
                }
            }
        });
    }

    /// Start topology maintenance tasks
    async fn start_topology_maintenance(&self) {
        let cluster_topology = self.cluster_topology.clone();
        let running = self.running.clone();

        tokio::spawn(async move {
            let mut interval = tokio::time::interval(Duration::from_secs(60)); // Maintain every minute

            loop {
                interval.tick().await;

                if !*running.read().await {
                    break;
                }

                // Perform topology maintenance
                Self::maintain_topology(&cluster_topology).await;
            }
        });
    }

    /// Update cluster topology based on discovered nodes
    async fn update_cluster_topology(
        discovered_nodes: &Arc<RwLock<HashMap<String, DiscoveredNode>>>,
        cluster_topology: &Arc<RwLock<ClusterTopology>>,
        local_node_id: &str,
        config: &AutoNetworkConfig,
    ) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
        let nodes = discovered_nodes.read().await;
        let active_nodes: Vec<_> = nodes
            .values()
            .filter(|n| n.is_fresh(Duration::from_secs(config.node_expiry)))
            .cloned()
            .collect();

        drop(nodes);

        if active_nodes.is_empty() {
            return Ok(());
        }

        debug!(
            "Updating cluster topology with {} active nodes",
            active_nodes.len()
        );

        // Create new topology based on active nodes
        let new_topology =
            Self::create_optimal_topology(&active_nodes, local_node_id, config).await?;

        // Update the topology
        let mut topology = cluster_topology.write().await;
        *topology = new_topology;

        debug!(
            "Cluster topology updated with {} clusters",
            topology.clusters.len()
        );
        Ok(())
    }

    /// Create optimal cluster topology
    async fn create_optimal_topology(
        active_nodes: &[DiscoveredNode],
        local_node_id: &str,
        config: &AutoNetworkConfig,
    ) -> Result<ClusterTopology, Box<dyn std::error::Error + Send + Sync>> {
        let node_count = active_nodes.len();

        // Determine optimal cluster configuration
        let (cluster_count, nodes_per_cluster) = Self::calculate_cluster_params(node_count, config);

        info!(
            "Creating topology: {} nodes -> {} clusters (~{} nodes each)",
            node_count, cluster_count, nodes_per_cluster
        );

        let mut clusters = Vec::new();
        let mut connections = HashMap::new();
        let mut node_roles = HashMap::new();

        if cluster_count == 1 {
            // Single cluster - all nodes connected
            let cluster = Self::create_single_cluster(active_nodes, local_node_id)?;

            // All nodes connect to all other nodes (full mesh for small clusters)
            for node in active_nodes {
                let peers: Vec<String> = active_nodes
                    .iter()
                    .filter(|n| n.node_id != node.node_id)
                    .map(|n| n.node_id.clone())
                    .collect();
                connections.insert(node.node_id.clone(), peers);

                // Assign roles
                let role = if node.node_id == local_node_id || node.node_id == cluster.center {
                    NodeRole::Seed
                } else {
                    NodeRole::Member
                };
                node_roles.insert(node.node_id.clone(), role);
            }

            clusters.push(cluster);
        } else {
            // Multiple clusters with bridges
            let (created_clusters, cluster_connections, roles) =
                Self::create_multi_cluster_topology(
                    active_nodes,
                    cluster_count,
                    nodes_per_cluster,
                    local_node_id,
                )?;

            clusters = created_clusters;
            connections = cluster_connections;
            node_roles = roles;
        }

        Ok(ClusterTopology {
            clusters,
            connections,
            node_roles,
            last_updated: SystemTime::now(),
            version: SystemTime::now()
                .duration_since(SystemTime::UNIX_EPOCH)
                .unwrap_or_default()
                .as_secs(),
        })
    }

    /// Calculate optimal cluster parameters
    fn calculate_cluster_params(node_count: usize, config: &AutoNetworkConfig) -> (usize, usize) {
        let max_cluster_size = config.max_nodes.min(8); // Limit cluster size for efficiency

        if node_count <= max_cluster_size {
            (1, node_count)
        } else {
            let cluster_count = node_count.div_ceil(max_cluster_size);
            let nodes_per_cluster = node_count.div_ceil(cluster_count);
            (cluster_count, nodes_per_cluster)
        }
    }

    /// Create a single cluster with all nodes
    fn create_single_cluster(
        nodes: &[DiscoveredNode],
        local_node_id: &str,
    ) -> Result<Cluster, Box<dyn std::error::Error + Send + Sync>> {
        let cluster_id = "cluster-main".to_string();
        let node_ids: Vec<String> = nodes.iter().map(|n| n.node_id.clone()).collect();

        // Use local node as center if present, otherwise first node
        let center = if node_ids.contains(&local_node_id.to_string()) {
            local_node_id.to_string()
        } else {
            nodes.first().map(|n| n.node_id.clone()).unwrap_or_default()
        };

        Ok(Cluster {
            id: cluster_id,
            nodes: node_ids,
            center,
            created_at: SystemTime::now(),
            healthy: true,
        })
    }

    /// Create multi-cluster topology with bridges
    fn create_multi_cluster_topology(
        nodes: &[DiscoveredNode],
        cluster_count: usize,
        nodes_per_cluster: usize,
        local_node_id: &str,
    ) -> ClusterTopologyResult {
        let mut clusters = Vec::new();
        let mut connections = HashMap::new();
        let mut node_roles = HashMap::new();

        // Distribute nodes across clusters
        let mut node_index = 0;
        for cluster_idx in 0..cluster_count {
            let cluster_id = format!("cluster-{cluster_idx}");
            let mut cluster_nodes = Vec::new();

            // Assign nodes to this cluster
            let cluster_size = if cluster_idx == cluster_count - 1 {
                // Last cluster gets remaining nodes
                nodes.len() - node_index
            } else {
                nodes_per_cluster
            };

            for _ in 0..cluster_size {
                if node_index < nodes.len() {
                    cluster_nodes.push(nodes[node_index].node_id.clone());
                    node_index += 1;
                }
            }

            if cluster_nodes.is_empty() {
                continue;
            }

            // Select cluster center (prefer local node if in this cluster)
            let center = if cluster_nodes.contains(&local_node_id.to_string()) {
                local_node_id.to_string()
            } else {
                cluster_nodes[0].clone()
            };

            let cluster = Cluster {
                id: cluster_id,
                nodes: cluster_nodes.clone(),
                center: center.clone(),
                created_at: SystemTime::now(),
                healthy: true,
            };

            // Create intra-cluster connections (each node connects to center + 2 neighbors)
            for (i, node_id) in cluster_nodes.iter().enumerate() {
                let mut peers = Vec::new();

                // Connect to cluster center
                if node_id != &center {
                    peers.push(center.clone());
                }

                // Connect to neighbors
                let prev_idx = if i == 0 {
                    cluster_nodes.len() - 1
                } else {
                    i - 1
                };
                let next_idx = (i + 1) % cluster_nodes.len();

                if cluster_nodes.len() > 2 {
                    if cluster_nodes[prev_idx] != *node_id
                        && !peers.contains(&cluster_nodes[prev_idx])
                    {
                        peers.push(cluster_nodes[prev_idx].clone());
                    }
                    if cluster_nodes[next_idx] != *node_id
                        && !peers.contains(&cluster_nodes[next_idx])
                    {
                        peers.push(cluster_nodes[next_idx].clone());
                    }
                }

                connections.insert(node_id.clone(), peers);

                // Assign node role
                let role = if node_id == &center {
                    NodeRole::Seed
                } else {
                    NodeRole::Member
                };
                node_roles.insert(node_id.clone(), role);
            }

            clusters.push(cluster);
        }

        // Create inter-cluster bridges
        Self::create_inter_cluster_bridges(&mut connections, &mut node_roles, &clusters)?;

        Ok((clusters, connections, node_roles))
    }

    /// Create bridges between clusters
    fn create_inter_cluster_bridges(
        connections: &mut HashMap<String, Vec<String>>,
        node_roles: &mut HashMap<String, NodeRole>,
        clusters: &[Cluster],
    ) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
        if clusters.len() < 2 {
            return Ok(());
        }

        // Connect cluster centers to each other for inter-cluster communication
        let cluster_centers: Vec<String> = clusters.iter().map(|c| c.center.clone()).collect();

        for cluster in clusters {
            let center = &cluster.center;

            // Add connections to other cluster centers
            if !connections.contains_key(center) {
                connections.insert(center.clone(), Vec::new());
            }
            let current_connections = connections.get_mut(center).unwrap();

            for other_center in &cluster_centers {
                if other_center != center && !current_connections.contains(other_center) {
                    current_connections.push(other_center.clone());
                }
            }

            // Mark cluster centers as bridge nodes
            node_roles.insert(center.clone(), NodeRole::Bridge);
        }

        debug!(
            "Created inter-cluster bridges between {} clusters",
            clusters.len()
        );
        Ok(())
    }

    /// Maintain topology health
    async fn maintain_topology(cluster_topology: &Arc<RwLock<ClusterTopology>>) {
        let mut topology = cluster_topology.write().await;

        // Check cluster health
        for cluster in &mut topology.clusters {
            // Simple health check - cluster is healthy if it has at least one node
            cluster.healthy = !cluster.nodes.is_empty();
        }

        // Remove empty clusters
        topology.clusters.retain(|c| c.healthy);

        // Clean up connections to removed nodes
        let all_node_ids: HashSet<String> = topology
            .clusters
            .iter()
            .flat_map(|c| c.nodes.iter())
            .cloned()
            .collect();

        topology
            .connections
            .retain(|node_id, _| all_node_ids.contains(node_id));

        for peers in topology.connections.values_mut() {
            peers.retain(|peer_id| all_node_ids.contains(peer_id));
        }

        topology
            .node_roles
            .retain(|node_id, _| all_node_ids.contains(node_id));

        debug!("Topology maintenance completed");
    }

    /// Get cluster statistics
    pub async fn get_cluster_stats(&self) -> ClusterStats {
        let topology = self.cluster_topology.read().await;
        let nodes = self.discovered_nodes.read().await;

        let total_nodes = nodes.len();
        let active_nodes = nodes
            .values()
            .filter(|n| n.is_fresh(Duration::from_secs(self.config.node_expiry)))
            .count();

        let cluster_count = topology.clusters.len();
        let healthy_clusters = topology.clusters.iter().filter(|c| c.healthy).count();

        ClusterStats {
            total_nodes,
            active_nodes,
            cluster_count,
            healthy_clusters,
            average_cluster_size: if cluster_count > 0 {
                topology
                    .clusters
                    .iter()
                    .map(|c| c.nodes.len())
                    .sum::<usize>() as f64
                    / cluster_count as f64
            } else {
                0.0
            },
            topology_version: topology.version,
            last_updated: topology.last_updated,
        }
    }
}

/// Cluster statistics
#[derive(Debug, Clone, Serialize)]
pub struct ClusterStats {
    pub total_nodes: usize,
    pub active_nodes: usize,
    pub cluster_count: usize,
    pub healthy_clusters: usize,
    pub average_cluster_size: f64,
    pub topology_version: u64,
    pub last_updated: SystemTime,
}

/// Cluster optimization utilities
pub mod cluster_utils {
    use super::*;

    /// Find optimal gossip fanout for a cluster
    pub fn calculate_gossip_fanout(cluster_size: usize) -> u32 {
        match cluster_size {
            1 => 1,
            2..=4 => 2,
            5..=8 => 3,
            9..=16 => 4,
            _ => 5,
        }
    }

    /// Calculate expected message propagation time
    pub fn estimate_propagation_time(
        cluster_size: usize,
        gossip_interval_ms: u64,
        gossip_fanout: u32,
    ) -> Duration {
        if cluster_size <= 1 {
            return Duration::from_millis(0);
        }

        // Simple estimate: log base fanout of cluster size
        let rounds = ((cluster_size as f64).ln() / (gossip_fanout as f64).ln()).ceil() as u64;
        Duration::from_millis(rounds * gossip_interval_ms)
    }

    /// Check if topology is well-connected
    pub fn is_topology_connected(topology: &ClusterTopology) -> bool {
        if topology.clusters.is_empty() {
            return false;
        }

        // Check if all clusters have at least one node
        topology.clusters.iter().all(|c| !c.nodes.is_empty())
    }

    /// Find bridge nodes in the topology
    pub fn find_bridge_nodes(topology: &ClusterTopology) -> Vec<String> {
        topology
            .node_roles
            .iter()
            .filter_map(|(node_id, role)| {
                if *role == NodeRole::Bridge {
                    Some(node_id.clone())
                } else {
                    None
                }
            })
            .collect()
    }
}
