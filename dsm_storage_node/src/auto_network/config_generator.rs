/// Configuration Generator Module
///
/// Automatically generates configuration files for DSM nodes based on network discovery.
/// This eliminates the need for manual configuration by creating optimized configs
/// based on discovered network topology.
use super::{AutoNetworkConfig, DiscoveredNode};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use tracing::{debug, info, warn};

/// Configuration template for storage nodes
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NodeConfigTemplate {
    pub api: ApiConfig,
    pub node: NodeInfo,
    pub cluster: ClusterConfig,
    pub storage: StorageConfig,
    pub network: NetworkConfig,
    pub security: SecurityConfig,
    pub staking: StakingConfig,
    pub mpc: MpcConfig,
    pub logging: LoggingConfig,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ApiConfig {
    pub bind_address: String,
    pub port: u16,
    pub enable_cors: bool,
    pub cors_allow_origins: Vec<String>,
    pub enable_rate_limits: bool,
    pub max_body_size: u64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NodeInfo {
    pub id: String,
    pub name: String,
    pub region: String,
    pub operator: String,
    pub version: String,
    pub description: String,
    pub public_key: String,
    pub endpoint: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ClusterConfig {
    pub enabled: bool,
    pub clusters: Vec<String>,
    pub overlap_factor: u32,
    pub target_cluster_size: u32,
    pub min_cluster_size: u32,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct StorageConfig {
    pub engine: String,
    pub capacity: u64,
    pub data_dir: String,
    pub database_path: String,
    pub assignment_strategy: String,
    pub replication_strategy: String,
    pub replica_count: u32,
    pub min_regions: u32,
    pub default_ttl: u64,
    pub enable_pruning: bool,
    pub pruning_interval: u64,
    pub epidemic: EpidemicConfig,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EpidemicConfig {
    pub gossip_interval_ms: u64,
    pub reconciliation_interval_ms: u64,
    pub topology_maintenance_interval_ms: u64,
    pub gossip_fanout: u32,
    pub max_reconciliation_diff: u32,
    pub replication_factor: u32,
    pub k_neighbors: u32,
    pub alpha: f64,
    pub max_long_links: u32,
    pub max_topology_connections: u32,
    pub topology_connection_timeout_ms: u64,
    pub cleanup_interval_ms: u64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NetworkConfig {
    pub listen_addr: String,
    pub public_endpoint: String,
    pub port: u16,
    pub max_connections: u32,
    pub connection_timeout: u64,
    pub enable_discovery: bool,
    pub discovery_interval: u64,
    pub max_peers: u32,
    pub peers: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SecurityConfig {
    pub private_key_path: String,
    pub public_key_path: String,
    pub enable_tls: bool,
    pub require_auth: bool,
    pub enable_rate_limits: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct StakingConfig {
    pub enable_staking: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MpcConfig {
    pub enabled: bool,
    pub threshold: u32,
    pub max_participants: u32,
    pub session_timeout: u64,
    pub contribution_timeout: u64,
    pub enable_blind_signatures: bool,
    pub dbrw_enabled: bool,
    pub max_concurrent_sessions: u32,
    pub cleanup_interval: u64,
    pub participant_discovery_timeout: u64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LoggingConfig {
    pub level: String,
    pub file_path: String,
    pub format: String,
    pub console_logging: bool,
}

/// Generate node configuration based on discovery
pub async fn generate_node_config(
    auto_config: &AutoNetworkConfig,
    discovered_nodes: &[DiscoveredNode],
    local_node: Option<&DiscoveredNode>,
) -> Result<String, Box<dyn std::error::Error + Send + Sync>> {
    info!("Generating automatic node configuration");
    debug!(
        "Discovered {} nodes for configuration",
        discovered_nodes.len()
    );

    let local_node = local_node.ok_or("Local node information required")?;

    // Create optimized configuration based on network topology
    let config = create_optimized_config(auto_config, discovered_nodes, local_node).await?;

    // Convert to TOML format
    let toml_string = toml::to_string_pretty(&config)?;

    info!("Generated configuration for node: {}", local_node.node_id);
    Ok(add_config_header(&toml_string, local_node))
}

/// Create optimized configuration based on network discovery
async fn create_optimized_config(
    auto_config: &AutoNetworkConfig,
    discovered_nodes: &[DiscoveredNode],
    local_node: &DiscoveredNode,
) -> Result<NodeConfigTemplate, Box<dyn std::error::Error + Send + Sync>> {
    // Determine optimal cluster size based on discovered nodes
    let cluster_size = (discovered_nodes.len() + 1).min(auto_config.max_nodes);
    let replication_factor = determine_replication_factor(cluster_size);
    let mpc_threshold = determine_mpc_threshold(cluster_size);

    // Select peers for gossip (all other discovered nodes)
    let peers: Vec<String> = discovered_nodes
        .iter()
        .filter(|node| node.node_id != local_node.node_id)
        .map(|node| node.endpoint())
        .collect();

    // Determine storage engine based on cluster size
    let storage_engine = if cluster_size > 3 {
        "epidemic"
    } else {
        "sqlite"
    };

    let config = NodeConfigTemplate {
        api: ApiConfig {
            bind_address: "0.0.0.0".to_string(),
            port: local_node.port,
            enable_cors: true,
            cors_allow_origins: vec!["*".to_string()],
            enable_rate_limits: false,
            max_body_size: 52428800, // 50MB
        },

        node: NodeInfo {
            id: local_node.node_id.clone(),
            name: local_node.name.clone(),
            region: "auto-detected".to_string(),
            operator: "DSM Auto Config".to_string(),
            version: "1.0.0".to_string(),
            description: format!(
                "Auto-configured DSM node (discovered {} peers)",
                peers.len()
            ),
            public_key: "".to_string(), // Will be generated on startup
            endpoint: local_node.endpoint(),
        },

        cluster: ClusterConfig {
            enabled: cluster_size > 1,
            clusters: vec![],
            overlap_factor: 1,
            target_cluster_size: cluster_size as u32,
            min_cluster_size: 1,
        },

        storage: StorageConfig {
            engine: storage_engine.to_string(),
            capacity: 1073741824, // 1GB default
            data_dir: format!("./data-auto-{}", local_node.node_id),
            database_path: format!("./data-auto-{}/storage.db", local_node.node_id),
            assignment_strategy: "DeterministicHashing".to_string(),
            replication_strategy: "FixedReplicas".to_string(),
            replica_count: replication_factor,
            min_regions: 1,
            default_ttl: 0,
            enable_pruning: true,
            pruning_interval: 3600,
            epidemic: create_epidemic_config(cluster_size, replication_factor),
        },

        network: NetworkConfig {
            listen_addr: "0.0.0.0".to_string(),
            public_endpoint: local_node.endpoint(),
            port: local_node.port,
            max_connections: 100,
            connection_timeout: 300,
            enable_discovery: true,
            discovery_interval: auto_config.discovery_interval,
            max_peers: auto_config.max_nodes as u32,
            peers,
        },

        security: SecurityConfig {
            private_key_path: format!("./keys/auto-{}.key", local_node.node_id),
            public_key_path: format!("./keys/auto-{}.pub", local_node.node_id),
            enable_tls: false, // Disabled for local development
            require_auth: false,
            enable_rate_limits: false,
        },

        staking: StakingConfig {
            enable_staking: false,
        },

        mpc: MpcConfig {
            enabled: true,
            threshold: mpc_threshold,
            max_participants: cluster_size as u32,
            session_timeout: 300,
            contribution_timeout: 60,
            enable_blind_signatures: true,
            dbrw_enabled: true,
            max_concurrent_sessions: 10,
            cleanup_interval: 300,
            participant_discovery_timeout: 30,
        },

        logging: LoggingConfig {
            level: "info".to_string(),
            file_path: format!("./logs/auto-{}.log", local_node.node_id),
            format: "text".to_string(),
            console_logging: true,
        },
    };

    Ok(config)
}

/// Create epidemic storage configuration
fn create_epidemic_config(cluster_size: usize, replication_factor: u32) -> EpidemicConfig {
    // Optimize epidemic parameters based on cluster size
    let gossip_fanout = (cluster_size / 2).clamp(1, 5) as u32;
    let k_neighbors = (cluster_size / 3).clamp(1, 8) as u32;

    EpidemicConfig {
        gossip_interval_ms: 5000,
        reconciliation_interval_ms: 30000,
        topology_maintenance_interval_ms: 60000,
        gossip_fanout,
        max_reconciliation_diff: 1000,
        replication_factor,
        k_neighbors,
        alpha: 0.7,
        max_long_links: 10,
        max_topology_connections: 50,
        topology_connection_timeout_ms: 5000,
        cleanup_interval_ms: 300000,
    }
}

/// Determine optimal replication factor based on cluster size
fn determine_replication_factor(cluster_size: usize) -> u32 {
    match cluster_size {
        1 => 1,
        2..=3 => 2,
        4..=6 => 3,
        _ => (cluster_size / 2).min(5) as u32,
    }
}

/// Determine MPC threshold based on cluster size
fn determine_mpc_threshold(cluster_size: usize) -> u32 {
    match cluster_size {
        1 => 1,
        2..=3 => 2,
        4..=6 => 3,
        _ => ((cluster_size * 2) / 3).max(3) as u32,
    }
}

/// Add configuration header with metadata
fn add_config_header(toml_content: &str, local_node: &DiscoveredNode) -> String {
    let timestamp = chrono::Utc::now().format("%Y-%m-%d %H:%M:%S UTC");

    format!(
        r#"# DSM Auto-Generated Configuration
# Node ID: {}
# Generated: {}
# Auto-discovery enabled: Configuration will be updated automatically

{}
"#,
        local_node.node_id, timestamp, toml_content
    )
}

/// Generate mobile configuration for client applications
pub async fn generate_mobile_config(
    discovered_nodes: &[DiscoveredNode],
) -> Result<String, Box<dyn std::error::Error + Send + Sync>> {
    info!(
        "Generating mobile configuration for {} discovered nodes",
        discovered_nodes.len()
    );

    // Filter for active storage nodes
    let storage_nodes: Vec<&DiscoveredNode> = discovered_nodes
        .iter()
        .filter(|node| node.service_type == "dsm-storage")
        .collect();

    if storage_nodes.is_empty() {
        warn!("No storage nodes discovered for mobile configuration");
    }

    let mobile_config = serde_json::json!({
        "development": {
            "discovery_nodes": storage_nodes.iter().map(|n| n.endpoint()).collect::<Vec<_>>(),
            "connection_timeout": "30000",
            "bootstrap_node": storage_nodes.first().map(|n| n.endpoint()).unwrap_or("http://localhost:8080".to_string())
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
            "bootstrap_node": storage_nodes.first().map(|n| n.endpoint()).unwrap_or("http://localhost:8080".to_string()),
            "nodes": storage_nodes.iter().map(|n| n.endpoint()).collect::<Vec<_>>()
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
            "discovery_interval": 60,
            "node_refresh_interval": 300,
            "fallback_nodes": storage_nodes.iter().map(|n| n.endpoint()).collect::<Vec<_>>()
        },
        "metadata": {
            "generated_at": chrono::Utc::now().to_rfc3339(),
            "node_count": storage_nodes.len(),
            "auto_generated": true,
            "generator_version": "1.0.0"
        }
    });

    let json_string = serde_json::to_string_pretty(&mobile_config)?;
    info!(
        "Generated mobile configuration with {} storage nodes",
        storage_nodes.len()
    );

    Ok(json_string)
}

/// Generate environment configuration for development
pub async fn generate_env_config(
    discovered_nodes: &[DiscoveredNode],
) -> Result<String, Box<dyn std::error::Error + Send + Sync>> {
    info!("Generating environment configuration");

    let storage_nodes: Vec<&DiscoveredNode> = discovered_nodes
        .iter()
        .filter(|node| node.service_type == "dsm-storage")
        .collect();

    let env_config = serde_json::json!({
        "protocol": "http",
        "lan_ip": storage_nodes.first()
            .map(|n| n.ip.to_string())
            .unwrap_or("127.0.0.1".to_string()),
        "ports": storage_nodes.iter().map(|n| n.port).collect::<Vec<_>>(),
        "nodes": storage_nodes.iter().map(|node| {
            serde_json::json!({
                "name": node.name,
                "endpoint": node.endpoint(),
                "node_id": node.node_id,
                "capabilities": node.capabilities
            })
        }).collect::<Vec<_>>(),
        "cluster": {
            "size": storage_nodes.len(),
            "replication_factor": determine_replication_factor(storage_nodes.len()),
            "mpc_threshold": determine_mpc_threshold(storage_nodes.len()),
            "auto_generated": true
        }
    });

    let json_string = serde_json::to_string_pretty(&env_config)?;
    Ok(json_string)
}

/// Configuration writer for saving generated configs
pub struct ConfigWriter;

impl ConfigWriter {
    /// Write node configuration to file
    pub async fn write_node_config(
        node_id: &str,
        config_content: &str,
    ) -> Result<String, Box<dyn std::error::Error + Send + Sync>> {
        let config_dir = "./auto_configs";
        tokio::fs::create_dir_all(config_dir).await?;

        let config_path = format!("{config_dir}/config-auto-{node_id}.toml");
        tokio::fs::write(&config_path, config_content).await?;

        info!("Wrote node configuration to: {}", config_path);
        Ok(config_path)
    }

    /// Write mobile configuration to file
    pub async fn write_mobile_config(
        config_content: &str,
    ) -> Result<String, Box<dyn std::error::Error + Send + Sync>> {
        let config_dir = "./auto_configs";
        tokio::fs::create_dir_all(config_dir).await?;

        let config_path = format!("{config_dir}/mobile_auto_config.json");
        tokio::fs::write(&config_path, config_content).await?;

        info!("Wrote mobile configuration to: {}", config_path);
        Ok(config_path)
    }

    /// Write environment configuration to file
    pub async fn write_env_config(
        config_content: &str,
    ) -> Result<String, Box<dyn std::error::Error + Send + Sync>> {
        let config_path = "./dsm_env_config.json";
        tokio::fs::write(config_path, config_content).await?;

        info!("Wrote environment configuration to: {}", config_path);
        Ok(config_path.to_string())
    }

    /// Write all configurations
    pub async fn write_all_configs(
        auto_config: &AutoNetworkConfig,
        discovered_nodes: &[DiscoveredNode],
        local_node: &DiscoveredNode,
    ) -> Result<HashMap<String, String>, Box<dyn std::error::Error + Send + Sync>> {
        let mut written_files = HashMap::new();

        // Generate and write node config
        if let Ok(node_config) =
            generate_node_config(auto_config, discovered_nodes, Some(local_node)).await
        {
            if let Ok(path) = Self::write_node_config(&local_node.node_id, &node_config).await {
                written_files.insert("node_config".to_string(), path);
            }
        }

        // Generate and write mobile config
        if let Ok(mobile_config) = generate_mobile_config(discovered_nodes).await {
            if let Ok(path) = Self::write_mobile_config(&mobile_config).await {
                written_files.insert("mobile_config".to_string(), path);
            }
        }

        // Generate and write env config
        if let Ok(env_config) = generate_env_config(discovered_nodes).await {
            if let Ok(path) = Self::write_env_config(&env_config).await {
                written_files.insert("env_config".to_string(), path);
            }
        }

        info!("Generated {} configuration files", written_files.len());
        Ok(written_files)
    }
}
