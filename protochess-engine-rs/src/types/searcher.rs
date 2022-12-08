use super::Move;

pub type Centipawns = i32;

pub enum SearchError {
    Timeout,
    Checkmate,
    Stalemate,
    // This is not an error, but an easy way to return the best move
    // Return the best move, its score, and the depth. If there is one, also return a backup move to avoid repetition
    BestMove(Move, Centipawns, Option<Move>),
}
pub enum SearchResult {
    // Return value of the search function (best move and its depth). If there is one, also return a backup move to avoid repetition
    BestMove(Move, Depth, Option<Move>),
    // Return the player who is checkmated (losing player)
    Checkmate(Player),
    Stalemate,
}

pub type Depth = u8;

// At most 8 players, since castling rights are stored in a u8
pub type Player = u8;


