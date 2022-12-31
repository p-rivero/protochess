use crate::types::{BIndex, Move, Player};

use crate::piece::PieceId;

use super::castled_players::CastledPlayers;

/// Properties that are hard to recover from a Move
#[derive(Debug)]
pub struct PositionProperties {
    pub zobrist_key: u64,
    pub move_played: Option<Move>,
    //If the last move was a promotion, promote_from is the previous piecetype
    pub promote_from: Option<PieceId>,
    pub castled_players: CastledPlayers,
    //EP square (square behind a double pawn push)
    pub ep_square: Option<BIndex>,
    pub ep_victim: BIndex, // Only valid if ep_square is Some
    // true if the piece that moved could castle
    pub moved_piece_castle: bool,
    // true if make_move() was called with update_reps = true
    pub update_reps: bool,
    // Full id (piece type + player num) of the captured pieces, if any.
    // Also store whether the captured piece could castle and the index where it was captured.
    // In regular chess, this will be a maximum of 1 piece. In atomic chess, there can be many.
    pub captured_pieces: Vec<(PieceId, Player, bool, BIndex)>,
}

impl Clone for PositionProperties {
    fn clone(&self) -> PositionProperties {
        PositionProperties{
            // Copy most fields
            zobrist_key: self.zobrist_key,
            move_played: self.move_played,
            promote_from: self.promote_from,
            castled_players: self.castled_players,
            ep_square: self.ep_square,
            ep_victim: self.ep_victim,
            moved_piece_castle: self.moved_piece_castle,
            update_reps: self.update_reps,
            captured_pieces: Vec::new(), // Don't clone captured pieces
        }
    }
}

impl Default for PositionProperties {
    fn default() -> PositionProperties {
        PositionProperties{
            zobrist_key: 0,
            move_played: None,
            promote_from: None,
            castled_players: CastledPlayers::new(),
            ep_square: None,
            ep_victim: 0,
            moved_piece_castle: false,
            update_reps: false,
            captured_pieces: Vec::new(),
        }
    }
}
