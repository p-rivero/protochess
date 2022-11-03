use crate::types::{Move, Depth, Centipawns};

#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum EntryFlag{
    EXACT = 0,
    ALPHA = 1,
    BETA = 2,
    NULL = 3,
}
impl EntryFlag {
    #[inline(always)]
    pub fn equal_or_better_than(self, other: EntryFlag) -> bool {
        // Values are ordered so that EXACT < ALPHA <= BETA < NULL (ALPHA and BETA are worth the same)
        (self as u8) <= (other as u8) || (self == EntryFlag::BETA && other == EntryFlag::ALPHA)
    }
}

#[derive(Clone, Copy, PartialEq, Debug)]
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
    #[inline(always)]
    pub fn equal_or_better_than(&self, other: &Entry) -> bool {
        if self.depth != other.depth {
            // A deeper entry is always better: if depths are different, prefer higher depth
            self.depth > other.depth
        } else {
            // If the depth is the same, the entry with the most useful value is better
            self.flag.equal_or_better_than(other.flag)
        }
    }
}
