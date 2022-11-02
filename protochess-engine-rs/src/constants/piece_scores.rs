use crate::types::Centipawns;

pub const KING_SCORE: Centipawns = 9999;
pub const QUEEN_SCORE: Centipawns = 900;
pub const ROOK_SCORE: Centipawns = 500;
pub const BISHOP_SCORE: Centipawns = 350;
pub const KNIGHT_SCORE: Centipawns = 300;
pub const PAWN_SCORE: Centipawns = 100;
pub const CASTLING_BONUS: Centipawns = 400;

//Multiplier for the piece square table
pub const PST_MULTIPLIER: Centipawns = 5;
