use crate::types::{Move, Depth, Centipawns};

#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum EntryFlag{
    ALPHA,
    EXACT,
    BETA,
    NULL,
}

#[derive(Clone, Copy, PartialEq)]
pub struct Entry {
    pub key: u64,
    pub flag: EntryFlag,
    pub value: Centipawns,
    pub mv: Move,
    pub depth: Depth,
    pub ancient: bool
}
impl Entry {
    pub fn null() -> Entry {
        Entry {
            key: 0,
            flag: EntryFlag::NULL,
            value: 0,
            mv: Move::null(),
            depth: 0,
            ancient: true
        }
    }
}
