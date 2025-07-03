//! # DSM Cryptography Module
//!
//! This module provides cryptographic primitives and operations for the DSM system, including:
//!
//! * Post-quantum secure encryption using Kyber
//! * Post-quantum secure signatures using SPHINCS+
//! * Hash functions (Blake3, SHA3)
//! * Pedersen commitments
//! * Secure RNG utilities
//! * Privacy-preserving random walks
//!
//! The module implements a hybrid encryption approach using post-quantum algorithms combined
//! with symmetric encryption (ChaCha20Poly1305) for data protection.
//!
//! ## Security Notice
//!
//! The current implementation uses a simple in-memory key store for development purposes.
//! In production, this would be replaced with secure storage (HSM, TEE, etc.).

use crate::error::StorageNodeError;

use chacha20poly1305::{
    aead::{Aead, KeyInit},
    ChaCha20Poly1305, Key, Nonce,
};
use std::collections::HashMap;
use std::sync::Mutex;
use tracing::{debug, warn};

pub mod blake3;
pub mod hash;
pub mod kyber;
pub mod pedersen;
pub mod random_walk_privacy;
pub mod rng;
pub mod sha3;
pub mod signatures;
pub mod sphincs;

// A simple in-memory key store for development purposes
// In production, this would be replaced with secure storage (HSM, TEE, etc.)
lazy_static::lazy_static! {
    static ref PRIVATE_KEY_STORE: Mutex<HashMap<String, Vec<u8>>> = Mutex::new(HashMap::new());
}

/// Stores a private key securely
///
/// In development, stores the key in memory. In production, this would use
/// hardware security modules or trusted execution environments.
///
/// # Arguments
///
/// * `id` - Identifier for the key
/// * `private_key` - The private key data to store
///
/// # Returns
///
/// Result indicating success or storage error
pub fn store_private_key(
    id: &str,
    private_key: &[u8],
) -> std::result::Result<(), StorageNodeError> {
    debug!("Storing private key for ID: {}", id);
    let mut store = PRIVATE_KEY_STORE.lock().map_err(|_| {
        StorageNodeError::internal(
            "Failed to acquire lock for private key store",
            None::<std::io::Error>,
        )
    })?;

    store.insert(id.to_string(), private_key.to_vec());
    Ok(())
}

/// Retrieves a private key
/// In a production environment, this would use a TEE, HSM, or secured storage
pub fn get_private_key(id: &str) -> std::result::Result<Vec<u8>, StorageNodeError> {
    let store = PRIVATE_KEY_STORE.lock().map_err(|_| {
        StorageNodeError::internal(
            "Failed to acquire lock for private key store",
            None::<std::io::Error>,
        )
    })?;

    match store.get(id) {
        Some(key) => Ok(key.clone()),
        None => {
            warn!("Private key not found for ID: {}", id);
            Err(StorageNodeError::not_found(
                format!("Private key not found for ID: {id}"),
                None::<std::io::Error>,
            ))
        }
    }
}

/// Initialize cryptography subsystem
/// 
/// # Returns
/// * `Ok(())` - If initialization was successful
/// * `Err(StorageNodeError)` - If initialization failed
pub fn init_crypto() -> Result<(), crate::error::StorageNodeError> {
    // Initialize the post-quantum cryptographic subsystem
    let _ = kyber::init_kyber();
    sphincs::init_sphincs()?; // Use independent storage node SPHINCS+ implementation
    rng::ensure_rng_initialization();
    debug!("Cryptography subsystem initialized");
    Ok(())
}

pub fn generate_keypair() -> (Vec<u8>, Vec<u8>, Vec<u8>, Vec<u8>) {
    // Generate Kyber key pair for encryption
    let (kyber_public, kyber_secret) = kyber::generate_kyber_keypair().unwrap();
    // Generate SPHINCS+ key pair for signatures using independent storage node implementation
    let (sphincs_public, sphincs_secret) = sphincs::generate_sphincs_keypair().unwrap();

    (kyber_public, kyber_secret, sphincs_public, sphincs_secret)
}

/// Verify a signature using SPHINCS+
///
/// # Parameters
///
/// - `data`: The data that was signed.
/// - `signature`: The signature to verify.
/// - `public_key`: The public key corresponding to the private key that was used to sign the data.
///
/// # Returns
///
/// `true` if the signature is valid, `false` otherwise.
///
/// Hash data using Blake3
///
/// # Parameters
/// - `data`: The data to hash.
///
/// # Returns
/// A `Vec<u8>` containing the hash of the data.
/// Sign data using SPHINCS+
pub fn sign_data(data: &[u8], private_key: &[u8]) -> Option<Vec<u8>> {
    sphincs::sphincs_sign(private_key, data).ok()
}

/// Verify signature using SPHINCS+
pub fn verify_signature(data: &[u8], signature: &[u8], public_key: &[u8]) -> bool {
    sphincs::sphincs_verify(public_key, data, signature).unwrap_or(false)
}

/// Fully implemented encryption for recipient using Kyber encapsulation and ChaCha20Poly1305 AEAD
pub fn encrypt_for_recipient(recipient_pk: &[u8], message: &[u8]) -> Option<Vec<u8>> {
    // Encapsulate a symmetric key using the recipient's public key.
    // This function is assumed to return a tuple of (symmetric_key, encapsulated_key)
    let encapsulation_result = match kyber::kyber_encapsulate(recipient_pk) {
        Ok((symmetric_key, encapsulated_key)) => (symmetric_key, encapsulated_key),
        Err(_) => return None,
    };

    let (symmetric_key, encapsulated_key) = encapsulation_result;

    // Ensure the symmetric key is 32 bytes.
    let key_bytes = if symmetric_key.len() >= 32 {
        &symmetric_key[..32]
    } else {
        // If the provided key is shorter, derive 32 deterministic bytes from it.
        &deterministic_random_bytes(&symmetric_key, 32)
    };

    // Initialize ChaCha20Poly1305 with the symmetric key.
    let cipher = ChaCha20Poly1305::new(Key::from_slice(key_bytes));

    // Generate a 12-byte nonce.
    let nonce_bytes = generate_nonce();
    let nonce = Nonce::from_slice(&nonce_bytes);

    // Encrypt the message.
    let ciphertext = cipher.encrypt(nonce, message).ok()?;

    // Package the output:
    // [2 bytes for encapsulated_key length][encapsulated_key][nonce (12 bytes)][ciphertext]
    let mut output = Vec::new();
    let encapsulated_len = encapsulated_key.len() as u16;
    output.extend_from_slice(&encapsulated_len.to_be_bytes());
    output.extend_from_slice(&encapsulated_key);
    output.extend_from_slice(&nonce_bytes);
    output.extend_from_slice(&ciphertext);

    Some(output)
}

pub fn derive_key(context: &str, key_material: &[u8]) -> [u8; 32] {
    let mut hasher = ::blake3::Hasher::new();
    hasher.update(context.as_bytes());
    hasher.update(key_material);
    let mut output = [0u8; 32];
    output.copy_from_slice(hasher.finalize().as_bytes());
    output
}

pub fn decrypt_from_sender(recipient_sk: &[u8], encrypted_data: &[u8]) -> Option<Vec<u8>> {
    // Ensure there are at least 2 bytes for the encapsulated_key length.
    if encrypted_data.len() < 2 {
        return None;
    }

    // Read the encapsulated_key length (first 2 bytes).
    let encapsulated_len = u16::from_be_bytes([encrypted_data[0], encrypted_data[1]]) as usize;
    let offset = 2;

    // Check there is enough data for encapsulated_key, nonce, and ciphertext.
    if encrypted_data.len() < offset + encapsulated_len + 12 {
        return None;
    }

    // Split the encrypted data.
    let encapsulated_key = &encrypted_data[offset..offset + encapsulated_len];
    let nonce_bytes = &encrypted_data[offset + encapsulated_len..offset + encapsulated_len + 12];
    let ciphertext = &encrypted_data[offset + encapsulated_len + 12..];

    // Decapsulate the symmetric key using the recipient's secret key.
    let symmetric_key = kyber::kyber_decapsulate(recipient_sk, encapsulated_key).ok()?;

    // Ensure the symmetric key is 32-bytes.
    let key_bytes = if symmetric_key.len() >= 32 {
        &symmetric_key[..32]
    } else {
        // If the provided key is shorter, derive 32 deterministic bytes from it.
        &deterministic_random_bytes(&symmetric_key, 32)
    };

    // Initialize ChaCha20Poly1305 with the symmetric key.
    let cipher = ChaCha20Poly1305::new(Key::from_slice(key_bytes));
    let nonce = Nonce::from_slice(nonce_bytes);

    // Decrypt the ciphertext.
    let plaintext = cipher.decrypt(nonce, ciphertext).ok()?;
    Some(plaintext)
}

pub fn deterministic_random_bytes(seed: &[u8], length: usize) -> Vec<u8> {
    // Example implementation using Blake3 for deterministic randomness
    let mut out = Vec::new();
    let mut hasher = ::blake3::Hasher::new();
    hasher.update(seed);
    let mut hash = hasher.finalize();
    while out.len() < length {
        out.extend_from_slice(hash.as_bytes());
        hasher = ::blake3::Hasher::new();
        hasher.update(hash.as_bytes());
        hash = hasher.finalize();
    }
    out.truncate(length);
    out
}

pub fn generate_nonce() -> Vec<u8> {
    rng::random_bytes(12)
}

// Replace hash_data function to use blake3 directly instead of a separate hash module
pub fn hash_data(data: &[u8]) -> Vec<u8> {
    let mut hasher = ::blake3::Hasher::new();
    hasher.update(data);
    hasher.finalize().as_bytes().to_vec()
}

#[cfg(test)]
mod tests {
    use super::*;

    static TEST_MESSAGE: &[u8] = b"This is a test message for cryptographic operations";

    #[test]
    fn test_derive_key_length() {
        let context = "test-context";
        let key_material = b"some key material for testing";
        let key = derive_key(context, key_material);
        assert_eq!(key.len(), 32);
    }

    #[test]
    fn test_sign_and_verify() {
        // Generate dummy keypairs from the available crypto functions.
        // generate_keypair returns (kyber_pk, kyber_sk, sphincs_pk, sphincs_sk)
        let (_kyber_pub, _kyber_sec, sphincs_public, sphincs_secret) = generate_keypair();

        // Attempt to sign a test message using SPHINCS+
        if let Some(signature) = sign_data(TEST_MESSAGE, &sphincs_secret) {
            // The signature should verify with the corresponding public key.
            let valid = verify_signature(TEST_MESSAGE, &signature, &sphincs_public);
            assert!(valid, "Signature verification failed");
        } else {
            panic!("Failed to sign the test message");
        }
    }

    #[test]
    fn test_encrypt_and_decrypt() {
        // Generate dummy keypair (using kyber keys)
        // We use generate_keypair; first two elements are kyber keys.
        let (kyber_public, kyber_secret, _sphincs_pub, _sphincs_sec) = generate_keypair();

        // Encrypt a test message for the recipient
        if let Some(encrypted) = encrypt_for_recipient(&kyber_public, TEST_MESSAGE) {
            // Decrypt the ciphertext using the recipient's secret key
            if let Some(decrypted) = decrypt_from_sender(&kyber_secret, &encrypted) {
                assert_eq!(decrypted, TEST_MESSAGE);
            } else {
                panic!("Decryption failed");
            }
        } else {
            panic!("Encryption failed");
        }
    }
}
