// DSM Storage Node API Module
//
// This module implements the HTTP REST API for the storage node, providing endpoints for data operations,
// node management, and administrative functions. The API is built using the Axum framework and provides
// a comprehensive set of endpoints for interacting with the DSM Storage Node.
//
// # API Endpoints
//
// The API is organized into several logical groups:
//
// * **Data Operations**: Core storage functionality (get, put, delete, list)
// * **Inbox API**: Message delivery for unilateral transactions
// * **Vault API**: Secure storage for sensitive data with access controls
// * **Rewards API**: Integration with the DSM staking and rewards system
// * **Node Management**: Status, configuration, and peer management
//
// # Authentication
//
// The API supports multiple authentication methods:
// - API tokens
// - Public key signatures
// - Certificate-based authentication
//
// # Error Handling
//
// All API endpoints use standardized error responses with consistent status codes
// and structured error messages to simplify client-side error handling.
//
// # Examples
//
// ## Basic Usage
//
// ```rust
// use dsm_storage_node::api::ApiServer;
// use dsm_storage_node::storage::SqliteStorageEngine;
// use dsm_storage_node::staking::StakingService;
// use std::sync::Arc;
//
// async fn start_server() -> Result<(), Box<dyn std::error::Error>> {
//     // Initialize storage and staking services
//     let storage = Arc::new(SqliteStorageEngine::new("data/storage.db")?);
//     let staking = Arc::new(StakingService::new(/* config */)?);
//
//     // Create and start API server
//     let api = ApiServer::new(storage, staking, "127.0.0.1:8765".to_string());
//     api.start().await?;
//
//     Ok(())
// }
// ```
//
// ## Using the API with curl
//
// ```bash
// # Store data
// curl -X POST -H "Content-Type: application/octet-stream" --data-binary "@file.bin" http://localhost:8765/data
//
// # Retrieve data
// curl -X GET http://localhost:8765/data/b43f1d...
//
// # Check node status
// curl -X GET http://localhost:8765/health
// ```

use crate::error::{Result, StorageNodeError};
// Removed unused import

use crate::identity::DsmIdentityManager;
use crate::staking::StakingService;
use axum::{
    extract::{Path, State},
    http::StatusCode,
    response::{IntoResponse, Response},
    routing::{delete, get, post, put},
    Json, Router,
};
use serde::Serialize;
use std::sync::Arc;
use tower_http::trace::TraceLayer;
use tracing::info;

pub mod axum_handlers;
mod handlers;
mod middleware;
pub mod mpc_api;
mod rewards_api;
mod unilateral_api;
mod unilateral_blinded;
mod vault_api;
pub use handlers::*;
pub use mpc_api::*;
pub use rewards_api::*;
pub use unilateral_api::*;
pub use vault_api::*;

// Explicitly re-export init_uptime for main binary
pub use handlers::init_uptime;

/// Application state shared with all routes.
///
/// This struct holds references to core components that are needed by
/// request handlers, such as the storage engine and staking service.
/// The state is cloned for each request but uses Arc internally to
/// avoid expensive deep copies.
#[derive(Clone)]
pub struct AppState {
    /// Storage engine for persisting and retrieving data
    pub storage: Arc<dyn crate::storage::StorageEngine + Send + Sync>,
    /// Staking service for rewards and validation
    pub staking_service: Arc<StakingService>,
    /// Identity manager for MPC sessions
    pub identity_manager: Option<Arc<DsmIdentityManager>>,
}

/// API Error response model.
///
/// This struct provides a standardized format for all error responses
/// from the API. It includes a human-readable message, a machine-readable
/// error code, and optional structured details for more complex errors.
///
/// Error codes are mapped to appropriate HTTP status codes in the
/// `IntoResponse` implementation.
#[derive(Debug, Serialize)]
pub struct ApiError {
    /// Human-readable error message
    pub message: String,
    /// Machine-readable error code (e.g., "NOT_FOUND", "BAD_REQUEST")
    pub code: String,
    /// Optional additional structured details about the error
    pub details: Option<serde_json::Value>,
}

impl IntoResponse for ApiError {
    fn into_response(self) -> Response {
        let status = match self.code.as_str() {
            "NOT_FOUND" => StatusCode::NOT_FOUND,
            "BAD_REQUEST" => StatusCode::BAD_REQUEST,
            "UNAUTHORIZED" => StatusCode::UNAUTHORIZED,
            "FORBIDDEN" => StatusCode::FORBIDDEN,
            "CONFLICT" => StatusCode::CONFLICT,
            "TOO_MANY_REQUESTS" => StatusCode::TOO_MANY_REQUESTS,
            _ => StatusCode::INTERNAL_SERVER_ERROR,
        };

        let body = Json(self);

        (status, body).into_response()
    }
}

/// Convert StorageNodeError to an API error.
///
/// This implementation maps internal storage node errors to API-friendly
/// error responses with appropriate status codes and messages.
impl From<StorageNodeError> for ApiError {
    fn from(err: StorageNodeError) -> Self {
        let (code, message) = match err {
            StorageNodeError::NetworkClientNotSet => {
                ("NETWORK_ERROR", "Network client not set".to_string())
            }
            StorageNodeError::InvalidNodeId(msg) => ("INVALID_NODE_ID", msg),
            StorageNodeError::Timeout => ("TIMEOUT", "Operation timed out".to_string()),
            StorageNodeError::Internal(msg) => ("INTERNAL_ERROR", msg),
            StorageNodeError::Configuration => {
                ("CONFIGURATION_ERROR", "Configuration error".to_string())
            }
            StorageNodeError::NotFound(msg) => ("NOT_FOUND", msg),
            StorageNodeError::Storage(msg) => ("STORAGE_ERROR", msg),
            StorageNodeError::CryptoError { message } => ("CRYPTO_ERROR", message),
            StorageNodeError::Config(msg) => ("CONFIG_ERROR", msg),
            StorageNodeError::Encryption(msg) => ("ENCRYPTION_ERROR", msg),
            StorageNodeError::Distribution(msg) => ("DISTRIBUTION_ERROR", msg),
            StorageNodeError::NodeManagement(msg) => ("NODE_MANAGEMENT_ERROR", msg),
            StorageNodeError::Staking(msg) => ("STAKING_ERROR", msg),
            StorageNodeError::Authentication(msg) => ("AUTHENTICATION_ERROR", msg),
            StorageNodeError::InvalidState(msg) => ("INVALID_STATE", msg),
            StorageNodeError::Database(msg) => ("DATABASE_ERROR", msg),
            StorageNodeError::Serialization(msg) => ("SERIALIZATION_ERROR", msg),
            StorageNodeError::IO(err) => ("IO_ERROR", err.to_string()),
            StorageNodeError::Json(err) => ("JSON_ERROR", err.to_string()),
            StorageNodeError::Sqlite(err) => ("SQLITE_ERROR", err.to_string()),
            StorageNodeError::Request(err) => ("REQUEST_ERROR", err.to_string()),
            StorageNodeError::Network(err) => ("NETWORK_ERROR", err.to_string()),
            StorageNodeError::Unknown(msg) => ("UNKNOWN_ERROR", msg),
            StorageNodeError::RateLimitExceeded(msg) => ("RATE_LIMIT_EXCEEDED", msg),
            StorageNodeError::TaskCancelled(msg) => ("TASK_CANCELLED", msg),
            StorageNodeError::TaskFailed(msg) => ("TASK_FAILED", msg),
            StorageNodeError::QueueFull(msg) => ("QUEUE_FULL", msg),
            StorageNodeError::ReceiveFailure(msg) => ("RECEIVE_FAILURE", msg),
            StorageNodeError::InvalidOperation(msg) => ("INVALID_OPERATION", msg),
            StorageNodeError::InvalidInput(msg) => ("INVALID_INPUT", msg),
            StorageNodeError::ConcurrencyLimitExceeded => (
                "CONCURRENCY_LIMIT_EXCEEDED",
                "Concurrency limit exceeded".to_string(),
            ),
            StorageNodeError::NotImplemented(msg) => ("NOT_IMPLEMENTED", msg),
            StorageNodeError::BadRequest(msg) => ("BAD_REQUEST", msg),
            StorageNodeError::Validation(msg) => ("VALIDATION_ERROR", msg),
            StorageNodeError::InvalidConfiguration(msg) => ("INVALID_CONFIGURATION", msg),
            StorageNodeError::Crypto { context } => ("CRYPTO_ERROR", context),
            StorageNodeError::Integrity { context } => ("INTEGRITY_ERROR", context),
            StorageNodeError::InvalidPublicKey => {
                ("INVALID_PUBLIC_KEY", "Invalid public key".to_string())
            }
            StorageNodeError::InvalidSecretKey => {
                ("INVALID_SECRET_KEY", "Invalid secret key".to_string())
            }
            StorageNodeError::InvalidKeyLength => {
                ("INVALID_KEY_LENGTH", "Invalid key length".to_string())
            }
            StorageNodeError::InvalidCiphertext => {
                ("INVALID_CIPHERTEXT", "Invalid ciphertext".to_string())
            }
            StorageNodeError::StateMachine(msg) => ("STATE_MACHINE_ERROR", msg),
            StorageNodeError::Genesis(msg) => ("GENESIS_ERROR", msg),
            StorageNodeError::Policy(msg) => ("POLICY_ERROR", msg),
            StorageNodeError::Vault(msg) => ("VAULT_ERROR", msg),
        };

        Self {
            message,
            code: code.to_string(),
            details: None,
        }
    }
}

/// The main API server for the DSM Storage Node.
///
/// This struct represents the HTTP server that exposes the storage node's
/// functionality via a RESTful API. It handles initialization, routing, and
/// starting the server on the specified device_id.
///
/// # Examples
///
/// ```rust,no_run
/// use dsm_storage_node::api::ApiServer;
/// use dsm_storage_node::storage::memory_storage::MemoryStorage;
/// use dsm_storage_node::staking::{StakingService, StakingConfig};
/// use std::sync::Arc;
///
/// async fn example() -> Result<(), Box<dyn std::error::Error>> {
///     let storage = Arc::new(MemoryStorage::new(Default::default()));
///     let staking_config = StakingConfig::default();
///     let staking = Arc::new(StakingService::new(staking_config));
///     
///     let server = ApiServer::new(storage, staking, None, "127.0.0.1:8765".to_string());
///     server.start().await?;
///     
///     Ok(())
/// }
/// ```
pub struct ApiServer {
    /// Application state shared with all request handlers
    app_state: Arc<AppState>,
    /// Server bind device_id in the format "IP:port"
    bind_device_id: String,
}

impl ApiServer {
    /// Create a new API server instance.
    ///
    /// This constructor initializes the API server with the required dependencies
    /// and prepares it for starting. It does not actually start the server -
    /// call `start()` to begin serving requests.
    ///
    /// # Parameters
    ///
    /// * `storage` - An Arc-wrapped storage engine implementation
    /// * `staking_service` - An Arc-wrapped staking service implementation
    /// * `identity_manager` - An optional Arc-wrapped identity manager for MPC sessions
    /// * `bind_device_id` - The device_id and port to bind the server to (e.g., "127.0.0.1:8765")
    ///
    /// # Returns
    ///
    /// A new `ApiServer` instance ready to be started
    pub fn new(
        storage: Arc<dyn crate::storage::StorageEngine + Send + Sync>,
        staking_service: Arc<StakingService>,
        identity_manager: Option<Arc<DsmIdentityManager>>,
        bind_device_id: String,
    ) -> Self {
        let app_state = Arc::new(AppState {
            storage,
            staking_service,
            identity_manager,
        });

        Self {
            app_state,
            bind_device_id,
        }
    }

    /// Start the API server and begin serving requests.
    ///
    /// This method binds to the configured device_id and port, sets up the
    /// HTTP server with all routes, and begins handling incoming requests.
    /// It is an async method that doesn't return until the server is shut down
    /// or encounters an error.
    ///
    /// # Returns
    ///
    /// * `Ok(())` if the server was successfully started and then gracefully shut down
    /// * `Err` if there was an error starting or running the server
    pub async fn start(&self) -> Result<()> {
        // Create router with routes
        let app = self.create_router().layer(TraceLayer::new_for_http());

        // Parse the bind device_id
        let addr = self
            .bind_device_id
            .parse()
            .map_err(|e| StorageNodeError::Config(format!("Invalid bind device_id: {e}")))?;

        info!("Starting API server on {}", self.bind_device_id);

        // Start the server
        axum::Server::bind(&addr)
            .serve(app.into_make_service())
            .await
            .map_err(|e| StorageNodeError::Config(format!("Server error: {e}")))?;

        Ok(())
    }

    /// Create the API router with all defined routes.
    ///
    /// This method defines all the HTTP endpoints for the API server,
    /// including their HTTP methods, paths, and handler functions.
    /// It also attaches the application state to the router.
    ///
    /// # Returns
    ///
    /// An Axum `Router` configured with all API endpoints
    fn create_router(&self) -> Router {
        // Build the router
        Router::new()
            // Health and status endpoints
            .route("/health", get(handlers::health_check))
            .route("/api/v1/health", get(handlers::health_check)) // Standard API health endpoint
            .route("/stats", get(handlers::node_stats))
            .route("/api/v1/status", get(handlers::node_stats)) // Standard API status endpoint
            // General data storage endpoints
            .route("/data", post(handlers::store_data))
            .route("/data/:blinded_id", get(handlers::retrieve_data))
            .route("/data/:blinded_id", delete(handlers::delete_data))
            .route("/data/:blinded_id/exists", get(handlers::exists_data))
            .route("/data", get(handlers::list_data))
            // Genesis creation endpoints (DSM Protocol Compliant - G = H(b1 || b2 || ... || bt || Aux))
            .route("/api/v1/genesis/create", post(create_genesis_handler))
            .route(
                "/api/v1/genesis/session/:session_id",
                get(get_genesis_session_handler),
            )
            .route("/api/v1/genesis/contribute", post(blind_contribute_handler)) // MPC contribution endpoint
            .route("/api/v1/blind/contribute", post(blind_contribute_handler))
            // Production-ready unilateral transaction inbox endpoints
            .route(
                "/api/v1/inbox/submit",
                post(unilateral_api::submit_inbox_transaction),
            )
            .route(
                "/api/v1/inbox/retrieve",
                post(unilateral_api::retrieve_inbox_transactions),
            )
            .route(
                "/api/v1/inbox/acknowledge",
                post(unilateral_api::acknowledge_inbox_transactions),
            )
            .route(
                "/api/v1/inbox/:mailbox_id/status",
                get(unilateral_api::get_inbox_status),
            )
            // Blinded inbox endpoints for trustless storage (storage node cannot decrypt)
            .route(
                "/api/v1/blinded/inbox/submit",
                post(unilateral_blinded::submit_inbox_transaction),
            )
            // Legacy unilateral transaction inbox endpoints (backward compatibility)
            // These are deprecated and mapped to the new API handlers for compatibility
            .route("/inbox", post(unilateral_api::submit_inbox_transaction))
            .route(
                "/inbox/:recipient_genesis",
                get(unilateral_api::retrieve_inbox_transactions),
            )
            .route(
                "/inbox/:recipient_genesis/:entry_id",
                delete(unilateral_api::acknowledge_inbox_transactions),
            )
            // Vault API endpoints
            .route("/vault", post(store_vault))
            .route("/vault/:vault_id", get(get_vault))
            .route("/vault/creator/:creator_id", get(get_vaults_by_creator))
            .route(
                "/vault/recipient/:recipient_id",
                get(get_vaults_by_recipient),
            )
            .route("/vault/:vault_id/status", put(update_vault_status))
            // Rewards API endpoints
            .merge(rewards_api::rewards_routes())
            // Share application state
            .with_state(self.app_state.clone())
    }
}

// Genesis API HTTP Handlers (DSM Protocol Compliant)

/// HTTP handler for creating Genesis device ID via MPC
/// Implements DSM protocol: G = H(b1 || b2 || ... || bt || Aux)
/// Genesis device ID is OUTPUT, not input
async fn create_genesis_handler(
    State(state): State<Arc<AppState>>,
    Json(request): Json<mpc_api::GenesisCreationRequest>,
) -> impl IntoResponse {
    if let Some(ref identity_manager) = state.identity_manager {
        match mpc_api::create_genesis_identity(request, identity_manager.clone()).await {
            Ok(response) => (StatusCode::OK, Json(response)).into_response(),
            Err(err) => {
                let api_error = ApiError::from(err);
                api_error.into_response()
            }
        }
    } else {
        let api_error = ApiError {
            message: "Identity manager not available".to_string(),
            code: "SERVICE_UNAVAILABLE".to_string(),
            details: None,
        };
        api_error.into_response()
    }
}

/// HTTP handler for getting Genesis session status
/// Returns the current state of MPC Genesis creation session
async fn get_genesis_session_handler(
    State(state): State<Arc<AppState>>,
    Path(session_id): Path<String>,
) -> impl IntoResponse {
    if let Some(ref identity_manager) = state.identity_manager {
        match mpc_api::get_genesis_session_status(session_id, identity_manager.clone()).await {
            Ok(response) => (StatusCode::OK, Json(response)).into_response(),
            Err(err) => {
                let api_error = ApiError::from(err);
                api_error.into_response()
            }
        }
    } else {
        let api_error = ApiError {
            message: "Identity manager not available".to_string(),
            code: "SERVICE_UNAVAILABLE".to_string(),
            details: None,
        };
        api_error.into_response()
    }
}

/// HTTP handler for contributing to MPC sessions
async fn blind_contribute_handler(
    State(state): State<Arc<AppState>>,
    Json(request): Json<mpc_api::MpcContributionRequest>,
) -> impl IntoResponse {
    if let Some(ref identity_manager) = state.identity_manager {
        // Get node ID from the identity manager or use the request's node_id as fallback
        let node_id = request.node_id.clone();

        match mpc_api::contribute_to_mpc_session(request, identity_manager.clone(), node_id).await {
            Ok(response) => (StatusCode::OK, Json(response)).into_response(),
            Err(err) => {
                let api_error = ApiError::from(err);
                api_error.into_response()
            }
        }
    } else {
        let api_error = ApiError {
            message: "Identity manager not available".to_string(),
            code: "SERVICE_UNAVAILABLE".to_string(),
            details: None,
        };
        api_error.into_response()
    }
}
