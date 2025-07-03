// Sparse Merkle Tree implementation for DSM protocol state management
//
// This module implements a Sparse Merkle Tree (SMT) as required by the DSM protocol
// for efficient state verification and sparse index support.

use blake3;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

/// Height of the Sparse Merkle Tree (256-bit keys)
pub const SMT_HEIGHT: usize = 256;

/// Empty hash for unoccupied tree positions
pub const EMPTY_HASH: [u8; 32] = [0u8; 32];

/// Sparse Merkle Tree node
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SmtNode {
    /// Node hash value
    pub hash: [u8; 32],
    /// Left child hash (if internal node)
    pub left: Option<[u8; 32]>,
    /// Right child hash (if internal node)
    pub right: Option<[u8; 32]>,
    /// Leaf value (if leaf node)
    pub value: Option<Vec<u8>>,
    /// Node height in tree
    pub height: usize,
}

/// Sparse Merkle Tree proof for membership/non-membership
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SmtProof {
    /// Path from root to leaf
    pub siblings: Vec<[u8; 32]>,
    /// Bitmap indicating left/right path
    pub path_bits: Vec<bool>,
    /// Leaf value (None for non-membership proof)
    pub leaf_value: Option<Vec<u8>>,
    /// Root hash this proof is for
    pub root_hash: [u8; 32],
}

/// Sparse index entry for efficient key-value lookups
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SparseIndexEntry {
    /// Key hash for tree positioning
    pub key_hash: [u8; 32],
    /// Original key
    pub key: String,
    /// Associated value
    pub value: Vec<u8>,
    /// Tree path to this entry
    pub tree_path: Vec<bool>,
    /// Timestamp when entry was created/updated
    pub timestamp: u64,
    /// State index/sequence number
    pub state_index: u64,
}

/// Sparse Merkle Tree implementation
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SparseMerkleTree {
    /// Tree nodes indexed by hash
    pub nodes: HashMap<[u8; 32], SmtNode>,
    /// Current root hash
    pub root_hash: [u8; 32],
    /// Sparse index for efficient lookups
    pub sparse_index: HashMap<String, SparseIndexEntry>,
}

impl Default for SparseMerkleTree {
    fn default() -> Self {
        Self::new()
    }
}

impl SparseMerkleTree {
    /// Create a new empty Sparse Merkle Tree
    pub fn new() -> Self {
        Self {
            nodes: HashMap::new(),
            root_hash: EMPTY_HASH,
            sparse_index: HashMap::new(),
        }
    }

    /// Insert or update a key-value pair in the SMT
    pub fn insert(
        &mut self,
        key: &str,
        value: Vec<u8>,
        state_index: u64,
    ) -> Result<[u8; 32], String> {
        // Hash the key to get tree position
        let key_hash = self.hash_key(key);
        let mut tree_path = self.key_to_path(&key_hash);
        let height = self.current_tree_height();
        tree_path.truncate(height);

        // Create sparse index entry
        let index_entry = SparseIndexEntry {
            key_hash,
            key: key.to_string(),
            value: value.clone(),
            tree_path: tree_path.clone(),
            timestamp: std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .unwrap_or_default()
                .as_secs(),
            state_index,
        };

        // Update sparse index
        self.sparse_index.insert(key.to_string(), index_entry);

        // Rebuild the entire tree from scratch to ensure correctness
        self.rebuild_tree()?;

        Ok(self.root_hash)
    }

    /// Get value for a key from the SMT
    pub fn get(&self, key: &str) -> Option<&Vec<u8>> {
        self.sparse_index.get(key).map(|entry| &entry.value)
    }

    /// Helper: get current tree height (log2 of number of keys, min 1)
    fn current_tree_height(&self) -> usize {
        let n = self.sparse_index.len().max(2); // avoid log2(0)
        (n as f64).log2().ceil() as usize
    }

    /// Generate a membership proof for a key
    pub fn prove_membership(&self, key: &str) -> Result<SmtProof, String> {
        let key_hash = self.hash_key(key);
        let mut tree_path = self.key_to_path(&key_hash);
        let height = self.current_tree_height();
        tree_path.truncate(height);

        if let Some(entry) = self.sparse_index.get(key) {
            let mut siblings = self.collect_siblings(&tree_path)?;
            siblings.truncate(height);
            Ok(SmtProof {
                siblings,
                path_bits: tree_path,
                leaf_value: Some(entry.value.clone()),
                root_hash: self.root_hash,
            })
        } else {
            Err("Key not found in tree".to_string())
        }
    }

    /// Generate a non-membership proof for a key
    pub fn prove_non_membership(&self, key: &str) -> Result<SmtProof, String> {
        let key_hash = self.hash_key(key);
        let mut tree_path = self.key_to_path(&key_hash);
        let height = self.current_tree_height();
        tree_path.truncate(height);

        if self.sparse_index.contains_key(key) {
            return Err("Key exists in tree, cannot generate non-membership proof".to_string());
        }

        let mut siblings = self.collect_siblings(&tree_path)?;
        siblings.truncate(height);
        Ok(SmtProof {
            siblings,
            path_bits: tree_path,
            leaf_value: None,
            root_hash: self.root_hash,
        })
    }

    /// Verify a membership or non-membership proof
    pub fn verify_proof(&self, proof: &SmtProof, key: &str) -> bool {
        let key_hash = self.hash_key(key);
        let mut expected_path = self.key_to_path(&key_hash);
        let height = proof.path_bits.len();
        expected_path.truncate(height);

        // Verify path bits match
        if proof.path_bits != expected_path {
            tracing::debug!("verify_proof: path bits mismatch");
            return false;
        }

        // Reconstruct root from proof
        let reconstructed_root = self.reconstruct_root_from_proof(proof, &key_hash);
        tracing::debug!(
            "verify_proof: reconstructed_root={:x?}, proof.root_hash={:x?}, self.root_hash={:x?}",
            reconstructed_root,
            proof.root_hash,
            self.root_hash
        );

        // Verify root matches
        reconstructed_root == proof.root_hash && proof.root_hash == self.root_hash
    }

    /// Get all entries in the sparse index (for state enumeration)
    pub fn get_all_entries(&self) -> Vec<&SparseIndexEntry> {
        self.sparse_index.values().collect()
    }

    /// Get entries by state index range
    pub fn get_entries_by_state_range(
        &self,
        from_index: u64,
        to_index: u64,
    ) -> Vec<&SparseIndexEntry> {
        self.sparse_index
            .values()
            .filter(|entry| entry.state_index >= from_index && entry.state_index <= to_index)
            .collect()
    }
    /// Rebuild the entire tree from the sparse index
    fn rebuild_tree(&mut self) -> Result<(), String> {
        // Clear existing tree nodes
        self.nodes.clear();

        if self.sparse_index.is_empty() {
            self.root_hash = EMPTY_HASH;
            return Ok(());
        }

        let height = self.current_tree_height();

        // Create leaf nodes for all entries
        let mut leaf_hashes = std::collections::HashMap::new();
        for entry in self.sparse_index.values() {
            let mut hasher = blake3::Hasher::new();
            hasher.update(b"DSM_SMT_LEAF:");
            hasher.update(&entry.value);
            let leaf_hash = *hasher.finalize().as_bytes();

            let leaf_node = SmtNode {
                hash: leaf_hash,
                left: None,
                right: None,
                value: Some(entry.value.clone()),
                height,
            };

            self.nodes.insert(leaf_hash, leaf_node);

            // Map path to leaf hash
            let path_key = entry
                .tree_path
                .iter()
                .take(height)
                .map(|&b| if b { "1" } else { "0" })
                .collect::<String>();
            leaf_hashes.insert(path_key, leaf_hash);
        }

        // Build tree bottom-up
        self.root_hash = self.build_tree_level(leaf_hashes, height)?;

        Ok(())
    }

    /// Build tree level by level from leaves to root
    fn build_tree_level(
        &mut self,
        current_level: std::collections::HashMap<String, [u8; 32]>,
        depth: usize,
    ) -> Result<[u8; 32], String> {
        if depth == 0 {
            // We should have exactly one root
            if current_level.len() == 1 {
                return Ok(current_level.into_values().next().unwrap());
            } else if current_level.is_empty() {
                return Ok(EMPTY_HASH);
            } else {
                return Err("Multiple roots at depth 0".to_string());
            }
        }

        let mut next_level = std::collections::HashMap::new();

        // Group by parent path (remove last bit)
        let mut parent_groups: std::collections::HashMap<String, Vec<(String, [u8; 32])>> =
            std::collections::HashMap::new();

        for (path, hash) in current_level {
            let parent_path = if !path.is_empty() {
                path[..path.len() - 1].to_string()
            } else {
                "".to_string()
            };
            parent_groups
                .entry(parent_path)
                .or_default()
                .push((path, hash));
        }

        // Create internal nodes for each parent group
        for (parent_path, children) in parent_groups {
            let mut left_hash = EMPTY_HASH;
            let mut right_hash = EMPTY_HASH;

            for (child_path, child_hash) in children {
                if child_path.ends_with('0') {
                    left_hash = child_hash;
                } else {
                    right_hash = child_hash;
                }
            }

            // Create internal node
            let mut hasher = blake3::Hasher::new();
            hasher.update(b"DSM_SMT_INTERNAL:");
            hasher.update(&left_hash);
            hasher.update(&right_hash);
            let internal_hash = *hasher.finalize().as_bytes();

            let internal_node = SmtNode {
                hash: internal_hash,
                left: if left_hash == EMPTY_HASH {
                    None
                } else {
                    Some(left_hash)
                },
                right: if right_hash == EMPTY_HASH {
                    None
                } else {
                    Some(right_hash)
                },
                value: None,
                height: depth - 1,
            };

            self.nodes.insert(internal_hash, internal_node);
            next_level.insert(parent_path, internal_hash);
        }

        self.build_tree_level(next_level, depth - 1)
    }

    // Private helper methods

    /// Hash a key to get its position in the tree
    fn hash_key(&self, key: &str) -> [u8; 32] {
        let mut hasher = blake3::Hasher::new();
        hasher.update(b"DSM_SMT_KEY:");
        hasher.update(key.as_bytes());
        *hasher.finalize().as_bytes()
    }

    /// Convert key hash to tree path (bit sequence)
    fn key_to_path(&self, key_hash: &[u8; 32]) -> Vec<bool> {
        let mut path = Vec::with_capacity(SMT_HEIGHT);
        for byte in key_hash {
            for bit in 0..8 {
                path.push((byte >> (7 - bit)) & 1 == 1);
            }
        }
        path
    }

    /// Collect sibling hashes along a path for proof generation
    fn collect_siblings(&self, path: &[bool]) -> Result<Vec<[u8; 32]>, String> {
        let mut siblings = Vec::new();
        let mut current_hash = self.root_hash;

        for &go_right in path {
            if let Some(node) = self.nodes.get(&current_hash) {
                if go_right {
                    siblings.push(node.left.unwrap_or(EMPTY_HASH));
                    current_hash = node.right.unwrap_or(EMPTY_HASH);
                } else {
                    siblings.push(node.right.unwrap_or(EMPTY_HASH));
                    current_hash = node.left.unwrap_or(EMPTY_HASH);
                }
            } else {
                // Instead of erroring, treat as empty
                siblings.push(EMPTY_HASH);
                current_hash = EMPTY_HASH;
            }
        }

        Ok(siblings)
    }

    /// Reconstruct root hash from proof
    fn reconstruct_root_from_proof(&self, proof: &SmtProof, _key_hash: &[u8; 32]) -> [u8; 32] {
        let mut current_hash = if let Some(ref value) = proof.leaf_value {
            // Membership proof - hash the leaf value
            let mut hasher = blake3::Hasher::new();
            hasher.update(b"DSM_SMT_LEAF:");
            hasher.update(value);
            *hasher.finalize().as_bytes()
        } else {
            // Non-membership proof
            EMPTY_HASH
        };

        // Walk up the tree using siblings
        for (i, &sibling_hash) in proof.siblings.iter().enumerate().rev() {
            let go_right = proof.path_bits[proof.path_bits.len() - 1 - i];
            tracing::debug!("reconstruct_root_from_proof: i={}, go_right={}, sibling_hash={:x?}, current_hash={:x?}", i, go_right, sibling_hash, current_hash);
            let (left_hash, right_hash) = if go_right {
                (sibling_hash, current_hash)
            } else {
                (current_hash, sibling_hash)
            };

            let mut hasher = blake3::Hasher::new();
            hasher.update(b"DSM_SMT_INTERNAL:");
            hasher.update(&left_hash);
            hasher.update(&right_hash);
            current_hash = *hasher.finalize().as_bytes();
        }

        tracing::debug!(
            "reconstruct_root_from_proof: final current_hash={:x?}",
            current_hash
        );
        current_hash
    }
}

/// DSM-specific SMT operations for state chain management
impl SparseMerkleTree {
    /// Insert a DSM state entry into the SMT
    pub fn insert_dsm_state(
        &mut self,
        state_hash: &str,
        prev_hash: &str,
        operation: &str,
        balance_deltas: &std::collections::HashMap<String, i64>,
        state_index: u64,
    ) -> Result<[u8; 32], String> {
        // Serialize state data
        let state_data = serde_json::json!({
            "state_hash": state_hash,
            "prev_hash": prev_hash,
            "operation": operation,
            "balance_deltas": balance_deltas,
            "state_index": state_index,
            "timestamp": std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .unwrap_or_default()
                .as_secs()
        });

        let state_bytes = serde_json::to_vec(&state_data)
            .map_err(|e| format!("Failed to serialize state data: {e}"))?;

        // Insert using state hash as key
        self.insert(state_hash, state_bytes, state_index)
    }

    /// Get DSM state from SMT
    pub fn get_dsm_state(&self, state_hash: &str) -> Option<serde_json::Value> {
        self.get(state_hash)
            .and_then(|bytes| serde_json::from_slice(bytes).ok())
    }

    /// Generate a state proof for DSM verification
    pub fn generate_state_proof(&self, state_hash: &str) -> Result<SmtProof, String> {
        self.prove_membership(state_hash)
    }

    /// Verify a DSM state proof
    pub fn verify_dsm_state_proof(&self, proof: &SmtProof, state_hash: &str) -> bool {
        self.verify_proof(proof, state_hash)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use tracing_subscriber;

    fn init_tracing() {
        let _ = tracing_subscriber::fmt()
            .with_max_level(tracing::Level::DEBUG)
            .with_test_writer()
            .try_init();
    }

    #[test]
    fn test_smt_insert_and_get() {
        init_tracing();

        let mut smt = SparseMerkleTree::new();

        let key = "test_key";
        let value = b"test_value".to_vec();

        let root = smt.insert(key, value.clone(), 1).unwrap();
        assert_ne!(root, EMPTY_HASH);

        let retrieved = smt.get(key).unwrap();
        assert_eq!(retrieved, &value);
    }

    #[test]
    fn test_smt_membership_proof() {
        init_tracing();

        let mut smt = SparseMerkleTree::new();

        let key = "test_key";
        let value = b"test_value".to_vec();

        smt.insert(key, value.clone(), 1).unwrap();

        let proof = smt.prove_membership(key).unwrap();
        assert!(smt.verify_proof(&proof, key));

        // Verify proof contains the value
        assert_eq!(proof.leaf_value, Some(value));
    }

    #[test]
    fn test_smt_non_membership_proof() {
        init_tracing();

        let mut smt = SparseMerkleTree::new();

        let existing_key = "existing_key";
        let existing_value = b"existing_value".to_vec();
        smt.insert(existing_key, existing_value, 1).unwrap();

        let non_existing_key = "non_existing_key";
        let proof = smt.prove_non_membership(non_existing_key).unwrap();

        assert!(smt.verify_proof(&proof, non_existing_key));
        assert!(proof.leaf_value.is_none());
    }

    #[test]
    fn test_dsm_state_operations() {
        init_tracing();

        let mut smt = SparseMerkleTree::new();

        let state_hash = "state_123";
        let prev_hash = "state_122";
        let operation = "transfer";
        let mut balance_deltas = std::collections::HashMap::new();
        balance_deltas.insert("token_a".to_string(), -100);
        balance_deltas.insert("token_b".to_string(), 100);

        let root = smt
            .insert_dsm_state(state_hash, prev_hash, operation, &balance_deltas, 123)
            .unwrap();

        assert_ne!(root, EMPTY_HASH);

        let retrieved_state = smt.get_dsm_state(state_hash).unwrap();
        assert_eq!(retrieved_state["state_hash"], state_hash);
        assert_eq!(retrieved_state["prev_hash"], prev_hash);
        assert_eq!(retrieved_state["operation"], operation);
        assert_eq!(retrieved_state["state_index"], 123);

        let proof = smt.generate_state_proof(state_hash).unwrap();
        assert!(smt.verify_dsm_state_proof(&proof, state_hash));
    }
}
