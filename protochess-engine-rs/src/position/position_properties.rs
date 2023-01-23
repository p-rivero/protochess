use crate::types::{BIndex, Move};

use crate::piece::PieceId;

/// Properties that are hard to recover from a Move
#[derive(Clone, Copy, Debug, Default)]
pub struct PositionProperties {
    pub zobrist_key: u64,
    pub move_played: Option<Move>,
    //If the last move was a promotion, promote_from is the previous piecetype
    pub promote_from: Option<PieceId>,
    //EP square (square behind a double pawn push)
    ep_square: Option<BIndex>,
    ep_victim: BIndex, // Only valid if ep_square is Some
    // true if the piece that moved could castle
    pub moved_piece_castle: bool,
    pub num_captures: u8,
}

impl PositionProperties {
    // Access EP square
    pub fn set_ep_square(&mut self, ep_square: BIndex, ep_victim: BIndex) {
        self.clear_ep_square();
        self.ep_square = Some(ep_square);
        self.ep_victim = ep_victim;
        // Update zobrist. For simplicity, use the ep index as the zobrist key
        self.zobrist_key ^= ep_square as u64;
    }
    pub fn clear_ep_square(&mut self) {
        if let Some(sq) = self.ep_square {
            // If the last prop had some ep square then we want to clear zob by xoring again
            self.zobrist_key ^= sq as u64;
        }
        self.ep_square = None;
    }
    pub fn get_ep_square(&self) -> Option<BIndex> {
        self.ep_square
    }
    pub fn get_ep_victim(&self) -> BIndex {
        assert!(self.ep_square.is_some(), "Attempted to get ep victim when ep square is None");
        self.ep_victim
    }
}
