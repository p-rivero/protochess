use std::sync::Mutex;

use crate::types::Depth;

pub mod entry;
pub use self::entry::Entry;
pub use self::entry::EntryFlag;


// Since we will be computing zobrist_key % TABLE_SIZE, we want it to be a power of 2
// 2^21 entries is about 2 million entries. Each entry is 4*32 = 128 bytes, so this is about 256 MB
const TABLE_SIZE: usize = 2_usize.pow(21);
const ENTRIES_PER_CLUSTER: usize = 4;


pub struct Cluster {
    // Use a separate mutex (instead of wrapping the cluster in a mutex) so that we can implement
    // a more relaxed locking strategy. All threads can read the cluster while another thread is writing,
    // but 2 threads can't write at the same time.
    // In the worst case, a thread will read an old value, but that's fine.
    // TODO: Remove mutex
    mutex: std::sync::Mutex<()>,
    entries: [Entry; ENTRIES_PER_CLUSTER]
}

pub struct TranspositionTable {
    data: Vec<Cluster>
}

impl TranspositionTable {
    pub fn new() -> TranspositionTable {
        let mut data = Vec::with_capacity(TABLE_SIZE);
        for _ in 0..TABLE_SIZE {
            data.push(Cluster { 
                mutex:Mutex::new(()),
                entries: [Entry::null(); ENTRIES_PER_CLUSTER]
            })
        }
        TranspositionTable {
            data
        }
    }

    /// Sets all the entries in the table to ancient, allowing them to be rewritten
    /// before any new entries are rewritten
    pub fn set_ancient(&mut self) {
        for cluster in self.data.iter_mut() {
            for entry in cluster.entries.iter_mut() {
                entry.ancient = true;
            }
        }
    }

    /// Inserts a new Entry item into the transposition table
    pub fn insert(&mut self, zobrist_key:u64, entry: Entry) {
        let cluster = &mut self.data[zobrist_key as usize % TABLE_SIZE];
        // Prevent multiple threads from writing to the same cluster at the same time (don't protect reads)
        let _guard = cluster.mutex.lock();
        // As a first option, replace the first exact match for this key that has lower depth
        for i in 0..ENTRIES_PER_CLUSTER {
            let tentry = cluster.entries[i];
            if tentry.key == zobrist_key && tentry.flag != EntryFlag::NULL {
                // Exact match, replace it only if the new entry is better
                if entry.equal_or_better_than(&tentry) {
                    cluster.entries[i] = entry;
                }
                // Drop _guard and return
                return;
            }
        }

        // No exact match found, we need to replace an entry for a different position
        // Replace the ancient entry with the lowest depth. If there are no ancient entries, replace the entry with the lowest depth
        let mut lowest_depth_and_ancient = Depth::MAX;
        let mut lowest_depth_and_ancient_indx: i32 = -1;

        let mut lowest_depth = Depth::MAX;
        let mut lowest_depth_index = 0;
        for i in 0..ENTRIES_PER_CLUSTER {
            if cluster.entries[i].ancient
                && cluster.entries[i].depth <= lowest_depth_and_ancient {
                lowest_depth_and_ancient = cluster.entries[i].depth;
                lowest_depth_and_ancient_indx = i as i32;
            }

            if cluster.entries[i].depth <= lowest_depth {
                lowest_depth = cluster.entries[i].depth;
                lowest_depth_index = i;
            }
        }

        if lowest_depth_and_ancient_indx != -1 {
            cluster.entries [lowest_depth_and_ancient_indx as usize] = entry;
        } else if entry.depth >= lowest_depth {
            // Only replace the entry if it's not shallower than all existing entries
            cluster.entries[lowest_depth_index] = entry;
        }
        // Drop _guard and return
    }

    /// Returns a handle to an Entry in the table, if it exists
    pub fn retrieve(&mut self, zobrist_key:u64) -> Option<&Entry> {
        let cluster = &self.data[zobrist_key as usize % TABLE_SIZE];
        for entry in &cluster.entries {
            if entry.key == zobrist_key && entry.flag != EntryFlag::NULL {
                return Some(entry)
            }
        }
        None
    }
}