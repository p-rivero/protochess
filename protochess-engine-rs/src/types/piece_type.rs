use crate::{constants::piece_scores::*, piece::PieceId};

// TODO: Remove this file

// Class for storing the value of a position, must allow negative values

pub fn char_to_pieceid(c:char) -> PieceId {
    match c.to_ascii_lowercase() {
        'k' =>{ID_KING}
        'q' =>{ID_QUEEN}
        'r' =>{ID_ROOK}
        'b' =>{ID_BISHOP}
        'n' =>{ID_KNIGHT}
        'p' =>{ID_PAWN}
        _ => {BASE_ID_CUSTOM + c as PieceId}
    }
}