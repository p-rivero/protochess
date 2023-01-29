#[cfg(test)]
mod move_generator_test {
    use protochess_engine_rs::searcher::transposition_table::{Entry, EntryFlag};
    use protochess_engine_rs::types::Move;

    #[test]
    fn test_entry() {
        const ORIGINAL_KEY: u64 = 0x1234567890ABCDEF;
        // Create a new entry
        let mut entry = Entry::new(ORIGINAL_KEY, EntryFlag::Alpha, 123, Move::null(), 3);
        assert_eq!(entry.key, ORIGINAL_KEY);
        // Mask the entry key
        entry.mask_key();
        assert_ne!(entry.key, ORIGINAL_KEY);
        // Unmask the entry key
        assert_eq!(entry.original_key(), ORIGINAL_KEY);
        // Corrupt the entry and check that the key is no longer valid
        entry.flag = EntryFlag::Beta;
        assert_ne!(entry.original_key(), ORIGINAL_KEY);
    }
    
    #[test]
    fn test_2_entries() {
        const ORIGINAL_KEY: u64 = 1234;
        // Create 2 different entries with the same key
        let mut entry1 = Entry::new(ORIGINAL_KEY, EntryFlag::Alpha, 123, Move::null(), 3);
        let mut entry2 = Entry::new(ORIGINAL_KEY, EntryFlag::Beta, 456, Move::null(), 2);
        assert_eq!(entry1.key, entry2.key);
        
        // The masked keys are different, but the original keys remain the same
        entry1.mask_key();
        entry2.mask_key();
        assert_ne!(entry1.key, entry2.key);
        assert_eq!(entry1.original_key(), entry2.original_key());
        assert_eq!(entry1.original_key(), ORIGINAL_KEY);
    }
}
