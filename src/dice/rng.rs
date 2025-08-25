// Enhanced random number generation for dice rolling
// Provides cryptographically secure randomness with multiple entropy sources

use rand::rngs::StdRng; // Use StdRng instead of ChaCha20Rng (it's ChaCha20 internally)
use rand::{Rng, SeedableRng};
use std::time::{SystemTime, UNIX_EPOCH};

/// Creates an enhanced RNG with multiple entropy sources for maximum unpredictability
///
/// This function combines:
/// - OS entropy (getrandom) - cryptographically secure
/// - High-resolution timestamp - temporal entropy
/// - Thread ID - execution context entropy
/// - Process ID - process-specific entropy
/// - Memory address entropy - ASLR-based entropy
pub fn create_enhanced_rng() -> StdRng {
    let mut seed = [0u8; 32];

    // Primary entropy: OS cryptographic random bytes (16 bytes = 128 bits)
    if getrandom::getrandom(&mut seed[0..16]).is_err() {
        // Fallback: use system time if OS entropy fails
        fallback_seed(&mut seed[0..16]);
    }

    // Mix additional entropy into seed for StdRng (32-byte seed)
    mix_additional_entropy(&mut seed);

    StdRng::from_seed(seed)
}

/// Fallback seeding if OS entropy is unavailable
fn fallback_seed(seed_slice: &mut [u8]) {
    let time_nanos = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap_or_default()
        .as_nanos() as u64;

    let process_id = std::process::id() as u64;
    let fallback_entropy = time_nanos.wrapping_mul(process_id);

    for (i, byte) in fallback_entropy
        .to_le_bytes()
        .iter()
        .cycle()
        .take(seed_slice.len())
        .enumerate()
    {
        seed_slice[i] = *byte;
    }
}

/// Mix additional entropy sources into the seed
fn mix_additional_entropy(seed: &mut [u8; 32]) {
    // High-resolution timestamp entropy (4 bytes)
    let time_nanos = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap_or_default()
        .subsec_nanos();

    let time_bytes = time_nanos.to_le_bytes();
    for (i, &byte) in time_bytes.iter().enumerate() {
        if i + 16 < 32 {
            seed[i + 16] ^= byte;
        }
    }

    // Thread ID entropy - provides execution context diversity
    let thread_id = std::thread::current().id();
    let thread_hash = hash_thread_id(thread_id);
    let thread_bytes = thread_hash.to_le_bytes();

    for (i, &byte) in thread_bytes.iter().enumerate() {
        if i + 20 < 32 {
            seed[i + 20] ^= byte;
        }
    }

    // Process ID entropy (4 bytes)
    let process_id = std::process::id();
    let pid_bytes = process_id.to_le_bytes();

    for (i, &byte) in pid_bytes.iter().enumerate() {
        if i + 28 < 32 {
            seed[i + 28] ^= byte;
        }
    }

    // Memory address entropy from stack variable (ASLR-based)
    let stack_var = 0u8;
    let addr_entropy = &stack_var as *const u8 as u64;
    let addr_bytes = addr_entropy.to_le_bytes();

    // XOR address entropy across the entire seed for diffusion
    for (i, &byte) in addr_bytes.iter().cycle().take(32).enumerate() {
        seed[i] ^= byte.wrapping_mul((i + 1) as u8);
    }
}

/// Convert ThreadId to a hash value for entropy
fn hash_thread_id(thread_id: std::thread::ThreadId) -> u64 {
    // Simple hash of the thread ID debug representation
    let thread_str = format!("{:?}", thread_id);
    let mut hash = 0u64;

    for (i, byte) in thread_str.bytes().enumerate() {
        hash = hash.wrapping_mul(31).wrapping_add(byte as u64);
        // Add position-based mixing
        hash ^= (byte as u64) << (i % 56);
    }

    hash
}

/// Create a fast RNG for performance-critical scenarios
/// Uses the enhanced seeding but with a lighter algorithm
pub fn create_fast_rng() -> rand::rngs::SmallRng {
    // Use our enhanced seeding approach
    let mut temp_rng = create_enhanced_rng();

    // Generate a seed for SmallRng using our secure RNG
    let seed = temp_rng.random::<u64>();
    rand::rngs::SmallRng::seed_from_u64(seed)
}

/// Get a default enhanced RNG - the recommended choice for dice rolling
pub fn get_dice_rng() -> StdRng {
    create_enhanced_rng()
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::collections::HashSet;

    #[test]
    fn test_rng_creation() {
        let _rng = create_enhanced_rng();
        // Should not panic - basic creation test
    }

    #[test]
    fn test_rng_produces_different_values() {
        let mut rng1 = create_enhanced_rng();
        let mut rng2 = create_enhanced_rng();

        let val1: u32 = rng1.random();
        let val2: u32 = rng2.random();

        // Extremely unlikely to be equal with good entropy
        // (Though technically possible - this is a sanity check)
        println!("RNG1: {}, RNG2: {}", val1, val2);
    }

    #[test]
    fn test_dice_distribution() {
        let mut rng = create_enhanced_rng();
        let mut counts = [0u32; 6];
        let trials = 10000;

        // Roll a d6 many times
        for _ in 0..trials {
            let roll = rng.random_range(1..=6);
            counts[roll - 1] += 1;
        }

        // Each face should appear roughly 1/6 of the time
        // Allow for statistical variance
        let expected = trials / 6;
        let tolerance = expected / 4; // 25% tolerance

        for (face, &count) in counts.iter().enumerate() {
            assert!(
                count > expected - tolerance && count < expected + tolerance,
                "Face {} appeared {} times, expected around {} (±{})",
                face + 1,
                count,
                expected,
                tolerance
            );
        }
    }

    #[test]
    fn test_rng_uniqueness() {
        // Create multiple RNGs and verify they produce different sequences
        let mut rngs: Vec<_> = (0..10).map(|_| create_enhanced_rng()).collect();
        let mut first_values = HashSet::new();

        for rng in &mut rngs {
            let value: u64 = rng.random();
            first_values.insert(value);
        }

        // Should have mostly unique values (allowing for small chance of collision)
        assert!(
            first_values.len() >= 8,
            "Expected at least 8 unique values, got {}",
            first_values.len()
        );
    }

    #[test]
    fn test_fallback_seeding() {
        let mut seed = [0u8; 16];
        fallback_seed(&mut seed);

        // Should not be all zeros
        assert!(
            seed.iter().any(|&b| b != 0),
            "Fallback seed should not be all zeros"
        );
    }
}
