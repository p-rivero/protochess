use crate::types::{Move, Depth, Centipawns, ZobKey};

#[derive(Clone, Copy, PartialEq, Eq, Debug)]
#[must_use]
pub enum EntryFlag{
    Exact = 0,
    Alpha = 1,
    Beta = 2,
    Null = 3,
}
impl EntryFlag {
    #[inline]
    pub fn equal_or_better_than(self, other: EntryFlag) -> bool {
        // Values are ordered so that EXACT < ALPHA <= BETA < NULL (ALPHA and BETA are worth the same)
        (self as u8) <= (other as u8) || (self == EntryFlag::Beta && other == EntryFlag::Alpha)
    }
}

/// A single entry in the transposition table, containing the zobrist key of the position,
/// some value (exact score, alpha or beta), the corresponding move, and the depth at which
/// the value was computed.
/// 
/// **WARNING**: If you add fields to this struct, make sure to 1) update the padding field
/// so that no bits are left uninitialized, and 2) update the get_hash_mask() function to XOR all the bits
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
#[must_use]
pub struct Entry {
    pub key: ZobKey,
    pub flag: EntryFlag,
    pub value: Centipawns,
    pub mv: Move,
    pub depth: Depth,
    // Make sure that all bits of the struct are used and initialized to 0,
    // otherwise the hash mask would contain uninitialized data
    padding: u16,
}
impl Entry {
    pub fn null() -> Entry {
        Entry::new(0, EntryFlag::Null, 0, Move::null(), 0)
    }
    
    #[inline]
    pub fn new(key: ZobKey, flag: EntryFlag, value: Centipawns, mv: Move, depth: Depth) -> Entry {
        Entry { key, flag, value, mv, depth, padding: 0 }
    }
    
    #[inline]
    pub fn equal_or_better_than(&self, other: &Entry) -> bool {
        if self.depth == other.depth {
            // If the depth is the same, the entry with the most useful value is better
            self.flag.equal_or_better_than(other.flag)
        } else {
            // A deeper entry is always better: if depths are different, prefer higher depth
            self.depth > other.depth
        }
    }
    
    /// Get the original key, before it was masked by the hash mask
    /// See https://craftychess.com/hyatt/hashing.html
    #[inline]
    pub fn original_key(&self) -> ZobKey {
        self.get_hash_mask()
    }
    /// Mask the key with the hash mask, so that it can be stored in memory
    /// See https://craftychess.com/hyatt/hashing.html
    #[inline]
    pub fn mask_key(&mut self) {
        self.key = self.get_hash_mask();
    }
    
    /// In a lockless transposition table, XOR the key with the value to get a hash.
    /// In case of 2 threads writing at the same time, the entry will become invalid (instead
    /// of being incorrectly detected as valid and reading the wrong data).
    /// See https://craftychess.com/hyatt/hashing.html
    #[inline]
    fn get_hash_mask(&self) -> ZobKey {
        // Unsafely cast the struct to a [u64; 3] to get the raw bytes
        let data: &[u64; 3] = unsafe { std::mem::transmute(self) };
        // XOR all the bytes
        data[0] ^ data[1] ^ data[2]
    }
}
