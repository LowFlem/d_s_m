//! Independent Policy Types for Storage Node
//!
//! This module provides independent implementations of policy-related types
//! to replace dependencies on the DSM client crate.

use serde::{Deserialize, Serialize};
use std::collections::HashMap;

/// Policy anchor for anchoring policies to blockchain
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PolicyAnchor {
    /// Unique policy ID
    pub policy_id: String,
    /// Policy hash for verification
    pub policy_hash: Vec<u8>,
    /// Blockchain transaction ID where policy is anchored
    pub transaction_id: String,
    /// Block height of the anchor transaction
    pub block_height: u64,
    /// Timestamp of anchor
    pub timestamp: u64,
    /// Additional metadata
    pub metadata: HashMap<String, String>,
}

/// Policy file containing policy rules and constraints
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PolicyFile {
    /// Policy identifier
    pub id: String,
    /// Policy name
    pub name: String,
    /// Policy version
    pub version: String,
    /// Policy rules in JSON format
    pub rules: serde_json::Value,
    /// Policy creator
    pub creator: String,
    /// Creation timestamp
    pub created_at: u64,
    /// Last modification timestamp
    pub modified_at: u64,
    /// Policy signature for integrity
    pub signature: Option<Vec<u8>>,
    /// Additional metadata
    pub metadata: HashMap<String, String>,
}

impl PolicyAnchor {
    /// Create a new policy anchor
    pub fn new(
        policy_id: String,
        policy_hash: Vec<u8>,
        transaction_id: String,
        block_height: u64,
    ) -> Self {
        Self {
            policy_id,
            policy_hash,
            transaction_id,
            block_height,
            timestamp: std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .unwrap_or_default()
                .as_secs(),
            metadata: HashMap::new(),
        }
    }

    /// Validate the policy anchor
    pub fn validate(&self) -> bool {
        !self.policy_id.is_empty()
            && !self.policy_hash.is_empty()
            && !self.transaction_id.is_empty()
            && self.block_height > 0
    }

    /// Create a policy anchor from a policy file
    pub fn from_policy(policy: &PolicyFile) -> Result<Self, String> {
        let policy_hash = policy.compute_hash();
        let policy_id = hex::encode(&policy_hash);

        Ok(Self {
            policy_id,
            policy_hash,
            transaction_id: String::new(), // Will be set when anchored to blockchain
            block_height: 0,               // Will be set when anchored to blockchain
            timestamp: std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .unwrap_or_default()
                .as_secs(),
            metadata: HashMap::new(),
        })
    }

    /// Convert policy anchor to hex string
    pub fn to_hex(&self) -> String {
        hex::encode(&self.policy_hash)
    }
}

impl PolicyFile {
    /// Create a new policy file
    pub fn new(
        id: String,
        name: String,
        version: String,
        rules: serde_json::Value,
        creator: String,
    ) -> Self {
        let now = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap_or_default()
            .as_secs();

        Self {
            id,
            name,
            version,
            rules,
            creator,
            created_at: now,
            modified_at: now,
            signature: None,
            metadata: HashMap::new(),
        }
    }

    /// Validate the policy file
    pub fn validate(&self) -> bool {
        !self.id.is_empty()
            && !self.name.is_empty()
            && !self.version.is_empty()
            && !self.creator.is_empty()
    }

    /// Compute hash of the policy file
    pub fn compute_hash(&self) -> Vec<u8> {
        let mut hasher = blake3::Hasher::new();
        hasher.update(self.id.as_bytes());
        hasher.update(self.name.as_bytes());
        hasher.update(self.version.as_bytes());
        hasher.update(self.rules.to_string().as_bytes());
        hasher.update(self.creator.as_bytes());
        hasher.finalize().as_bytes().to_vec()
    }
}
