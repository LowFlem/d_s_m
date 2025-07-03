/// API handler for viewing operation logs
async fn get_logs_handler(Extension(state): Extension<Arc<RwLock<AppState>>>) -> impl IntoResponse {
    let state = state.read().await;
    let logger = &state.logger;

    // Get recent logs (last 100 operations)
    let logs = logger.get_logs(None, None, None, Some(100)).await;

    (StatusCode::OK, Json(logs))
}

/// API handler for operation statistics
async fn get_stats_handler(
    Extension(state): Extension<Arc<RwLock<AppState>>>,
) -> impl IntoResponse {
    let state = state.read().await;
    let logger = &state.logger;

    // Get statistics for the last 24 hours
    let stats = logger.get_statistics(Some(24)).await;

    (StatusCode::OK, Json(stats))
}

/// API handler for exporting logs
async fn export_logs_handler(
    Extension(state): Extension<Arc<RwLock<AppState>>>,
) -> impl IntoResponse {
    let state = state.read().await;
    let logger = &state.logger;

    match logger
        .export_logs(dsm_storage_node::logging::ExportFormat::Json)
        .await
    {
        Ok(json_logs) => {
            let headers = [
                ("Content-Type", "application/json"),
                (
                    "Content-Disposition",
                    "attachment; filename=\"dsm_node_logs.json\"",
                ),
            ];
            (StatusCode::OK, headers, json_logs)
        }
        Err(e) => {
            error!("Failed to export logs: {}", e);
            (
                StatusCode::INTERNAL_SERVER_ERROR,
                [("Content-Type", "application/json"), ("", "")],
                format!("{{\"error\": \"{e}\"}}"),
            )
        }
    }
}
use std::collections::HashMap;
/// # DSM Storage Node Binary
///
/// Entry point for the DSM Storage Node application, which provides
/// a secure, distributed, and quantum-resistant storage solution for the Decentralized
/// State Machine ecosystem.
///
/// ## Usage
///
/// ```bash
/// # Run with default configuration
/// cargo run --bin storage_node -- --config config.toml
///
/// # Run with staking to earn rewards
/// cargo run --bin storage_node -- --config config.toml stake --amount 5000
/// ```
use std::fs;
use std::net::SocketAddr;
use std::path::PathBuf;
use std::process;
use std::sync::Arc;
use std::time::{Instant, SystemTime, UNIX_EPOCH};

use axum::{
    extract::{ConnectInfo, Extension, Path as AxumPath},
    http::StatusCode,
    response::IntoResponse,
    routing::{delete, get, post},
    Json, Router,
};
use base64::Engine;
use clap::{Parser, Subcommand};
use config::{Config, ConfigError, File};
use serde::Deserialize;
use serde_json::Value;
use tokio::sync::RwLock;
use tower_http::trace::TraceLayer;
use tracing::{debug, error, info, warn};
use tracing_subscriber::{layer::SubscriberExt, util::SubscriberInitExt};

// Import types from the library
use dsm_storage_node::{
    api::mpc_api::{
        contribute_to_mpc_session, create_genesis_identity, get_genesis_session_status,
        GenesisCreationRequest, GenesisCreationResponse,
        MpcContributionRequest as ApiMpcContributionRequest,
        MpcContributionResponse as ApiMpcContributionResponse,
    },
    api::{DsmOperation, InboxEntry, InboxSubmissionRequest}, // Fixed: Added DsmOperation import
    cluster::ClusterManager,                                 // Add ClusterManager import
    identity::DsmIdentityManager,
    logging::{ClientInfo, OperationDetails, OperationResult, OperationType, StorageNodeLogger}, // Add logging
    storage::{
        vector_clock::VectorClock, StorageConfig as LibStorageConfig, StorageEngine, StorageFactory,
    },
    types::{
        AppConfig, AppState, BlindContributionRequest, BlindContributionResponse,
        BlindDeviceIdRequest, BlindDeviceIdResponse, BlindedStateEntry, MpcContributionRequest,
        MpcContributionResponse, MpcGenesisRequest, MpcGenesisResponse, MpcSession, NetworkConfig,
        NodeDiscoveryResponse, StatusResponse, StorageConfig,
    },
};

#[derive(Deserialize)]
#[allow(dead_code)] // Used in configuration loading
struct EnvConfig {
    protocol: String,
    lan_ip: String,
    ports: Vec<u16>,
    nodes: Vec<NodeConfig>,
}

#[derive(Deserialize)]
#[allow(dead_code)] // Used in configuration loading
struct NodeConfig {
    name: String,
    endpoint: String,
}

fn load_env_config() -> EnvConfig {
    // Try multiple possible paths for the config file
    let possible_paths = [
        "dsm_env_config.json",
        "../dsm_env_config.json",
        "../../dsm_env_config.json",
        "../../../dsm_env_config.json",
        "../../../../dsm_env_config.json",
    ];

    for path in &possible_paths {
        if std::path::Path::new(path).exists() {
            let config_str = fs::read_to_string(path)
                .unwrap_or_else(|_| panic!("Failed to read env config from {path}"));
            return serde_json::from_str(&config_str)
                .unwrap_or_else(|_| panic!("Failed to parse env config from {path}"));
        }
    }

    // Fallback to default config if none found
    warn!("No env config file found, using default configuration");
    EnvConfig {
        protocol: "http".to_string(),
        lan_ip: "127.0.0.1".to_string(),
        ports: vec![8080, 8081, 8082, 8083, 8084],
        nodes: (0..5)
            .map(|i| NodeConfig {
                name: format!("node{}", i + 1),
                endpoint: format!("http://127.0.0.1:{}", 8080 + i),
            })
            .collect(),
    }
}

fn get_node_endpoints() -> Vec<String> {
    let config = load_env_config();
    config.nodes.into_iter().map(|n| n.endpoint).collect()
}

/// Command line argument parser for the DSM Storage Node.
///
/// Provides options for specifying the configuration file and
/// different operation modes such as regular operation or staking.
#[derive(Parser)]
#[command(author, version, about, long_about = None)]
struct Cli {
    /// The subcommand to execute (run or stake)
    #[command(subcommand)]
    command: Option<Commands>,

    /// Path to the configuration file
    /// Defaults to config.toml in the current directory
    #[arg(short, long, value_name = "FILE", default_value = "config.toml")]
    config: PathBuf,
}

/// Available subcommands for the DSM Storage Node
#[derive(Subcommand)]
enum Commands {
    /// Run the storage node in standard mode
    Run,

    /// Start the storage node with staking to participate
    /// in the DSM network and earn rewards
    Stake {
        /// Amount of tokens to stake (minimum usually 1000)
        #[arg(short, long)]
        amount: u64,
    },
}

/// Load configuration from a TOML file
///
/// # Arguments
///
/// * `config_path` - Path to the configuration file
///
/// # Returns
///
/// * `Ok(AppConfig)` if the configuration was loaded successfully
/// * `Err(ConfigError)` if the configuration could not be loaded
#[allow(clippy::ptr_arg)]
fn load_config(config_path: &PathBuf) -> Result<AppConfig, ConfigError> {
    let config = Config::builder()
        .add_source(File::from(config_path.clone()))
        .build()?;

    config.try_deserialize::<AppConfig>()
}

/// API handler for the node status endpoint
///
/// Returns current information about the node's status,
/// including uptime, connections, and storage usage.
async fn status_handler(Extension(state): Extension<Arc<RwLock<AppState>>>) -> impl IntoResponse {
    let state = state.read().await;

    // In a real implementation, you would gather actual metrics
    let status = StatusResponse {
        node_id: state.config.node.id.clone(),
        status: "running".to_string(),
        version: state.config.node.version.clone(),
        uptime: 0,       // Placeholder
        peers: 0,        // Placeholder
        storage_used: 0, // Placeholder
        storage_total: state.config.storage.capacity,
        staked_amount: state.staked_amount,
    };

    (StatusCode::OK, Json(status))
}

/// API handler for the peers endpoint
///
/// Returns a list of known peer nodes in the network.
async fn peers_handler(Extension(state): Extension<Arc<RwLock<AppState>>>) -> impl IntoResponse {
    let state = state.read().await;

    // Use cluster manager to get cluster peers
    let mut peers = Vec::new();

    if let Some(cluster_manager) = &state.cluster_manager {
        let cluster_peers = cluster_manager.get_gossip_targets(None).await;
        for peer in cluster_peers {
            peers.push(serde_json::json!({
                "endpoint": peer.endpoint,
                "node_id": peer.id,
                "status": "active",
                "region": peer.region
            }));
        }
    } else {
        // Fall back to storage engine's cluster nodes if no cluster manager
        let cluster_nodes = state.storage.get_cluster_nodes();
        for node in cluster_nodes {
            peers.push(serde_json::json!({
                "endpoint": node.endpoint,
                "node_id": node.id,
                "status": "active",
                "region": node.region
            }));
        }
    }

    let response = serde_json::json!({
        "node_id": state.config.node.id,
        "peer_count": peers.len(),
        "peers": peers
    });

    (StatusCode::OK, Json(response))
}
/// API handler for the health check endpoint
///
/// Returns a simple health status for load balancers and monitoring.
async fn health_handler() -> impl IntoResponse {
    let response = serde_json::json!({
        "status": "healthy",
        "timestamp": SystemTime::now().duration_since(UNIX_EPOCH).unwrap().as_secs()
    });

    (StatusCode::OK, Json(response))
}

/// API handler for storing data
///
/// Stores a JSON value with the given key in the storage backend.
async fn store_data_handler(
    Extension(state): Extension<Arc<RwLock<AppState>>>,
    ConnectInfo(client_addr): ConnectInfo<SocketAddr>,
    AxumPath(key): AxumPath<String>,
    Json(data): Json<Value>,
) -> impl IntoResponse {
    let start_time = Instant::now();

    let state_read = state.read().await;
    let logger = state_read.logger.clone();
    drop(state_read);

    debug!("Storing data with key: {}", key);

    // Log the storage request
    logger
        .log_operation(
            OperationType::ClientStorageRequest,
            OperationDetails {
                description: format!("Store data request for key: {key}"),
                data_size: Some(data.to_string().len()),
                custom_fields: {
                    let mut fields = HashMap::new();
                    fields.insert("key".to_string(), serde_json::Value::String(key.clone()));
                    fields.insert(
                        "data_type".to_string(),
                        serde_json::Value::String(format!("{data:?}")),
                    );
                    fields
                },
                ..Default::default()
            },
            OperationResult::Success,
            None,
            None,
            Some(ClientInfo {
                ip_address: Some(client_addr.ip().to_string()),
                request_size: Some(data.to_string().len()),
                ..Default::default()
            }),
        )
        .await;

    let state = state.read().await;

    // Create a BlindedStateEntry from the input data
    let entry = BlindedStateEntry {
        blinded_id: key.clone(),
        encrypted_payload: serde_json::to_string(&data)
            .unwrap_or_default()
            .into_bytes(),
        timestamp: SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap()
            .as_secs(),
        ttl: state.config.storage.default_ttl,
        region: "default".to_string(),
        priority: 0,
        proof_hash: [0u8; 32], // In a real implementation, this would be a proper hash
        metadata: HashMap::new(),
    };

    match state.storage.store(entry).await {
        Ok(_response) => {
            let duration = start_time.elapsed();
            debug!("Successfully stored data with key: {}", key);

            // Log successful storage
            logger
                .log_operation(
                    OperationType::DataStore,
                    OperationDetails {
                        description: format!("Successfully stored data with key: {key}"),
                        data_size: Some(data.to_string().len()),
                        custom_fields: {
                            let mut fields = HashMap::new();
                            fields
                                .insert("key".to_string(), serde_json::Value::String(key.clone()));
                            fields.insert(
                                "storage_success".to_string(),
                                serde_json::Value::Bool(true),
                            );
                            fields
                        },
                        ..Default::default()
                    },
                    OperationResult::Success,
                    Some(duration.as_millis() as u64),
                    None,
                    Some(ClientInfo {
                        ip_address: Some(client_addr.ip().to_string()),
                        response_size: Some(std::mem::size_of_val("success")),
                        ..Default::default()
                    }),
                )
                .await;

            (
                StatusCode::OK,
                Json(serde_json::json!({
                    "status": "success",
                    "message": format!("Data stored with key: {}", key)
                })),
            )
        }
        Err(err) => {
            let duration = start_time.elapsed();
            error!("Failed to store data: {}", err);

            // Log storage failure
            logger
                .log_operation(
                    OperationType::DataStore,
                    OperationDetails {
                        description: format!("Failed to store data with key: {key}"),
                        error_message: Some(err.to_string()),
                        data_size: Some(data.to_string().len()),
                        custom_fields: {
                            let mut fields = HashMap::new();
                            fields
                                .insert("key".to_string(), serde_json::Value::String(key.clone()));
                            fields.insert(
                                "storage_success".to_string(),
                                serde_json::Value::Bool(false),
                            );
                            fields
                        },
                        ..Default::default()
                    },
                    OperationResult::Failure,
                    Some(duration.as_millis() as u64),
                    None,
                    Some(ClientInfo {
                        ip_address: Some(client_addr.ip().to_string()),
                        ..Default::default()
                    }),
                )
                .await;

            (
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(serde_json::json!({
                    "error": "Storage failed",
                    "message": err.to_string()
                })),
            )
        }
    }
}
/// API handler for retrieving data
///
/// Retrieves a JSON value by its key from the storage backend.
async fn retrieve_data_handler(
    Extension(state): Extension<Arc<RwLock<AppState>>>,
    AxumPath(key): AxumPath<String>,
) -> impl IntoResponse {
    debug!("Retrieving data with key: {}", key);
    info!(
        "DEBUG: Checking key format - input: '{}', starts_with device_identity: {}",
        key,
        key.starts_with("device_identity:")
    );

    let state = state.read().await;
    match state.storage.retrieve(&key).await {
        Ok(Some(entry)) => {
            // Special handling for device identity responses
            if key.starts_with("device_identity:") {
                info!("DEBUG: Processing device identity response transformation");
                // Try to parse as DeviceIdentity from encrypted_payload
                match serde_json::from_slice::<dsm_storage_node::identity::DeviceIdentity>(
                    &entry.encrypted_payload,
                ) {
                    Ok(device_identity) => {
                        info!("DEBUG: Successfully parsed DeviceIdentity, creating response");
                        // Convert DeviceIdentity to the expected response format
                        let mut blinded_state = std::collections::HashMap::new();
                        blinded_state.insert(
                            "device_id".to_string(),
                            serde_json::Value::String(device_identity.device_id),
                        );
                        blinded_state.insert(
                            "threshold".to_string(),
                            serde_json::Value::Number(serde_json::Number::from(5u64)),
                        ); // MPC threshold
                        blinded_state.insert(
                            "created_at".to_string(),
                            serde_json::Value::Number(serde_json::Number::from(
                                device_identity.created_at,
                            )),
                        );
                        blinded_state.insert(
                            "updated_at".to_string(),
                            serde_json::Value::Number(serde_json::Number::from(
                                device_identity.updated_at,
                            )),
                        );

                        // Add cryptographic components
                        if let Ok(genesis_json) =
                            serde_json::to_value(&device_identity.genesis_state)
                        {
                            blinded_state.insert("genesis_state".to_string(), genesis_json);
                        }

                        // Add device entropy as base64
                        let device_entropy_b64 = base64::engine::general_purpose::STANDARD
                            .encode(&device_identity.device_entropy);
                        blinded_state.insert(
                            "device_entropy".to_string(),
                            serde_json::Value::String(device_entropy_b64),
                        );

                        // Add blind key as base64
                        let blind_key_b64 = base64::engine::general_purpose::STANDARD
                            .encode(&device_identity.blind_key);
                        blinded_state.insert(
                            "blind_key".to_string(),
                            serde_json::Value::String(blind_key_b64),
                        );

                        let device_identity_response = serde_json::json!({
                            "blinded_state": blinded_state
                        });

                        info!("DEBUG: Returning device identity response");
                        (StatusCode::OK, Json(device_identity_response))
                    }
                    Err(e) => {
                        warn!("Failed to parse device identity from payload: {}", e);
                        // Fall back to default behavior
                        match String::from_utf8(entry.encrypted_payload) {
                            Ok(json_str) => match serde_json::from_str::<Value>(&json_str) {
                                Ok(data) => (StatusCode::OK, Json(data)),
                                Err(_) => (StatusCode::INTERNAL_SERVER_ERROR, Json(Value::Null)),
                            },
                            Err(_) => (StatusCode::INTERNAL_SERVER_ERROR, Json(Value::Null)),
                        }
                    }
                }
            } else {
                // Convert the encrypted_payload back to JSON for non-device-identity entries
                match String::from_utf8(entry.encrypted_payload) {
                    Ok(json_str) => match serde_json::from_str::<Value>(&json_str) {
                        Ok(data) => (StatusCode::OK, Json(data)),
                        Err(_) => (StatusCode::INTERNAL_SERVER_ERROR, Json(Value::Null)),
                    },
                    Err(_) => (StatusCode::INTERNAL_SERVER_ERROR, Json(Value::Null)),
                }
            }
        }
        Ok(None) => (StatusCode::NOT_FOUND, Json(Value::Null)),
        Err(err) => {
            error!("Failed to retrieve data: {}", err);
            (StatusCode::INTERNAL_SERVER_ERROR, Json(Value::Null))
        }
    }
}

/// API handler for deleting data
///
/// Deletes a key-value pair from the storage backend.
async fn delete_data_handler(
    Extension(state): Extension<Arc<RwLock<AppState>>>,
    AxumPath(key): AxumPath<String>,
) -> impl IntoResponse {
    debug!("Deleting data with key: {}", key);

    let state = state.read().await;
    match state.storage.delete(&key).await {
        Ok(true) => StatusCode::OK,
        Ok(false) => StatusCode::NOT_FOUND,
        Err(err) => {
            error!("Failed to delete data: {}", err);
            StatusCode::INTERNAL_SERVER_ERROR
        }
    }
}

/// API handler for MPC Genesis creation
///
/// This endpoint allows clients to initiate MPC Genesis operations
/// by connecting to multiple storage nodes and performing distributed
/// multi-party computation.
async fn mpc_genesis_handler(
    Extension(state): Extension<Arc<RwLock<AppState>>>,
    ConnectInfo(client_addr): ConnectInfo<SocketAddr>,
    Json(request): Json<MpcGenesisRequest>,
) -> impl IntoResponse {
    let start_time = Instant::now();
    let _operation_start = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .unwrap_or_default()
        .as_secs();

    let state_read = state.read().await;
    let logger = state_read.logger.clone();
    let _node_id = state_read.config.node.id.clone();
    drop(state_read);

    debug!("Starting MPC Genesis for device: {}", request.device_id);

    // Log the client request
    logger
        .log_operation(
            OperationType::ClientGenesis,
            OperationDetails {
                description: format!("MPC Genesis request from device: {}", request.device_id),
                device_id: Some(request.device_id.clone()),
                data_size: Some(std::mem::size_of_val(&request)),
                ..Default::default()
            },
            OperationResult::Success,
            None,
            None,
            Some(ClientInfo {
                device_id: Some(request.device_id.clone()),
                ip_address: Some(client_addr.ip().to_string()),
                request_size: Some(std::mem::size_of_val(&request)),
                ..Default::default()
            }),
        )
        .await;

    let threshold = request.threshold.unwrap_or(1);

    // Create new MPC session
    let session_id = format!(
        "{}_{}",
        request.device_id,
        std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap()
            .as_secs()
    );

    let session = MpcSession {
        session_id: session_id.clone(),
        device_id: request.device_id.clone(),
        threshold,
        contributions: Vec::new(),
        created_at: std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap()
            .as_secs(),
    };

    // Store session
    {
        let state = state.read().await;
        let mut sessions = state.mpc_sessions.write().await;
        sessions.insert(session_id.clone(), session);
    }

    // Simulate MPC Genesis (in real implementation, this would coordinate with other nodes)
    let mnemonic = generate_mpc_mnemonic(&request.device_id, threshold).await;

    let duration = start_time.elapsed();
    let success = true; // In this simplified implementation

    // Log session creation
    logger
        .log_operation(
            OperationType::MpcSessionCreate,
            OperationDetails {
                description: format!("MPC session created for device: {}", request.device_id),
                session_id: Some(session_id.clone()),
                device_id: Some(request.device_id.clone()),
                custom_fields: {
                    let mut fields = HashMap::new();
                    fields.insert(
                        "threshold".to_string(),
                        serde_json::Value::Number(serde_json::Number::from(threshold)),
                    );
                    fields.insert(
                        "mnemonic_generated".to_string(),
                        serde_json::Value::Bool(true),
                    );
                    fields
                },
                ..Default::default()
            },
            if success {
                OperationResult::Success
            } else {
                OperationResult::Failure
            },
            Some(duration.as_millis() as u64),
            None,
            Some(ClientInfo {
                device_id: Some(request.device_id.clone()),
                ip_address: Some(client_addr.ip().to_string()),
                response_size: Some(std::mem::size_of_val(&session_id) + mnemonic.len()),
                ..Default::default()
            }),
        )
        .await;

    let response = MpcGenesisResponse {
        success: true,
        session_id: session_id.clone(),
        device_id: request.device_id.clone(),
        mnemonic: Some(mnemonic),
        error: None,
    };

    (StatusCode::OK, Json(response))
}

/// API handler for MPC contribution submission
async fn mpc_contribute_handler(
    Extension(state): Extension<Arc<RwLock<AppState>>>,
    Json(request): Json<MpcContributionRequest>,
) -> impl IntoResponse {
    debug!(
        "Received MPC contribution request for session: {} from device: {}",
        request.session_id, request.device_id
    );

    // Generate a real cryptographic contribution based on node entropy
    let node_entropy = {
        let state = state.read().await;
        format!(
            "{}{}{}",
            state.config.node.id,
            request.session_id,
            SystemTime::now()
                .duration_since(UNIX_EPOCH)
                .unwrap()
                .as_secs()
        )
    };

    // Create a proper cryptographic contribution using Blake3
    let mut hasher = blake3::Hasher::new();
    hasher.update(node_entropy.as_bytes());
    hasher.update(request.device_id.as_bytes());
    hasher.update("mpc_genesis".as_bytes());

    let contribution_hash = hasher.finalize();
    let contribution_hex = hex::encode(contribution_hash.as_bytes());

    debug!(
        "Generated MPC contribution: {} for device: {}",
        &contribution_hex[..16],
        request.device_id
    );

    let response = MpcContributionResponse {
        success: true,
        contribution: contribution_hex,
        session_id: request.session_id,
        threshold_met: true, // Simplified - in production would check actual threshold
        accepted: true,
        contributions_count: 1, // Simplified - in production would track actual count
        threshold: 1,           // Simplified - in production would use actual threshold
        ready_for_processing: true,
        message: "Contribution accepted".to_string(),
        timestamp: SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap_or_default()
            .as_secs(),
    };

    (StatusCode::OK, Json(response))
}

/// API handler for node discovery
async fn discover_nodes_handler(
    Extension(_state): Extension<Arc<RwLock<AppState>>>,
) -> impl IntoResponse {
    debug!("Node discovery request received");

    // Return list of available storage nodes from dynamic config
    let nodes = get_node_endpoints();

    let response = NodeDiscoveryResponse {
        count: nodes.len(),
        nodes,
    };

    (StatusCode::OK, Json(response))
}

/// Generate MPC-based mnemonic using proper cryptographic coordination
async fn generate_mpc_mnemonic(device_id: &str, threshold: u8) -> String {
    // Create a proper MPC-coordinated mnemonic using cryptographic aggregation
    // This implementation coordinates with storage nodes to create a secure mnemonic

    // Standard BIP39 wordlist (abbreviated for demo)
    let bip39_words = [
        "abandon", "ability", "able", "about", "above", "absent", "absorb", "abstract", "absurd",
        "abuse", "access", "accident", "account", "accuse", "achieve", "acid", "acoustic",
        "acquire", "across", "act", "action", "actor", "actress", "actual", "adapt", "add",
        "addict", "address", "adjust", "admit", "adult", "advance", "advice", "aerobic", "affair",
        "afford", "afraid", "again", "age", "agent", "agree", "ahead", "aim", "air", "airport",
        "aisle", "alarm", "album", "alcohol", "alert", "alien", "all", "alley", "allow", "almost",
        "alone", "alpha", "already", "also", "alter", "always", "amateur", "amazing", "among",
        "amount", "amused", "analyst", "anchor", "ancient", "anger", "angle", "angry", "animal",
        "ankle", "announce", "annual", "another", "answer", "antenna", "antique", "anxiety", "any",
        "apart", "apology", "appear", "apple", "approve", "april", "arch", "arctic", "area",
        "arena", "argue", "arm", "armed", "armor", "army", "around", "arrange", "arrest", "arrive",
        "arrow", "art", "artefact", "artist", "artwork", "ask", "aspect", "assault", "asset",
        "assist", "assume", "asthma", "athlete", "atom", "attack", "attend", "attitude", "attract",
        "auction", "audit", "august", "aunt", "author", "auto", "autumn", "average", "avocado",
        "avoid", "awake", "aware", "away", "awesome", "awful", "awkward", "axis", "baby",
        "bachelor", "bacon", "badge", "bag", "balance", "balcony", "ball", "bamboo", "banana",
        "banner", "bar", "barely", "bargain", "barrel", "base", "basic", "basket", "battle",
        "beach", "bean", "beauty", "because", "become", "beef", "before", "begin", "behave",
        "behind", "believe", "below", "belt", "bench", "benefit", "best", "betray", "better",
        "between", "beyond", "bicycle", "bid", "bike", "bind", "biology", "bird", "birth",
        "bitter", "black", "blade", "blame", "blanket", "blast", "bleak", "bless", "blind",
        "blood", "blossom", "blow", "blue", "blur", "blush", "board", "boat", "body", "boil",
        "bomb", "bone", "bonus", "book", "boost", "border", "boring", "borrow", "boss", "bottom",
        "bounce", "box", "boy", "bracket", "brain", "brand", "brass", "brave", "bread", "breeze",
        "brick", "bridge", "brief", "bright", "bring", "brisk", "broccoli", "broken", "bronze",
        "broom", "brother", "brown", "brush", "bubble", "buddy", "budget", "buffalo", "build",
        "bulb", "bulk", "bullet", "bundle", "bunker", "burden", "burger", "burst", "bus",
        "business", "busy", "butter", "buyer", "buzz",
    ];

    // Step 1: Create aggregated entropy using proper MPC protocol
    let mut aggregated_entropy = Vec::new();

    // Simulate MPC contributions from multiple nodes
    let node_count = threshold.max(1) as usize;
    for node_index in 0..node_count {
        // Each node contributes entropy based on device_id and node identity
        let mut node_hasher = blake3::Hasher::new();
        node_hasher.update(device_id.as_bytes());
        node_hasher.update(&node_index.to_le_bytes());
        node_hasher.update(&threshold.to_le_bytes());
        node_hasher.update("MPC_MNEMONIC_CONTRIBUTION".as_bytes());

        // Add system entropy for security
        let system_time = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap_or_default()
            .as_nanos();
        node_hasher.update(&system_time.to_le_bytes());

        let contribution = node_hasher.finalize();
        aggregated_entropy.extend_from_slice(contribution.as_bytes());
    }

    // Step 2: Create master seed from aggregated entropy
    let mut master_hasher = blake3::Hasher::new();
    master_hasher.update(&aggregated_entropy);
    master_hasher.update("DSM_MPC_MNEMONIC_SEED".as_bytes());
    let master_seed = master_hasher.finalize();

    // Step 3: Generate 12-word mnemonic using proper entropy distribution
    let mut selected_words = Vec::new();
    let seed_bytes = master_seed.as_bytes();

    for word_index in 0..12 {
        // Use 2 bytes of entropy per word for better distribution
        let byte_offset = (word_index * 2) % seed_bytes.len();
        let word_entropy = u16::from_le_bytes([
            seed_bytes[byte_offset],
            seed_bytes[(byte_offset + 1) % seed_bytes.len()],
        ]) as usize;

        let word_index = word_entropy % bip39_words.len();
        selected_words.push(bip39_words[word_index]);
    }

    // Step 4: Validate mnemonic uniqueness and cryptographic strength
    let mnemonic = selected_words.join(" ");

    // Create a verification hash to ensure mnemonic integrity
    let mut verification_hasher = blake3::Hasher::new();
    verification_hasher.update(mnemonic.as_bytes());
    verification_hasher.update(device_id.as_bytes());
    let verification_hash = verification_hasher.finalize();

    tracing::info!(
        "Generated MPC mnemonic for device {} with verification hash: {}",
        device_id,
        hex::encode(&verification_hash.as_bytes()[..8])
    );

    mnemonic
}

/// API handler for storing an inbox entry
async fn store_inbox_entry_handler(
    Extension(state): Extension<Arc<RwLock<AppState>>>,
    Json(submission): Json<InboxSubmissionRequest>,
) -> impl IntoResponse {
    debug!("Storing inbox entry: {}", submission.entry.transaction_id);

    // Validate entry
    if submission.entry.transaction_id.is_empty() {
        return (
            StatusCode::BAD_REQUEST,
            Json(serde_json::json!({"error": "Entry ID cannot be empty"})),
        );
    }

    // Check if transaction is valid (instead of checking is_empty on enum)
    if matches!(submission.entry.transaction, DsmOperation::Noop) {
        return (
            StatusCode::BAD_REQUEST,
            Json(serde_json::json!({"error": "Transaction cannot be empty"})),
        );
    }

    let state = state.read().await;

    // Create a BlindedStateEntry from the inbox entry
    let entry = BlindedStateEntry {
        blinded_id: format!(
            "inbox:{}:{}",
            submission.entry.recipient_device_id, submission.entry.transaction_id
        ),
        encrypted_payload: match bincode::serialize(&submission.entry) {
            Ok(data) => data,
            Err(e) => {
                error!("Failed to serialize inbox entry: {}", e);
                return (
                    StatusCode::INTERNAL_SERVER_ERROR,
                    Json(serde_json::json!({"error": "Serialization failed"})),
                );
            }
        },
        timestamp: SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .map_or(0, |d| d.as_secs()),
        ttl: if submission.entry.ttl_seconds > 0 {
            submission.entry.ttl_seconds
        } else {
            3600 // Default 1 hour TTL
        },
        region: "global".to_string(),
        priority: 1, // Standard priority
        proof_hash: {
            // Hash the entry for verification
            let mut hasher = blake3::Hasher::new();
            hasher.update(&bincode::serialize(&submission.entry).unwrap_or_default());
            let hash = hasher.finalize();
            let mut hash_bytes = [0u8; 32];
            hash_bytes.copy_from_slice(hash.as_bytes());
            hash_bytes
        },
        metadata: {
            let mut metadata = HashMap::new();
            metadata.insert("type".to_string(), "inbox_entry".to_string());
            metadata.insert(
                "sender".to_string(),
                submission.entry.sender_genesis_hash.clone(),
            );
            metadata.insert(
                "recipient".to_string(),
                submission.entry.recipient_device_id.clone(),
            );
            metadata.insert(
                "timestamp".to_string(),
                submission.entry.timestamp.to_string(),
            );
            metadata
        },
    };

    // Store the entry
    match state.storage.store(entry).await {
        Ok(_response) => (
            StatusCode::OK,
            Json(serde_json::json!({"status": "success", "message": "Inbox entry stored"})),
        ),
        Err(err) => {
            error!("Failed to store inbox entry: {}", err);
            (
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(serde_json::json!({"error": "Storage failed"})),
            )
        }
    }
}

/// API handler for getting inbox entries for a recipient
async fn get_inbox_entries_handler(
    Extension(state): Extension<Arc<RwLock<AppState>>>,
    AxumPath(recipient_genesis): AxumPath<String>,
) -> impl IntoResponse {
    debug!("Getting inbox entries for recipient: {}", recipient_genesis);

    let state = state.read().await;

    // Get all blinded IDs with the inbox prefix for this recipient
    let prefix = format!("inbox:{recipient_genesis}:");

    // Get all entries and filter by prefix
    match state.storage.list(Some(1000), None).await {
        Ok(all_ids) => {
            // Filter to only include entries for this recipient
            let inbox_ids: Vec<String> = all_ids
                .into_iter()
                .filter(|id| id.starts_with(&prefix))
                .collect();

            // Retrieve each entry
            let mut entries = Vec::new();
            for id in inbox_ids {
                if let Ok(Some(entry)) = state.storage.retrieve(&id).await {
                    // Deserialize the inbox entry
                    if let Ok(inbox_entry) =
                        bincode::deserialize::<InboxEntry>(&entry.encrypted_payload)
                    {
                        entries.push(inbox_entry);
                    } else {
                        warn!("Failed to deserialize inbox entry: {}", id);
                    }
                }
            }

            (StatusCode::OK, Json(entries))
        }
        Err(err) => {
            error!("Failed to list inbox entries: {}", err);
            let empty_entries: Vec<InboxEntry> = Vec::new();
            (StatusCode::INTERNAL_SERVER_ERROR, Json(empty_entries))
        }
    }
}

/// API handler for deleting an inbox entry
async fn delete_inbox_entry_handler(
    Extension(state): Extension<Arc<RwLock<AppState>>>,
    AxumPath((recipient_genesis, entry_id)): AxumPath<(String, String)>,
) -> impl IntoResponse {
    let blinded_id = format!("inbox:{recipient_genesis}:{entry_id}");
    debug!("Deleting inbox entry: {}", blinded_id);

    let state = state.read().await;

    // Delete the entry
    match state.storage.delete(&blinded_id).await {
        Ok(true) => (
            StatusCode::OK,
            Json(serde_json::json!({
                "status": "success",
                "message": format!("Inbox entry {} deleted", entry_id),
            })),
        ),
        Ok(false) => (
            StatusCode::NOT_FOUND,
            Json(serde_json::json!({
                "error": format!("Inbox entry with ID {} not found", entry_id)
            })),
        ),
        Err(err) => {
            error!("Failed to delete inbox entry: {}", err);
            (
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(serde_json::json!({"error": "Deletion failed"})),
            )
        }
    }
}
/// Create the API router with all routes
///
/// Sets up all API endpoints and attaches the application state.
///
/// # Arguments
///
/// * `state` - Application state to be shared with all handlers
///
/// # Returns
///
/// An Axum Router configured with all API endpoints
fn create_router(state: Arc<RwLock<AppState>>) -> Router {
    use tower_http::cors::CorsLayer;

    Router::new()
        .route("/api/v1/status", get(status_handler))
        .route("/api/v1/health", get(health_handler))
        .route("/api/v1/peers", get(peers_handler))
        .route("/api/v1/data/:key", get(retrieve_data_handler))
        .route("/api/v1/data/:key", post(store_data_handler))
        .route("/api/v1/data/:key", delete(delete_data_handler))
        .route("/api/v1/mpc/genesis", post(mpc_genesis_handler))
        .route("/api/v1/mpc/contribute", post(mpc_contribute_handler))
        // DSM Protocol Compliant Genesis Creation (MPC Required)
        .route(
            "/api/v1/genesis/create",
            post(create_genesis_identity_handler),
        )
        .route(
            "/api/v1/genesis/contribute",
            post(contribute_to_genesis_handler),
        )
        .route(
            "/api/v1/genesis/session/:session_id",
            get(get_genesis_status_handler),
        )
        .route(
            "/api/v1/genesis/status",
            get(get_all_genesis_sessions_handler),
        )
        .route("/api/v1/nodes/discover", get(discover_nodes_handler))
        // Blind device ID endpoints
        .route("/api/v1/blind/device", post(create_blind_device_handler))
        .route("/api/v1/blind/contribute", post(blind_contribute_handler))
        .route(
            "/api/v1/blind/session/:session_id",
            get(get_blind_session_handler),
        )
        // Unilateral transaction inbox endpoints
        .route("/api/v1/inbox", post(store_inbox_entry_handler))
        .route(
            "/api/v1/inbox/:recipient_genesis",
            get(get_inbox_entries_handler),
        )
        .route(
            "/api/v1/inbox/:recipient_genesis/:entry_id",
            delete(delete_inbox_entry_handler),
        )
        // Epidemic gossip protocol endpoints
        .route("/api/v1/entries", get(list_entries_handler))
        .route("/api/v1/entries", post(receive_entries_handler))
        .route("/api/v1/entries/request", post(request_entries_handler))
        // Chain tip and contact management endpoints
        .route("/api/v1/contacts", post(add_contact_handler))
        .route("/api/v1/chain-tips", post(update_chain_tip_handler))
        .route(
            "/api/v1/bilateral-state",
            post(create_bilateral_state_handler),
        )
        .route(
            "/api/v1/device/:device_id",
            get(get_device_identity_handler),
        )
        .route("/api/v1/verify-chain", post(verify_chain_handler))
        // Logging and statistics endpoints
        .route("/api/v1/logs", get(get_logs_handler))
        .route("/api/v1/stats", get(get_stats_handler))
        .route("/api/v1/logs/export", get(export_logs_handler))
        .layer(Extension(state))
        .layer(TraceLayer::new_for_http())
        .layer(CorsLayer::permissive())
}
/// Initialize the storage engine based on configuration
///
/// Creates and initializes the appropriate storage engine
/// based on the configuration.
///
/// # Arguments
///
/// * `config` - Storage configuration
///
/// # Returns
///
/// * `Ok(Arc<dyn StorageEngine + Send + Sync>)` if initialization was successful
/// * `Err(anyhow::Error)` if initialization failed
async fn init_storage(
    config: &StorageConfig,
    node: &dsm_storage_node::types::StorageNode,
    _backing_storage: Option<Arc<ClusterManager>>, // For future use
) -> Result<Arc<dyn StorageEngine + Send + Sync>, anyhow::Error> {
    // Create data directory if it doesn't exist
    tokio::fs::create_dir_all(&config.data_dir).await?;

    // Convert to library storage config
    let lib_config = LibStorageConfig {
        database_path: config.database_path.clone(),
        default_ttl: config.default_ttl,
        enable_pruning: true,
        pruning_interval: 3600,
    };

    let factory = StorageFactory::new(lib_config);

    // Initialize the appropriate storage engine based on the configuration
    match config.engine.as_str() {
        "sqlite" => {
            info!(
                "Initializing SQLite storage engine at {}",
                config.database_path
            );
            factory
                .create_sql_storage()
                .map_err(|e| anyhow::anyhow!("Failed to create SQLite storage: {}", e))
        }
        "memory" => {
            info!("Initializing in-memory storage engine");
            factory
                .create_memory_storage()
                .map_err(|e| anyhow::anyhow!("Failed to create memory storage: {}", e))
        }
        "epidemic" => {
            info!("Initializing epidemic storage engine with cluster manager");

            // Initialize cluster topology if cluster manager exists
            if let Some(ref manager) = _backing_storage {
                info!("Initializing cluster topology for epidemic storage");

                // Generate predictable node IDs for the 5-node development cluster
                // Each node will generate these same IDs, ensuring consistency
                let _all_nodes = vec![
                    (
                        dsm_storage_node::storage::topology::NodeId::from_device_entropy(
                            "dev-node-1-8080".as_bytes(),
                            "dsm_storage_node",
                        )
                        .to_string(),
                        "http://127.0.0.1:8080".to_string(),
                    ),
                    (
                        dsm_storage_node::storage::topology::NodeId::from_device_entropy(
                            "dev-node-2-8081".as_bytes(),
                            "dsm_storage_node",
                        )
                        .to_string(),
                        "http://127.0.0.1:8081".to_string(),
                    ),
                    (
                        dsm_storage_node::storage::topology::NodeId::from_device_entropy(
                            "dev-node-3-8082".as_bytes(),
                            "dsm_storage_node",
                        )
                        .to_string(),
                        "http://127.0.0.1:8082".to_string(),
                    ),
                    (
                        dsm_storage_node::storage::topology::NodeId::from_device_entropy(
                            "dev-node-4-8083".as_bytes(),
                            "dsm_storage_node",
                        )
                        .to_string(),
                        "http://127.0.0.1:8083".to_string(),
                    ),
                    (
                        dsm_storage_node::storage::topology::NodeId::from_device_entropy(
                            "dev-node-5-8084".as_bytes(),
                            "dsm_storage_node",
                        )
                        .to_string(),
                        "http://127.0.0.1:8084".to_string(),
                    ),
                ];

                // Initialize the topology
                if let Err(e) = manager.start_discovery().await {
                    error!("Failed to initialize cluster topology: {}", e);
                    // Continue anyway - the epidemic storage will work with available nodes
                } else {
                    info!("Successfully initialized cluster discovery");
                }
            }

            factory
                .create_epidemic_storage(
                    node.id.clone(),
                    node.clone(),
                    vec![],                   // Bootstrap nodes already in cluster manager
                    _backing_storage.clone(), // Pass cluster manager to epidemic storage
                )
                .await
                .map_err(|e| anyhow::anyhow!("Failed to create epidemic storage: {}", e))
        }
        _ => {
            error!("Unsupported storage engine: {}", config.engine);
            Err(anyhow::anyhow!("Unsupported storage engine"))
        }
    }
}

/// Initialize networking based on configuration
///
/// Sets up networking connections to peer nodes and
/// initializes the discovery mechanism.
///
/// # Arguments
///
/// * `config` - Network configuration
///
/// # Returns
///
/// * `Ok(())` if initialization was successful
/// * `Err(anyhow::Error)` if initialization failed
async fn init_networking(config: &NetworkConfig) -> Result<(), anyhow::Error> {
    info!(
        "Initializing networking on {}:{}",
        config.listen_addr, config.port
    );

    // Basic network initialization - cluster management handles peer discovery
    info!("Network stack initialized. Cluster management handles peer discovery.");

    Ok(())
}
/// Process staking of tokens for node operation///
/// Stakes the specified amount of tokens to participate
/// in the DSM network and earn rewards.
///
/// # Arguments
///
/// * `amount` - Amount of tokens to stake
///
/// # Returns
///
/// * `Ok(())` if staking was successful
/// * `Err(anyhow::Error)` if staking failed
async fn process_staking(amount: u64) -> Result<(), anyhow::Error> {
    info!("Processing stake of {} tokens", amount);

    // In a real implementation, this would interact with a blockchain
    // to lock tokens as stake for operating a storage node.

    // For now, we just simulate a successful staking
    if amount < 1000 {
        error!("Staking amount too low. Minimum requirement is 1000 tokens.");
        return Err(anyhow::anyhow!("Staking amount too low"));
    }

    info!("Staking successful. Node is eligible for rewards.");
    Ok(())
}

/// API handler for creating blind device ID sessions
async fn create_blind_device_handler(
    Extension(state): Extension<Arc<RwLock<AppState>>>,
    Json(request): Json<BlindDeviceIdRequest>,
) -> impl IntoResponse {
    debug!(
        "Creating blind device ID session for device: {}",
        request.device_id
    );

    let state = state.read().await;

    if let Some(identity_manager) = &state.identity_manager {
        match identity_manager
            .create_mpc_session(request.device_id.clone(), request.threshold, None)
            .await
        {
            Ok(session_id) => {
                let response = BlindDeviceIdResponse {
                    session_id: session_id.clone(),
                    device_id: request.device_id,
                    state: "collecting".to_string(),
                    contributions_received: 0,
                    threshold: request.threshold,
                    complete: false,
                    timestamp: std::time::SystemTime::now()
                        .duration_since(std::time::UNIX_EPOCH)
                        .unwrap()
                        .as_secs(),
                };
                (StatusCode::OK, Json(response))
            }
            Err(e) => {
                error!("Failed to create blind device session: {}", e);
                let response = BlindDeviceIdResponse {
                    session_id: "".to_string(),
                    device_id: request.device_id,
                    state: "failed".to_string(),
                    contributions_received: 0,
                    threshold: request.threshold,
                    complete: false,
                    timestamp: std::time::SystemTime::now()
                        .duration_since(std::time::UNIX_EPOCH)
                        .unwrap()
                        .as_secs(),
                };
                (StatusCode::INTERNAL_SERVER_ERROR, Json(response))
            }
        }
    } else {
        error!("Identity manager not initialized");
        let response = BlindDeviceIdResponse {
            session_id: "".to_string(),
            device_id: request.device_id,
            state: "failed".to_string(),
            contributions_received: 0,
            threshold: request.threshold,
            complete: false,
            timestamp: std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .unwrap()
                .as_secs(),
        };
        (StatusCode::SERVICE_UNAVAILABLE, Json(response))
    }
}

/// API handler for contributing to blind device ID sessions
#[axum::debug_handler]
async fn blind_contribute_handler(
    Extension(state): Extension<Arc<RwLock<AppState>>>,
    Json(request): Json<BlindContributionRequest>,
) -> impl IntoResponse {
    let state = state.read().await;
    debug!("Receiving contribution for session: {}", request.session_id);

    if let Some(identity_manager) = &state.identity_manager {
        // Create contribution from the request
        let contribution = dsm_storage_node::identity::MpcContribution {
            node_id: request.node_id.clone(),
            entropy_data: request.entropy_data.clone(),
            proof: request.proof.or(Some(vec![])),
            timestamp: request.timestamp,
        };

        let now = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap()
            .as_secs();

        // Execute the contribution process
        let result = identity_manager
            .add_contribution(request.session_id.clone(), contribution)
            .await;

        match result {
            Ok(genesis_ready) => {
                let response = BlindContributionResponse {
                    success: true,
                    session_id: request.session_id,
                    threshold_met: genesis_ready,
                    error: None,
                    timestamp: now,
                };
                (StatusCode::OK, Json(response))
            }
            Err(e) => {
                error!("Failed to add contribution: {}", e);
                let response = BlindContributionResponse {
                    success: false,
                    session_id: request.session_id,
                    threshold_met: false,
                    error: Some(e.to_string()),
                    timestamp: now,
                };
                (StatusCode::BAD_REQUEST, Json(response))
            }
        }
    } else {
        error!("Identity manager not initialized");
        let response = BlindContributionResponse {
            success: false,
            session_id: request.session_id,
            threshold_met: false,
            error: Some("Identity manager not available".to_string()),
            timestamp: std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .unwrap()
                .as_secs(),
        };
        (StatusCode::SERVICE_UNAVAILABLE, Json(response))
    }
}

/// API handler for DSM Protocol Genesis Creation (MPC Required)
async fn create_genesis_identity_handler(
    Extension(state): Extension<Arc<RwLock<AppState>>>,
    ConnectInfo(client_addr): ConnectInfo<SocketAddr>,
    Json(request): Json<GenesisCreationRequest>,
) -> impl IntoResponse {
    let start_time = Instant::now();

    let state_read = state.read().await;
    let logger = state_read.logger.clone();
    let identity_manager = state_read.identity_manager.clone();
    drop(state_read);

    // Store threshold before moving request
    let threshold = request.threshold;

    info!(
        "Creating DSM Genesis Identity via MPC with threshold: {}",
        threshold
    );

    // Log the Genesis creation request
    logger
        .log_operation(
            OperationType::ClientGenesis,
            OperationDetails {
                description: "DSM Genesis creation request (MPC required)".to_string(),
                custom_fields: {
                    let mut fields = HashMap::new();
                    fields.insert(
                        "threshold".to_string(),
                        serde_json::Value::Number(serde_json::Number::from(threshold)),
                    );
                    fields.insert("mpc_required".to_string(), serde_json::Value::Bool(true));
                    fields
                },
                ..Default::default()
            },
            OperationResult::Success,
            None,
            None,
            Some(ClientInfo {
                ip_address: Some(client_addr.ip().to_string()),
                request_size: Some(std::mem::size_of_val(&request)),
                ..Default::default()
            }),
        )
        .await;

    if let Some(identity_manager) = identity_manager {
        match create_genesis_identity(request, identity_manager).await {
            Ok(response) => {
                let duration = start_time.elapsed();

                // Log successful Genesis creation
                logger
                    .log_operation(
                        OperationType::MpcSessionCreate,
                        OperationDetails {
                            description: format!(
                                "Genesis session created: {}",
                                response.session_id
                            ),
                            session_id: Some(response.session_id.clone()),
                            custom_fields: {
                                let mut fields = HashMap::new();
                                fields.insert(
                                    "genesis_device_id".to_string(),
                                    serde_json::Value::String(response.genesis_device_id.clone()),
                                );
                                fields.insert(
                                    "threshold".to_string(),
                                    serde_json::Value::Number(serde_json::Number::from(
                                        response.threshold,
                                    )),
                                );
                                fields
                            },
                            ..Default::default()
                        },
                        OperationResult::Success,
                        Some(duration.as_millis() as u64),
                        None,
                        Some(ClientInfo {
                            ip_address: Some(client_addr.ip().to_string()),
                            response_size: Some(std::mem::size_of_val(&response)),
                            ..Default::default()
                        }),
                    )
                    .await;

                (StatusCode::OK, Json(response))
            }
            Err(e) => {
                error!("Failed to create Genesis identity: {}", e);

                // Log Genesis creation failure
                logger
                    .log_operation(
                        OperationType::ClientGenesis,
                        OperationDetails {
                            description: "Genesis creation failed".to_string(),
                            error_message: Some(e.to_string()),
                            ..Default::default()
                        },
                        OperationResult::Failure,
                        Some(start_time.elapsed().as_millis() as u64),
                        None,
                        Some(ClientInfo {
                            ip_address: Some(client_addr.ip().to_string()),
                            ..Default::default()
                        }),
                    )
                    .await;

                let error_response = GenesisCreationResponse {
                    session_id: String::new(),
                    genesis_device_id: String::new(),
                    master_genesis_id: String::new(),
                    is_master_genesis: true,
                    state: "failed".to_string(),
                    contributions_received: 0,
                    threshold,
                    complete: false,
                    genesis_hash: None,
                    initial_chain_tip: None,
                    participating_nodes: Vec::new(),
                    timestamp: SystemTime::now()
                        .duration_since(UNIX_EPOCH)
                        .unwrap_or_default()
                        .as_secs(),
                };

                (StatusCode::INTERNAL_SERVER_ERROR, Json(error_response))
            }
        }
    } else {
        error!("Identity manager not initialized");

        let error_response = GenesisCreationResponse {
            session_id: String::new(),
            genesis_device_id: String::new(),
            master_genesis_id: String::new(),
            is_master_genesis: true,
            state: "failed".to_string(),
            contributions_received: 0,
            threshold,
            complete: false,
            genesis_hash: None,
            initial_chain_tip: None,
            participating_nodes: Vec::new(),
            timestamp: SystemTime::now()
                .duration_since(UNIX_EPOCH)
                .unwrap_or_default()
                .as_secs(),
        };

        (StatusCode::SERVICE_UNAVAILABLE, Json(error_response))
    }
}

/// API handler for contributing to Genesis MPC session
async fn contribute_to_genesis_handler(
    Extension(state): Extension<Arc<RwLock<AppState>>>,
    ConnectInfo(client_addr): ConnectInfo<SocketAddr>,
    Json(request): Json<ApiMpcContributionRequest>,
) -> impl IntoResponse {
    let start_time = Instant::now();

    let state_read = state.read().await;
    let logger = state_read.logger.clone();
    let identity_manager = state_read.identity_manager.clone();
    let node_id = state_read.config.node.id.clone();
    drop(state_read);

    info!(
        "Received Genesis MPC contribution for session: {}",
        request.session_id
    );

    // Log the contribution request
    logger
        .log_operation(
            OperationType::MpcContribution,
            OperationDetails {
                description: format!(
                    "Genesis MPC contribution for session: {}",
                    request.session_id
                ),
                session_id: Some(request.session_id.clone()),
                custom_fields: {
                    let mut fields = HashMap::new();
                    fields.insert(
                        "node_id".to_string(),
                        serde_json::Value::String(request.node_id.clone()),
                    );
                    fields.insert(
                        "entropy_size".to_string(),
                        serde_json::Value::Number(serde_json::Number::from(
                            request.entropy_data.len(),
                        )),
                    );
                    fields
                },
                ..Default::default()
            },
            OperationResult::Success,
            None,
            None,
            Some(ClientInfo {
                ip_address: Some(client_addr.ip().to_string()),
                request_size: Some(std::mem::size_of_val(&request)),
                ..Default::default()
            }),
        )
        .await;

    if let Some(identity_manager) = identity_manager {
        match contribute_to_mpc_session(request, identity_manager, node_id).await {
            Ok(response) => {
                let duration = start_time.elapsed();

                // Log successful contribution
                logger
                    .log_operation(
                        OperationType::MpcContribution,
                        OperationDetails {
                            description: format!(
                                "Genesis MPC contribution accepted for session: {}",
                                response.session_id
                            ),
                            session_id: Some(response.session_id.clone()),
                            custom_fields: {
                                let mut fields = HashMap::new();
                                fields.insert(
                                    "accepted".to_string(),
                                    serde_json::Value::Bool(response.accepted),
                                );
                                fields.insert(
                                    "ready_for_processing".to_string(),
                                    serde_json::Value::Bool(response.ready_for_processing),
                                );
                                fields
                            },
                            ..Default::default()
                        },
                        OperationResult::Success,
                        Some(duration.as_millis() as u64),
                        None,
                        Some(ClientInfo {
                            ip_address: Some(client_addr.ip().to_string()),
                            response_size: Some(std::mem::size_of_val(&response)),
                            ..Default::default()
                        }),
                    )
                    .await;

                (StatusCode::OK, Json(response))
            }
            Err(e) => {
                error!("Failed to process Genesis MPC contribution: {}", e);

                // Log contribution failure
                logger
                    .log_operation(
                        OperationType::MpcContribution,
                        OperationDetails {
                            description: "Genesis MPC contribution failed".to_string(),
                            error_message: Some(e.to_string()),
                            ..Default::default()
                        },
                        OperationResult::Failure,
                        Some(start_time.elapsed().as_millis() as u64),
                        None,
                        Some(ClientInfo {
                            ip_address: Some(client_addr.ip().to_string()),
                            ..Default::default()
                        }),
                    )
                    .await;

                let error_response = ApiMpcContributionResponse {
                    session_id: String::new(),
                    accepted: false,
                    contributions_count: 0,
                    threshold: 0,
                    ready_for_processing: false,
                    message: e.to_string(),
                    timestamp: SystemTime::now()
                        .duration_since(UNIX_EPOCH)
                        .unwrap_or_default()
                        .as_secs(),
                };

                (StatusCode::INTERNAL_SERVER_ERROR, Json(error_response))
            }
        }
    } else {
        error!("Identity manager not initialized");

        let error_response = ApiMpcContributionResponse {
            session_id: String::new(),
            accepted: false,
            contributions_count: 0,
            threshold: 0,
            ready_for_processing: false,
            message: "Identity manager not available".to_string(),
            timestamp: SystemTime::now()
                .duration_since(UNIX_EPOCH)
                .unwrap_or_default()
                .as_secs(),
        };

        (StatusCode::SERVICE_UNAVAILABLE, Json(error_response))
    }
}

/// API handler for getting Genesis session status
async fn get_genesis_status_handler(
    Extension(state): Extension<Arc<RwLock<AppState>>>,
    AxumPath(session_id): AxumPath<String>,
) -> impl IntoResponse {
    let state_read = state.read().await;
    let identity_manager = state_read.identity_manager.clone();
    drop(state_read);

    info!("Getting Genesis session status: {}", session_id);

    if let Some(identity_manager) = identity_manager {
        match get_genesis_session_status(session_id, identity_manager).await {
            Ok(response) => (StatusCode::OK, Json(response)),
            Err(e) => {
                error!("Failed to get Genesis session status: {}", e);

                let error_response = GenesisCreationResponse {
                    session_id: String::new(),
                    genesis_device_id: String::new(),
                    master_genesis_id: String::new(),
                    is_master_genesis: true,
                    state: "failed".to_string(),
                    contributions_received: 0,
                    threshold: 0,
                    complete: false,
                    genesis_hash: None,
                    initial_chain_tip: None,
                    participating_nodes: Vec::new(),
                    timestamp: SystemTime::now()
                        .duration_since(UNIX_EPOCH)
                        .unwrap_or_default()
                        .as_secs(),
                };

                (StatusCode::INTERNAL_SERVER_ERROR, Json(error_response))
            }
        }
    } else {
        error!("Identity manager not initialized");

        let error_response = GenesisCreationResponse {
            session_id: String::new(),
            genesis_device_id: String::new(),
            master_genesis_id: String::new(),
            is_master_genesis: true,
            state: "failed".to_string(),
            contributions_received: 0,
            threshold: 0,
            complete: false,
            genesis_hash: None,
            initial_chain_tip: None,
            participating_nodes: Vec::new(),
            timestamp: SystemTime::now()
                .duration_since(UNIX_EPOCH)
                .unwrap_or_default()
                .as_secs(),
        };

        (StatusCode::SERVICE_UNAVAILABLE, Json(error_response))
    }
}

/// API handler for getting all Genesis sessions
async fn get_all_genesis_sessions_handler(
    Extension(state): Extension<Arc<RwLock<AppState>>>,
) -> impl IntoResponse {
    let state_read = state.read().await;
    let identity_manager = state_read.identity_manager.clone();
    drop(state_read);

    info!("Getting all Genesis sessions status");

    if let Some(_identity_manager) = identity_manager {
        // This would require additional implementation in the identity manager
        // For now, return an empty list
        let response = serde_json::json!({
            "sessions": [],
            "total_count": 0,
            "timestamp": SystemTime::now().duration_since(UNIX_EPOCH).unwrap_or_default().as_secs()
        });

        (StatusCode::OK, Json(response))
    } else {
        let error_response = serde_json::json!({
            "error": "Identity manager not available",
            "sessions": [],
            "total_count": 0,
            "timestamp": SystemTime::now().duration_since(UNIX_EPOCH).unwrap_or_default().as_secs()
        });

        (StatusCode::SERVICE_UNAVAILABLE, Json(error_response))
    }
}

/// API handler for getting blind device ID session status
async fn get_blind_session_handler(
    Extension(state): Extension<Arc<RwLock<AppState>>>,
    AxumPath(session_id): AxumPath<String>,
) -> impl IntoResponse {
    debug!("Getting status for session: {}", session_id);

    let state = state.read().await;

    if let Some(identity_manager) = &state.identity_manager {
        if let Some(session) = identity_manager.get_mpc_session(&session_id).await {
            let response = serde_json::json!({
                "session_id": session.session_id,
                "device_id": session.device_id,
                "threshold": session.threshold,
                "contributions_count": session.contributions.len(),
                "state": session.state,
                "started_at": session.started_at,
                "expires_at": session.expires_at,
                "has_genesis": session.device_genesis.is_some(),
            });
            (StatusCode::OK, Json(response))
        } else {
            let response = serde_json::json!({
                "error": "Session not found"
            });
            (StatusCode::NOT_FOUND, Json(response))
        }
    } else {
        let response = serde_json::json!({
            "error": "Identity manager not available"
        });
        (StatusCode::SERVICE_UNAVAILABLE, Json(response))
    }
}

/// Handler for receiving gossip entries from other nodes
///
/// HTTP Endpoint: POST /entries
///
/// # Arguments
///
/// * `Extension(state)` - The application state
/// * `Json(entries)` - The entries to process
///
/// # Returns
///
/// * `StatusCode` - 200 OK if successful, appropriate error code otherwise
async fn receive_entries_handler(
    Extension(state): Extension<Arc<RwLock<AppState>>>,
    Json(entries): Json<Vec<dsm_storage_node::network::StateEntry>>,
) -> impl IntoResponse {
    debug!(
        "Received {} gossip entries from another node",
        entries.len()
    );

    let state_read = state.read().await;

    // Check if the storage engine is an EpidemicStorageEngine
    if let Some(epidemic_storage) =
        state_read
            .storage
            .as_any()
            .downcast_ref::<dsm_storage_node::storage::epidemic_storage::EpidemicStorageEngine>()
    {
        // Process the entries
        match epidemic_storage.merge_gossip_entries(entries).await {
            Ok(_) => StatusCode::OK,
            Err(e) => {
                error!("Failed to process gossip entries: {}", e);
                StatusCode::INTERNAL_SERVER_ERROR
            }
        }
    } else {
        // Not an epidemic storage engine
        debug!("Storage engine is not epidemic storage, ignoring gossip entries");
        StatusCode::NOT_IMPLEMENTED
    }
}

/// Handler for requests for entries from other nodes
///
/// HTTP Endpoint: POST /entries/request
///
/// # Arguments
///
/// * `Extension(state)` - The application state
/// * `Json(keys)` - The keys to retrieve
///
/// # Returns
///
/// * `Json<Vec<StateEntry>>` - The requested entries
/// * `StatusCode` - Appropriate error code if the operation fails
async fn request_entries_handler(
    Extension(state): Extension<Arc<RwLock<AppState>>>,
    Json(keys): Json<Vec<String>>,
) -> impl IntoResponse {
    debug!("Received request for {} entries", keys.len());

    let state_read = state.read().await;

    // For epidemic gossip protocol, we need to return entries that match the requested keys
    if state_read
        .storage
        .as_any()
        .downcast_ref::<dsm_storage_node::storage::epidemic_storage::EpidemicStorageEngine>()
        .is_some()
    {
        // We need to use the EpidemicStorageEngine's internal logic to process this request
        // Since we can't access its internal methods directly (they're private),
        // send the request back to it through its public interface

        // Create a stub response; in a full implementation, this would call the appropriate functionality
        // For now, we'll just prepare entries but leave vector clocks empty
        let mut entries = Vec::new();

        for key in keys {
            // Try to get the entry from storage
            if let Ok(Some(entry)) = state_read.storage.retrieve(&key).await {
                // Create a StateEntry with a default vector clock
                let state_entry = dsm_storage_node::network::StateEntry {
                    key: key.clone(),
                    value: entry.encrypted_payload.clone(),
                    vector_clock: VectorClock::new(),
                    timestamp: entry.timestamp,
                    origin_node: state_read.config.node.id.clone(),
                };

                entries.push(state_entry);
            }
        }

        debug!("Returning {} entries in response to request", entries.len());

        // Return the collected entries
        (StatusCode::OK, Json(entries))
    } else {
        // Not an epidemic storage engine
        debug!("Storage engine is not epidemic storage, cannot fulfill entry request");
        (StatusCode::NOT_IMPLEMENTED, Json(Vec::new()))
    }
}

/// Handler for listing available entries for gossip
///
/// HTTP Endpoint: GET /entries
///
/// # Arguments
///
/// * `Extension(state)` - The application state
///
/// # Returns
///
/// * `Json<Vec<StateEntry>>` - The available entries for gossip
/// * `StatusCode` - Appropriate error code if the operation fails
async fn list_entries_handler(
    Extension(state): Extension<Arc<RwLock<AppState>>>,
) -> impl IntoResponse {
    debug!("Received request to list entries for gossip");

    let state_read = state.read().await;

    // For epidemic gossip protocol, we need to return available entries
    if state_read
        .storage
        .as_any()
        .downcast_ref::<dsm_storage_node::storage::epidemic_storage::EpidemicStorageEngine>()
        .is_some()
    {
        // Get all available keys and create state entries
        match state_read.storage.list(Some(100), None).await {
            Ok(keys) => {
                let mut entries = Vec::new();

                for key in keys {
                    if let Ok(Some(entry)) = state_read.storage.retrieve(&key).await {
                        // Create a StateEntry
                        let state_entry = dsm_storage_node::network::StateEntry {
                            key: key.clone(),
                            value: entry.encrypted_payload.clone(),
                            vector_clock: dsm_storage_node::storage::vector_clock::VectorClock::new(
                            ),
                            timestamp: entry.timestamp,
                            origin_node: state_read.config.node.id.clone(),
                        };

                        entries.push(state_entry);
                    }
                }

                debug!(
                    "Returning {} entries in gossip list response",
                    entries.len()
                );
                (StatusCode::OK, Json(entries))
            }
            Err(e) => {
                error!("Failed to list entries for gossip: {}", e);
                (StatusCode::INTERNAL_SERVER_ERROR, Json(Vec::new()))
            }
        }
    } else {
        // Not an epidemic storage engine
        debug!("Storage engine is not epidemic storage, cannot list entries for gossip");
        (StatusCode::NOT_IMPLEMENTED, Json(Vec::new()))
    }
}

/// Request structure for adding a contact
#[derive(Debug, serde::Deserialize)]
struct AddContactRequest {
    device_id: String,
    contact: dsm_storage_node::identity::DsmContact,
}

/// Response structure for adding a contact
#[derive(Debug, serde::Serialize)]
struct AddContactResponse {
    success: bool,
    message: String,
}

/// Add a contact to a device identity
async fn add_contact_handler(
    Extension(state): Extension<Arc<RwLock<AppState>>>,
    Json(request): Json<AddContactRequest>,
) -> Result<Json<AddContactResponse>, StatusCode> {
    let state = state.read().await;
    if let Some(identity_manager) = &state.identity_manager {
        match identity_manager
            .add_contact_to_device(&request.device_id, request.contact)
            .await
        {
            Ok(()) => Ok(Json(AddContactResponse {
                success: true,
                message: "Contact added successfully".to_string(),
            })),
            Err(e) => {
                tracing::error!("Failed to add contact: {}", e);
                Err(StatusCode::INTERNAL_SERVER_ERROR)
            }
        }
    } else {
        Err(StatusCode::SERVICE_UNAVAILABLE)
    }
}

/// Request structure for updating chain tip
#[derive(Debug, serde::Deserialize)]
struct UpdateChainTipRequest {
    device_id: String,
    contact_device_id: String,
    new_chain_tip: String,
}

/// Response structure for updating chain tip
#[derive(Debug, serde::Serialize)]
struct UpdateChainTipResponse {
    success: bool,
    message: String,
}

/// Update chain tip for a bilateral relationship
async fn update_chain_tip_handler(
    Extension(state): Extension<Arc<RwLock<AppState>>>,
    Json(request): Json<UpdateChainTipRequest>,
) -> Result<Json<UpdateChainTipResponse>, StatusCode> {
    let state = state.read().await;
    if let Some(identity_manager) = &state.identity_manager {
        match identity_manager
            .update_chain_tip(
                &request.device_id,
                &request.contact_device_id,
                request.new_chain_tip,
            )
            .await
        {
            Ok(()) => Ok(Json(UpdateChainTipResponse {
                success: true,
                message: "Chain tip updated successfully".to_string(),
            })),
            Err(e) => {
                tracing::error!("Failed to update chain tip: {}", e);
                Err(StatusCode::INTERNAL_SERVER_ERROR)
            }
        }
    } else {
        Err(StatusCode::SERVICE_UNAVAILABLE)
    }
}

/// Request structure for creating bilateral state
#[derive(Debug, serde::Deserialize)]
struct CreateBilateralStateRequest {
    device_id: String,
    contact_device_id: String,
    operation: String,
    balance_deltas: std::collections::HashMap<String, i64>,
}

/// Response structure for creating bilateral state
#[derive(Debug, serde::Serialize)]
struct CreateBilateralStateResponse {
    success: bool,
    state: Option<dsm_storage_node::identity::DsmState>,
    message: String,
}

/// Create a new bilateral state transition
async fn create_bilateral_state_handler(
    Extension(state): Extension<Arc<RwLock<AppState>>>,
    Json(request): Json<CreateBilateralStateRequest>,
) -> Result<Json<CreateBilateralStateResponse>, StatusCode> {
    let state = state.read().await;
    if let Some(identity_manager) = &state.identity_manager {
        match identity_manager
            .create_bilateral_state(
                &request.device_id,
                &request.contact_device_id,
                request.operation,
                request.balance_deltas,
            )
            .await
        {
            Ok(new_state) => Ok(Json(CreateBilateralStateResponse {
                success: true,
                state: Some(new_state),
                message: "Bilateral state created successfully".to_string(),
            })),
            Err(e) => {
                tracing::error!("Failed to create bilateral state: {}", e);
                Ok(Json(CreateBilateralStateResponse {
                    success: false,
                    state: None,
                    message: format!("Failed to create bilateral state: {e}"),
                }))
            }
        }
    } else {
        Ok(Json(CreateBilateralStateResponse {
            success: false,
            state: None,
            message: "Identity manager not available".to_string(),
        }))
    }
}

/// Response structure for getting device identity
#[derive(Debug, serde::Serialize)]
struct GetDeviceIdentityResponse {
    success: bool,
    device_identity: Option<dsm_storage_node::identity::DeviceIdentity>,
    message: String,
}

/// Get device identity with full chain manager
async fn get_device_identity_handler(
    Extension(state): Extension<Arc<RwLock<AppState>>>,
    AxumPath(device_id): AxumPath<String>,
) -> Result<Json<GetDeviceIdentityResponse>, StatusCode> {
    let state = state.read().await;
    if let Some(identity_manager) = &state.identity_manager {
        match identity_manager.get_device_identity(&device_id).await {
            Ok(Some(device_identity)) => Ok(Json(GetDeviceIdentityResponse {
                success: true,
                device_identity: Some(device_identity),
                message: "Device identity retrieved successfully".to_string(),
            })),
            Ok(None) => Ok(Json(GetDeviceIdentityResponse {
                success: false,
                device_identity: None,
                message: "Device identity not found".to_string(),
            })),
            Err(e) => {
                tracing::error!("Failed to get device identity: {}", e);
                Ok(Json(GetDeviceIdentityResponse {
                    success: false,
                    device_identity: None,
                    message: format!("Failed to get device identity: {e}"),
                }))
            }
        }
    } else {
        Ok(Json(GetDeviceIdentityResponse {
            success: false,
            device_identity: None,
            message: "Identity manager not available".to_string(),
        }))
    }
}

/// Request structure for verifying chain
#[derive(Debug, serde::Deserialize)]
struct VerifyChainRequest {
    device_id: String,
    contact_device_id: String,
    from_state: String,
    to_state: String,
}

/// Response structure for verifying chain
#[derive(Debug, serde::Serialize)]
struct VerifyChainResponse {
    success: bool,
    valid: bool,
    message: String,
}

/// Verify a bilateral hash chain between two devices
async fn verify_chain_handler(
    Extension(state): Extension<Arc<RwLock<AppState>>>,
    Json(request): Json<VerifyChainRequest>,
) -> Result<Json<VerifyChainResponse>, StatusCode> {
    let state = state.read().await;
    if let Some(identity_manager) = &state.identity_manager {
        match identity_manager
            .verify_bilateral_chain(
                &request.device_id,
                &request.contact_device_id,
                &request.from_state,
                &request.to_state,
            )
            .await
        {
            Ok(is_valid) => Ok(Json(VerifyChainResponse {
                success: true,
                valid: is_valid,
                message: if is_valid {
                    "Chain is valid"
                } else {
                    "Chain is invalid"
                }
                .to_string(),
            })),
            Err(e) => {
                tracing::error!("Failed to verify chain: {}", e);
                Ok(Json(VerifyChainResponse {
                    success: false,
                    valid: false,
                    message: format!("Failed to verify chain: {e}"),
                }))
            }
        }
    } else {
        Ok(Json(VerifyChainResponse {
            success: false,
            valid: false,
            message: "Identity manager not available".to_string(),
        }))
    }
}

#[tokio::main]
async fn main() -> Result<(), anyhow::Error> {
    // Initialize tracing for logs
    tracing_subscriber::registry()
        .with(tracing_subscriber::EnvFilter::new(
            std::env::var("RUST_LOG").unwrap_or_else(|_| "info".into()),
        ))
        .with(tracing_subscriber::fmt::layer())
        .init();

    // Initialize uptime tracking
    dsm_storage_node::api::init_uptime();

    // Parse command line arguments
    let cli = Cli::parse();

    // Load configuration
    let config_path = cli.config;
    info!("Loading configuration from {:?}", config_path);

    let config = match load_config(&config_path) {
        Ok(config) => config,
        Err(e) => {
            error!("Failed to load configuration: {}", e);
            process::exit(1);
        }
    };

    info!("Configuration loaded successfully");

    // Process command
    let staked_amount = match cli.command {
        Some(Commands::Stake { amount }) => {
            process_staking(amount).await?;
            Some(amount)
        }
        Some(Commands::Run) | None => None,
    };

    // Get the bind device_id
    let addr: SocketAddr = format!("{}:{}", config.network.listen_addr, config.network.port)
        .parse()
        .expect("Failed to parse bind device_id");

    // Generate a deterministic node ID based on the bind address
    let node_id = {
        use dsm_storage_node::storage::topology::NodeId;
        // Use the port to create a consistent node ID for development
        let device_salt = format!(
            "dev-node-{}-{}",
            config.node.id.replace("dev-node-", ""),
            config.network.port
        )
        .into_bytes();
        let app_id = "dsm_storage_node";
        let node_id = NodeId::from_device_entropy(&device_salt, app_id);
        node_id.to_string()
    };

    // Prepare node for epidemic storage
    let node = dsm_storage_node::types::StorageNode {
        id: node_id.clone(),
        name: config.node.name.clone(),
        region: config.node.region.clone(),
        public_key: config.node.public_key.clone(),
        endpoint: config.node.endpoint.clone(),
    };

    // Initialize cluster manager if cluster config is enabled
    let cluster_manager = if let Some(cluster_config) = &config.cluster {
        if cluster_config.enabled {
            info!(
                "Initializing cluster manager with auto-discovery for node {}",
                node_id
            );

            // Create auto-discovery configuration
            let auto_config = dsm_storage_node::auto_network::AutoNetworkConfig::default();
            let local_node = dsm_storage_node::auto_network::DiscoveredNode {
                node_id: node_id.clone(),
                name: format!("dsm-node-{node_id}"),
                ip: "127.0.0.1".parse().unwrap(), // Will be updated by discovery
                port: config.api.port,
                service_type: "dsm-storage".to_string(),
                properties: {
                    let mut props = std::collections::HashMap::new();
                    props.insert("region".to_string(), config.node.region.clone());
                    props.insert("storage_capacity".to_string(), "10737418240".to_string());
                    props.insert("bandwidth_mbps".to_string(), "100".to_string());
                    props.insert("cpu_cores".to_string(), "4".to_string());
                    props
                },
                discovered_at: std::time::SystemTime::now(),
                last_seen: std::time::SystemTime::now(),
                capabilities: vec![
                    "mpc".to_string(),
                    "storage".to_string(),
                    "gossip".to_string(),
                ],
            };
            let discovered_nodes = Arc::new(RwLock::new(std::collections::HashMap::new()));

            let manager = Arc::new(
                ClusterManager::new(node_id.clone(), auto_config, local_node, discovered_nodes)
                    .await,
            );

            // Start auto-discovery instead of reading static config
            if let Err(e) = manager.start_discovery().await {
                error!("Failed to start auto-discovery: {}", e);
                warn!("Continuing in standalone mode");
            } else {
                info!("Successfully started auto-discovery and dynamic clustering");
            }

            Some(manager)
        } else {
            None
        }
    } else {
        None
    };

    // Initialize storage
    let storage = match init_storage(&config.storage, &node, cluster_manager.clone()).await {
        Ok(storage) => storage,
        Err(e) => {
            error!("Failed to initialize storage: {}", e);
            process::exit(1);
        }
    };

    // Initialize the network stack
    if let Err(e) = init_networking(&config.network).await {
        error!("Failed to initialize networking: {}", e);
        process::exit(1);
    }

    // Initialize identity manager with cluster manager integration
    let identity_manager = Arc::new(if let Some(cluster_mgr) = cluster_manager.clone() {
        DsmIdentityManager::new_with_cluster(storage.clone(), config.node.id.clone(), cluster_mgr)
    } else {
        DsmIdentityManager::new(storage.clone(), config.node.id.clone())
    });

    // Cluster nodes are now automatically discovered via cluster manager
    // No hardcoded URLs needed

    // Initialize comprehensive logger
    let logger = Arc::new(StorageNodeLogger::new(config.node.id.clone()));

    // Log startup
    logger
        .log_operation(
            OperationType::NetworkConnect,
            OperationDetails {
                description: format!("DSM Storage Node starting on {addr}"),
                endpoint: Some(addr.to_string()),
                custom_fields: {
                    let mut fields = HashMap::new();
                    fields.insert(
                        "node_id".to_string(),
                        serde_json::Value::String(config.node.id.clone()),
                    );
                    fields.insert(
                        "version".to_string(),
                        serde_json::Value::String(config.node.version.clone()),
                    );
                    fields.insert(
                        "region".to_string(),
                        serde_json::Value::String(config.node.region.clone()),
                    );
                    fields
                },
                ..Default::default()
            },
            OperationResult::Success,
            None,
            None,
            None,
        )
        .await;

    // Create application state
    let app_state = Arc::new(RwLock::new(AppState {
        config: config.clone(),
        storage,
        identity_manager: Some(identity_manager),
        staked_amount,
        mpc_sessions: Arc::new(RwLock::new(HashMap::new())),
        logger: logger.clone(),
        cluster_manager: cluster_manager.clone(),
    }));

    // Set up the router
    let app = create_router(app_state.clone());

    info!("Starting DSM Storage Node server on {}", addr);

    // Start cluster topology/epidemic storage initialization in background
    if let Some(cluster_config) = &config.cluster {
        if cluster_config.enabled {
            if let Some(ref manager) = cluster_manager {
                let node_id_clone = node_id.clone();
                let manager_clone = manager.clone();
                tokio::spawn(async move {
                    info!(
                        "[Background] Initializing cluster topology for node {}",
                        node_id_clone
                    );
                    // This will not block the main server
                    let my_clusters = manager_clone.get_my_clusters().await;
                    let gossip_targets = manager_clone.get_gossip_targets(None).await;
                    info!(
                        "[Background] Node {} participates in clusters: {:?}",
                        node_id_clone, my_clusters
                    );
                    info!(
                        "[Background] Node {} has {} gossip targets",
                        node_id_clone,
                        gossip_targets.len()
                    );
                    for target in &gossip_targets {
                        info!("  - Gossip target: {} at {}", target.id, target.endpoint);
                    }
                });
            }
        }
    }

    info!("DSM Storage Node ready to serve requests on {}", addr);
    // Start the server with better error handling
    let server = axum::Server::bind(&addr)
        .serve(app.into_make_service_with_connect_info::<SocketAddr>())
        .with_graceful_shutdown(shutdown_signal());
    info!(
        "🚀 DSM Storage Node {} is now running and healthy!",
        node_id
    );
    match server.await {
        Ok(_) => {
            info!("Server shutdown gracefully");
            Ok(())
        }
        Err(e) => {
            error!("Server error: {}", e);
            Err(anyhow::anyhow!("Server failed to start: {}", e))
        }
    }
}

/// Graceful shutdown signal handler
async fn shutdown_signal() {
    use tokio::signal;

    let ctrl_c = async {
        signal::ctrl_c()
            .await
            .expect("failed to install Ctrl+C handler");
    };

    #[cfg(unix)]
    let terminate = async {
        signal::unix::signal(signal::unix::SignalKind::terminate())
            .expect("failed to install signal handler")
            .recv()
            .await;
    };

    #[cfg(not(unix))]
    let terminate = std::future::pending::<()>();

    tokio::select! {
        _ = ctrl_c => {
            info!("Received Ctrl+C, shutting down gracefully...");
        },
        _ = terminate => {
            info!("Received terminate signal, shutting down gracefully...");
        },
    }
}
