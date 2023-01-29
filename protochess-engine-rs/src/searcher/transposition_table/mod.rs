use crate::types::Depth;
use crate::types::ZobKey;

pub mod entry;
pub use self::entry::Entry;
pub use self::entry::EntryFlag;

#[cfg(feature = "parallel")]
mod handle_multithread;
#[cfg(feature = "parallel")]
pub use handle_multithread::TranspositionHandle;

#[cfg(not(feature = "parallel"))]
mod handle_singlethread;
#[cfg(not(feature = "parallel"))]
pub use handle_singlethread::TranspositionHandle;


// Since we will be computing zobrist_key % TABLE_SIZE, we want it to be a power of 2
// 2^21 clusters is about 2 million clusters. Each cluster is 4*24 = 96 bytes, so this is about 192 MB
const TABLE_SIZE: usize = 2_usize.pow(21);
const ENTRIES_PER_CLUSTER: usize = 4;


#[derive(Debug, Clone, Copy)]
pub struct Cluster {
    entries: [Entry; ENTRIES_PER_CLUSTER]
}

#[derive(Debug, Clone)]
pub struct TranspositionTable {
    data: Vec<Cluster>
}

impl TranspositionTable {
    pub fn new() -> TranspositionTable {
        let mut data = Vec::with_capacity(TABLE_SIZE);
        for _ in 0..TABLE_SIZE {
            data.push(Cluster { entries: [Entry::null(); ENTRIES_PER_CLUSTER] });
        }
        TranspositionTable {
            data
        }
    }

    /// Inserts a new Entry item into the transposition table
    pub fn insert(&mut self, mut entry: Entry) {
        // T = W1(p) & MASK;
        // W1(T) = W1(p) ^ W2(p)
        // W2(T) = W2(p)
        // https://craftychess.com/hyatt/hashing.html
        let original_key = entry.key;
        entry.mask_key();
        
        let cluster = &mut self.data[original_key as usize % TABLE_SIZE];
        // As a first option, replace the first exact match for this key that has lower depth
        for i in 0..ENTRIES_PER_CLUSTER {
            let table_entry = cluster.entries[i];
            if table_entry.original_key() == original_key && table_entry.flag != EntryFlag::Null {
                // Exact match, replace it only if the new entry is better
                if entry.equal_or_better_than(&table_entry) {
                    cluster.entries[i] = entry;
                }
                return;
            }
        }

        // No exact match found, we need to replace an entry for a different position
        // Replace the entry with the lowest depth
        let mut lowest_depth = Depth::MAX;
        let mut lowest_depth_index = 0;
        for i in 0..ENTRIES_PER_CLUSTER {
            if cluster.entries[i].depth <= lowest_depth {
                lowest_depth = cluster.entries[i].depth;
                lowest_depth_index = i;
            }
        }

        if entry.depth >= lowest_depth {
            // Only replace the entry if it's not shallower than all existing entries
            cluster.entries[lowest_depth_index] = entry;
        }
    }

    /// Returns a handle to an Entry in the table, if it exists
    pub fn retrieve(&self, original_key: ZobKey) -> Option<&Entry> {
        let cluster = &self.data[original_key as usize % TABLE_SIZE];
        for i in 0..ENTRIES_PER_CLUSTER {
            let entry = &cluster.entries[i];
            if entry.original_key() == original_key && entry.flag != EntryFlag::Null {
                return Some(entry)
            }
        }
        None
    }
}