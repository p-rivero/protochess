use crate::types::{Player, BCoord, Bitboard, BDimensions};
use crate::piece::PieceId;
use super::to_index;
use crate::{Position, PieceDefinition};


pub fn make_custom_position(piece_types: &Vec<PieceDefinition>,
    valid_squares: &Vec<(BCoord, BCoord)>, pieces: &[(Player, BCoord, BCoord, PieceId)]) -> Position
{
    let mut width = 0;
    let mut height = 0;
    let mut bounds = Bitboard::zero();
    for sq in valid_squares {
        if sq.0 >= width { width = sq.0 + 1; }
        if sq.1 >= height { height = sq.1 + 1; }
        bounds.set_bit_at(sq.0, sq.1);
    }

    let pieces = pieces.iter()
        .map(|(owner, x, y, piece)| (*owner, to_index(*x, *y), *piece))
        .collect();
    Position::custom(BDimensions{width, height, bounds}, piece_types, pieces)
}
