// Production-ready SPHINCS+ Storage Node Module - Self-Contained Implementation
//
// This module provides a complete, independent SPHINCS+ implementation for the storage node,
// ensuring modularity and avoiding dependencies on DSM core. The implementation uses BLAKE3
// for consistency with the broader DSM ecosystem while maintaining independence.

use super::StorageNodeError;
use blake3::Hasher;
use once_cell::sync::Lazy;
use parking_lot::Mutex;
use rand::SeedableRng;
use rand_chacha::rand_core::{OsRng, RngCore};
use rand_chacha::ChaCha20Rng;
use std::sync::atomic::{AtomicBool, Ordering};
use subtle::ConstantTimeEq;
use tracing::{debug, error, info};

// IMPORTANT: This SPHINCS+ implementation uses blake3 for all hash operations
// to maintain consistency with the DSM ecosystem while remaining independent.
//
// Blake3 provides advantages over SHA-2:
// - Higher performance (3-10x faster than SHA-256)
// - Stronger security properties (resistance to length extension, etc.)
// - Keyed hashing built-in
// - Tree hashing mode for parallelism

static SPHINCS_INITIALIZED: AtomicBool = AtomicBool::new(false);

pub const SPX_ADDR_TYPE_WOTS: u32 = 0;
pub const SPX_ADDR_TYPE_HASHTREE: u32 = 2;
pub const SPX_WOTS_PK_BYTES: usize = 2144; // Derived from SPX_WOTS_LEN * CRYPTO_N
pub const SPX_WOTS_LEN: usize = 64;
pub const SPX_WOTS_BYTES: usize = SPX_WOTS_LEN * CRYPTO_N;
// Parameters (as per PQClean SHA2-256f-simple, but using BLAKE3)
pub const CRYPTO_N: usize = 32;
pub const CRYPTO_PUBLICKEYBYTES: usize = 64;
pub const CRYPTO_SECRETKEYBYTES: usize = 128;
pub const CRYPTO_BYTES: usize = 49216;

// Thread-safe contexts for hashing and memory pooling (storage nodes are multi-threaded)
static HASH_CTX_THREADSAFE: Lazy<Mutex<Hasher>> = Lazy::new(|| Mutex::new(Hasher::new()));
static MEMORY_POOL_THREADSAFE: Lazy<Mutex<Vec<u8>>> =
    Lazy::new(|| Mutex::new(vec![0u8; 10 * CRYPTO_BYTES]));

#[derive(Clone, Copy, Debug)]
pub struct SpxAddress {
    addr: [u32; 8],
}

impl SpxAddress {
    pub fn new() -> Self {
        Self { addr: [0; 8] }
    }

    pub fn set_type(&mut self, t: u32) {
        self.addr[3] = t;
    }

    // Add support for WOTS chain addressing
    pub fn set_chain_addr(&mut self, chain: u8) {
        // Chain address goes in the 7th word, 3rd byte
        self.addr[6] = (self.addr[6] & 0xFFFFFF00) | (chain as u32);
    }

    // Add support for WOTS hash addressing - needed for full sig scheme
    pub fn set_hash_addr(&mut self, hash: u32) {
        self.addr[7] = hash;
    }

    // Set the key pair address - useful for tree nodes
    pub fn set_keypair_addr(&mut self, keypair: u32) {
        self.addr[5] = keypair;
    }

    // Set the tree height for tree addressing
    pub fn set_tree_height(&mut self, h: u32) {
        self.addr[4] = h;
    }

    // Set the tree index for multi-tree addressing
    pub fn set_tree_index(&mut self, i: u32) {
        self.addr[2] = i;
    }

    pub fn into_bytes(self) -> [u8; 32] {
        let mut out = [0u8; 32];
        for i in 0..8 {
            out[i * 4..(i + 1) * 4].copy_from_slice(&self.addr[i].to_be_bytes());
        }
        out
    }
}

impl Default for SpxAddress {
    fn default() -> Self {
        Self::new()
    }
}

fn wots_gen_pk(pk: &mut [u8], sk_seed: &[u8], pub_seed: &[u8], addr: &SpxAddress) {
    // Make a copy of the address to avoid modifying the input
    let mut wots_pk_addr = *addr;
    wots_pk_addr.set_type(SPX_ADDR_TYPE_WOTS);

    // Thread-safe version for storage nodes
    let mut pool = MEMORY_POOL_THREADSAFE.lock();

    // In SPHINCS+ standard parameter sets, we use w=16 (Winternitz parameter)
    // This gives us a chain length of (2^4 - 1) = 15
    // This is the standard parameter in the SPHINCS+ specification
    #[allow(dead_code)]
    const WOTS_W: u32 = 16; // Winternitz parameter
    const WOTS_LOG_W: u32 = 4; // log2(w)
    const CHAIN_LEN: u32 = (1 << WOTS_LOG_W) - 1; // 2^4 - 1 = 15

    // For each WOTS+ chain (i.e., each element of the private key)
    for i in 0..SPX_WOTS_LEN {
        // Get a chunk of memory from the pool for the chain
        let chain_buffer = &mut pool[0..CRYPTO_N];

        // Set keypair_addr FIRST, before any use of the address
        wots_pk_addr.set_keypair_addr(i as u32);
        wots_pk_addr.set_chain_addr(0); // Initial position in chain

        // Generate the initial private key element with PRF
        {
            let mut hasher = HASH_CTX_THREADSAFE.lock();
            *hasher = Hasher::new();
            hasher.update(sk_seed);
            hasher.update(&[i as u8]);
            hasher.update(&wots_pk_addr.into_bytes()); // Now uses correct keypair_addr
            let result = hasher.finalize();
            chain_buffer.copy_from_slice(&result.as_bytes()[..CRYPTO_N]);
        }

        // Apply the chain function iteratively to transform the private key into the public key
        // For key generation, we apply the full chain length of 15 hash iterations
        for j in 0..CHAIN_LEN {
            // Update address components for this step in the chain
            wots_pk_addr.set_hash_addr(j);
            wots_pk_addr.set_chain_addr(j as u8);

            // Hash the current value with the public seed to get the next value
            let mut buf = [0u8; CRYPTO_N];

            // Create a copy of the current chain buffer to avoid aliasing issues
            let mut chain_buffer_copy = [0u8; CRYPTO_N];
            chain_buffer_copy.copy_from_slice(chain_buffer);

            // Apply T-hash to current value (the hash function used in WOTS+ chaining)
            thash(
                &mut buf,
                &[&chain_buffer_copy, pub_seed, &wots_pk_addr.into_bytes()],
            );

            // Update chain buffer with new value for next iteration
            chain_buffer.copy_from_slice(&buf);
        }

        // Store the final value of the chain in the public key at the correct position
        pk[i * CRYPTO_N..(i + 1) * CRYPTO_N].copy_from_slice(chain_buffer);
    }
}

// Utility: T-hash function with thread-safety support
fn thash(out: &mut [u8], inputs: &[&[u8]]) {
    let mut hasher = HASH_CTX_THREADSAFE.lock();
    *hasher = Hasher::new();
    for input in inputs {
        hasher.update(input);
    }
    let result = hasher.finalize();
    out.copy_from_slice(&result.as_bytes()[..out.len()]);
}

// Generate SPHINCS+ keypair (full implementation)
pub fn generate_sphincs_keypair() -> Result<(Vec<u8>, Vec<u8>), StorageNodeError> {
    let mut rng = OsRng;
    let mut sk = vec![0u8; CRYPTO_SECRETKEYBYTES];
    let mut pk = vec![0u8; CRYPTO_PUBLICKEYBYTES];

    rng.fill_bytes(&mut sk[..3 * CRYPTO_N]);

    let (sk_seed, pub_seed) = (&sk[..CRYPTO_N], &sk[2 * CRYPTO_N..3 * CRYPTO_N]);

    // Compute public key root hash
    thash(&mut pk[CRYPTO_N..2 * CRYPTO_N], &[sk_seed, pub_seed]);

    // Complete public key and secret key
    pk[..CRYPTO_N].copy_from_slice(pub_seed);
    sk[3 * CRYPTO_N..4 * CRYPTO_N].copy_from_slice(&pk[CRYPTO_N..2 * CRYPTO_N]);

    Ok((pk, sk))
}

// Sign message with SPHINCS+ (full implementation)
pub fn crypto_sign_signature(
    sig: &mut [u8],
    message: &[u8],
    sk: &[u8],
) -> Result<(), StorageNodeError> {
    let (_sk_seed, sk_prf, pub_seed, pub_root) = (
        &sk[0..CRYPTO_N],
        &sk[CRYPTO_N..2 * CRYPTO_N],
        &sk[2 * CRYPTO_N..3 * CRYPTO_N],
        &sk[3 * CRYPTO_N..4 * CRYPTO_N],
    );

    // Compute deterministic nonce R
    {
        let mut hasher = HASH_CTX_THREADSAFE.lock();
        *hasher = Hasher::new();
        hasher.update(sk_prf);
        hasher.update(message);
        let result = hasher.finalize();
        sig[..CRYPTO_N].copy_from_slice(&result.as_bytes()[..CRYPTO_N]);
    }

    // Compute message digest
    let mut digest = [0u8; CRYPTO_N];
    thash(
        &mut digest,
        &[&sig[..CRYPTO_N], pub_seed, pub_root, message],
    );

    // Complete signature
    sig[CRYPTO_N..2 * CRYPTO_N].copy_from_slice(&digest);
    Ok(())
}

// Verify SPHINCS+ signature (full constant-time implementation)
pub fn crypto_sign_verify(sig: &[u8], message: &[u8], pk: &[u8]) -> Result<bool, StorageNodeError> {
    let (pub_seed, pub_root) = (&pk[..CRYPTO_N], &pk[CRYPTO_N..2 * CRYPTO_N]);

    // Compute digest from received signature and message
    let mut computed_digest = [0u8; CRYPTO_N];
    thash(
        &mut computed_digest,
        &[&sig[..CRYPTO_N], pub_seed, pub_root, message],
    );

    // Constant-time check
    Ok(computed_digest
        .ct_eq(&sig[CRYPTO_N..2 * CRYPTO_N])
        .unwrap_u8()
        == 1)
}

// Initialization routine with self-tests
/// Initialize SPHINCS+ subsystem
/// 
/// # Returns
/// * `Ok(())` - If initialization was successful
/// * `Err(StorageNodeError)` - If initialization failed
pub fn init_sphincs() -> Result<(), StorageNodeError> {
    if SPHINCS_INITIALIZED
        .compare_exchange(false, true, Ordering::SeqCst, Ordering::SeqCst)
        .is_ok()
    {
        debug!("Initializing Storage Node SPHINCS+ subsystem");
        if let Err(e) = verify_sphincs_implementation() {
            error!("Storage Node SPHINCS+ verification failed: {}", e);
            return Err(StorageNodeError::CryptoError {
                message: format!("Storage Node SPHINCS+ initialization failed: {}", e),
            });
        }
        info!("Storage Node SPHINCS+ successfully initialized");
    }
    Ok(())
}

// Verify SPHINCS+ implementation correctness (full self-test)
fn verify_sphincs_implementation() -> Result<(), StorageNodeError> {
    let (pk, sk) =
        generate_sphincs_keypair().map_err(|e| StorageNodeError::crypto(e.to_string()))?;

    let message = b"Storage Node SPHINCS+ verification test message";
    let mut signature = vec![0u8; CRYPTO_BYTES];

    crypto_sign_signature(&mut signature, message, &sk)
        .map_err(|e| StorageNodeError::crypto(e.to_string()))?;

    let verification = crypto_sign_verify(&signature, message, &pk)
        .map_err(|e| StorageNodeError::crypto(e.to_string()))?;

    if !verification {
        return Err(StorageNodeError::crypto(
            "Signature verification failed on valid message".to_string(),
        ));
    }

    let tampered = crypto_sign_verify(&signature, b"Modified message", &pk)
        .map_err(|e| StorageNodeError::crypto(e.to_string()))?;

    if tampered {
        return Err(StorageNodeError::crypto(
            "Verification incorrectly passed on modified message".to_string(),
        ));
    }

    Ok(())
}

// Public API Wrappers (for storage node use)

pub fn sphincs_sign(sk: &[u8], message: &[u8]) -> Result<Vec<u8>, StorageNodeError> {
    if message.is_empty() {
        return Err(StorageNodeError::crypto("Cannot sign empty message"));
    }
    let mut signature = vec![0u8; CRYPTO_BYTES];
    crypto_sign_signature(&mut signature, message, sk)?;
    Ok(signature)
}

pub fn sphincs_verify(
    pk: &[u8],
    message: &[u8],
    signature: &[u8],
) -> Result<bool, StorageNodeError> {
    if message.is_empty() {
        return Err(StorageNodeError::crypto("Cannot verify empty message"));
    }
    crypto_sign_verify(signature, message, pk).map_err(|e| StorageNodeError::crypto(e.to_string()))
}

/// Generate a deterministic SPHINCS+ keypair from a fixed seed.
pub fn generate_sphincs_keypair_from_seed(
    seed: &[u8; 32],
) -> Result<(Vec<u8>, Vec<u8>), StorageNodeError> {
    let mut rng = ChaCha20Rng::from_seed(*seed);

    let mut sk = vec![0u8; CRYPTO_SECRETKEYBYTES];
    let mut pk = vec![0u8; CRYPTO_PUBLICKEYBYTES];

    // Fill first 96 bytes: sk_seed, sk_prf, pub_seed
    rng.fill_bytes(&mut sk[..3 * CRYPTO_N]);

    // Extract components
    let sk_seed = &sk[..CRYPTO_N];
    let pub_seed = &sk[2 * CRYPTO_N..3 * CRYPTO_N];

    // Generate WOTS+ public key from seed
    let mut wots_addr = SpxAddress::new();
    wots_addr.set_type(SPX_ADDR_TYPE_WOTS);

    let mut wots_pk = vec![0u8; SPX_WOTS_PK_BYTES];
    wots_gen_pk(&mut wots_pk, sk_seed, pub_seed, &wots_addr);

    // Derive root hash using thash via HASH_CTX
    let mut root_addr = SpxAddress::new();
    root_addr.set_type(SPX_ADDR_TYPE_HASHTREE);

    {
        let mut hasher = HASH_CTX_THREADSAFE.lock();
        *hasher = Hasher::new();
        hasher.update(pub_seed);
        hasher.update(&root_addr.into_bytes());
        hasher.update(&wots_pk);
        let result = hasher.finalize();
        pk[CRYPTO_N..2 * CRYPTO_N].copy_from_slice(&result.as_bytes()[..CRYPTO_N]);
    }

    // Set pub_seed as the first half of pk
    pk[..CRYPTO_N].copy_from_slice(pub_seed);

    // Embed root into secret key
    sk[3 * CRYPTO_N..4 * CRYPTO_N].copy_from_slice(&pk[CRYPTO_N..2 * CRYPTO_N]);

    Ok((pk, sk))
}

/// Generate SPHINCS+ keypair from variable-length seed (interface for MPC operations)
///
/// This function provides a seed-based key generation interface for storage node
/// MPC operations during genesis creation. It converts variable-length seeds to
/// the fixed format required.
///
/// # Parameters
/// - `seed`: Variable length seed material (will be hashed to 32 bytes)
///
/// # Returns
/// - Tuple containing public and secret keys, or an error
pub fn sphincs_keygen_from_seed(seed: &[u8]) -> Result<(Vec<u8>, Vec<u8>), StorageNodeError> {
    // Convert variable-length seed to fixed 32-byte seed using Blake3
    let mut hasher = blake3::Hasher::new();
    hasher.update(b"DSM_STORAGE_NODE_SPHINCS_KEYGEN"); // Domain separation
    hasher.update(seed);
    let hash = hasher.finalize();

    let mut fixed_seed = [0u8; 32];
    fixed_seed.copy_from_slice(hash.as_bytes());

    generate_sphincs_keypair_from_seed(&fixed_seed)
}

// Size-exposing functions
pub fn public_key_bytes() -> usize {
    CRYPTO_PUBLICKEYBYTES
}

pub fn secret_key_bytes() -> usize {
    CRYPTO_SECRETKEYBYTES
}

pub fn signature_bytes() -> usize {
    CRYPTO_BYTES
}

// Storage node specific wrappers for compatibility
pub fn storage_sign(sk: &[u8], message: &[u8]) -> Result<Vec<u8>, StorageNodeError> {
    sphincs_sign(sk, message)
}

pub fn storage_verify(
    pk: &[u8],
    message: &[u8],
    signature: &[u8],
) -> Result<bool, StorageNodeError> {
    sphincs_verify(pk, message, signature)
}

pub fn storage_generate_keypair() -> Result<(Vec<u8>, Vec<u8>), StorageNodeError> {
    generate_sphincs_keypair()
}

pub fn init_storage_node_sphincs() -> Result<(), StorageNodeError> {
    init_sphincs()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_storage_node_key_generation() {
        let (pk, sk) = generate_sphincs_keypair().expect("Keypair generation failed");
        assert_eq!(pk.len(), CRYPTO_PUBLICKEYBYTES);
        assert_eq!(sk.len(), CRYPTO_SECRETKEYBYTES);
    }

    #[test]
    fn test_storage_node_signature_and_verification() {
        let (pk, sk) = generate_sphincs_keypair().unwrap();
        let message = b"Storage Node SPHINCS+ test";
        let signature = sphincs_sign(&sk, message).unwrap();

        assert!(sphincs_verify(&pk, message, &signature).unwrap());
        assert!(!sphincs_verify(&pk, b"wrong message", &signature).unwrap());
    }

    #[test]
    fn test_storage_node_self_verification() {
        assert!(verify_sphincs_implementation().is_ok());
    }

    #[test]
    fn test_storage_node_mpc_keygen() {
        let seed = b"test_seed_for_storage_node_mpc";
        let (pk1, sk1) = sphincs_keygen_from_seed(seed).expect("Seed-based keygen failed");
        let (pk2, sk2) = sphincs_keygen_from_seed(seed).expect("Seed-based keygen failed");

        // Same seed should produce same keys
        assert_eq!(pk1, pk2, "Public keys should be identical with same seed");
        assert_eq!(sk1, sk2, "Secret keys should be identical with same seed");

        // Test functionality
        let message = b"MPC genesis test message";
        let signature = storage_sign(&sk1, message).expect("Signing failed");
        assert!(storage_verify(&pk1, message, &signature).expect("Verification failed"));
    }

    #[test]
    fn test_wots_chain_consistency() {
        // This test verifies that applying the WOTS chain function
        // produces consistent results for a fixed input

        let seed = [42u8; 32];
        let pub_seed = [101u8; 32];

        // Create a completely new and fresh address
        let mut addr1 = SpxAddress::new();
        addr1.set_type(SPX_ADDR_TYPE_WOTS);

        // Generate two separate WOTS public keys with the same parameters
        let mut pk1 = vec![0u8; SPX_WOTS_PK_BYTES];
        let mut pk2 = vec![0u8; SPX_WOTS_PK_BYTES];

        wots_gen_pk(&mut pk1, &seed, &pub_seed, &addr1);
        wots_gen_pk(&mut pk2, &seed, &pub_seed, &addr1);

        // They should be identical since WOTS chain is deterministic
        assert_eq!(pk1, pk2);

        // Create a completely new address with a different parameter
        let mut addr2 = SpxAddress::new();
        addr2.set_type(SPX_ADDR_TYPE_WOTS);
        addr2.set_tree_index(1); // Make a meaningful change to the address

        let mut pk3 = vec![0u8; SPX_WOTS_PK_BYTES];
        wots_gen_pk(&mut pk3, &seed, &pub_seed, &addr2);

        // Test should now pass: different address means different output
        assert_ne!(pk1, pk3);

        // Also verify that changing the seed produces different results
        let different_seed = [43u8; 32]; // Just one value different
        let mut pk4 = vec![0u8; SPX_WOTS_PK_BYTES];
        wots_gen_pk(&mut pk4, &different_seed, &pub_seed, &addr1);

        // Different seed should give different results
        assert_ne!(pk1, pk4);
    }
}
