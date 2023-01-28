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

#[derive(Clone, Copy, PartialEq, Eq, Debug)]
#[must_use]
pub struct Entry {
    pub key: ZobKey,
    pub flag: EntryFlag,
    pub value: Centipawns,
    pub mv: Move,
    pub depth: Depth,
}
impl Entry {
    pub fn null() -> Entry {
        Entry {
            key: 0,
            flag: EntryFlag::Null,
            value: 0,
            mv: Move::null(),
            depth: 0,
        }
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
}
