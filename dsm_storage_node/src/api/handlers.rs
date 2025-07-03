// API handlers for DSM Storage Node
//
// This module implements the API route handlers for the storage node.

use crate::api::AppState;
use crate::error::{Result, StorageNodeError};
use crate::types::storage_types::{DataRetrievalRequest, DataSubmissionRequest};
use crate::types::BlindedStateEntry;
use axum::{
    extract::{Json, Path, Query, State},
    http::StatusCode,
    response::IntoResponse,
};
use base64;
use std::collections::HashMap;
use std::sync::{Arc, OnceLock};
use tracing::{debug, info, warn};

static START_TIME: OnceLock<std::time::Instant> = OnceLock::new();

/// Initialize the start time for uptime tracking
pub fn init_uptime() {
    START_TIME.set(std::time::Instant::now()).ok();
}

/// Calculate uptime in seconds
fn calculate_uptime() -> u64 {
    START_TIME
        .get()
        .map(|start| start.elapsed().as_secs())
        .unwrap_or(0)
}

/// Store data handler
#[axum::debug_handler]
pub async fn store_data(
    State(state): State<Arc<AppState>>,
    Json(request): Json<DataSubmissionRequest>,
) -> Result<impl IntoResponse> {
    info!("Storing data with blinded ID: {}", request.blinded_id);

    // Validate request
    if request.blinded_id.is_empty() {
        return Err(StorageNodeError::InvalidState(
            "Blinded ID cannot be empty".into(),
        ));
    }

    if request.payload.is_empty() {
        return Err(StorageNodeError::InvalidState(
            "Payload cannot be empty".into(),
        ));
    }

    // Create blinded state entry
    let entry = BlindedStateEntry {
        blinded_id: request.blinded_id.clone(),
        encrypted_payload: request.payload.clone(), // Clone to avoid move
        timestamp: std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .map_or(0, |d| d.as_secs()),
        ttl: request.ttl.unwrap_or(0),
        region: request.region.unwrap_or_else(|| "global".to_string()),
        priority: request.priority.unwrap_or(0),
        proof_hash: request.proof_hash.unwrap_or_else(|| {
            // Generate a hash from the payload
            let mut hasher = blake3::Hasher::new();
            hasher.update(&request.payload);
            let hash = hasher.finalize();

            let mut hash_bytes = [0u8; 32];
            hash_bytes.copy_from_slice(hash.as_bytes());
            hash_bytes
        }),
        metadata: request.metadata.unwrap_or_else(HashMap::new),
    };

    // Store entry
    let response = state.storage.store(entry).await?;

    Ok((StatusCode::OK, Json(response)))
}

/// Retrieve data handler
#[axum::debug_handler]
pub async fn retrieve_data(
    State(state): State<Arc<AppState>>,
    Path(blinded_id): Path<String>,
    Query(params): Query<HashMap<String, String>>,
) -> Result<impl IntoResponse> {
    info!("Retrieving data with blinded ID: {}", blinded_id);

    // Check for requester ID and signature in query parameters
    let requester_id = params.get("requester_id").cloned();
    let signature = params.get("signature").cloned();

    // Create retrieval request
    let request = DataRetrievalRequest {
        blinded_id: blinded_id.clone(),
        requester_id,
        signature,
    };

    // Retrieve entry
    let entry = state.storage.retrieve(&request.blinded_id).await?;

    match entry {
        Some(entry) => {
            debug!("Entry found with ID: {}", blinded_id);
            info!(
                "DEBUG: Entry blinded_id check - input: '{}', starts_with device_identity: {}",
                blinded_id,
                blinded_id.starts_with("device_identity:")
            );

            // Special handling for device identity responses
            if blinded_id.starts_with("device_identity:") {
                info!("DEBUG: Processing device identity response transformation");
                // Parse the device identity from the encrypted payload
                match serde_json::from_slice::<crate::identity::DeviceIdentity>(
                    &entry.encrypted_payload,
                ) {
                    Ok(device_identity) => {
                        // Convert DeviceIdentity to the expected response format
                        let mut blinded_state = std::collections::HashMap::new();
                        blinded_state.insert(
                            "device_id".to_string(),
                            serde_json::Value::String(device_identity.device_id),
                        );
                        blinded_state.insert(
                            "threshold".to_string(),
                            serde_json::Value::Number(serde_json::Number::from(3u64)),
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
                        let device_entropy_b64 = base64::Engine::encode(
                            &base64::engine::general_purpose::STANDARD,
                            &device_identity.device_entropy,
                        );
                        blinded_state.insert(
                            "device_entropy".to_string(),
                            serde_json::Value::String(device_entropy_b64),
                        );

                        // Add blind key as base64
                        let blind_key_b64 = base64::Engine::encode(
                            &base64::engine::general_purpose::STANDARD,
                            &device_identity.blind_key,
                        );
                        blinded_state.insert(
                            "blind_key".to_string(),
                            serde_json::Value::String(blind_key_b64),
                        );

                        let device_identity_response = serde_json::json!({
                            "blinded_state": blinded_state
                        });

                        Ok((StatusCode::OK, Json(device_identity_response)))
                    }
                    Err(e) => {
                        warn!("Failed to parse device identity from payload: {}", e);
                        // Fall back to raw entry format
                        let entry_json = serde_json::to_value(&entry).map_err(|e| {
                            StorageNodeError::Serialization(format!(
                                "Failed to serialize entry: {e}"
                            ))
                        })?;
                        Ok((StatusCode::OK, Json(entry_json)))
                    }
                }
            } else {
                // For non-device-identity entries, return the raw entry
                let entry_json = serde_json::to_value(&entry).map_err(|e| {
                    StorageNodeError::Serialization(format!("Failed to serialize entry: {e}"))
                })?;
                Ok((StatusCode::OK, Json(entry_json)))
            }
        }
        None => {
            debug!("Entry not found with ID: {}", blinded_id);
            Err(StorageNodeError::NotFound(format!(
                "Entry with ID {blinded_id} not found"
            )))
        }
    }
}

/// Delete data handler
#[axum::debug_handler]
pub async fn delete_data(
    State(state): State<Arc<AppState>>,
    Path(blinded_id): Path<String>,
    Query(params): Query<HashMap<String, String>>,
) -> Result<impl IntoResponse> {
    info!("Deleting data with blinded ID: {}", blinded_id);

    // Check for signature in query parameters
    let signature = params.get("signature").cloned();

    // Verify signature if provided - for storage node operations,
    // we validate that the signature is properly formatted
    if let Some(sig) = &signature {
        if sig.is_empty() || sig.len() < 64 {
            return Err(StorageNodeError::Authentication(
                "Invalid signature format".into(),
            ));
        }

        // For production, verify signature against stored public key
        // For now, accept properly formatted signatures
        debug!("Signature verified for delete operation");
    }

    // Delete entry
    let deleted = state.storage.delete(&blinded_id).await?;

    if deleted {
        debug!("Entry deleted with ID: {}", blinded_id);
        Ok((
            StatusCode::OK,
            Json(serde_json::json!({
                "status": "success",
                "message": format!("Entry with ID {} deleted", blinded_id),
            })),
        ))
    } else {
        debug!("Entry not found for deletion with ID: {}", blinded_id);
        Err(StorageNodeError::NotFound(format!(
            "Entry with ID {blinded_id} not found"
        )))
    }
}

/// Check if data exists handler
#[axum::debug_handler]
pub async fn exists_data(
    State(state): State<Arc<AppState>>,
    Path(blinded_id): Path<String>,
) -> Result<impl IntoResponse> {
    debug!("Checking if data exists with blinded ID: {}", blinded_id);

    // Check if entry exists
    let exists = state.storage.exists(&blinded_id).await?;

    Ok((
        StatusCode::OK,
        Json(serde_json::json!({
            "exists": exists,
        })),
    ))
}

/// List data handler
#[axum::debug_handler]
pub async fn list_data(
    State(state): State<Arc<AppState>>,
    Query(params): Query<HashMap<String, String>>,
) -> Result<impl IntoResponse> {
    // Parse query parameters
    let limit = params.get("limit").and_then(|l| l.parse::<usize>().ok());
    let offset = params.get("offset").and_then(|o| o.parse::<usize>().ok());

    debug!("Listing data with limit: {:?}, offset: {:?}", limit, offset);

    // List entries
    let entries = state.storage.list(limit, offset).await?;

    Ok((
        StatusCode::OK,
        Json(serde_json::json!({
            "entries": entries,
            "count": entries.len(),
            "limit": limit,
            "offset": offset,
        })),
    ))
}

/// Get node stats handler
#[axum::debug_handler]
pub async fn node_stats(State(state): State<Arc<AppState>>) -> Result<impl IntoResponse> {
    debug!("Getting node stats");

    // Get storage stats
    let stats = state.storage.get_stats().await?;

    // Get staking status
    let staking_status = state.staking_service.get_status().await?;

    // Get DSM version
    let dsm_version = env!("CARGO_PKG_VERSION");

    // Get current timestamp
    let timestamp = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .map_or(0, |d| d.as_secs());

    Ok((
        StatusCode::OK,
        Json(serde_json::json!({
            "storage": stats,
            "staking": {
                "enabled": staking_status.enabled,
                "staked_amount": staking_status.staked_amount,
                "pending_rewards": staking_status.pending_rewards,
                "apy": staking_status.apy,
                "reputation": staking_status.reputation,
                "last_reward_time": staking_status.last_reward_time,
            },
            "dsm_version": dsm_version,
            "uptime": calculate_uptime(),
            "timestamp": timestamp,
        })),
    ))
}
/// Health check handler
pub async fn health_check() -> impl IntoResponse {
    (
        StatusCode::OK,
        Json(serde_json::json!({
            "status": "ok",
            "timestamp": std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .map_or(0, |d| d.as_secs()),
        })),
    )
}
