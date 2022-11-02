pub enum SearcherError {
    Timeout,
    Checkmate,
    Stalemate,
}
pub enum GameResult {
    Checkmate,
    Stalemate,
}

pub type Depth = u8;

// At most 8 players, since castling rights are stored in a u8
pub type Player = u8;


