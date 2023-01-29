use crate::types::ZobKey;

use super::{TranspositionTable, Entry};

// In single-threaded builds, the transposition table is stored directly in the Searcher struct.

#[derive(Debug, Clone)]
pub struct TranspositionHandle {
    table: TranspositionTable
}

impl TranspositionHandle {
    #[inline]
    pub fn insert(&mut self, entry: Entry) {
        self.table.insert(entry);
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
