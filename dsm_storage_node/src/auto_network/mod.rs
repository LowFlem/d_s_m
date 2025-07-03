pub mod auto_cluster;
pub mod config_generator;
/// Auto Network Configuration Module
///
/// This module provides automatic network discovery and configuration for DSM nodes.
/// It eliminates the need for manual network setup by providing:
/// - mDNS/Bonjour service discovery
/// - Dynamic configuration generation
/// - Mobile device discovery
/// - Automatic gossip network formation
pub mod discovery;
pub mod mobile_discovery;

use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::net::IpAddr;
use std::sync::Arc;
use std::time::{Duration, SystemTime};
use tokio::sync::RwLock;
use tracing::{debug, info};

/// Discovered network node information
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DiscoveredNode {
    /// Node unique identifier
    pub node_id: String,
    /// Node name/hostname
    pub name: String,
    /// IP address
    pub ip: IpAddr,
    /// Port number
    pub port: u16,
    /// Service type (e.g., "dsm-storage", "dsm-client")
    pub service_type: String,
    /// Additional service properties
    pub properties: HashMap<String, String>,
    /// When this node was first discovered
    pub discovered_at: SystemTime,
    /// Last seen timestamp
    pub last_seen: SystemTime,
    /// Node capabilities
    pub capabilities: Vec<String>,
}

impl DiscoveredNode {
    /// Get the full endpoint URL for this node
    pub fn endpoint(&self) -> String {
        format!("http://{}:{}", self.ip, self.port)
    }

    /// Check if this node is still fresh (seen recently)
    pub fn is_fresh(&self, max_age: Duration) -> bool {
        self.last_seen.elapsed().unwrap_or(Duration::MAX) < max_age
    }

    /// Update last seen timestamp
    pub fn update_last_seen(&mut self) {
        self.last_seen = SystemTime::now();
    }
}

/// Network discovery configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AutoNetworkConfig {
    /// Enable automatic service discovery
    pub enable_discovery: bool,
    /// Service name for mDNS registration
    pub service_name: String,
    /// Service type for mDNS (e.g., "_dsm-storage._tcp")
    pub service_type: String,
    /// Discovery interval in seconds
    pub discovery_interval: u64,
    /// Node expiry time in seconds
    pub node_expiry: u64,
    /// Auto-generate configuration files
    pub auto_generate_configs: bool,
    /// Base port for auto-assigned ports
    pub base_port: u16,
    /// Maximum number of nodes to discover
    pub max_nodes: usize,
    /// Enable mobile discovery API
    pub enable_mobile_api: bool,
    /// Mobile API port
    pub mobile_api_port: u16,
}

impl Default for AutoNetworkConfig {
    fn default() -> Self {
        Self {
            enable_discovery: true,
            service_name: "DSM Storage Node".to_string(),
            service_type: "_dsm-storage._tcp".to_string(),
            discovery_interval: 30,
            node_expiry: 300, // 5 minutes
            auto_generate_configs: true,
            base_port: 8080,
            max_nodes: 10,
            enable_mobile_api: true,
            mobile_api_port: 9090,
        }
    }
}

/// Main auto-network manager
pub struct AutoNetworkManager {
    config: AutoNetworkConfig,
    discovered_nodes: Arc<RwLock<HashMap<String, DiscoveredNode>>>,
    local_node_info: Option<DiscoveredNode>,
    discovery_service: Option<Arc<discovery::DiscoveryService>>,
    mobile_api: Option<Arc<mobile_discovery::MobileDiscoveryApi>>,
}

impl AutoNetworkManager {
    /// Create a new auto-network manager
    pub fn new(config: AutoNetworkConfig) -> Self {
        Self {
            config,
            discovered_nodes: Arc::new(RwLock::new(HashMap::new())),
            local_node_info: None,
            discovery_service: None,
            mobile_api: None,
        }
    }

    /// Initialize the auto-network system
    pub async fn initialize(
        &mut self,
        node_id: String,
        port: u16,
    ) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
        info!("Initializing auto-network system for node: {}", node_id);

        // Get local IP address
        let local_ip = self.get_local_ip().await?;

        // Create local node info
        let local_node = DiscoveredNode {
            node_id: node_id.clone(),
            name: format!("DSM-Node-{node_id}"),
            ip: local_ip,
            port,
            service_type: "dsm-storage".to_string(),
            properties: {
                let mut props = HashMap::new();
                props.insert("version".to_string(), "1.0.0".to_string());
                props.insert("capabilities".to_string(), "storage,mpc,gossip".to_string());
                props
            },
            discovered_at: SystemTime::now(),
            last_seen: SystemTime::now(),
            capabilities: vec![
                "storage".to_string(),
                "mpc".to_string(),
                "gossip".to_string(),
            ],
        };

        self.local_node_info = Some(local_node.clone());

        // Initialize discovery service
        if self.config.enable_discovery {
            let discovery = Arc::new(
                discovery::DiscoveryService::new(
                    self.config.clone(),
                    local_node,
                    self.discovered_nodes.clone(),
                )
                .await?,
            );

            discovery.start().await?;
            self.discovery_service = Some(discovery);
            info!("Service discovery started");
        }

        // Initialize mobile discovery API
        if self.config.enable_mobile_api {
            let mobile_api = Arc::new(
                mobile_discovery::MobileDiscoveryApi::new(
                    self.config.mobile_api_port,
                    self.discovered_nodes.clone(),
                )
                .await?,
            );

            mobile_api.start().await?;
            self.mobile_api = Some(mobile_api);
            info!(
                "Mobile discovery API started on port {}",
                self.config.mobile_api_port
            );
        }

        // Start the cleanup task
        self.start_cleanup_task().await;

        info!("Auto-network system initialized successfully");
        Ok(())
    }

    /// Get discovered nodes
    pub async fn get_discovered_nodes(&self) -> Vec<DiscoveredNode> {
        let nodes = self.discovered_nodes.read().await;
        nodes.values().cloned().collect()
    }

    /// Get nodes suitable for gossip (excluding self)
    pub async fn get_gossip_peers(&self) -> Vec<DiscoveredNode> {
        let nodes = self.discovered_nodes.read().await;
        let local_id = self.local_node_info.as_ref().map(|n| &n.node_id);

        nodes
            .values()
            .filter(|node| {
                // Exclude self and ensure node is fresh
                Some(&node.node_id) != local_id
                    && node.is_fresh(Duration::from_secs(self.config.node_expiry))
            })
            .cloned()
            .collect()
    }

    /// Generate automatic configuration
    pub async fn generate_auto_config(
        &self,
    ) -> Result<String, Box<dyn std::error::Error + Send + Sync>> {
        let nodes = self.get_discovered_nodes().await;
        config_generator::generate_node_config(&self.config, &nodes, self.local_node_info.as_ref())
            .await
    }

    /// Get local IP address automatically
    async fn get_local_ip(&self) -> Result<IpAddr, Box<dyn std::error::Error + Send + Sync>> {
        // Try to get the primary network interface IP
        use std::net::UdpSocket;

        // Connect to a dummy address to determine local IP
        let socket = UdpSocket::bind("0.0.0.0:0")?;
        socket.connect("8.8.8.8:80")?;
        let local_addr = socket.local_addr()?;

        Ok(local_addr.ip())
    }

    /// Start cleanup task to remove stale nodes
    async fn start_cleanup_task(&self) {
        let discovered_nodes = self.discovered_nodes.clone();
        let expiry_duration = Duration::from_secs(self.config.node_expiry);

        tokio::spawn(async move {
            let mut interval = tokio::time::interval(Duration::from_secs(60)); // Cleanup every minute

            loop {
                interval.tick().await;

                let mut nodes = discovered_nodes.write().await;
                let before_count = nodes.len();

                // Remove expired nodes
                nodes.retain(|_, node| node.is_fresh(expiry_duration));

                let after_count = nodes.len();
                if before_count != after_count {
                    debug!("Cleaned up {} expired nodes", before_count - after_count);
                }
            }
        });
    }

    /// Shutdown the auto-network system
    pub async fn shutdown(&mut self) {
        info!("Shutting down auto-network system");

        if let Some(discovery) = &self.discovery_service {
            discovery.stop().await;
        }

        if let Some(mobile_api) = &self.mobile_api {
            mobile_api.stop().await;
        }

        info!("Auto-network system shutdown complete");
    }
}

/// Utility functions for network operations
pub mod utils {
    use std::net::IpAddr;

    /// Check if an IP address is on the local network
    pub fn is_local_network(ip: &IpAddr) -> bool {
        match ip {
            IpAddr::V4(ipv4) => {
                let octets = ipv4.octets();
                // Check for private network ranges
                matches!(octets[0], 10) ||                           // 10.0.0.0/8
                (octets[0] == 172 && octets[1] >= 16 && octets[1] <= 31) || // 172.16.0.0/12
                (octets[0] == 192 && octets[1] == 168) ||            // 192.168.0.0/16
                (octets[0] == 127) // 127.0.0.0/8 (localhost)
            }
            IpAddr::V6(_) => false, // Simplification - only handle IPv4 for now
        }
    }

    /// Generate a unique node ID based on machine characteristics
    pub fn generate_node_id() -> String {
        use blake3::Hasher;
        use std::time::{SystemTime, UNIX_EPOCH};

        let mut hasher = Hasher::new();

        // Add hostname
        if let Ok(hostname) = hostname::get() {
            if let Ok(hostname_str) = hostname.into_string() {
                hasher.update(hostname_str.as_bytes());
            }
        }

        // Add current time (for uniqueness)
        let now = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap()
            .as_nanos();
        hasher.update(&now.to_le_bytes());

        // Add some randomness
        use rand::RngCore;
        let mut rng = rand::thread_rng();
        let random_bytes: [u8; 16] = {
            let mut bytes = [0u8; 16];
            rng.fill_bytes(&mut bytes);
            bytes
        };
        hasher.update(&random_bytes);

        let hash = hasher.finalize();
        hex::encode(&hash.as_bytes()[..8]) // Use first 8 bytes as node ID
    }

    /// Find available port starting from base_port
    pub async fn find_available_port(base_port: u16) -> Result<u16, std::io::Error> {
        use tokio::net::TcpListener;

        for port in base_port..base_port + 100 {
            if let Ok(listener) = TcpListener::bind(format!("0.0.0.0:{port}")).await {
                drop(listener);
                return Ok(port);
            }
        }

        Err(std::io::Error::new(
            std::io::ErrorKind::AddrInUse,
            "No available ports found",
        ))
    }
}

pub use auto_cluster::AutoClusterManager;
pub use config_generator::{generate_mobile_config, generate_node_config};
/// Re-export commonly used types
pub use discovery::DiscoveryService;
pub use mobile_discovery::MobileDiscoveryApi;
