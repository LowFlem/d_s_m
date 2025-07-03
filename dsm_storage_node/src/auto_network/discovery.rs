/// Service Discovery Module
///
/// Provides automatic service discovery using mDNS/Bonjour for DSM storage nodes.
/// This allows nodes to automatically find each other on the local network without
/// manual configuration.
use super::{AutoNetworkConfig, DiscoveredNode};
use std::collections::HashMap;
use std::net::IpAddr;
use std::sync::Arc;
use std::time::{Duration, SystemTime};
use tokio::sync::RwLock;
use tokio::time::interval;
use tracing::{debug, info, warn};

/// Service discovery implementation
pub struct DiscoveryService {
    config: AutoNetworkConfig,
    local_node: DiscoveredNode,
    discovered_nodes: Arc<RwLock<HashMap<String, DiscoveredNode>>>,
    running: Arc<RwLock<bool>>,
    mdns_service: Option<MdnsService>,
}

impl DiscoveryService {
    /// Create a new discovery service
    pub async fn new(
        config: AutoNetworkConfig,
        local_node: DiscoveredNode,
        discovered_nodes: Arc<RwLock<HashMap<String, DiscoveredNode>>>,
    ) -> Result<Self, Box<dyn std::error::Error + Send + Sync>> {
        let mdns_service = MdnsService::new(&config, &local_node).await?;

        Ok(Self {
            config,
            local_node,
            discovered_nodes,
            running: Arc::new(RwLock::new(false)),
            mdns_service: Some(mdns_service),
        })
    }

    /// Start the discovery service
    pub async fn start(&self) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
        info!(
            "Starting service discovery for node: {}",
            self.local_node.node_id
        );

        *self.running.write().await = true;

        // Start mDNS service
        if let Some(mdns) = &self.mdns_service {
            mdns.start().await?;
        }

        // Start discovery loop
        self.start_discovery_loop().await;

        info!("Service discovery started successfully");
        Ok(())
    }

    /// Stop the discovery service
    pub async fn stop(&self) {
        info!("Stopping service discovery");

        *self.running.write().await = false;

        // Stop mDNS service
        if let Some(mdns) = &self.mdns_service {
            mdns.stop().await;
        }

        info!("Service discovery stopped");
    }

    /// Start the discovery loop
    async fn start_discovery_loop(&self) {
        let discovered_nodes = self.discovered_nodes.clone();
        let running = self.running.clone();
        let discovery_interval = self.config.discovery_interval;
        let local_node_id = self.local_node.node_id.clone();

        tokio::spawn(async move {
            let mut interval = interval(Duration::from_secs(discovery_interval));

            loop {
                interval.tick().await;

                // Check if we should continue running
                if !*running.read().await {
                    break;
                }

                // Perform network scan
                match Self::scan_network().await {
                    Ok(found_nodes) => {
                        let mut nodes = discovered_nodes.write().await;

                        for node in found_nodes {
                            // Don't add ourselves
                            if node.node_id == local_node_id {
                                continue;
                            }

                            // Update existing node or add new one
                            if let Some(existing) = nodes.get_mut(&node.node_id) {
                                existing.update_last_seen();
                                existing.ip = node.ip;
                                existing.port = node.port;
                                existing.properties = node.properties;
                            } else {
                                info!(
                                    "Discovered new node: {} at {}:{}",
                                    node.node_id, node.ip, node.port
                                );
                                nodes.insert(node.node_id.clone(), node);
                            }
                        }
                    }
                    Err(e) => {
                        warn!("Network scan failed: {}", e);
                    }
                }
            }
        });
    }

    /// Scan the network for DSM nodes
    async fn scan_network() -> Result<Vec<DiscoveredNode>, Box<dyn std::error::Error + Send + Sync>>
    {
        let mut found_nodes = Vec::new();

        // Scan common ports for DSM services
        let base_ports = [8080, 8081, 8082, 8083, 8084, 9090, 9091, 9092];
        let local_networks = Self::get_local_networks().await?;

        for network in local_networks {
            for port in base_ports {
                if let Ok(nodes) = Self::scan_network_range(&network, port).await {
                    found_nodes.extend(nodes);
                }
            }
        }

        Ok(found_nodes)
    }

    /// Get local network ranges to scan
    async fn get_local_networks() -> Result<Vec<String>, Box<dyn std::error::Error + Send + Sync>> {
        let mut networks = Vec::new();

        // Get all network interfaces
        match get_if_addrs::get_if_addrs() {
            Ok(interfaces) => {
                for interface in interfaces {
                    if !interface.is_loopback() {
                        if let IpAddr::V4(ipv4) = interface.ip() {
                            let octets = ipv4.octets();
                            // Generate network range based on common subnet masks
                            if octets[0] == 192 && octets[1] == 168 {
                                // /24 network
                                networks.push(format!("192.168.{}.0/24", octets[2]));
                            } else if octets[0] == 10 {
                                // /24 network within 10.x.x.x
                                networks.push(format!("10.{}.{}.0/24", octets[1], octets[2]));
                            } else if octets[0] == 172 && octets[1] >= 16 && octets[1] <= 31 {
                                // /24 network within 172.16-31.x.x
                                networks.push(format!("172.{}.{}.0/24", octets[1], octets[2]));
                            }
                        }
                        // Skip IPv6 for now
                    }
                }
            }
            Err(e) => {
                warn!("Failed to get network interfaces: {}", e);
                // Fallback to common networks
                networks.push("192.168.1.0/24".to_string());
                networks.push("192.168.0.0/24".to_string());
            }
        }

        if networks.is_empty() {
            // Final fallback
            networks.push("192.168.1.0/24".to_string());
        }

        debug!("Scanning networks: {:?}", networks);
        Ok(networks)
    }

    /// Scan a specific network range and port
    async fn scan_network_range(
        network: &str,
        port: u16,
    ) -> Result<Vec<DiscoveredNode>, Box<dyn std::error::Error + Send + Sync>> {
        let mut found_nodes = Vec::new();

        // Parse network (simplified - only handle /24 subnets)
        if let Some(base) = network.strip_suffix("/24") {
            let base_parts: Vec<&str> = base.split('.').collect();
            if base_parts.len() == 4 {
                let base_ip = format!("{}.{}.{}.", base_parts[0], base_parts[1], base_parts[2]);

                // Scan first 50 IPs in the range (to avoid excessive scanning)
                let mut scan_tasks = Vec::new();

                for i in 1..=50 {
                    let ip = format!("{base_ip}{i}");
                    let scan_task = Self::check_dsm_service(ip, port);
                    scan_tasks.push(scan_task);
                }

                // Execute scans with limited concurrency
                let results = futures::future::join_all(scan_tasks).await;

                for result in results {
                    if let Ok(Some(node)) = result {
                        found_nodes.push(node);
                    }
                }
            }
        }

        Ok(found_nodes)
    }

    /// Check if a specific IP:port hosts a DSM service
    async fn check_dsm_service(
        ip: String,
        port: u16,
    ) -> Result<Option<DiscoveredNode>, Box<dyn std::error::Error + Send + Sync>> {
        let timeout_duration = Duration::from_secs(2);
        let url = format!("http://{ip}:{port}/api/v1/status");

        // Create HTTP client with timeout
        let client = reqwest::Client::builder()
            .timeout(timeout_duration)
            .build()?;

        match client.get(&url).send().await {
            Ok(response) => {
                if response.status().is_success() {
                    // Try to parse the response to confirm it's a DSM node
                    if let Ok(text) = response.text().await {
                        if let Ok(status) = serde_json::from_str::<serde_json::Value>(&text) {
                            // Extract node information from status response
                            let node_id = status
                                .get("node_id")
                                .and_then(|v| v.as_str())
                                .unwrap_or(&format!("unknown-{ip}-{port}"))
                                .to_string();

                            let ip_addr: IpAddr = ip.parse()?;

                            let mut properties = HashMap::new();
                            if let Some(version) = status.get("version").and_then(|v| v.as_str()) {
                                properties.insert("version".to_string(), version.to_string());
                            }
                            if let Some(uptime) = status.get("uptime").and_then(|v| v.as_u64()) {
                                properties.insert("uptime".to_string(), uptime.to_string());
                            }

                            let node = DiscoveredNode {
                                node_id,
                                name: format!("DSM-Node-{ip}"),
                                ip: ip_addr,
                                port,
                                service_type: "dsm-storage".to_string(),
                                properties,
                                discovered_at: SystemTime::now(),
                                last_seen: SystemTime::now(),
                                capabilities: vec!["storage".to_string(), "mpc".to_string()],
                            };

                            debug!("Found DSM service at {}:{}", ip, port);
                            return Ok(Some(node));
                        }
                    }
                }
            }
            Err(_) => {
                // Service not available or not a DSM node - this is normal
            }
        }

        Ok(None)
    }
}

/// mDNS service for registration and discovery
struct MdnsService {
    service_name: String,
    service_type: String,
    #[allow(dead_code)]
    port: u16,
    #[allow(dead_code)]
    properties: HashMap<String, String>,
}

impl MdnsService {
    async fn new(
        config: &AutoNetworkConfig,
        local_node: &DiscoveredNode,
    ) -> Result<Self, Box<dyn std::error::Error + Send + Sync>> {
        Ok(Self {
            service_name: format!("{}-{}", config.service_name, local_node.node_id),
            service_type: config.service_type.clone(),
            port: local_node.port,
            properties: local_node.properties.clone(),
        })
    }

    async fn start(&self) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
        info!(
            "Starting mDNS service registration: {} ({})",
            self.service_name, self.service_type
        );

        // Note: In a full implementation, you would use a proper mDNS library like mdns-sd
        // For now, we'll use the HTTP-based discovery implemented above

        // This is where you would register the service with mDNS:
        // let mdns = ServiceDaemon::new()?;
        // let service_info = ServiceInfo::new(
        //     &self.service_type,
        //     &self.service_name,
        //     "localhost",
        //     self.port,
        //     &self.properties,
        // )?;
        // mdns.register(service_info)?;

        info!("mDNS service registration simulated (HTTP discovery active)");
        Ok(())
    }

    async fn stop(&self) {
        info!("Stopping mDNS service: {}", self.service_name);
        // In a full implementation, you would unregister the service
    }
}

/// Simple network scanner for discovering services
pub struct NetworkScanner;

impl NetworkScanner {
    /// Discover DSM nodes on the local network
    pub async fn discover_nodes(
    ) -> Result<Vec<DiscoveredNode>, Box<dyn std::error::Error + Send + Sync>> {
        info!("Starting network scan for DSM nodes");

        let mut discovered = Vec::new();

        // Get local IP to determine scanning range
        let local_ip = Self::get_local_ip().await?;
        info!("Local IP: {}", local_ip);

        // Determine network range based on local IP
        let scan_range = Self::get_scan_range(&local_ip);
        info!("Scanning range: {}", scan_range);

        // Scan common DSM ports
        let ports = [8080, 8081, 8082, 8083, 8084];

        for port in ports {
            if let Ok(nodes) = Self::scan_range(&scan_range, port).await {
                discovered.extend(nodes);
            }
        }

        info!("Network scan completed, found {} nodes", discovered.len());
        Ok(discovered)
    }

    async fn get_local_ip() -> Result<IpAddr, Box<dyn std::error::Error + Send + Sync>> {
        use std::net::UdpSocket;

        let socket = UdpSocket::bind("0.0.0.0:0")?;
        socket.connect("8.8.8.8:80")?;
        Ok(socket.local_addr()?.ip())
    }

    fn get_scan_range(local_ip: &IpAddr) -> String {
        match local_ip {
            IpAddr::V4(ipv4) => {
                let octets = ipv4.octets();
                format!("{}.{}.{}.0/24", octets[0], octets[1], octets[2])
            }
            IpAddr::V6(_) => "::1/128".to_string(), // Simplified IPv6
        }
    }

    async fn scan_range(
        range: &str,
        port: u16,
    ) -> Result<Vec<DiscoveredNode>, Box<dyn std::error::Error + Send + Sync>> {
        let mut nodes = Vec::new();

        // Simple range scanning (only handle IPv4 /24 for now)
        if let Some(base) = range.strip_suffix("/24") {
            let parts: Vec<&str> = base.split('.').collect();
            if parts.len() == 4 {
                let base_ip = format!("{}.{}.{}.", parts[0], parts[1], parts[2]);

                // Scan first 20 IPs to avoid excessive network traffic
                for i in 1..=20 {
                    let ip = format!("{base_ip}{i}");

                    // Quick connectivity check
                    if let Ok(Some(node)) = Self::check_node(&ip, port).await {
                        nodes.push(node);
                    }
                }
            }
        }

        Ok(nodes)
    }

    async fn check_node(
        ip: &str,
        port: u16,
    ) -> Result<Option<DiscoveredNode>, Box<dyn std::error::Error + Send + Sync>> {
        let url = format!("http://{ip}:{port}/api/v1/status");
        let client = reqwest::Client::builder()
            .timeout(Duration::from_secs(1))
            .build()?;

        match client.get(&url).send().await {
            Ok(response) if response.status().is_success() => {
                if let Ok(text) = response.text().await {
                    if let Ok(status) = serde_json::from_str::<serde_json::Value>(&text) {
                        let node_id = status
                            .get("node_id")
                            .and_then(|v| v.as_str())
                            .unwrap_or(&format!("node-{ip}-{port}"))
                            .to_string();

                        let node = DiscoveredNode {
                            node_id,
                            name: format!("DSM-{ip}"),
                            ip: ip.parse()?,
                            port,
                            service_type: "dsm-storage".to_string(),
                            properties: HashMap::new(),
                            discovered_at: SystemTime::now(),
                            last_seen: SystemTime::now(),
                            capabilities: vec!["storage".to_string()],
                        };

                        return Ok(Some(node));
                    }
                }
            }
            _ => {} // Not a DSM node or not available
        }

        Ok(None)
    }
}
