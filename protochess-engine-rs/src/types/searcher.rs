// Type for storing the value of a position, must allow negative values
pub type Centipawns = i32;

// Type for storing a zobrist hash key
pub type ZobKey = u64;

// Don't make this too big, PV is an array of size Depth::MAX
pub type Depth = u8;

// At most 8 players, since castling rights are stored in a u8
// Currently only 2 players are supported, since the current alpha-beta search cannot be generalized to more players
pub type Player = u8;



#[derive(Debug)]
pub struct SearchTimeout;
impl std::fmt::Display for SearchTimeout {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "Search timed out")
    }
}
impl std::error::Error for SearchTimeout { }
