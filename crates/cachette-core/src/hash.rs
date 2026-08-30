//! The state hash.
//!
//! The engine hashes its whole state each frame and compares the result
//! against a stored file. The padding rule stops that test from failing
//! falsely.[^1]
//!
//! The function is FNV-1a over the little-endian bytes of the state. It is
//! not a cryptographic hash. It detects a changed byte, which is all the
//! golden test needs.
//!
//! # References
//!
//! [^1]: ADR-0001, Determinism as the primary constraint, decisions D9 and D11. `docs/adrs/draft/adr-0001-determinism.md`

/// The offset basis of the 64-bit FNV-1a hash.
const OFFSET_BASIS: u64 = 0xcbf2_9ce4_8422_2325;
/// The prime of the 64-bit FNV-1a hash.
const PRIME: u64 = 0x0000_0100_0000_01b3;

/// A running hash of the simulation state.
#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct StateHash(u64);

impl Default for StateHash {
    fn default() -> Self {
        Self::new()
    }
}

impl StateHash {
    /// Builds an empty hash.
    #[must_use]
    pub const fn new() -> Self {
        Self(OFFSET_BASIS)
    }

    /// Absorbs a slice of bytes.
    #[must_use]
    pub const fn write(mut self, bytes: &[u8]) -> Self {
        let mut index = 0;
        while index < bytes.len() {
            self.0 ^= bytes[index] as u64;
            self.0 = self.0.wrapping_mul(PRIME);
            index += 1;
        }
        self
    }

    /// Absorbs one 64-bit integer in little-endian order.
    #[must_use]
    pub const fn write_u64(self, value: u64) -> Self {
        self.write(&value.to_le_bytes())
    }

    /// Returns the hash value.
    #[must_use]
    pub const fn finish(self) -> u64 {
        self.0
    }
}

impl core::fmt::Display for StateHash {
    fn fmt(&self, formatter: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        write!(formatter, "{:016x}", self.0)
    }
}
