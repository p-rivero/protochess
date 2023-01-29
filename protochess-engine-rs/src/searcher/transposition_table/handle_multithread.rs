use std::sync::Arc;

use crate::types::ZobKey;

use super::{TranspositionTable, Entry};

/// Wrapper around a pointer to a TranspositionTable, which is used to pass the table to the threads.
/// It's responsible for implementing the ugly unsafe code to access the table.
pub struct TranspositionHandle {
    ptr: *mut TranspositionTable
}

impl TranspositionHandle {
    #[inline]
    pub fn insert(&mut self, entry: Entry) {
        unsafe {
            (*self.ptr).insert(entry);
        }
    }
    #[inline]
    pub fn retrieve(&self, key: ZobKey) -> Option<&Entry> {
        unsafe {
            (*self.ptr).retrieve(key)
        }
    }
}


impl From<Arc<TranspositionTable>> for TranspositionHandle {
    fn from(arc: Arc<TranspositionTable>) -> Self {
        let ptr = Arc::into_raw(arc);
        let ptr = ptr as *mut TranspositionTable;
        TranspositionHandle { ptr }
    }
}

// Delegate Drop to Arc
impl Drop for TranspositionHandle {
    fn drop(&mut self) {
        unsafe {
            let arc = Arc::from_raw(self.ptr);
            drop(arc);
        }
    }
}

impl std::fmt::Debug for TranspositionHandle {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_fmt(format_args!("TranspositionHandle({:?}): {:?}", self.ptr, unsafe { &*self.ptr }))
    }
}

// Delegate Clone to Arc
impl Clone for TranspositionHandle {
    fn clone(&self) -> Self {
        let arc = unsafe { Arc::from_raw(self.ptr) };
        arc.clone().into()
    }
}
