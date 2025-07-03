//! # Independent Genesis and Identity Management for Storage Node
//!
//! This module provides independent implementations of genesis ID derivation and identity
//! management for the DSM storage node, maintaining protocol compatibility without
//! depending on Android client code.

use crate::error::{Result, StorageNodeError};
use blake3;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

/// SPHINCS+ public key for quantum-resistant signatures
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SigningPublicKey {
    pub bytes: Vec<u8>,
    pub algorithm: String,
}

/// Kyber public key for post-quantum key encapsulation
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct KyberPublicKey {
    pub bytes: Vec<u8>,
    pub algorithm: String,
}

/// Signing key for SPHINCS+ signatures
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SigningKey {
    pub public_key: SigningPublicKey,
    pub key_id: String,
    pub algorithm: String,
}

/// Kyber key for post-quantum key exchange
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct KyberKey {
    pub public_key: KyberPublicKey,
    pub key_id: String,
    pub algorithm: String,
}

/// Contribution for MPC Genesis ID creation
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Contribution {
    /// Node ID providing the contribution
    pub node_id: String,
    /// Signing key contribution
    pub signing_key: SigningKey,
    /// Kyber key contribution  
    pub kyber_key: KyberKey,
    /// Random entropy contribution
    pub entropy: Vec<u8>,
    /// Timestamp of contribution
    pub timestamp: u64,
    /// Cryptographic proof (simplified for now)
    pub proof: Option<Vec<u8>>,
}

/// Genesis state containing identity information
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct GenesisState {
    /// Unique genesis ID derived from contributions
    pub genesis_id: String,
    /// Hash of the genesis state
    pub genesis_hash: Vec<u8>,
    /// Contributing nodes
    pub contributors: Vec<String>,
    /// Threshold used for MPC
    pub threshold: u32,
    /// Timestamp of genesis creation
    pub created_at: u64,
    /// Genesis metadata
    pub metadata: HashMap<String, String>,
}

impl GenesisState {
    /// Create a new genesis state
    pub fn new(genesis_id: String, contributors: Vec<String>, threshold: u32) -> Self {
        let created_at = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap_or_default()
            .as_secs();

        let mut state = Self {
            genesis_id: genesis_id.clone(),
            genesis_hash: blake3::hash(&[]).as_bytes().to_vec(), // Will be computed later
            contributors,
            threshold,
            created_at,
            metadata: HashMap::new(),
        };

        // Compute genesis hash from state data
        state.genesis_hash = state.compute_hash();
        state
    }

    /// Compute hash of the genesis state
    pub fn compute_hash(&self) -> Vec<u8> {
        let mut hasher = blake3::Hasher::new();
        hasher.update(self.genesis_id.as_bytes());
        hasher.update(&self.threshold.to_le_bytes());
        hasher.update(&self.created_at.to_le_bytes());

        for contributor in &self.contributors {
            hasher.update(contributor.as_bytes());
        }

        hasher.finalize().as_bytes().to_vec()
    }

    /// Validate the genesis state
    pub fn validate(&self) -> Result<()> {
        if self.genesis_id.is_empty() {
            return Err(StorageNodeError::genesis("Genesis ID cannot be empty"));
        }

        if self.contributors.is_empty() {
            return Err(StorageNodeError::genesis(
                "Contributors list cannot be empty",
            ));
        }

        if self.threshold == 0 {
            return Err(StorageNodeError::genesis(
                "Threshold must be greater than 0",
            ));
        }

        if self.threshold > self.contributors.len() as u32 {
            return Err(StorageNodeError::genesis(
                "Threshold cannot exceed number of contributors",
            ));
        }

        // Verify hash integrity
        let computed_hash = self.compute_hash();
        if computed_hash != self.genesis_hash {
            return Err(StorageNodeError::genesis("Genesis hash validation failed"));
        }

        Ok(())
    }
}

/// Create a Genesis ID from MPC contributions (CORRECT IMPLEMENTATION)
/// This function generates a Genesis device ID purely from MPC contributions
/// The Genesis device ID is derived from the cryptographic material, not predetermined
pub fn create_genesis_from_mpc(
    contributions: &[Contribution],
    threshold: u32,
    session_entropy: Option<&[u8]>,
) -> Result<GenesisState> {
    if contributions.is_empty() {
        return Err(StorageNodeError::genesis("No contributions provided"));
    }

    if contributions.len() < threshold as usize {
        return Err(StorageNodeError::genesis(
            "Insufficient contributions for threshold",
        ));
    }

    // Sort contributions by node ID for deterministic ordering
    let mut sorted_contributions = contributions.to_vec();
    sorted_contributions.sort_by(|a, b| a.node_id.cmp(&b.node_id));

    // **CRITICAL**: Derive Genesis device ID from MPC contributions ONLY
    // No pre-specified device ID - the Genesis ID is the OUTPUT of this process
    let mut hasher = blake3::Hasher::new();

    // Add a fixed protocol identifier to ensure we're creating a Genesis
    hasher.update(b"DSM_GENESIS_V1");
    hasher.update(&threshold.to_le_bytes());

    // Add session entropy if provided (from client)
    if let Some(entropy) = session_entropy {
        hasher.update(entropy);
    }

    // Aggregate all MPC contributions
    for contribution in &sorted_contributions {
        hasher.update(contribution.node_id.as_bytes());
        hasher.update(&contribution.signing_key.public_key.bytes);
        hasher.update(&contribution.kyber_key.public_key.bytes);
        hasher.update(&contribution.entropy);
        hasher.update(&contribution.timestamp.to_le_bytes());

        // Include proof if available
        if let Some(proof) = &contribution.proof {
            hasher.update(proof);
        }
    }

    let genesis_hash = hasher.finalize();

    // Create Genesis device ID with proper formatting
    let genesis_id = format!(
        "dsm_genesis_{}",
        hex::encode(&genesis_hash.as_bytes()[..16])
    );

    let contributors: Vec<String> = sorted_contributions
        .iter()
        .map(|c| c.node_id.clone())
        .collect();

    let mut genesis_state = GenesisState::new(genesis_id, contributors, threshold);

    // Compute the correct hash from the final state fields
    genesis_state.genesis_hash = genesis_state.compute_hash();

    // Add metadata about the creation process
    genesis_state
        .metadata
        .insert("creation_method".to_string(), "mpc_v1".to_string());
    genesis_state.metadata.insert(
        "contribution_count".to_string(),
        sorted_contributions.len().to_string(),
    );

    Ok(genesis_state)
}

/// Generate a contribution for MPC Genesis ID creation
pub fn generate_contribution(node_id: &str) -> Result<Contribution> {
    // Generate cryptographic keys
    let (signing_pk, _signing_sk) = generate_signing_keypair()?;
    let (kyber_pk, _kyber_sk) = generate_kyber_keypair()?;

    // Generate random entropy
    let mut entropy = vec![0u8; 32];
    use rand::RngCore;
    rand::rngs::OsRng.fill_bytes(&mut entropy);

    let timestamp = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .unwrap_or_default()
        .as_secs();

    let contribution = Contribution {
        node_id: node_id.to_string(),
        signing_key: SigningKey {
            public_key: signing_pk,
            key_id: format!("signing_{node_id}"),
            algorithm: "SPHINCS+".to_string(),
        },
        kyber_key: KyberKey {
            public_key: kyber_pk,
            key_id: format!("kyber_{node_id}"),
            algorithm: "ML-KEM-512".to_string(),
        },
        entropy,
        timestamp,
        proof: None, // Simplified for now
    };

    Ok(contribution)
}

/// Generate a SPHINCS+ signing keypair
/// This is a placeholder implementation for production use
fn generate_signing_keypair() -> Result<(SigningPublicKey, Vec<u8>)> {
    // In production, this would use a proper SPHINCS+ implementation
    // For now, we'll use a placeholder with secure random bytes
    let mut public_key_bytes = vec![0u8; 32]; // SPHINCS+ public key is typically larger
    let mut private_key_bytes = vec![0u8; 64]; // SPHINCS+ private key is typically larger

    use rand::RngCore;
    rand::rngs::OsRng.fill_bytes(&mut public_key_bytes);
    rand::rngs::OsRng.fill_bytes(&mut private_key_bytes);

    let public_key = SigningPublicKey {
        bytes: public_key_bytes,
        algorithm: "SPHINCS+".to_string(),
    };

    Ok((public_key, private_key_bytes))
}

/// Generate a Kyber keypair for post-quantum key encapsulation
/// This is a placeholder implementation for production use
fn generate_kyber_keypair() -> Result<(KyberPublicKey, Vec<u8>)> {
    // In production, this would use a proper Kyber/ML-KEM implementation
    // For now, we'll use a placeholder with secure random bytes
    let mut public_key_bytes = vec![0u8; 800]; // ML-KEM-512 public key size
    let mut private_key_bytes = vec![0u8; 1632]; // ML-KEM-512 private key size

    use rand::RngCore;
    rand::rngs::OsRng.fill_bytes(&mut public_key_bytes);
    rand::rngs::OsRng.fill_bytes(&mut private_key_bytes);

    let public_key = KyberPublicKey {
        bytes: public_key_bytes,
        algorithm: "ML-KEM-512".to_string(),
    };

    Ok((public_key, private_key_bytes))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_genesis_state_creation() {
        let genesis_id = "test_genesis_123".to_string();
        let contributors = vec![
            "node1".to_string(),
            "node2".to_string(),
            "node3".to_string(),
        ];
        let threshold = 2;

        let genesis_state = GenesisState::new(genesis_id.clone(), contributors.clone(), threshold);

        assert_eq!(genesis_state.genesis_id, genesis_id);
        assert_eq!(genesis_state.contributors, contributors);
        assert_eq!(genesis_state.threshold, threshold);
        assert!(genesis_state.created_at > 0);
    }

    #[test]
    fn test_genesis_state_validation() {
        let genesis_state = GenesisState::new(
            "valid_genesis".to_string(),
            vec!["node1".to_string(), "node2".to_string()],
            2,
        );

        assert!(genesis_state.validate().is_ok());

        // Test invalid cases
        let invalid_state = GenesisState {
            genesis_id: String::new(),
            ..genesis_state.clone()
        };
        assert!(invalid_state.validate().is_err());
    }

    #[test]
    fn test_contribution_generation() {
        let node_id = "test_node";
        let contribution = generate_contribution(node_id).unwrap();

        assert_eq!(contribution.node_id, node_id);
        assert!(!contribution.entropy.is_empty());
        assert!(contribution.timestamp > 0);
        assert_eq!(contribution.signing_key.algorithm, "SPHINCS+");
        assert_eq!(contribution.kyber_key.algorithm, "ML-KEM-512");
    }

    #[test]
    fn test_create_genesis_from_mpc() {
        let node_ids = ["node1", "node2", "node3"];
        let threshold = 2;

        let contributions: Vec<Contribution> = node_ids
            .iter()
            .map(|&node_id| generate_contribution(node_id).unwrap())
            .collect();

        let genesis_state =
            create_genesis_from_mpc(&contributions, threshold, Some(b"test_session")).unwrap();

        assert!(!genesis_state.genesis_id.is_empty());
        assert!(genesis_state.genesis_id.starts_with("dsm_genesis_"));
        assert_eq!(genesis_state.threshold, threshold);
        assert_eq!(genesis_state.contributors.len(), node_ids.len());
        assert!(genesis_state.validate().is_ok());
    }

    #[test]
    fn test_create_genesis_from_mpc_deterministic() {
        let contributions = vec![
            generate_contribution("node1").unwrap(),
            generate_contribution("node2").unwrap(),
        ];
        let session_entropy = b"deterministic_session";

        let genesis1 = create_genesis_from_mpc(&contributions, 2, Some(session_entropy)).unwrap();
        let genesis2 = create_genesis_from_mpc(&contributions, 2, Some(session_entropy)).unwrap();

        // Should be deterministic for same inputs and session entropy
        assert_eq!(genesis1.genesis_id, genesis2.genesis_id);
    }
}
