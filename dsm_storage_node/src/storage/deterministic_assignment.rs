// DSM: Deterministic Storage Assignment and Consistency Enforcement
use blake3::Hasher;
use std::collections::{HashMap, HashSet};

use crate::error::Result;
use crate::types::StorageNode;

/// Unique identifier for an object (e.g., genesis state, token policy)
pub type ObjectId = [u8; 32];

/// Node ID (could be a public key, hash, or address)
pub type NodeId = [u8; 32];

#[derive(Debug, Clone)]
pub struct Node {
    pub id: NodeId,
    pub assigned_objects: HashSet<ObjectId>,
    pub stored_objects: HashSet<ObjectId>,
    pub reward_balance: u64,
}

impl Node {
    /// Returns true if this node has all objects it is assigned.
    pub fn is_consistent(&self) -> bool {
        self.assigned_objects.is_subset(&self.stored_objects)
    }

    /// Update reward balance based on consistency (protocol reward logic).
    pub fn update_reward(&mut self, base_reward: u64) {
        if self.is_consistent() {
            self.reward_balance += base_reward;
        } else {
            // Inconsistent nodes are ineligible for rewards
            self.reward_balance = 0;
        }
    }

    /// Audit function: returns missing object IDs, if any.
    pub fn missing_assignments(&self) -> HashSet<ObjectId> {
        self.assigned_objects
            .difference(&self.stored_objects)
            .copied()
            .collect()
    }
}

/// Deterministic storage assignment function.
/// For object D, assigns to r nodes from the network.
/// Returns the set of NodeId's responsible.
pub fn assignment(object_id: &ObjectId, r: usize, node_ids: &[NodeId]) -> HashSet<NodeId> {
    let mut assigned = HashSet::new();
    let node_count = node_ids.len();

    for k in 1..=r {
        // Concatenate object ID and replica index (k)
        let mut hasher = Hasher::new();
        hasher.update(object_id);
        hasher.update(&k.to_le_bytes());
        let hash = hasher.finalize();

        let idx = (u64::from_le_bytes(hash.as_bytes()[..8].try_into().unwrap()) % node_count as u64)
            as usize;
        assigned.insert(node_ids[idx]);
    }
    assigned
}

// Example: Protocol round for reward calculation and auditing
pub fn protocol_round(nodes: &mut [Node], base_reward: u64) {
    for node in nodes.iter_mut() {
        node.update_reward(base_reward);
        if !node.is_consistent() {
            let missing = node.missing_assignments();
            tracing::info!(
                "Node {:?} is inconsistent. Missing objects: {:?}",
                node.id,
                missing
            );
        }
    }
}

/// DSM Storage Assignment Manager
pub struct AssignmentManager {
    /// Replication factor (number of nodes that should store each object)
    replication_factor: usize,

    /// Assignment threshold for probabilistic assignment
    assignment_threshold: u64,
}

impl AssignmentManager {
    pub fn new(replication_factor: usize) -> Self {
        // Set threshold to 1/3 of u64::MAX for ~33% probability
        let assignment_threshold = u64::MAX / 3;
        Self {
            replication_factor,
            assignment_threshold,
        }
    }

    /// Check if a node is responsible for storing an object
    pub fn is_responsible(&self, object_id: &ObjectId, node_id: &NodeId) -> bool {
        let combined = [object_id.as_slice(), node_id.as_slice()].concat();
        let mut hasher = Hasher::new();
        hasher.update(&combined);
        let hash = hasher.finalize();

        let hash_value = u64::from_le_bytes([
            hash.as_bytes()[0],
            hash.as_bytes()[1],
            hash.as_bytes()[2],
            hash.as_bytes()[3],
            hash.as_bytes()[4],
            hash.as_bytes()[5],
            hash.as_bytes()[6],
            hash.as_bytes()[7],
        ]);

        hash_value < self.assignment_threshold
    }

    /// Get the set of nodes responsible for storing an object
    pub fn get_responsible_nodes(
        &self,
        object_id: &ObjectId,
        all_nodes: &[StorageNode],
    ) -> Vec<String> {
        let responsible_nodes: Vec<String> = all_nodes
            .iter()
            .filter_map(|node| {
                // Convert string node ID to NodeId
                let mut node_id = [0u8; 32];
                let node_bytes = node.id.as_bytes();
                let len = node_bytes.len().min(32);
                node_id[..len].copy_from_slice(&node_bytes[..len]);

                if self.is_responsible(object_id, &node_id) {
                    Some(node.id.clone())
                } else {
                    None
                }
            })
            .take(self.replication_factor)
            .collect();

        // If not enough nodes match, fall back to deterministic selection
        if responsible_nodes.is_empty() && !all_nodes.is_empty() {
            all_nodes
                .iter()
                .take(self.replication_factor.min(all_nodes.len()))
                .map(|node| node.id.clone())
                .collect()
        } else {
            responsible_nodes
        }
    }

    /// Verify that all assigned objects are present on the appropriate nodes
    pub fn verify_consistency(
        &self,
        nodes: &HashMap<NodeId, Node>,
        objects: &[ObjectId],
    ) -> Result<bool> {
        let node_ids: Vec<NodeId> = nodes.keys().copied().collect();

        for object_id in objects {
            let responsible = assignment(object_id, self.replication_factor, &node_ids);

            let mut found_count = 0;
            for node_id in &responsible {
                if let Some(node) = nodes.get(node_id) {
                    if node.stored_objects.contains(object_id) {
                        found_count += 1;
                    }
                }
            }

            // Require at least one copy for consistency
            if found_count == 0 {
                return Ok(false);
            }
        }

        Ok(true)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use rand::Rng;

    #[test]
    fn test_dsm_assignment_and_consistency() {
        // Simulate 20 nodes, 100 objects, r=4 redundancy
        let mut rng = rand::thread_rng();
        let node_ids: Vec<NodeId> = (0..20)
            .map(|_| {
                let mut id = [0u8; 32];
                rng.fill(&mut id);
                id
            })
            .collect();

        // Assign 100 objects
        let objects: Vec<ObjectId> = (0..100)
            .map(|_| {
                let mut id = [0u8; 32];
                rng.fill(&mut id);
                id
            })
            .collect();

        // Build node map
        let mut nodes: Vec<Node> = node_ids
            .iter()
            .map(|id| Node {
                id: *id,
                assigned_objects: HashSet::new(),
                stored_objects: HashSet::new(),
                reward_balance: 0,
            })
            .collect();

        // Assignment: for each object, assign r=4 nodes
        let r = 4;
        for object in &objects {
            let assigned_ids = assignment(object, r, &node_ids);
            for node in nodes.iter_mut() {
                if assigned_ids.contains(&node.id) {
                    node.assigned_objects.insert(*object);
                    // For the test, simulate a perfect storage node
                    node.stored_objects.insert(*object);
                }
            }
        }

        // Intentionally make one node inconsistent
        if let Some(bad_node) = nodes.get_mut(0) {
            // Remove one assignment
            if let Some(obj) = bad_node.assigned_objects.iter().next().copied() {
                bad_node.stored_objects.remove(&obj);
            }
        }

        protocol_round(&mut nodes, 100);
        for node in &nodes {
            if node.is_consistent() {
                assert_eq!(node.reward_balance, 100);
            } else {
                assert_eq!(node.reward_balance, 0);
            }
        }
    }
}
