// hash.rs
//
// Higher-level hash operations for DSM that build upon the lower-level Blake3 implementations.
// This module provides semantic functions that map to the concepts described in the whitepaper.
//
// Note: Core hashing operations are implemented in blake3.rs. This module provides a more
// domain-specific abstraction layer on top of those primitives.

use super::StorageNodeError;

pub use blake3::Hash as HashOutput;

/// Hash data using Blake3
///
/// This implementation follows the whitepaper section 3.1 for straight hash chain verification
///
/// # Arguments
/// * `data` - The data to hash
///
/// # Returns
/// * `HashOutput` - Blake3 hash of the data
pub fn blake3(data: &[u8]) -> HashOutput {
    crate::crypto::blake3::hash_blake3(data)
}

/// Hash data and return as bytes
///
/// # Arguments
/// * `data` - The data to hash
///
/// # Returns
/// * `Vec<u8>` - Blake3 hash of the data as bytes
pub fn hash_to_bytes(data: &[u8]) -> Vec<u8> {
    blake3(data).as_bytes().to_vec()
}

/// Calculate deterministic entropy for state transition
///
/// This implements the entropy evolution for state transitions as described
/// in whitepaper section 7.1:
/// en+1 = H(en || opn+1 || (n+1))
///
/// # Arguments
/// * `current_entropy` - Current entropy value
/// * `operation` - Operation data
/// * `state_number` - State number for the transition
///
/// # Returns
/// * `HashOutput` - Next entropy value
pub fn calculate_next_entropy(
    current_entropy: &[u8],
    operation: &[u8],
    state_number: u64,
) -> HashOutput {
    crate::crypto::blake3::generate_deterministic_entropy(current_entropy, operation, state_number)
}

/// Generate a verification seed for random walk verification
///
/// This implements the random walk verification approach from whitepaper sections
/// 13 and 14, which provides efficient verification without hardware TEE.
///
/// # Arguments
/// * `state_hash` - Hash of the state
/// * `additional_entropy` - Additional entropy
///
/// # Returns
/// * `HashOutput` - Verification seed
pub fn generate_verification_seed(state_hash: &[u8], additional_entropy: &[u8]) -> HashOutput {
    let mut data = Vec::with_capacity(state_hash.len() + additional_entropy.len());
    data.extend_from_slice(state_hash);
    data.extend_from_slice(additional_entropy);

    blake3(&data)
}

/// Calculate state hash using deterministic algorithm
///
/// This implements the state hashing approach from whitepaper section
/// 3.1, creating a hash chain that links states together.
///
/// # Arguments
/// * `state_number` - State number
/// * `previous_state_hash` - Hash of the previous state
/// * `operation` - Operation data
/// * `entropy` - Entropy for this state
/// * `additional_data` - Additional data for the state
///
/// # Returns
/// * `Result<HashOutput, StorageNodeError>` - State hash or error
pub fn calculate_state_hash(
    state_number: u64,
    previous_state_hash: &[u8],
    operation: &[u8],
    entropy: &[u8],
    additional_data: &[u8],
) -> Result<HashOutput, StorageNodeError> {
    let mut data = Vec::new();

    // Add state number
    data.extend_from_slice(&state_number.to_be_bytes());

    // Add previous state hash
    data.extend_from_slice(previous_state_hash);

    // Add operation data
    data.extend_from_slice(operation);

    // Add entropy
    data.extend_from_slice(entropy);

    // Add additional data
    data.extend_from_slice(additional_data);

    Ok(blake3(&data))
}

/// Combine multiple hashes into a single hash
///
/// # Arguments
/// * `hashes` - Hashes to combine
///
/// # Returns
/// * `HashOutput` - Combined hash
pub fn combine_hashes(hashes: &[HashOutput]) -> HashOutput {
    let mut hasher = blake3::Hasher::new();

    for hash in hashes {
        // Create a reference to the hash bytes that lasts for the entire update operation
        let hash_bytes_ref = hash.as_bytes();
        hasher.update(hash_bytes_ref);
    }

    hasher.finalize()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_blake3_hashing() {
        let data = b"test data";
        let hash = blake3(data);

        // Test hash is not empty
        assert!(!hash.as_bytes().is_empty());

        // Test determinism (same input gives same output)
        let hash2 = blake3(data);
        assert_eq!(hash, hash2);

        // Test different input gives different output
        let different_data = b"different data";
        let different_hash = blake3(different_data);
        assert_ne!(hash, different_hash);
    }

    #[test]
    fn test_calculate_next_entropy() {
        let current_entropy = b"current entropy";
        let operation = b"test operation";
        let state_number = 42;

        let next_entropy = calculate_next_entropy(current_entropy, operation, state_number);

        // Test determinism (same input gives same output)
        let next_entropy2 = calculate_next_entropy(current_entropy, operation, state_number);
        assert_eq!(next_entropy, next_entropy2);

        // Test different input gives different output
        let different_entropy =
            calculate_next_entropy(current_entropy, b"different operation", state_number);
        assert_ne!(next_entropy, different_entropy);
    }

    #[test]
    fn test_calculate_state_hash() {
        let state_number = 42;
        let binding = blake3(b"previous state");
        let previous_state_hash = binding.as_bytes();
        let operation = b"test operation";
        let entropy = b"test entropy";
        let additional_data = b"additional data";

        let state_hash = calculate_state_hash(
            state_number,
            previous_state_hash,
            operation,
            entropy,
            additional_data,
        )
        .unwrap();

        // Test determinism (same input gives same output)
        let state_hash2 = calculate_state_hash(
            state_number,
            previous_state_hash,
            operation,
            entropy,
            additional_data,
        )
        .unwrap();
        assert_eq!(state_hash, state_hash2);

        // Test different input gives different output
        let different_hash = calculate_state_hash(
            state_number + 1,
            previous_state_hash,
            operation,
            entropy,
            additional_data,
        )
        .unwrap();
        assert_ne!(state_hash, different_hash);
    }

    #[test]
    fn test_combine_hashes() {
        let hash1 = blake3(b"data1");
        let hash2 = blake3(b"data2");
        let hash3 = blake3(b"data3");

        let combined = combine_hashes(&[hash1, hash2, hash3]);

        // Test determinism (same input gives same output)
        let combined2 = combine_hashes(&[hash1, hash2, hash3]);
        assert_eq!(combined, combined2);

        // Test different input gives different output
        let different_combined = combine_hashes(&[hash1, hash3, hash2]);
        assert_ne!(combined, different_combined);
    }
}
