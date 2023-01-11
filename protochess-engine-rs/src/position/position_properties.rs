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
    ep_square: Option<BIndex>,
    ep_victim: BIndex, // Only valid if ep_square is Some
    // true if the piece that moved could castle
    pub moved_piece_castle: bool,
    // Full id (piece type + player num) of the captured pieces, if any.
    // Also store whether the captured piece could castle and the index where it was captured.
    // In regular chess, this will be a maximum of 1 piece. In atomic chess, there can be many.
    pub captured_pieces: Vec<(PieceId, Player, bool, BIndex)>,
}

impl PositionProperties {
    pub fn cheap_clone(&self) -> PositionProperties {
        PositionProperties{
            // Copy most fields
            zobrist_key: self.zobrist_key,
            move_played: self.move_played,
            promote_from: self.promote_from,
            castled_players: self.castled_players,
            ep_square: self.ep_square,
            ep_victim: self.ep_victim,
            moved_piece_castle: self.moved_piece_castle,
            captured_pieces: Vec::new(), // Don't clone captured pieces
        }
    }
    
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
            captured_pieces: Vec::new(),
        }
    }
}
