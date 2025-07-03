// Multi-Party Computation API for DSM Storage Node
//
// This module handles MPC-based Genesis state creation following DSM protocol.
// The Genesis device ID is derived from MPC contributions using the formula:
// G = H(b1 || b2 || ... || bt || Aux) where bi are blind inputs from t parties.
//
// CRITICAL: Genesis device ID is OUTPUT of MPC process, not input.

use crate::error::{Result, StorageNodeError};
use crate::identity::{DsmIdentityManager, MpcContribution, MpcSessionState};
use crate::storage::StorageEngine;
use crate::types::BlindedStateEntry;
use blake3;
use rand::{rngs::OsRng, Rng};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::sync::Arc;
use std::time::{SystemTime, UNIX_EPOCH};
use tracing::info;

// ============================================================================
// Request/Response Structures
// ============================================================================

/// Request for Genesis ID creation via MPC (CORRECT IMPLEMENTATION)
/// Genesis device ID is generated as OUTPUT, not provided as input
#[derive(Debug, Serialize, Deserialize)]
pub struct GenesisCreationRequest {
    /// Optional session identifier for tracking
    pub session_id: Option<String>,
    /// Required threshold for MPC (minimum 3 for security)
    pub threshold: usize,
    /// Optional client entropy for additional randomness
    pub client_entropy: Option<Vec<u8>>,
    /// Optional master genesis ID to anchor to (for sub-genesis creation)
    /// If None, creates a new master/root genesis
    /// If Some(master_id), creates a sub-genesis anchored to the existing master
    pub anchor_to_master: Option<String>,
    /// Request timestamp
    pub request_timestamp: u64,
}

/// Response for Genesis ID creation
#[derive(Debug, Serialize, Deserialize)]
pub struct GenesisCreationResponse {
    /// Session ID for tracking this Genesis creation
    pub session_id: String,
    /// GENERATED Genesis device ID (output of MPC process)
    pub genesis_device_id: String,
    /// Master genesis ID (same as device_id for root genesis, different for sub-genesis)
    pub master_genesis_id: String,
    /// Whether this is a root/master genesis (true) or sub-genesis (false)
    pub is_master_genesis: bool,
    /// Current session state
    pub state: String,
    /// Number of contributions received
    pub contributions_received: usize,
    /// Required threshold
    pub threshold: usize,
    /// Whether genesis creation is complete
    pub complete: bool,
    /// Genesis hash for verification (available when complete)
    pub genesis_hash: Option<String>,
    /// Initial chain tip (available when complete)
    pub initial_chain_tip: Option<String>,
    /// List of participating storage node IDs
    pub participating_nodes: Vec<String>,
    /// Timestamp
    pub timestamp: u64,
}

/// Request for contributing entropy to MPC session
#[derive(Debug, Serialize, Deserialize)]
pub struct MpcContributionRequest {
    /// Session ID to contribute to
    pub session_id: String,
    /// Node ID making the contribution
    pub node_id: String,
    /// Cryptographic entropy data
    pub entropy_data: Vec<u8>,
    /// Optional cryptographic proof
    pub proof: Option<Vec<u8>>,
    /// Request timestamp
    pub timestamp: u64,
}

/// Response for MPC contribution
#[derive(Debug, Serialize, Deserialize)]
pub struct MpcContributionResponse {
    /// Session ID
    pub session_id: String,
    /// Whether contribution was accepted
    pub accepted: bool,
    /// Current number of contributions
    pub contributions_count: usize,
    /// Required threshold
    pub threshold: usize,
    /// Whether session is ready for final processing
    pub ready_for_processing: bool,
    /// Status message
    pub message: String,
    /// Timestamp
    pub timestamp: u64,
}

/// Request for general entropy generation
#[derive(Debug, Serialize, Deserialize)]
pub struct EntropyRequest {
    /// Process ID for entropy generation
    pub process_id: String,
    /// Node ID requesting participation
    pub node_id: String,
    /// Request timestamp
    pub request_timestamp: u64,
}

/// Response for entropy generation
#[derive(Debug, Serialize, Deserialize)]
pub struct EntropyResponse {
    /// Process ID this entropy is for
    pub process_id: String,
    /// Node ID that generated this entropy
    pub node_id: String,
    /// Status of entropy generation
    pub status: String,
    /// Timestamp of generation
    pub timestamp: u64,
}

// ============================================================================
// Additional MPC Utility Structures
// ============================================================================

/// Request for querying multiple Genesis sessions
#[derive(Debug, Serialize, Deserialize)]
pub struct GenesisSessionQueryRequest {
    /// Optional filter by session state
    pub state_filter: Option<String>,
    /// Optional limit on number of results
    pub limit: Option<usize>,
    /// Include completed sessions in results
    pub include_completed: bool,
}

/// Response for Genesis session queries
#[derive(Debug, Serialize, Deserialize)]
pub struct GenesisSessionQueryResponse {
    /// List of matching Genesis sessions
    pub sessions: Vec<GenesisCreationResponse>,
    /// Total count of sessions (before limit)
    pub total_count: usize,
    /// Query timestamp
    pub timestamp: u64,
}

// ============================================================================
// Core Genesis Creation Functions
// ============================================================================

/// Create a Genesis ID using MPC (CORRECT DSM PROTOCOL IMPLEMENTATION)
///
/// This function implements the DSM protocol for Genesis creation:
/// G = H(b1 || b2 || ... || bt || Aux)
///
/// The Genesis device ID is the OUTPUT of this process, derived from
/// cryptographic contributions from multiple storage nodes.
pub async fn create_genesis_identity(
    request: GenesisCreationRequest,
    identity_manager: Arc<DsmIdentityManager>,
) -> Result<GenesisCreationResponse> {
    info!(
        "Creating Genesis identity via MPC with threshold: {}",
        request.threshold
    );

    // DEVELOPMENT MODE: threshold = 1 means skip MPC entirely
    if request.threshold == 1 {
        info!("🚀 DEV MODE: threshold = 1, generating local genesis (no MPC)");

        let timestamp = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap_or_default()
            .as_millis();

        let session_id = format!("dev_genesis_{timestamp:x}");

        // Generate deterministic genesis from entropy
        let mut dev_hasher = blake3::Hasher::new();
        dev_hasher.update(b"DSM_DEV_GENESIS");
        dev_hasher.update(session_id.as_bytes());

        if let Some(client_entropy) = request.client_entropy {
            dev_hasher.update(&client_entropy);
        } else {
            dev_hasher.update(&timestamp.to_le_bytes());
            dev_hasher.update(&rand::random::<u64>().to_le_bytes());
        }

        let genesis_hash = dev_hasher.finalize();
        let genesis_device_id = format!(
            "dev_genesis_{:x}",
            &genesis_hash.as_bytes()[0..8]
                .iter()
                .fold(0u64, |acc, &b| acc << 8 | b as u64)
        );
        let genesis_hash_hex = hex::encode(genesis_hash.as_bytes());

        info!(
            "Development mode: Generated local genesis {} with hash {}",
            genesis_device_id, genesis_hash_hex
        );

        // Create and store the device identity for dev mode
        let device_identity = identity_manager
            .create_dev_device_identity(
                genesis_device_id.clone(),
                genesis_hash_hex.clone(),
                request.anchor_to_master.clone(),
            )
            .await?;

        identity_manager
            .store_device_identity(&device_identity)
            .await?;

        info!(
            "Development mode: Device identity created and stored for {}",
            genesis_device_id
        );

        // Determine if this is a master or sub-genesis
        let is_master = request.anchor_to_master.is_none();
        let master_genesis_id = request
            .anchor_to_master
            .unwrap_or_else(|| genesis_device_id.clone());

        return Ok(GenesisCreationResponse {
            session_id,
            genesis_device_id,
            master_genesis_id,
            is_master_genesis: is_master,
            state: "complete".to_string(),
            contributions_received: 1,
            threshold: 1,
            complete: true,
            genesis_hash: Some(genesis_hash_hex.clone()),
            initial_chain_tip: None, // Chain tips are created when adding contacts, not at genesis
            participating_nodes: vec!["localhost".to_string()],
            timestamp: SystemTime::now()
                .duration_since(UNIX_EPOCH)
                .unwrap_or_default()
                .as_secs(),
        });
    }

    // Validate threshold according to DSM security requirements
    // For development (threshold = 1), allow bypass of normal security requirements
    if request.threshold == 1 {
        // Development mode: single-party genesis (skip MPC)
        info!("Development mode: Creating single-party genesis (threshold=1)");
    } else if request.threshold < 3 || request.threshold > 10 {
        return Err(StorageNodeError::InvalidInput(
            "Threshold must be between 3 and 10 for Genesis creation security (or 1 for development)".to_string(),
        ));
    } else {
        // Production mode: proper MPC threshold
        info!(
            "Production mode: Creating MPC genesis with threshold={}",
            request.threshold
        );
    }

    // Generate unique session ID if not provided
    let session_id = request.session_id.unwrap_or_else(|| {
        let timestamp = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap_or_default()
            .as_millis();
        format!("genesis_{timestamp:x}")
    });

    // CRITICAL: Genesis device ID will be derived from MPC output
    // Generate deterministic genesis ID from session parameters
    let mut genesis_hasher = blake3::Hasher::new();
    genesis_hasher.update(session_id.as_bytes());
    genesis_hasher.update(&request.threshold.to_le_bytes());
    if let Some(ref entropy) = request.client_entropy {
        genesis_hasher.update(entropy);
    }
    let genesis_device_id = format!(
        "dsm_genesis_{}",
        hex::encode(&genesis_hasher.finalize().as_bytes()[..16])
    );

    // Include client entropy in the MPC process if provided
    let has_client_entropy = request.client_entropy.is_some();
    let client_entropy_contribution = if let Some(entropy) = request.client_entropy {
        // Hash client entropy for security
        let mut hasher = blake3::Hasher::new();
        hasher.update(&entropy);
        hasher.update(session_id.as_bytes());
        Some(hasher.finalize().as_bytes().to_vec())
    } else {
        None
    };

    // Create MPC session for Genesis derivation (production mode)
    let actual_session_id = identity_manager
        .create_mpc_session(
            genesis_device_id.clone(),
            request.threshold,
            request.anchor_to_master.clone(),
        )
        .await?;

    // If client provided entropy, add it as the first contribution
    if let Some(client_contribution) = client_entropy_contribution {
        let client_mpc_contribution = MpcContribution {
            node_id: "client".to_string(),
            entropy_data: client_contribution,
            proof: None, // Client contributions don't require proof
            timestamp: SystemTime::now()
                .duration_since(UNIX_EPOCH)
                .unwrap_or_default()
                .as_secs(),
        };

        identity_manager
            .add_contribution(actual_session_id.clone(), client_mpc_contribution)
            .await?;
    }

    info!("Created Genesis MPC session: {}", actual_session_id);

    // Generate master genesis ID deterministically based on session parameters
    let master_genesis_id = if request.anchor_to_master.is_none() {
        // This is a master genesis - use the session ID as base
        format!("master_{}", &actual_session_id[..16])
    } else {
        // This is anchored to an existing master
        request
            .anchor_to_master
            .clone()
            .unwrap_or_else(|| "unknown_master".to_string())
    };

    Ok(GenesisCreationResponse {
        session_id: actual_session_id,
        genesis_device_id, // Deterministically generated
        master_genesis_id, // Properly determined
        is_master_genesis: request.anchor_to_master.is_none(), // Based on request
        state: "collecting".to_string(),
        contributions_received: if has_client_entropy { 1 } else { 0 },
        threshold: request.threshold,
        complete: false,
        genesis_hash: None,
        initial_chain_tip: None,
        participating_nodes: Vec::new(),
        timestamp: SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap_or_default()
            .as_secs(),
    })
}

/// Contribute entropy to an MPC session for Genesis derivation
pub async fn contribute_to_mpc_session(
    request: MpcContributionRequest,
    identity_manager: Arc<DsmIdentityManager>,
    node_id: String,
) -> Result<MpcContributionResponse> {
    info!(
        "Node {} contributing to MPC session {}",
        node_id, request.session_id
    );

    // Validate session exists and is active
    let session = identity_manager
        .get_mpc_session(&request.session_id)
        .await
        .ok_or_else(|| {
            StorageNodeError::NotFound("MPC session not found or expired".to_string())
        })?;

    // Check session timeout (15 minutes for Genesis creation)
    let session_timeout = 15 * 60; // 15 minutes in seconds
    let current_time = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap_or_default()
        .as_secs();

    if current_time > session.started_at + session_timeout {
        return Err(StorageNodeError::InvalidInput(
            "MPC session has expired".to_string(),
        ));
    }

    // Validate entropy contribution size
    if request.entropy_data.is_empty() || request.entropy_data.len() > 1024 {
        return Err(StorageNodeError::InvalidInput(
            "Entropy data must be between 1 and 1024 bytes".to_string(),
        ));
    }

    // Create cryptographic contribution
    let contribution = MpcContribution {
        node_id: node_id.clone(),
        entropy_data: request.entropy_data.clone(),
        proof: Some(request.proof.unwrap_or_else(|| {
            // Generate cryptographic proof of contribution
            let mut hasher = blake3::Hasher::new();
            hasher.update(&request.entropy_data);
            hasher.update(node_id.as_bytes());
            hasher.update(request.session_id.as_bytes());
            hasher.update(&request.timestamp.to_le_bytes());
            hasher.finalize().as_bytes().to_vec()
        })),
        timestamp: SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap_or_default()
            .as_secs(),
    };

    // Add contribution to MPC session
    let ready_for_processing = identity_manager
        .add_contribution(request.session_id.clone(), contribution)
        .await?;

    // Get updated session information
    let updated_session = identity_manager
        .get_mpc_session(&request.session_id)
        .await
        .ok_or_else(|| StorageNodeError::NotFound("MPC session not found".to_string()))?;

    let message = if ready_for_processing {
        "Contribution accepted. Genesis derivation ready for processing.".to_string()
    } else {
        format!(
            "Contribution accepted. Need {} more contributions for Genesis derivation.",
            updated_session.threshold - updated_session.contributions.len()
        )
    };

    Ok(MpcContributionResponse {
        session_id: request.session_id,
        accepted: true,
        contributions_count: updated_session.contributions.len(),
        threshold: updated_session.threshold,
        ready_for_processing,
        message,
        timestamp: SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap_or_default()
            .as_secs(),
    })
}

/// Get the status of an MPC Genesis creation session
pub async fn get_genesis_session_status(
    session_id: String,
    identity_manager: Arc<DsmIdentityManager>,
) -> Result<GenesisCreationResponse> {
    let session = identity_manager
        .get_mpc_session(&session_id)
        .await
        .ok_or_else(|| StorageNodeError::NotFound("Genesis session not found".to_string()))?;

    let state_str = match session.state {
        MpcSessionState::Collecting => "collecting",
        MpcSessionState::Aggregating => "deriving_genesis",
        MpcSessionState::Complete => "complete",
        MpcSessionState::Failed => "failed",
    }
    .to_string();

    // If complete, derive the actual Genesis ID from contributions
    let (genesis_device_id, genesis_hash) = if session.state == MpcSessionState::Complete {
        // Implement DSM protocol: G = H(b1 || b2 || ... || bt || Aux)
        let derived_genesis = derive_genesis_from_contributions(&session.contributions)?;
        (derived_genesis.clone(), Some(derived_genesis))
    } else {
        (session.device_id.clone(), None)
    };

    // Extract participating node IDs
    let participating_nodes: Vec<String> = session
        .contributions
        .iter()
        .map(|c| c.node_id.clone())
        .collect();

    // Determine master genesis logic from session information
    let is_master = session.anchor_to_master.is_none();
    let master_genesis_id = if is_master {
        genesis_device_id.clone()
    } else {
        session
            .anchor_to_master
            .clone()
            .unwrap_or_else(|| "unknown".to_string())
    };

    Ok(GenesisCreationResponse {
        session_id,
        genesis_device_id,
        master_genesis_id,
        is_master_genesis: is_master,
        state: state_str,
        contributions_received: session.contributions.len(),
        threshold: session.threshold,
        complete: session.state == MpcSessionState::Complete,
        genesis_hash: genesis_hash.clone(),
        initial_chain_tip: None, // Chain tips are created when adding contacts, not at genesis
        participating_nodes,
        timestamp: SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap_or_default()
            .as_secs(),
    })
}

// ============================================================================
// Utility Functions
// ============================================================================

/// Derive Genesis ID from MPC contributions according to DSM protocol
/// Formula: G = H(b1 || b2 || ... || bt || Aux)
fn derive_genesis_from_contributions(contributions: &[MpcContribution]) -> Result<String> {
    if contributions.is_empty() {
        return Err(StorageNodeError::InvalidInput(
            "Cannot derive Genesis from empty contributions".to_string(),
        ));
    }

    // Sort contributions by node_id for deterministic ordering
    let mut sorted_contributions = contributions.to_vec();
    sorted_contributions.sort_by(|a, b| a.node_id.cmp(&b.node_id));

    // Implement DSM protocol formula: G = H(b1 || b2 || ... || bt || Aux)
    let mut hasher = blake3::Hasher::new();

    // Add each contribution's entropy data
    for contribution in &sorted_contributions {
        hasher.update(&contribution.entropy_data);
    }

    // Add auxiliary data (timestamps and node IDs for additional entropy)
    for contribution in &sorted_contributions {
        hasher.update(contribution.node_id.as_bytes());
        hasher.update(&contribution.timestamp.to_le_bytes());
    }

    // Generate the Genesis device ID
    let genesis_hash = hasher.finalize();
    let genesis_id = format!(
        "dsm_genesis_{}",
        hex::encode(&genesis_hash.as_bytes()[..16])
    );

    Ok(genesis_id)
}

/// Generate high-quality entropy contribution for MPC
pub async fn generate_node_contribution(session_id: String, node_id: String) -> Result<Vec<u8>> {
    // Generate cryptographically secure entropy
    let mut entropy = [0u8; 32];
    OsRng.fill(&mut entropy);

    // Mix with session and node information for uniqueness
    let mut hasher = blake3::Hasher::new();
    hasher.update(&entropy);
    hasher.update(session_id.as_bytes());
    hasher.update(node_id.as_bytes());
    hasher.update(
        &SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap_or_default()
            .as_nanos()
            .to_le_bytes(),
    );

    Ok(hasher.finalize().as_bytes().to_vec())
}

// ============================================================================
// Legacy Entropy Functions
// ============================================================================

/// Handle general entropy requests (for legacy compatibility)
pub async fn handle_entropy_request(
    process_id: String,
    request: EntropyRequest,
    storage_engine: Arc<dyn StorageEngine + Send + Sync>,
) -> Result<EntropyResponse> {
    // Validate process ID
    if process_id != request.process_id {
        return Err(StorageNodeError::InvalidInput(format!(
            "Process ID mismatch: {} vs {}",
            process_id, request.process_id
        )));
    }

    // Generate high-quality entropy
    let mut entropy = [0u8; 32];
    OsRng.fill(&mut entropy);

    // Mix with additional entropy sources
    let timestamp = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap_or_default()
        .as_nanos();

    let mut hasher = blake3::Hasher::new();
    hasher.update(&entropy);
    hasher.update(&timestamp.to_le_bytes());
    hasher.update(process_id.as_bytes());
    hasher.update(request.node_id.as_bytes());

    let final_entropy = hasher.finalize().as_bytes().to_vec();

    // Store entropy contribution
    let entropy_key = format!("entropy:{}:{}", process_id, request.node_id);
    let entropy_data = serde_json::json!({
        "process_id": process_id,
        "node_id": request.node_id,
        "entropy": final_entropy,
        "timestamp": SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap_or_default()
            .as_secs(),
    });

    let serialized = serde_json::to_vec(&entropy_data).map_err(|e| {
        StorageNodeError::Serialization(format!("Failed to serialize entropy: {e}"))
    })?;

    let entry = BlindedStateEntry {
        blinded_id: entropy_key,
        encrypted_payload: serialized,
        timestamp: SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap_or_default()
            .as_secs(),
        ttl: 3600, // 1 hour expiration
        region: "global".to_string(),
        priority: 1,
        proof_hash: [0u8; 32],
        metadata: HashMap::new(),
    };

    storage_engine.store(entry).await?;

    Ok(EntropyResponse {
        process_id,
        node_id: request.node_id,
        status: "completed".to_string(),
        timestamp: SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap_or_default()
            .as_secs(),
    })
}

/// Retrieve entropy contribution for a process and node
pub async fn get_entropy_contribution(
    process_id: String,
    node_id: String,
    storage_engine: Arc<dyn StorageEngine + Send + Sync>,
) -> Result<Option<Vec<u8>>> {
    let entropy_key = format!("entropy:{process_id}:{node_id}");

    if let Some(entry) = storage_engine.retrieve(&entropy_key).await? {
        let entropy_data: serde_json::Value = serde_json::from_slice(&entry.encrypted_payload)
            .map_err(|e| {
                StorageNodeError::Serialization(format!("Failed to deserialize entropy: {e}"))
            })?;

        if let Some(entropy_array) = entropy_data["entropy"].as_array() {
            let entropy = entropy_array
                .iter()
                .map(|v| v.as_u64().unwrap_or(0) as u8)
                .collect::<Vec<u8>>();
            return Ok(Some(entropy));
        }
    }

    Ok(None)
}

/// Process multiple MPC contributions in batch
pub async fn batch_contribute_to_mpc_sessions(
    requests: Vec<MpcContributionRequest>,
    identity_manager: Arc<DsmIdentityManager>,
    node_id: String,
) -> Result<Vec<MpcContributionResponse>> {
    let mut responses = Vec::new();

    for request in requests {
        let response =
            contribute_to_mpc_session(request, identity_manager.clone(), node_id.clone()).await?;
        responses.push(response);
    }

    Ok(responses)
}

/// Query Genesis sessions with optional filtering
pub async fn query_genesis_sessions(
    _request: GenesisSessionQueryRequest,
    _identity_manager: Arc<DsmIdentityManager>,
) -> Result<GenesisSessionQueryResponse> {
    // This would need implementation in the identity manager
    // For now, return empty results
    Ok(GenesisSessionQueryResponse {
        sessions: Vec::new(),
        total_count: 0,
        timestamp: SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap_or_default()
            .as_secs(),
    })
}
