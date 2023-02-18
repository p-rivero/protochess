
use std::hash::Hash;
use std::collections::hash_map::{Entry, HashMap};

// Check if two vectors are equal in any order
// https://users.rust-lang.org/t/assert-vectors-equal-in-any-order/38716/10
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
