// (REMOVED) DBRW (Dual-Binding Random Walk) for Hardware Entropy Extraction
// This module is not used by the storage node and should not be referenced.
// 
// This module implements the RandomWalkMemoryInterrogation algorithm to extract
/// device-specific entropy from memory timing characteristics. This hardware-derived
/// entropy is then integrated into the MPC (Multi-Party Computation) process
/// to bind device identity directly to hardware characteristics.
/// 
/// The algorithm performs a deterministic random walk through memory, measuring
/// access timing characteristics that are influenced by hardware variations,
/// even among devices of identical make and model.

use blake3::{Hash, Hasher};
use rand::RngCore;
use rand_core::OsRng;
use std::time::{Duration, Instant};
use tracing::{debug, error, warn};

const MEMORY_BLOCK_SIZE: usize = 1024 * 1024; // 1MB memory block
const DEFAULT_WALK_LENGTH: usize = 64; // Number of memory locations to interrogate
const DEFAULT_MEASUREMENT_ATTEMPTS: usize = 5; // Multiple measurements for stability
const WARMUP_ITERATIONS: usize = 10; // CPU/memory warmup iterations
const TIMING_QUANTIZATION_BITS: usize = 4; // Quantize timing to 4-bit resolution

/// RandomWalkMemoryInterrogation extracts hardware-specific entropy
/// through memory timing measurements using a deterministic random walk pattern
pub struct RandomWalkMemoryInterrogation {
    memory_block: Vec<u8>,
    walk_length: usize,
    measurement_attempts: usize,
    seed: [u8; 32],
}

impl RandomWalkMemoryInterrogation {
    /// Create a new memory interrogation instance with a deterministic seed
    /// 
    /// # Parameters
    /// * `seed` - Deterministic seed for generating walk addresses
    pub fn new(seed: [u8; 32]) -> Self {
        let mut memory_block = vec![0u8; MEMORY_BLOCK_SIZE];
        
        // Initialize memory block with deterministic pattern for consistent measurements
        for (i, byte) in memory_block.iter_mut().enumerate() {
            *byte = ((i ^ (i >> 8) ^ (i >> 16)) & 0xFF) as u8;
        }
        
        Self {
            memory_block,
            walk_length: DEFAULT_WALK_LENGTH,
            measurement_attempts: DEFAULT_MEASUREMENT_ATTEMPTS,
            seed,
        }
    }

    /// Create a new instance with custom parameters
    pub fn new_with_params(seed: [u8; 32], walk_length: usize, measurement_attempts: usize) -> Self {
        let mut instance = Self::new(seed);
        instance.walk_length = walk_length;
        instance.measurement_attempts = measurement_attempts;
        instance
    }

    /// Generate deterministic walk addresses using Blake3 hash chain
    fn generate_walk_addresses(&self) -> Vec<usize> {
        let mut addresses = Vec::with_capacity(self.walk_length);
        let mut current_hash = self.seed;

        for _ in 0..self.walk_length {
            let mut hasher = Hasher::new();
            hasher.update(&current_hash);
            let result = hasher.finalize();
            let bytes = result.as_bytes();

            // Extract address from hash, ensuring it's within memory block bounds
            let raw_address = u64::from_le_bytes(
                bytes[..8].try_into().unwrap_or([0u8; 8])
            );
            let address = (raw_address as usize) % (MEMORY_BLOCK_SIZE - 64); // Leave safety margin
            addresses.push(address);

            // Update hash for next iteration
            current_hash = *bytes;
        }

        addresses
    }

    /// Perform memory access timing measurements
    fn measure_memory_access(&mut self, addresses: &[usize]) -> Vec<u64> {
        let mut timings = Vec::with_capacity(addresses.len());

        // Warmup phase to stabilize CPU/memory state
        for _ in 0..WARMUP_ITERATIONS {
            for &address in addresses {
                // Touch memory locations to warm up caches
                self.memory_block[address] = self.memory_block[address].wrapping_add(1);
            }
        }

        // Perform actual timing measurements
        for &address in addresses {
            // Flush CPU cache by accessing distant memory locations
            for i in 0..16 {
                let flush_addr = (address + i * 4096) % MEMORY_BLOCK_SIZE;
                self.memory_block[flush_addr] = (i & 0xFF) as u8;
            }

            // Memory barrier - force completion of previous operations
            std::sync::atomic::fence(std::sync::atomic::Ordering::SeqCst);

            // Measure access time
            let start_time = Instant::now();
            
            // Perform memory access
            let _value = self.memory_block[address];
            
            // Force memory access completion
            std::sync::atomic::fence(std::sync::atomic::Ordering::SeqCst);
            
            let end_time = Instant::now();
            let timing = end_time.duration_since(start_time).as_nanos() as u64;
            
            timings.push(timing);
        }

        timings
    }

    /// Process multiple timing measurements into a stable hardware fingerprint
    fn process_measurements(&self, all_measurements: Vec<Vec<u64>>) -> Vec<u8> {
        if all_measurements.is_empty() {
            error!("No timing measurements available for processing");
            return vec![0u8; 32];
        }

        let mut processed_timings = vec![0u64; self.walk_length];

        // Calculate median timing for each address position
        for addr_idx in 0..self.walk_length {
            let mut timings_at_address: Vec<u64> = all_measurements
                .iter()
                .filter_map(|measurement| measurement.get(addr_idx).copied())
                .collect();

            if timings_at_address.is_empty() {
                warn!("No timing measurements for address index {}", addr_idx);
                continue;
            }

            timings_at_address.sort_unstable();
            let median_timing = timings_at_address[timings_at_address.len() / 2];
            
            // Quantize timing to reduce noise and improve stability
            processed_timings[addr_idx] = quantize_timing(median_timing);
        }

        // Convert processed timings to fingerprint using Blake3
        let mut hasher = Hasher::new();
        hasher.update(b"DBRW_HARDWARE_FINGERPRINT");
        hasher.update(&self.seed);
        
        for timing in processed_timings {
            hasher.update(&timing.to_le_bytes());
        }

        hasher.finalize().as_bytes().to_vec()
    }

    /// Perform the complete random walk memory interrogation
    pub fn perform_interrogation(&mut self) -> Result<Vec<u8>, Box<dyn std::error::Error>> {
        debug!("Starting DBRW memory interrogation with {} addresses", self.walk_length);

        let addresses = self.generate_walk_addresses();
        let mut all_measurements = Vec::with_capacity(self.measurement_attempts);

        for attempt in 0..self.measurement_attempts {
            debug!("Performing measurement attempt {}/{}", attempt + 1, self.measurement_attempts);
            
            let timings = self.measure_memory_access(&addresses);
            
            if timings.len() == self.walk_length {
                all_measurements.push(timings);
            } else {
                warn!("Measurement attempt {} yielded incomplete timing data", attempt + 1);
            }
        }

        if all_measurements.is_empty() {
            return Err("Failed to obtain any valid timing measurements".into());
        }

        let hardware_fingerprint = self.process_measurements(all_measurements);
        
        debug!("DBRW interrogation completed, generated {}-byte fingerprint", 
               hardware_fingerprint.len());

        Ok(hardware_fingerprint)
    }
}

/// Extract environment fingerprint for dual-binding
pub fn extract_environment_fingerprint() -> Vec<u8> {
    let mut hasher = Hasher::new();
    hasher.update(b"DBRW_ENVIRONMENT_FINGERPRINT");

    // Process ID - unique per application instance
    hasher.update(&std::process::id().to_le_bytes());

    // Current executable path
    if let Ok(exe_path) = std::env::current_exe() {
        hasher.update(exe_path.to_string_lossy().as_bytes());
    }

    // Current working directory
    if let Ok(cwd) = std::env::current_dir() {
        hasher.update(cwd.to_string_lossy().as_bytes());
    }

    // Environment variables that indicate execution context
    let env_vars = ["HOSTNAME", "USER", "HOME", "PATH", "PWD"];
    for var in &env_vars {
        if let Ok(value) = std::env::var(var) {
            hasher.update(var.as_bytes());
            hasher.update(value.as_bytes());
        }
    }

    // System time as additional environment context
    if let Ok(system_time) = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH) 
    {
        // Quantize to hour resolution to avoid temporal drift issues
        let hour_timestamp = system_time.as_secs() / 3600;
        hasher.update(&hour_timestamp.to_le_bytes());
    }

    hasher.finalize().as_bytes().to_vec()
}

/// Quantize timing measurement to reduce noise
fn quantize_timing(timing: u64) -> u64 {
    // Quantize to reduce noise while preserving hardware-specific characteristics
    let quantization_factor = 1u64 << TIMING_QUANTIZATION_BITS;
    (timing / quantization_factor) * quantization_factor
}

/// Generate hardware entropy for MPC integration
pub fn generate_dbrw_mpc_entropy() -> Result<Vec<u8>, Box<dyn std::error::Error>> {
    // Generate deterministic seed for memory interrogation
    let mut seed = [0u8; 32];
    
    // Use a deterministic seed based on system characteristics
    let mut hasher = Hasher::new();
    hasher.update(b"DBRW_MPC_SEED");
    
    // Include system characteristics in seed generation
    if let Ok(hostname) = std::env::var("HOSTNAME") {
        hasher.update(hostname.as_bytes());
    }
    
    // Use processor information if available
    #[cfg(target_arch = "x86_64")]
    {
        hasher.update(b"x86_64");
    }
    #[cfg(target_arch = "aarch64")]
    {
        hasher.update(b"aarch64");
    }
    
    seed.copy_from_slice(hasher.finalize().as_bytes());

    // Perform hardware entropy extraction
    let mut interrogation = RandomWalkMemoryInterrogation::new(seed);
    let hardware_entropy = interrogation.perform_interrogation()?;

    // Extract environment fingerprint
    let environment_entropy = extract_environment_fingerprint();

    // Combine hardware and environment entropy using dual-binding
    let mut dual_binding_hasher = Hasher::new();
    dual_binding_hasher.update(b"DBRW_DUAL_BINDING");
    dual_binding_hasher.update(&hardware_entropy);
    dual_binding_hasher.update(&environment_entropy);

    let dual_bound_entropy = dual_binding_hasher.finalize().as_bytes().to_vec();
    
    debug!("Generated DBRW dual-bound entropy: {} bytes", dual_bound_entropy.len());
    
    Ok(dual_bound_entropy)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_deterministic_walk_addresses() {
        let seed = [42u8; 32];
        let interrogation1 = RandomWalkMemoryInterrogation::new(seed);
        let interrogation2 = RandomWalkMemoryInterrogation::new(seed);

        let addresses1 = interrogation1.generate_walk_addresses();
        let addresses2 = interrogation2.generate_walk_addresses();

        assert_eq!(addresses1, addresses2, "Walk addresses should be deterministic");
    }

    #[test]
    fn test_different_seeds_produce_different_addresses() {
        let seed1 = [1u8; 32];
        let seed2 = [2u8; 32];
        
        let interrogation1 = RandomWalkMemoryInterrogation::new(seed1);
        let interrogation2 = RandomWalkMemoryInterrogation::new(seed2);

        let addresses1 = interrogation1.generate_walk_addresses();
        let addresses2 = interrogation2.generate_walk_addresses();

        assert_ne!(addresses1, addresses2, "Different seeds should produce different addresses");
    }

    #[test]
    fn test_timing_quantization() {
        assert_eq!(quantize_timing(100), 96); // 100 quantized to 4-bit resolution
        assert_eq!(quantize_timing(200), 192);
    }

    #[test]
    fn test_environment_fingerprint_generation() {
        let fingerprint1 = extract_environment_fingerprint();
        let fingerprint2 = extract_environment_fingerprint();
        
        // Should be consistent within the same environment
        assert_eq!(fingerprint1, fingerprint2);
        assert!(!fingerprint1.is_empty());
    }
}
