use super::Move;

// Class for storing the value of a position, must allow negative values
pub type Centipawns = i32;

pub enum SearchError {
    Timeout,
    // This is not an error, but an easy way to return the best move
    // Return the best move, its score, and the depth.
    BestMove(Move, Centipawns),
}

pub type Depth = u8;

// At most 8 players, since castling rights are stored in a u8
pub type Player = u8;


