
use std::convert::TryInto;
use std::hash::Hash;
use std::collections::hash_map::{Entry, HashMap};

use crate::types::GameMode;

/// Check if two vectors are equal in any order
/// https://users.rust-lang.org/t/assert-vectors-equal-in-any-order/38716/10
pub fn eq_anyorder<T: Eq + Hash>(a: &[T], b: &[T]) -> bool {
    fn get_lookup<T: Eq + Hash>(iter:impl Iterator<Item = T>) -> HashMap<T, usize> {
        let mut lookup = HashMap::<T, usize>::new();
        for value in iter {
            match lookup.entry(value) {
                Entry::Occupied(entry) => { *entry.into_mut() += 1; },
                Entry::Vacant(entry) => { entry.insert(0); }
            }
        }
        lookup
    }
    get_lookup(a.iter()) == get_lookup(b.iter())
}


/// Splits the FEN into the standard FEN and a hardcoded variant name. Used for testing.
/// The variant name is the last part of the FEN, separated by whitespace.
/// 
/// See `GameMode` for a list of valid variant names.
pub fn split_debug_fen(fen: &str) -> (String, GameMode) {
    let fen_vec: Vec<_> = fen.split_whitespace().collect();
    let variant_str = *fen_vec.last().unwrap();
    let variant = variant_str.try_into();
    if variant.is_err() {
        return (fen.to_string(), GameMode::Standard);
    }
    // Join all parts of the FEN except the last one
    let std_fen = fen_vec[0..fen_vec.len()-1].join(" ");
    (std_fen, variant.unwrap())
}
