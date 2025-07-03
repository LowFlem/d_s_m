//! Token Policy Store for DSM Storage Nodes
//!
//! This module provides a global policy store for token policies, allowing storage nodes
//! to maintain a global reference of all token policies in the network. Client applications
//! can query these policies when needed and cache them locally.

use std::{collections::HashMap, sync::Arc};

use crate::types::policy_types::{PolicyAnchor, PolicyFile};
use parking_lot::RwLock;
use serde::{Deserialize, Serialize};
use tracing::info;

use crate::error::{Result, StorageNodeError};
use crate::storage::StorageEngine;

/// Storage response for policy data
#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct PolicyStorageEntry {
    /// Policy ID
    pub id: String,
    /// Policy content hash
    pub hash: String,
    /// Serialized policy data
    pub data: Vec<u8>,
    /// Policy metadata
    pub metadata: HashMap<String, String>,
    /// Timestamp when policy was created/updated
    pub timestamp: u64,
}

/// Request to store a new policy
#[derive(Debug, Serialize, Deserialize)]
pub struct StorePolicyRequest {
    /// Policy file content
    pub policy: PolicyFile,
    /// Sender's signature (optional for verification)
    pub signature: Option<Vec<u8>>,
}

/// Response when storing a policy
#[derive(Debug, Serialize, Deserialize)]
pub struct StorePolicyResponse {
    /// Generated policy anchor ID
    pub policy_id: String,
    /// Policy content hash
    pub policy_hash: String,
    /// Whether the policy was stored successfully
    pub success: bool,
    /// Timestamp of the operation
    pub timestamp: u64,
}

/// Request to retrieve a policy
#[derive(Debug, Serialize, Deserialize)]
pub struct GetPolicyRequest {
    /// Policy anchor ID to retrieve
    pub policy_id: String,
}

/// Response when retrieving a policy
#[derive(Debug, Serialize, Deserialize)]
pub struct GetPolicyResponse {
    /// Policy ID
    pub policy_id: String,
    /// Whether the policy was found
    pub found: bool,
    /// Policy file if found
    pub policy: Option<PolicyFile>,
    /// Timestamp of the operation
    pub timestamp: u64,
}

/// Token Policy Store for maintaining a global reference of all token policies
pub struct PolicyStore {
    /// Storage engine for persisting policies
    storage_engine: Arc<dyn StorageEngine>,
    /// In-memory cache of policies for quick access
    policy_cache: RwLock<HashMap<String, PolicyStorageEntry>>,
}

impl PolicyStore {
    /// Create a new policy store with the given storage engine
    pub fn new(storage_engine: Arc<dyn StorageEngine>) -> Self {
        Self {
            storage_engine,
            policy_cache: RwLock::new(HashMap::new()),
        }
    }

    /// Initialize the policy store by loading cached policies from storage
    pub async fn initialize(&self) -> Result<()> {
        info!("Initializing policy store");

        // Load policies from storage
        let policies = self
            .storage_engine
            .list_policies()
            .await
            .map_err(|e| StorageNodeError::Storage(format!("Failed to load policies: {e}")))?;

        // Populate the cache
        let mut cache = self.policy_cache.write();
        for entry in policies {
            cache.insert(entry.id.clone(), entry);
        }

        info!("Policy store initialized with {} policies", cache.len());
        Ok(())
    }

    /// Store a new policy in the global reference
    pub async fn store_policy(&self, request: StorePolicyRequest) -> Result<StorePolicyResponse> {
        // Generate the policy anchor from the policy file
        let policy_anchor = PolicyAnchor::from_policy(&request.policy)
            .map_err(|e| StorageNodeError::BadRequest(format!("Invalid policy file: {e}")))?;

        let policy_id = policy_anchor.to_hex();
        let policy_hash = policy_id.clone();

        // Check if policy already exists
        {
            let cache = self.policy_cache.read();
            if cache.contains_key(&policy_id) {
                // Policy already exists, return success
                return Ok(StorePolicyResponse {
                    policy_id,
                    policy_hash,
                    success: true,
                    timestamp: std::time::SystemTime::now()
                        .duration_since(std::time::UNIX_EPOCH)
                        .unwrap_or_default()
                        .as_secs(),
                });
            }
        }

        // Serialize the policy
        let policy_data = serde_json::to_vec(&request.policy)
            .map_err(|e| StorageNodeError::Storage(format!("Failed to serialize policy: {e}")))?;

        // Create metadata for the policy
        let mut metadata = HashMap::new();
        metadata.insert("name".to_string(), request.policy.name.clone());
        metadata.insert("version".to_string(), request.policy.version.clone());
        metadata.insert("creator".to_string(), request.policy.creator.clone());

        // Create storage entry
        let entry = PolicyStorageEntry {
            id: policy_id.clone(),
            hash: policy_hash.clone(),
            data: policy_data,
            metadata,
            timestamp: std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .unwrap_or_default()
                .as_secs(),
        };

        // Store the policy in storage engine
        self.storage_engine
            .store_policy(&entry)
            .await
            .map_err(|e| StorageNodeError::Storage(format!("Failed to store policy: {e}")))?;

        // Add to cache
        {
            let mut cache = self.policy_cache.write();
            cache.insert(policy_id.clone(), entry);
        }

        info!("Stored new policy with ID: {}", policy_id);

        Ok(StorePolicyResponse {
            policy_id,
            policy_hash,
            success: true,
            timestamp: std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .unwrap_or_default()
                .as_secs(),
        })
    }

    /// Get a policy by its ID
    pub async fn get_policy(&self, request: GetPolicyRequest) -> Result<GetPolicyResponse> {
        // Try to get from cache first
        {
            let cache = self.policy_cache.read();
            if let Some(entry) = cache.get(&request.policy_id) {
                // Deserialize the policy
                let policy = serde_json::from_slice::<PolicyFile>(&entry.data).map_err(|e| {
                    StorageNodeError::Storage(format!("Failed to deserialize policy: {e}"))
                })?;

                return Ok(GetPolicyResponse {
                    policy_id: request.policy_id,
                    found: true,
                    policy: Some(policy),
                    timestamp: std::time::SystemTime::now()
                        .duration_since(std::time::UNIX_EPOCH)
                        .unwrap_or_default()
                        .as_secs(),
                });
            }
        }

        // If not in cache, try to get from storage
        match self.storage_engine.get_policy(&request.policy_id).await {
            Ok(Some(entry)) => {
                // Add to cache
                {
                    let mut cache = self.policy_cache.write();
                    cache.insert(request.policy_id.clone(), entry.clone());
                }

                // Deserialize the policy
                let policy = serde_json::from_slice::<PolicyFile>(&entry.data).map_err(|e| {
                    StorageNodeError::Storage(format!("Failed to deserialize policy: {e}"))
                })?;

                Ok(GetPolicyResponse {
                    policy_id: request.policy_id,
                    found: true,
                    policy: Some(policy),
                    timestamp: std::time::SystemTime::now()
                        .duration_since(std::time::UNIX_EPOCH)
                        .unwrap_or_default()
                        .as_secs(),
                })
            }
            Ok(None) => {
                // Policy not found
                Ok(GetPolicyResponse {
                    policy_id: request.policy_id,
                    found: false,
                    policy: None,
                    timestamp: std::time::SystemTime::now()
                        .duration_since(std::time::UNIX_EPOCH)
                        .unwrap_or_default()
                        .as_secs(),
                })
            }
            Err(e) => Err(StorageNodeError::Storage(format!(
                "Failed to get policy: {e}"
            ))),
        }
    }

    /// List all policies (with optional pagination)
    pub async fn list_policies(
        &self,
        limit: Option<usize>,
        offset: Option<usize>,
    ) -> Result<Vec<String>> {
        // Get all policy IDs from cache
        let cache = self.policy_cache.read();
        let mut policy_ids: Vec<String> = cache.keys().cloned().collect();

        // Sort by policy ID
        policy_ids.sort();

        // Apply pagination if specified
        let policy_ids = if let (Some(limit), Some(offset)) = (limit, offset) {
            policy_ids.into_iter().skip(offset).take(limit).collect()
        } else if let Some(limit) = limit {
            policy_ids.into_iter().take(limit).collect()
        } else {
            policy_ids
        };

        Ok(policy_ids)
    }

    /// Remove a policy from the store (for administrative purposes)
    pub async fn remove_policy(&self, policy_id: &str) -> Result<bool> {
        // Remove from storage first
        let result = self
            .storage_engine
            .remove_policy(policy_id)
            .await
            .map_err(|e| StorageNodeError::Storage(format!("Failed to remove policy: {e}")))?;

        // If successfully removed from storage, remove from cache
        if result {
            let mut cache = self.policy_cache.write();
            cache.remove(policy_id);
            info!("Removed policy with ID: {}", policy_id);
        }

        Ok(result)
    }
}

#[cfg(test)]
mod tests {
    #[allow(unused_imports)]
    use super::*;

    // Tests for PolicyStore would go here
}
