// DSM Identity Management Integration for Storage Node
//
// This module integrates independent DSM identity, genesis, and device management
// functionality with the storage node to enable proper MPC blind device ID creation.

pub mod genesis;

use crate::cluster::ClusterManager;
use crate::error::{Result, StorageNodeError};
use crate::storage::StorageEngine;
use crate::types::BlindedStateEntry;
use genesis::GenesisState;

use blake3;
use rand::{rngs::OsRng, Rng};
use std::collections::HashMap;
use std::sync::Arc;
use std::time::{SystemTime, UNIX_EPOCH};
use tokio::sync::RwLock;
use tracing::{info, warn};

/// Multi-Party Computation contribution for Genesis ID creation
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct MpcContribution {
    /// Node providing the contribution
    pub node_id: String,
    /// Entropy data contributed by this node
    pub entropy_data: Vec<u8>,
    /// Cryptographic proof of contribution validity (optional for now)
    pub proof: Option<Vec<u8>>,
    /// Timestamp when contribution was made
    pub timestamp: u64,
}

/// MPC session states for coordinated Genesis ID creation
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize, PartialEq)]
pub enum MpcSessionState {
    /// Collecting contributions from nodes
    Collecting,
    /// Threshold reached, aggregating contributions
    Aggregating,
    /// Genesis ID creation complete
    Complete,
    /// Session expired or failed
    Failed,
}

/// MPC session for coordinated Genesis ID creation across storage nodes
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct MpcBlindSession {
    /// Unique session identifier
    pub session_id: String,
    /// Device requesting Genesis ID
    pub device_id: String,
    /// Required threshold of contributions
    pub threshold: usize,
    /// Collected contributions from storage nodes
    pub contributions: Vec<MpcContribution>,
    /// Current session state
    pub state: MpcSessionState,
    /// Master genesis state (aggregated from all contributions)
    pub master_genesis: Option<GenesisState>,
    /// Device-specific genesis state
    pub device_genesis: Option<GenesisState>,
    /// Optional master genesis ID to anchor to (for sub-genesis creation)
    pub anchor_to_master: Option<String>,
    /// Session creation timestamp
    pub started_at: u64,
    /// Session expiration timestamp
    pub expires_at: u64,
    /// Facilitator node ID (node that initiated the session)
    pub facilitator_node: String,
    /// List of storage nodes participating in this MPC
    pub participating_nodes: Vec<String>,
}

/// DSM Contact entry for bilateral relationship management
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct DsmContact {
    /// User-chosen alias for the contact
    pub alias: String,
    /// Cryptographic device identifier (immutable)
    pub device_id: String,
    /// Genesis hash from decentralized storage (for verification)
    pub genesis_hash: String,
    /// Latest bilateral state hash (chain tip)
    pub chain_tip: Option<String>,
    /// Timestamp when contact was added
    pub added_at: u64,
    /// Last transaction timestamp
    pub last_tx_at: Option<u64>,
}

/// DSM State entry for forward-only hash chain with SMT support
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct DsmState {
    /// Current state hash
    pub state_hash: String,
    /// Previous state hash (for chain validation)
    pub prev_hash: String,
    /// State randomness/entropy
    pub randomness: Vec<u8>,
    /// Operation that created this state
    pub operation: String,
    /// Token balance deltas (if applicable)
    pub balance_deltas: std::collections::HashMap<String, i64>,
    /// Merkle root for sparse merkle tree
    pub merkle_root: [u8; 32],
    /// Sparse merkle tree proof for state verification
    pub smt_proof: Option<crate::smt::SmtProof>,
    /// Timestamp of state creation
    pub timestamp: u64,
    /// State index/sequence number
    pub state_index: u64,
}

/// Chain tip manager for bilateral relationships with SMT integration
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct ChainTipManager {
    /// Map of device_id -> latest state hash for bilateral relationships
    pub chain_tips: std::collections::HashMap<String, String>,
    /// Map of device_id -> contact information
    pub contacts: std::collections::HashMap<String, DsmContact>,
    /// Map of state_hash -> full state for verification
    pub states: std::collections::HashMap<String, DsmState>,
    /// Sparse Merkle Tree for efficient state proofs
    pub smt: crate::smt::SparseMerkleTree,
}

impl Default for ChainTipManager {
    fn default() -> Self {
        Self::new()
    }
}

impl ChainTipManager {
    /// Create a new chain tip manager
    pub fn new() -> Self {
        Self {
            chain_tips: std::collections::HashMap::new(),
            contacts: std::collections::HashMap::new(),
            states: std::collections::HashMap::new(),
            smt: crate::smt::SparseMerkleTree::new(),
        }
    }

    /// Add a contact with genesis verification
    pub fn add_contact(&mut self, contact: DsmContact) -> Result<()> {
        // Verify genesis hash format
        if !contact.genesis_hash.starts_with("dsm_genesis_") {
            return Err(StorageNodeError::InvalidInput(
                "Invalid genesis hash format".to_string(),
            ));
        }

        // Store the contact
        self.contacts.insert(contact.device_id.clone(), contact);
        Ok(())
    }

    /// Update chain tip for a bilateral relationship
    pub fn update_chain_tip(&mut self, device_id: &str, new_chain_tip: String) -> Result<()> {
        if let Some(contact) = self.contacts.get_mut(device_id) {
            contact.chain_tip = Some(new_chain_tip.clone());
            contact.last_tx_at = Some(
                std::time::SystemTime::now()
                    .duration_since(std::time::UNIX_EPOCH)
                    .unwrap_or_default()
                    .as_secs(),
            );
            self.chain_tips.insert(device_id.to_string(), new_chain_tip);
            Ok(())
        } else {
            Err(StorageNodeError::NotFound(
                "Contact not found for chain tip update".to_string(),
            ))
        }
    }

    /// Get chain tip for a device
    pub fn get_chain_tip(&self, device_id: &str) -> Option<&String> {
        self.chain_tips.get(device_id)
    }

    /// Create a new state in the forward-only hash chain
    pub fn create_next_state(
        &mut self,
        prev_state_hash: &str,
        operation: String,
        randomness: Vec<u8>,
        balance_deltas: std::collections::HashMap<String, i64>,
    ) -> Result<DsmState> {
        // Generate new state hash using DSM protocol formula: S_n = H(S_{n-1} || R_n)
        let mut hasher = blake3::Hasher::new();
        hasher.update(prev_state_hash.as_bytes());
        hasher.update(&randomness);
        hasher.update(operation.as_bytes());

        // Include balance deltas in hash
        for (token, delta) in &balance_deltas {
            hasher.update(token.as_bytes());
            hasher.update(&delta.to_le_bytes());
        }

        let state_hash = hex::encode(hasher.finalize().as_bytes());

        // Get next state index
        let state_index = if prev_state_hash.is_empty() {
            0 // Genesis state
        } else {
            // Find previous state and increment
            if let Some(prev_state) = self.states.get(prev_state_hash) {
                prev_state.state_index + 1
            } else {
                return Err(StorageNodeError::NotFound(
                    "Previous state not found in chain".to_string(),
                ));
            }
        };

        // Insert state into SMT and get merkle root
        let merkle_root = self
            .smt
            .insert_dsm_state(
                &state_hash,
                prev_state_hash,
                &operation,
                &balance_deltas,
                state_index,
            )
            .map_err(|e| {
                StorageNodeError::InvalidOperation(format!("SMT insertion failed: {e}"))
            })?;

        // Generate SMT proof for the new state
        let smt_proof = self.smt.generate_state_proof(&state_hash).map_err(|e| {
            StorageNodeError::InvalidOperation(format!("SMT proof generation failed: {e}"))
        })?;

        let new_state = DsmState {
            state_hash: state_hash.clone(),
            prev_hash: prev_state_hash.to_string(),
            randomness,
            operation,
            balance_deltas,
            merkle_root,
            smt_proof: Some(smt_proof),
            timestamp: std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .unwrap_or_default()
                .as_secs(),
            state_index,
        };

        // Store the new state
        self.states.insert(state_hash.clone(), new_state.clone());

        Ok(new_state)
    }

    /// Verify a hash chain from state_i to state_j
    pub fn verify_chain(&self, from_hash: &str, to_hash: &str) -> Result<bool> {
        let mut current_hash = to_hash;
        let mut visited = std::collections::HashSet::new();

        // Walk backwards through the chain to find from_hash
        while current_hash != from_hash {
            if visited.contains(current_hash) {
                return Err(StorageNodeError::InvalidOperation(
                    "Circular reference detected in chain".to_string(),
                ));
            }
            visited.insert(current_hash);

            if let Some(state) = self.states.get(current_hash) {
                if state.prev_hash.is_empty() {
                    // Reached genesis without finding from_hash
                    return Ok(false);
                }
                current_hash = &state.prev_hash;
            } else {
                return Err(StorageNodeError::NotFound(format!(
                    "State {current_hash} not found in chain"
                )));
            }
        }

        Ok(true)
    }

    /// Generate a state proof using SMT
    pub fn generate_state_proof(&self, state_hash: &str) -> Result<crate::smt::SmtProof> {
        self.smt.generate_state_proof(state_hash).map_err(|e| {
            StorageNodeError::InvalidOperation(format!("State proof generation failed: {e}"))
        })
    }

    /// Verify a state proof using SMT
    pub fn verify_state_proof(&self, proof: &crate::smt::SmtProof, state_hash: &str) -> bool {
        self.smt.verify_dsm_state_proof(proof, state_hash)
    }

    /// Get all states in a given index range (for sparse index queries)
    pub fn get_states_by_index_range(&self, from_index: u64, to_index: u64) -> Vec<&DsmState> {
        self.states
            .values()
            .filter(|state| state.state_index >= from_index && state.state_index <= to_index)
            .collect()
    }

    /// Get SMT root hash for current state
    pub fn get_smt_root(&self) -> [u8; 32] {
        self.smt.root_hash
    }

    /// Verify chain tip with SMT proof
    pub fn verify_chain_tip_with_proof(&self, device_id: &str, expected_tip: &str) -> Result<bool> {
        if let Some(actual_tip) = self.get_chain_tip(device_id) {
            if actual_tip == expected_tip {
                // Verify the tip exists in SMT
                if let Some(state) = self.states.get(expected_tip) {
                    if let Some(ref proof) = state.smt_proof {
                        return Ok(self.verify_state_proof(proof, expected_tip));
                    }
                }
            }
        }
        Ok(false)
    }
}

/// Device identity containing Genesis state and cryptographic keys  
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct DeviceIdentity {
    /// DSM Genesis device identifier (derived from MPC process)
    pub device_id: String,
    /// Master DSM Genesis ID (the primary identity)
    pub master_genesis_id: String,
    /// Device-specific Genesis state
    pub genesis_state: GenesisState,
    /// Device-specific entropy
    pub device_entropy: Vec<u8>,
    /// Blind encryption key for secure storage
    pub blind_key: Vec<u8>,
    /// Creation timestamp
    pub created_at: u64,
    /// Last update timestamp
    pub updated_at: u64,
    /// Chain tip manager for bilateral relationships
    pub chain_manager: ChainTipManager,
    /// SMT root hash for state verification
    pub smt_root: [u8; 32],
}

/// DSM Identity Manager with proper MPC coordination
pub struct DsmIdentityManager {
    /// Storage engine for persistence
    storage: Arc<dyn StorageEngine>,

    /// Current node ID
    node_id: String,

    /// Active MPC sessions
    pub mpc_sessions: Arc<RwLock<HashMap<String, MpcBlindSession>>>,

    /// Cluster manager for node discovery
    cluster_manager: Option<Arc<ClusterManager>>,

    /// Known storage nodes for MPC coordination (fallback)
    cluster_nodes: Arc<RwLock<Vec<String>>>,
}

impl DsmIdentityManager {
    /// Create a new identity manager
    pub fn new(storage: Arc<dyn StorageEngine>, node_id: String) -> Self {
        Self {
            storage,
            node_id,
            mpc_sessions: Arc::new(RwLock::new(HashMap::new())),
            cluster_manager: None,
            cluster_nodes: Arc::new(RwLock::new(Vec::new())),
        }
    }

    /// Create a new identity manager with cluster manager for production use
    pub fn new_with_cluster(
        storage: Arc<dyn StorageEngine>,
        node_id: String,
        cluster_manager: Arc<ClusterManager>,
    ) -> Self {
        Self {
            storage,
            node_id,
            mpc_sessions: Arc::new(RwLock::new(HashMap::new())),
            cluster_manager: Some(cluster_manager),
            cluster_nodes: Arc::new(RwLock::new(Vec::new())),
        }
    }

    /// Create MPC session - wrapper for create_genesis_mpc_session for API compatibility
    pub async fn create_mpc_session(
        &self,
        device_id: String,
        threshold: usize,
        anchor_to_master: Option<String>,
    ) -> Result<String> {
        // Use the provided device_id for non-genesis sessions
        let session_id = self.generate_session_id("mpc_request")?;
        let facilitator_node = self.node_id.clone();
        let participating_nodes = vec![self.node_id.clone()];
        let session = MpcBlindSession {
            session_id: session_id.clone(),
            device_id: device_id.clone(),
            threshold,
            contributions: Vec::new(),
            state: MpcSessionState::Collecting,
            master_genesis: None,
            anchor_to_master,
            device_genesis: None,
            started_at: SystemTime::now()
                .duration_since(UNIX_EPOCH)
                .unwrap_or_default()
                .as_secs(),
            expires_at: SystemTime::now()
                .duration_since(UNIX_EPOCH)
                .unwrap_or_default()
                .as_secs()
                + 3600, // 1 hour expiry
            facilitator_node,
            participating_nodes,
        };
        self.mpc_sessions
            .write()
            .await
            .insert(session_id.clone(), session);
        Ok(session_id)
    }

    /// Initialize MPC session for Genesis ID creation with proper coordination
    /// UPDATED: For true Genesis creation, we don't need a pre-existing device_id
    /// The Genesis device ID will be derived from the MPC process itself
    pub async fn create_genesis_mpc_session(
        &self,
        threshold: usize,
        anchor_to_master: Option<String>,
    ) -> Result<String> {
        // Generate unique session ID for this Genesis creation
        let session_id = self.generate_session_id("genesis_request")?;

        // **CRITICAL CHANGE**: Use session ID as placeholder, Genesis ID will be derived later
        let session = MpcBlindSession {
            session_id: session_id.clone(),
            device_id: format!("genesis_pending_{}", &session_id[8..16]), // Temporary placeholder
            threshold,
            contributions: Vec::new(),
            state: MpcSessionState::Collecting,
            master_genesis: None,
            anchor_to_master,
            device_genesis: None,
            started_at: SystemTime::now()
                .duration_since(UNIX_EPOCH)
                .unwrap_or_default()
                .as_secs(),
            expires_at: SystemTime::now()
                .duration_since(UNIX_EPOCH)
                .unwrap_or_default()
                .as_secs()
                + 3600, // 1 hour expiration
            facilitator_node: self.node_id.clone(),
            participating_nodes: self.get_cluster_node_urls().await,
        };

        // Store session
        {
            let mut sessions = self.mpc_sessions.write().await;
            sessions.insert(session_id.clone(), session);
        }

        // Persist session
        self.persist_mpc_session(&session_id).await?;

        // Auto-request contributions from peer nodes
        self.request_peer_contributions(&session_id).await?;

        Ok(session_id)
    }

    /// Add a contribution to an MPC session
    pub async fn add_contribution(
        &self,
        session_id: String,
        contribution: MpcContribution,
    ) -> Result<bool> {
        let mut sessions = self.mpc_sessions.write().await;

        if let Some(session) = sessions.get_mut(&session_id) {
            // Check session state
            if session.state != MpcSessionState::Collecting {
                return Err(StorageNodeError::InvalidInput(
                    "Session is not accepting contributions".to_string(),
                ));
            }

            // Check if contribution is from a new node
            if session
                .contributions
                .iter()
                .any(|c| c.node_id == contribution.node_id)
            {
                return Err(StorageNodeError::InvalidInput(
                    "Node has already contributed to this session".to_string(),
                ));
            }

            // Add contribution
            session.contributions.push(contribution);

            // Check if we have enough contributions
            if session.contributions.len() >= session.threshold {
                session.state = MpcSessionState::Aggregating;
                drop(sessions); // Release lock before async operations

                // Process the session
                return self.process_mpc_session(session_id).await;
            }

            // Persist updated session
            drop(sessions);
            self.persist_mpc_session(&session_id).await?;

            Ok(false) // Not yet ready to process
        } else {
            Err(StorageNodeError::NotFound(
                "MPC session not found".to_string(),
            ))
        }
    }

    /// Process an MPC session to create blind device ID
    async fn process_mpc_session(&self, session_id: String) -> Result<bool> {
        let session = {
            let sessions = self.mpc_sessions.read().await;
            sessions.get(&session_id).cloned()
        };

        if let Some(mut session) = session {
            match self.create_device_genesis(&mut session).await {
                Ok(device_identity) => {
                    // Store the device identity in storage for client retrieval
                    let device_key = format!("device_identity:{}", device_identity.device_id);
                    let device_value = serde_json::to_vec(&device_identity)
                        .map_err(|e| StorageNodeError::Serialization(e.to_string()))?;

                    let device_entry = BlindedStateEntry {
                        blinded_id: device_key,
                        encrypted_payload: device_value,
                        timestamp: SystemTime::now()
                            .duration_since(UNIX_EPOCH)
                            .unwrap_or_default()
                            .as_secs(),
                        ttl: 86400, // 24 hours
                        region: "default".to_string(),
                        priority: 1,
                        proof_hash: [0u8; 32],
                        metadata: std::collections::HashMap::new(),
                    };

                    // Store device identity in storage
                    self.storage
                        .store(device_entry)
                        .await
                        .map_err(|e| StorageNodeError::Storage(e.to_string()))?;

                    tracing::info!(
                        "Stored device identity for device: {}",
                        device_identity.device_id
                    );

                    // Update session state
                    session.state = MpcSessionState::Complete;

                    // Update session
                    {
                        let mut sessions = self.mpc_sessions.write().await;
                        sessions.insert(session_id.clone(), session.clone());
                    }

                    // Persist changes
                    self.persist_mpc_session(&session_id).await?;

                    Ok(true)
                }
                Err(e) => {
                    // Mark session as failed
                    session.state = MpcSessionState::Failed;

                    {
                        let mut sessions = self.mpc_sessions.write().await;
                        sessions.insert(session_id.clone(), session.clone());
                    }

                    self.persist_mpc_session(&session_id).await?;

                    Err(StorageNodeError::InvalidOperation(format!(
                        "Failed to create device genesis: {e}"
                    )))
                }
            }
        } else {
            Err(StorageNodeError::NotFound(
                "MPC session not found".to_string(),
            ))
        }
    }
    /// Create device genesis from MPC contributions (CORRECTED IMPLEMENTATION)
    /// This now properly derives the Genesis device ID from MPC output, not from input
    async fn create_device_genesis(&self, session: &mut MpcBlindSession) -> Result<DeviceIdentity> {
        // Verify we have enough contributions for threshold
        if session.contributions.len() < session.threshold {
            return Err(StorageNodeError::InvalidOperation(format!(
                "Insufficient contributions: {} < {}",
                session.contributions.len(),
                session.threshold
            )));
        }

        // **CRITICAL**: Create Genesis ID from MPC contributions using the protocol formula
        // G = H(b1 ∥ b2 ∥ ... ∥ bt ∥ A)
        let genesis_contributions: Vec<genesis::Contribution> = session
            .contributions
            .iter()
            .map(|mpc_contrib| genesis::Contribution {
                node_id: mpc_contrib.node_id.clone(),
                signing_key: genesis::SigningKey {
                    public_key: genesis::SigningPublicKey {
                        bytes: mpc_contrib.entropy_data.clone(),
                        algorithm: "SPHINCS+".to_string(),
                    },
                    key_id: format!("signing_{}", mpc_contrib.node_id),
                    algorithm: "SPHINCS+".to_string(),
                },
                kyber_key: genesis::KyberKey {
                    public_key: genesis::KyberPublicKey {
                        bytes: mpc_contrib.entropy_data.clone(),
                        algorithm: "Kyber".to_string(),
                    },
                    key_id: format!("kyber_{}", mpc_contrib.node_id),
                    algorithm: "Kyber".to_string(),
                },
                entropy: mpc_contrib.entropy_data.clone(),
                timestamp: mpc_contrib.timestamp,
                proof: mpc_contrib.proof.clone(),
            })
            .collect();

        // **KEY CHANGE**: Use the new function that generates Genesis ID from MPC
        let genesis_state = genesis::create_genesis_from_mpc(
            &genesis_contributions,
            session.threshold as u32,
            None, // No additional session entropy for now
        )?;

        // **CRITICAL**: Extract the GENERATED Genesis device ID
        let genesis_device_id = genesis_state.genesis_id.clone();

        // Update the session with the real Genesis device ID
        session.device_id = genesis_device_id.clone();

        // Generate device-specific entropy (for DBRW compatibility)
        let device_entropy = self.generate_device_entropy(&genesis_device_id)?;

        // Generate blind encryption key from the genesis state
        let blind_key = self.generate_blind_key(&genesis_state)?;

        // Store genesis states in session
        session.master_genesis = Some(genesis_state.clone());
        session.device_genesis = Some(genesis_state.clone());

        // Create chain manager with initial SMT state
        // Note: Chain tips are only created when adding contacts (bilateral relationships)
        // The device's own genesis is not a "chain tip" - it's the starting point
        let chain_manager = ChainTipManager::new();
        let smt_root = chain_manager.get_smt_root();

        // Create device identity with the GENERATED Genesis device ID
        // Determine master genesis ID based on anchoring
        let master_genesis_id = if let Some(anchor_master) = &session.anchor_to_master {
            // This is a sub-genesis, anchor to existing master
            anchor_master.clone()
        } else {
            // This is a master/root genesis, device_id becomes the master
            genesis_device_id.clone()
        };

        let device_identity = DeviceIdentity {
            device_id: genesis_device_id.clone(), // This is now the OUTPUT of MPC process
            master_genesis_id,
            genesis_state,
            device_entropy,
            blind_key,
            created_at: SystemTime::now()
                .duration_since(UNIX_EPOCH)
                .unwrap_or_default()
                .as_secs(),
            updated_at: SystemTime::now()
                .duration_since(UNIX_EPOCH)
                .unwrap_or_default()
                .as_secs(),
            chain_manager,
            smt_root,
        };

        Ok(device_identity)
    }
    /// Generate device-specific entropy for MPC (Dual-Binding Random Walk protocol)
    ///
    /// This function simulates dual-binding entropy for the storage node, as required by DeviceBinding.md.
    /// The storage node MUST NOT use DBRW or hardware entropy, but MUST combine software entropy with a per-session salt
    /// (device_id, node_id, timestamp) to ensure each MPC contribution is unique and protocol-compliant.
    ///
    /// This ensures the storage node's entropy is always salted/member-randomized per MPC session, as required by the protocol.
    fn generate_device_entropy(
        &self,
        device_id: &str,
    ) -> std::result::Result<Vec<u8>, StorageNodeError> {
        let mut entropy = [0u8; 32];
        OsRng.fill(&mut entropy);

        // Derive a per-session salt (simulate member/session salt for MPC)
        let mut salt_hasher = blake3::Hasher::new();
        salt_hasher.update(device_id.as_bytes());
        salt_hasher.update(self.node_id.as_bytes());
        salt_hasher.update(
            &SystemTime::now()
                .duration_since(UNIX_EPOCH)
                .unwrap_or_default()
                .as_nanos()
                .to_le_bytes(),
        );
        let salt = salt_hasher.finalize().as_bytes().to_vec();

        // Use hash to combine entropy and salt for MPC compliance
        let mut hmac_hasher = blake3::Hasher::new();
        hmac_hasher.update(&salt);
        hmac_hasher.update(&entropy);
        let mpc_entropy = hmac_hasher.finalize().as_bytes().to_vec();
        Ok(mpc_entropy)
    }

    /// Generate blind encryption key
    fn generate_blind_key(
        &self,
        genesis: &GenesisState,
    ) -> std::result::Result<Vec<u8>, StorageNodeError> {
        // Derive blind key from genesis entropy
        let mut hasher = blake3::Hasher::new();
        hasher.update(&genesis.genesis_hash);
        hasher.update(b"BLIND_KEY");
        hasher.update(self.node_id.as_bytes());

        Ok(hasher.finalize().as_bytes().to_vec())
    }

    /// Generate session ID with enhanced security
    fn generate_session_id(&self, device_id: &str) -> Result<String> {
        use std::time::{SystemTime, UNIX_EPOCH};
        let timestamp = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap_or_default()
            .as_millis();

        // Add some randomness for uniqueness
        let mut entropy = [0u8; 8];
        OsRng.fill(&mut entropy);

        let mut hasher = blake3::Hasher::new();
        hasher.update(device_id.as_bytes());
        hasher.update(self.node_id.as_bytes());
        hasher.update(&timestamp.to_le_bytes());
        hasher.update(&entropy);

        let session_hash = hasher.finalize();
        // Use hex-like encoding for consistency
        let hex_chars = "0123456789abcdef";
        let hash_bytes = session_hash.as_bytes();
        let mut hex_string = String::with_capacity(32);
        for &byte in &hash_bytes[..16] {
            hex_string.push(hex_chars.chars().nth((byte >> 4) as usize).unwrap());
            hex_string.push(hex_chars.chars().nth((byte & 0xf) as usize).unwrap());
        }

        // For genesis sessions, use genesis_ prefix
        if device_id.starts_with("genesis") {
            Ok(format!("genesis_{hex_string}"))
        } else {
            Ok(format!("session_{hex_string}"))
        }
    }

    // NOTE: The MPC aggregated genesis creation is handled by the genesis module
    // via genesis::create_genesis_from_mpc() which provides the proper DSM whitepaper
    // implementation of G = H(b1 ∥ b2 ∥ ... ∥ bt ∥ A). The local implementation
    // was removed to eliminate code duplication and ensure consistency.

    /// Persist MPC session to storage (cluster-wide through epidemic storage)
    async fn persist_mpc_session(&self, session_id: &str) -> Result<()> {
        let session = {
            let sessions = self.mpc_sessions.read().await;
            sessions.get(session_id).cloned()
        };

        if let Some(session) = session {
            let key = format!("mpc_session:{session_id}");
            let value = serde_json::to_vec(&session)
                .map_err(|e| StorageNodeError::Serialization(e.to_string()))?;

            let entry = BlindedStateEntry {
                blinded_id: key,
                encrypted_payload: value,
                timestamp: std::time::SystemTime::now()
                    .duration_since(std::time::UNIX_EPOCH)
                    .unwrap()
                    .as_secs(),
                ttl: 3600, // 1 hour
                region: "default".to_string(),
                priority: 1,
                proof_hash: [0u8; 32],
                metadata: std::collections::HashMap::new(),
            };

            // Store through epidemic storage engine for cluster-wide availability
            self.storage
                .store(entry)
                .await
                .map_err(|e| StorageNodeError::Storage(e.to_string()))?;

            tracing::info!(
                "Persisted MPC session {} to cluster-wide storage",
                session_id
            );
        }

        Ok(())
    }

    /// Get MPC session by ID (checks both local and cluster-wide storage)
    pub async fn get_mpc_session(&self, session_id: &str) -> Option<MpcBlindSession> {
        // First check local in-memory cache
        {
            let sessions = self.mpc_sessions.read().await;
            if let Some(session) = sessions.get(session_id).cloned() {
                tracing::debug!("Found MPC session {} in local cache", session_id);
                return Some(session);
            }
        }

        // If not in local cache, check cluster-wide storage via epidemic storage
        let key = format!("mpc_session:{session_id}");

        match self.storage.retrieve(&key).await {
            Ok(Some(entry)) => {
                match serde_json::from_slice::<MpcBlindSession>(&entry.encrypted_payload) {
                    Ok(session) => {
                        tracing::info!(
                            "Found MPC session {} in cluster storage, caching locally",
                            session_id
                        );

                        // Cache the session locally for faster future access
                        {
                            let mut sessions = self.mpc_sessions.write().await;
                            sessions.insert(session_id.to_string(), session.clone());
                        }

                        Some(session)
                    }
                    Err(e) => {
                        tracing::error!("Failed to deserialize MPC session {}: {}", session_id, e);
                        None
                    }
                }
            }
            Ok(None) => {
                tracing::debug!("MPC session {} not found in cluster storage", session_id);
                None
            }
            Err(e) => {
                tracing::error!(
                    "Error retrieving MPC session {} from cluster storage: {}",
                    session_id,
                    e
                );
                None
            }
        }
    }

    /// Clean up expired sessions
    pub async fn cleanup_expired_sessions(&self) -> Result<usize> {
        let current_time = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap_or_default()
            .as_secs();

        let mut cleaned_count = 0;
        let expired_sessions: Vec<String> = {
            let mut sessions = self.mpc_sessions.write().await;
            let mut expired = Vec::new();

            for (session_id, session) in sessions.iter_mut() {
                if session.expires_at < current_time && session.state == MpcSessionState::Collecting
                {
                    session.state = MpcSessionState::Failed;
                    expired.push(session_id.clone());
                }
            }

            expired
        };

        // Remove expired sessions from memory and storage
        for session_id in expired_sessions {
            {
                let mut sessions = self.mpc_sessions.write().await;
                sessions.remove(&session_id);
            }

            let key = format!("mpc_session:{session_id}");
            let _ = self.storage.delete(&key).await; // Ignore errors for cleanup
            cleaned_count += 1;
        }

        Ok(cleaned_count)
    }

    /// Request contributions from peer nodes for the MPC session
    async fn request_peer_contributions(&self, session_id: &str) -> Result<()> {
        // Get the actual cluster nodes using the cluster manager
        let cluster_nodes = self.get_cluster_node_urls().await;

        info!(
            "Requesting MPC contributions from {} cluster nodes for session {}",
            cluster_nodes.len(),
            session_id
        );

        for peer_url in cluster_nodes {
            // Skip if this is our own URL
            if peer_url.contains(&self.node_id) {
                continue;
            }

            let session_id = session_id.to_string();
            let peer_url = peer_url.to_string();

            // Spawn async task to request contribution from this peer
            tokio::spawn(async move {
                let client = reqwest::Client::new();

                // FIXED: Generate proper cryptographic entropy for each peer
                let mut entropy = [0u8; 32];
                rand::rngs::OsRng.fill(&mut entropy);

                let timestamp = std::time::SystemTime::now()
                    .duration_since(std::time::UNIX_EPOCH)
                    .unwrap_or_default()
                    .as_secs();

                // Create proper MPC contribution request
                let payload = serde_json::json!({
                    "session_id": session_id,
                    "node_id": format!("peer_{}", peer_url.split(':').next_back().unwrap_or("unknown")),
                    "entropy_data": entropy.to_vec(),
                    "timestamp": timestamp
                });

                // Use correct MPC contribution endpoint
                let url = format!("{peer_url}/api/v1/mpc/contribute");
                match client.post(&url).json(&payload).send().await {
                    Ok(response) => {
                        if response.status().is_success() {
                            info!("Successfully requested MPC contribution from {}", peer_url);
                        } else {
                            warn!(
                                "Peer {} returned error status: {}",
                                peer_url,
                                response.status()
                            );
                        }
                    }
                    Err(e) => {
                        warn!(
                            "Failed to request MPC contribution from {}: {}",
                            peer_url, e
                        );
                    }
                }
            });
        }

        Ok(())
    }

    /// Initialize cluster nodes from configuration
    pub async fn initialize_cluster_nodes(&self, node_urls: Vec<String>) {
        let mut nodes = self.cluster_nodes.write().await;
        *nodes = node_urls;
    }

    /// Get cluster node URLs using cluster manager
    async fn get_cluster_node_urls(&self) -> Vec<String> {
        if let Some(cluster_manager) = &self.cluster_manager {
            // Get gossip targets from cluster manager
            let targets = cluster_manager.get_gossip_targets(None).await;
            if !targets.is_empty() {
                return targets.into_iter().map(|node| node.endpoint).collect();
            }
        }

        // Fallback to manually registered nodes if cluster manager not available or no targets
        let nodes = self.cluster_nodes.read().await;
        nodes.clone()
    }

    /// Add a cluster node
    pub async fn add_cluster_node(&self, node_url: String) {
        let mut nodes = self.cluster_nodes.write().await;
        if !nodes.contains(&node_url) {
            nodes.push(node_url);
        }
    }

    /// Remove a cluster node
    pub async fn remove_cluster_node(&self, node_url: &str) {
        let mut nodes = self.cluster_nodes.write().await;
        nodes.retain(|url| url != node_url);
    }

    /// Get cluster node count
    pub async fn get_cluster_size(&self) -> usize {
        self.cluster_nodes.read().await.len()
    }

    /// Add a contact to a device identity
    pub async fn add_contact_to_device(&self, device_id: &str, contact: DsmContact) -> Result<()> {
        let device_key = format!("device_identity:{device_id}");

        // Retrieve device identity
        if let Some(entry) = self
            .storage
            .retrieve(&device_key)
            .await
            .map_err(|e| StorageNodeError::Storage(e.to_string()))?
        {
            let mut device_identity: DeviceIdentity =
                serde_json::from_slice(&entry.encrypted_payload)
                    .map_err(|e| StorageNodeError::Serialization(e.to_string()))?;

            // Add the contact
            let contact_alias = contact.alias.clone();
            device_identity.chain_manager.add_contact(contact)?;
            device_identity.updated_at = SystemTime::now()
                .duration_since(UNIX_EPOCH)
                .unwrap_or_default()
                .as_secs();

            // Store updated device identity
            let updated_value = serde_json::to_vec(&device_identity)
                .map_err(|e| StorageNodeError::Serialization(e.to_string()))?;

            let updated_entry = BlindedStateEntry {
                blinded_id: device_key,
                encrypted_payload: updated_value,
                timestamp: device_identity.updated_at,
                ttl: 86400, // 24 hours
                region: "default".to_string(),
                priority: 1,
                proof_hash: [0u8; 32],
                metadata: std::collections::HashMap::new(),
            };

            self.storage
                .store(updated_entry)
                .await
                .map_err(|e| StorageNodeError::Storage(e.to_string()))?;

            tracing::info!("Added contact {} to device {}", contact_alias, device_id);
            Ok(())
        } else {
            Err(StorageNodeError::NotFound(format!(
                "Device identity {device_id} not found"
            )))
        }
    }

    /// Update chain tip for a bilateral relationship
    pub async fn update_chain_tip(
        &self,
        device_id: &str,
        contact_device_id: &str,
        new_chain_tip: String,
    ) -> Result<()> {
        let device_key = format!("device_identity:{device_id}");

        // Retrieve device identity
        if let Some(entry) = self
            .storage
            .retrieve(&device_key)
            .await
            .map_err(|e| StorageNodeError::Storage(e.to_string()))?
        {
            let mut device_identity: DeviceIdentity =
                serde_json::from_slice(&entry.encrypted_payload)
                    .map_err(|e| StorageNodeError::Serialization(e.to_string()))?;

            // Update chain tip
            device_identity
                .chain_manager
                .update_chain_tip(contact_device_id, new_chain_tip)?;
            device_identity.updated_at = SystemTime::now()
                .duration_since(UNIX_EPOCH)
                .unwrap_or_default()
                .as_secs();

            // Store updated device identity
            let updated_value = serde_json::to_vec(&device_identity)
                .map_err(|e| StorageNodeError::Serialization(e.to_string()))?;

            let updated_entry = BlindedStateEntry {
                blinded_id: device_key,
                encrypted_payload: updated_value,
                timestamp: device_identity.updated_at,
                ttl: 86400, // 24 hours
                region: "default".to_string(),
                priority: 1,
                proof_hash: [0u8; 32],
                metadata: std::collections::HashMap::new(),
            };

            self.storage
                .store(updated_entry)
                .await
                .map_err(|e| StorageNodeError::Storage(e.to_string()))?;

            tracing::info!(
                "Updated chain tip for {} -> {}",
                device_id,
                contact_device_id
            );
            Ok(())
        } else {
            Err(StorageNodeError::NotFound(format!(
                "Device identity {device_id} not found"
            )))
        }
    }

    /// Create a new bilateral state transition
    pub async fn create_bilateral_state(
        &self,
        device_id: &str,
        contact_device_id: &str,
        operation: String,
        balance_deltas: std::collections::HashMap<String, i64>,
    ) -> Result<DsmState> {
        let device_key = format!("device_identity:{device_id}");

        // Retrieve device identity
        if let Some(entry) = self
            .storage
            .retrieve(&device_key)
            .await
            .map_err(|e| StorageNodeError::Storage(e.to_string()))?
        {
            let mut device_identity: DeviceIdentity =
                serde_json::from_slice(&entry.encrypted_payload)
                    .map_err(|e| StorageNodeError::Serialization(e.to_string()))?;

            // Get current chain tip or use genesis as starting point
            let prev_state_hash = device_identity
                .chain_manager
                .get_chain_tip(contact_device_id)
                .cloned()
                .unwrap_or_else(|| hex::encode(&device_identity.genesis_state.genesis_hash));

            // Generate new randomness for state transition
            let mut randomness = vec![0u8; 32];
            use rand::RngCore;
            rand::rngs::OsRng.fill_bytes(&mut randomness);

            // Create next state in the forward-only chain
            let new_state = device_identity.chain_manager.create_next_state(
                &prev_state_hash,
                operation,
                randomness,
                balance_deltas,
            )?;

            // Update chain tip
            device_identity
                .chain_manager
                .update_chain_tip(contact_device_id, new_state.state_hash.clone())?;

            device_identity.updated_at = SystemTime::now()
                .duration_since(UNIX_EPOCH)
                .unwrap_or_default()
                .as_secs();

            // Store updated device identity
            let updated_value = serde_json::to_vec(&device_identity)
                .map_err(|e| StorageNodeError::Serialization(e.to_string()))?;

            let updated_entry = BlindedStateEntry {
                blinded_id: device_key,
                encrypted_payload: updated_value,
                timestamp: device_identity.updated_at,
                ttl: 86400, // 24 hours
                region: "default".to_string(),
                priority: 1,
                proof_hash: [0u8; 32],
                metadata: std::collections::HashMap::new(),
            };

            self.storage
                .store(updated_entry)
                .await
                .map_err(|e| StorageNodeError::Storage(e.to_string()))?;

            tracing::info!(
                "Created new bilateral state {} for {} -> {}",
                new_state.state_hash,
                device_id,
                contact_device_id
            );

            Ok(new_state)
        } else {
            Err(StorageNodeError::NotFound(format!(
                "Device identity {device_id} not found"
            )))
        }
    }

    /// Get device identity with full chain manager
    pub async fn get_device_identity(&self, device_id: &str) -> Result<Option<DeviceIdentity>> {
        let device_key = format!("device_identity:{device_id}");

        if let Some(entry) = self
            .storage
            .retrieve(&device_key)
            .await
            .map_err(|e| StorageNodeError::Storage(e.to_string()))?
        {
            let device_identity: DeviceIdentity = serde_json::from_slice(&entry.encrypted_payload)
                .map_err(|e| StorageNodeError::Serialization(e.to_string()))?;
            Ok(Some(device_identity))
        } else {
            Ok(None)
        }
    }

    /// Verify a bilateral hash chain between two devices
    pub async fn verify_bilateral_chain(
        &self,
        device_id: &str,
        _contact_device_id: &str,
        from_state: &str,
        to_state: &str,
    ) -> Result<bool> {
        if let Some(device_identity) = self.get_device_identity(device_id).await? {
            device_identity
                .chain_manager
                .verify_chain(from_state, to_state)
        } else {
            Err(StorageNodeError::NotFound(format!(
                "Device identity {device_id} not found"
            )))
        }
    }

    /// Create a dev mode MPC session for development/testing
    pub async fn create_dev_mode_session(
        &self,
        session_id: String,
        device_id: String,
        anchor_to_master: Option<String>,
    ) -> Result<MpcBlindSession> {
        Ok(MpcBlindSession {
            session_id,
            device_id,
            threshold: 1,
            contributions: Vec::new(),
            state: MpcSessionState::Collecting,
            master_genesis: None,
            anchor_to_master,
            device_genesis: None,
            started_at: SystemTime::now()
                .duration_since(UNIX_EPOCH)
                .unwrap_or_default()
                .as_secs(),
            expires_at: SystemTime::now()
                .duration_since(UNIX_EPOCH)
                .unwrap_or_default()
                .as_secs()
                + 3600, // 1 hour
            facilitator_node: "dev_node".to_string(),
            participating_nodes: vec!["dev_node".to_string()],
        })
    }

    /// Create a simple device identity for dev mode
    pub async fn create_dev_device_identity(
        &self,
        device_id: String,
        genesis_hash: String,
        anchor_to_master: Option<String>,
    ) -> Result<DeviceIdentity> {
        // Create a simple genesis state for dev mode
        let genesis_state = GenesisState {
            genesis_id: device_id.clone(),
            genesis_hash: hex::decode(&genesis_hash).map_err(|_| {
                StorageNodeError::InvalidInput("Invalid genesis hash format".to_string())
            })?,
            contributors: vec!["dev_node".to_string()],
            threshold: 1,
            created_at: SystemTime::now()
                .duration_since(UNIX_EPOCH)
                .unwrap_or_default()
                .as_secs(),
            metadata: std::collections::HashMap::new(),
        };

        // Generate simple entropy for dev mode
        let device_entropy = vec![0u8; 32]; // Simple for dev mode

        // Generate simple blind key
        let blind_key = vec![1u8; 32]; // Simple for dev mode

        // Create chain manager with initial chain tip
        let mut chain_manager = ChainTipManager::new();
        chain_manager
            .chain_tips
            .insert(device_id.clone(), genesis_hash.clone());
        let smt_root = chain_manager.get_smt_root();

        // Determine master genesis ID
        let master_genesis_id = anchor_to_master.unwrap_or_else(|| device_id.clone());

        let device_identity = DeviceIdentity {
            device_id: device_id.clone(),
            master_genesis_id,
            genesis_state,
            device_entropy,
            blind_key,
            created_at: SystemTime::now()
                .duration_since(UNIX_EPOCH)
                .unwrap_or_default()
                .as_secs(),
            updated_at: SystemTime::now()
                .duration_since(UNIX_EPOCH)
                .unwrap_or_default()
                .as_secs(),
            chain_manager,
            smt_root,
        };

        Ok(device_identity)
    }

    /// Store raw data in the storage engine (for dev mode utilities)
    pub async fn store_raw_data(&self, key: String, data: Vec<u8>) -> Result<()> {
        let entry = BlindedStateEntry {
            blinded_id: key,
            encrypted_payload: data,
            timestamp: SystemTime::now()
                .duration_since(UNIX_EPOCH)
                .unwrap_or_default()
                .as_secs(),
            ttl: 86400, // 24 hours
            region: "default".to_string(),
            priority: 1,
            proof_hash: [0u8; 32],
            metadata: std::collections::HashMap::new(),
        };

        self.storage
            .store(entry)
            .await
            .map_err(|e| StorageNodeError::Storage(e.to_string()))?;

        Ok(())
    }

    /// Create device identity from MPC session (public wrapper)
    pub async fn create_device_identity_from_session(
        &self,
        session: &mut MpcBlindSession,
    ) -> Result<DeviceIdentity> {
        self.create_device_genesis(session).await
    }

    /// Store a device identity in the storage engine
    pub async fn store_device_identity(&self, device_identity: &DeviceIdentity) -> Result<()> {
        let device_key = format!("device_identity:{}", device_identity.device_id);

        let serialized_identity = serde_json::to_vec(device_identity)
            .map_err(|e| StorageNodeError::Serialization(e.to_string()))?;

        let entry = BlindedStateEntry {
            blinded_id: device_key,
            encrypted_payload: serialized_identity,
            timestamp: device_identity.updated_at,
            ttl: 86400, // 24 hours
            region: "default".to_string(),
            priority: 1,
            proof_hash: [0u8; 32],
            metadata: std::collections::HashMap::new(),
        };

        self.storage
            .store(entry)
            .await
            .map_err(|e| StorageNodeError::Storage(e.to_string()))?;

        tracing::info!(
            "Stored device identity for device {}",
            device_identity.device_id
        );
        Ok(())
    }
}

#[cfg(test)]
mod tests;
