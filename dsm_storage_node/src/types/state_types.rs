//! Independent State Types for Storage Node
//!
//! This module provides independent implementations of state-related types
//! to replace dependencies on the DSM client crate.

use serde::{Deserialize, Serialize};
use std::collections::HashMap;

/// Device information for state management
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DeviceInfo {
    /// Device identifier
    pub device_id: String,
    /// Device public key
    pub public_key: Vec<u8>,
    /// Device type
    pub device_type: String,
    /// Creation timestamp
    pub created_at: u64,
    /// Last activity timestamp
    pub last_active: u64,
    /// Device metadata
    pub metadata: HashMap<String, String>,
}

/// State management for DSM operations
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct State {
    /// State identifier
    pub state_id: String,
    /// State hash for integrity
    pub state_hash: Vec<u8>,
    /// Previous state hash
    pub previous_hash: Option<Vec<u8>>,
    /// State data
    pub data: serde_json::Value,
    /// State creator
    pub creator: String,
    /// Creation timestamp
    pub timestamp: u64,
    /// Block height reference
    pub block_height: u64,
    /// State signature
    pub signature: Option<Vec<u8>>,
    /// Additional metadata
    pub metadata: HashMap<String, String>,
}

impl DeviceInfo {
    /// Create a new device info
    pub fn new(device_id: &str, public_key: Vec<u8>) -> Self {
        let now = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap_or_default()
            .as_secs();

        Self {
            device_id: device_id.to_string(),
            public_key,
            device_type: "mobile".to_string(),
            created_at: now,
            last_active: now,
            metadata: HashMap::new(),
        }
    }

    /// Update last activity timestamp
    pub fn update_activity(&mut self) {
        self.last_active = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap_or_default()
            .as_secs();
    }

    /// Validate device info
    pub fn validate(&self) -> bool {
        !self.device_id.is_empty() && !self.public_key.is_empty()
    }
}

impl State {
    /// Create a new genesis state
    pub fn new_genesis(state_id: String, data: serde_json::Value, creator: String) -> Self {
        let now = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap_or_default()
            .as_secs();

        let mut state = Self {
            state_id,
            state_hash: vec![],
            previous_hash: None,
            data,
            creator,
            timestamp: now,
            block_height: 0,
            signature: None,
            metadata: HashMap::new(),
        };

        // Compute initial hash
        state.state_hash = state.compute_hash();
        state
    }

    /// Create a new state from previous state
    pub fn new_from_previous(
        state_id: String,
        data: serde_json::Value,
        creator: String,
        previous_state: &State,
        block_height: u64,
    ) -> Self {
        let now = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap_or_default()
            .as_secs();

        let mut state = Self {
            state_id,
            state_hash: vec![],
            previous_hash: Some(previous_state.state_hash.clone()),
            data,
            creator,
            timestamp: now,
            block_height,
            signature: None,
            metadata: HashMap::new(),
        };

        // Compute hash
        state.state_hash = state.compute_hash();
        state
    }

    /// Compute hash of the state
    pub fn compute_hash(&self) -> Vec<u8> {
        let mut hasher = blake3::Hasher::new();
        hasher.update(self.state_id.as_bytes());
        hasher.update(self.data.to_string().as_bytes());
        hasher.update(self.creator.as_bytes());
        hasher.update(&self.timestamp.to_le_bytes());
        hasher.update(&self.block_height.to_le_bytes());

        if let Some(prev_hash) = &self.previous_hash {
            hasher.update(prev_hash);
        }

        hasher.finalize().as_bytes().to_vec()
    }

    /// Validate the state
    pub fn validate(&self) -> bool {
        !self.state_id.is_empty()
            && !self.creator.is_empty()
            && self.state_hash == self.compute_hash()
    }

    /// Get state age in seconds
    pub fn get_age(&self) -> u64 {
        std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap_or_default()
            .as_secs()
            .saturating_sub(self.timestamp)
    }
}
