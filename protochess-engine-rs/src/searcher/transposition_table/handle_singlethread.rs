use crate::types::ZobKey;

use super::{TranspositionTable, Entry};

// In single-threaded builds, the transposition table is stored directly in the Searcher struct.

#[derive(Debug, Clone)]
pub struct TranspositionHandle {
    table: TranspositionTable
}

impl TranspositionHandle {
    #[inline]
    pub fn insert(&mut self, key: ZobKey, entry: Entry) {
        self.table.insert(key, entry);
    }
    #[inline]
    pub fn retrieve(&self, key: ZobKey) -> Option<&Entry> {
        self.table.retrieve(key)
    }
}

impl From<TranspositionTable> for TranspositionHandle {
    fn from(table: TranspositionTable) -> Self {
        TranspositionHandle { table }
    }
}
