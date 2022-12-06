use crate::{types::Centipawns, position::piece::PieceId};

pub const KING_SCORE: Centipawns = 320 * CRITICAL_PIECE_MULTIPLIER;
pub const QUEEN_SCORE: Centipawns = 1040;
pub const ROOK_SCORE: Centipawns = 520;
pub const BISHOP_SCORE: Centipawns = 370;
pub const KNIGHT_SCORE: Centipawns = 320;
pub const PAWN_SCORE: Centipawns = 100;

pub const CASTLING_BONUS: Centipawns = 200;
pub const CRITICAL_PIECE_MULTIPLIER: Centipawns = 2;


//king = 0
//queen = 1
//rook = 2
//bishop = 3
//knight = 4
//pawn = 5
    
pub const ID_KING: PieceId = 0;
pub const ID_QUEEN: PieceId = 1;
pub const ID_ROOK: PieceId = 2;
pub const ID_BISHOP: PieceId = 3;
pub const ID_KNIGHT: PieceId = 4;
pub const ID_PAWN: PieceId = 5;
pub const BASE_ID_CUSTOM: PieceId = 100;
