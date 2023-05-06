use crate::types::{BCoord, Move};
use crate::Position;

use super::from_index;

/// Converts an (x, y) location to chess rank-file notation
/// Ex: `to_rank_file(0, 1)` = a2
pub fn to_rank_file(x: BCoord, y: BCoord) -> String {
    format!("{}{}", (b'a' + x) as char, (y + 1))
}
pub fn tuple_to_rank_file((x, y): (BCoord, BCoord)) -> String {
    to_rank_file(x, y)
}


/// Converts the move to user-friendly algebraic notation
/// Call this **after** making the move
pub fn get_algebraic_notation(pos: &mut Position, mv: &Move, _all_moves: &[Move]) -> String {
    let piece = pos.piece_at(mv.get_to()).unwrap();
    let piece_prefix = piece.get_notation_prefix();
    let to = tuple_to_rank_file(from_index(mv.get_to()));
    
    // TODO: Naive implementation, fix later
    return format!("{piece_prefix}{to}");
}
