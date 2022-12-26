use std::sync::Arc;

use crate::types::{BIndex, Move, Player};

use crate::piece::PieceId;

use super::castled_players::CastledPlayers;

/// Properties that are hard to recover from a Move
#[derive(Clone, Debug)]
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
    // Full id (piece type + player num) of the captured piece, if any.
    // Also store whether the captured piece could castle
    pub captured_piece: Option<(PieceId, Player, bool)>,
    pub prev_properties: Option<Arc<PositionProperties>>,
}

impl PositionProperties {
    pub fn get_prev(&self) -> Option<Arc<PositionProperties>> {
        self.prev_properties.as_ref().cloned()
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
            captured_piece: None,
            prev_properties: None,
        }
    }
}