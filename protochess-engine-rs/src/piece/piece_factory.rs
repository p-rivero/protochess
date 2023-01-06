use super::{PieceDefinition, PieceId};
use crate::types::BDimensions;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum GameMode {
    Standard,
    Atomic,
    Horde,
}
pub struct PieceFactory {
    mode: GameMode
}

impl PieceFactory {
    pub fn new(game_mode: &str) -> PieceFactory {
        let mode = match game_mode.to_ascii_lowercase().as_str() {
            "standard" => GameMode::Standard,
            "atomic" => GameMode::Atomic,
            "horde" => GameMode::Horde,
            _ => panic!("Invalid game mode: {}", game_mode)
        };
        PieceFactory { mode }
    }
    
    pub fn make_pawn(&self, id: PieceId, is_white: bool, dims: &BDimensions, promotions: Vec<PieceId>) -> PieceDefinition {
        let promotion_rank = { if is_white { dims.height - 1 } else { 0 } };
        let double_move_rank1 = { if is_white { 1 } else { dims.height - 2 } };
        let double_move_rank2 = { if is_white { 0 } else { dims.height - 1 } }; // Needed for horde
        let mut promotion_squares = vec![];
        let mut double_jump_squares = vec![];
        for i in 0..dims.width {
            promotion_squares.push((i, promotion_rank));
            double_jump_squares.push((i, double_move_rank1));
            double_jump_squares.push((i, double_move_rank2));
        }
        let move_dir = { if is_white { 1 } else { -1 } };
        
        PieceDefinition {
            id,
            char_rep: 'P',
            available_for: if is_white { vec![0] } else { vec![1] },
            is_leader: self.mode == GameMode::Horde && is_white,
            castle_files: None,
            is_castle_rook: false,
            explodes: self.mode == GameMode::Atomic,
            immune_to_explosion: true,
            promotion_squares,
            double_jump_squares,
            promo_vals: promotions,
            attack_sliding_deltas: vec![],
            attack_jump_deltas: vec![(-1, move_dir), (1, move_dir)],
            attack_north: false,
            attack_south: false,
            attack_east: false,
            attack_west: false,
            attack_northeast: false,
            attack_northwest: false,
            attack_southeast: false,
            attack_southwest: false,
            translate_jump_deltas: vec![(0, move_dir)],
            translate_sliding_deltas: vec![],
            translate_north: false,
            translate_south: false,
            translate_east: false,
            translate_west: false,
            translate_northeast: false,
            translate_northwest: false,
            translate_southeast: false,
            translate_southwest: false,
        }
    }
    
    pub fn make_knight(&self, id: PieceId) -> PieceDefinition {
        PieceDefinition {
            id,
            char_rep: 'N',
            available_for: vec![0, 1],
            is_leader: false,
            castle_files: None,
            is_castle_rook: false,
            explodes: self.mode == GameMode::Atomic,
            immune_to_explosion: false,
            promotion_squares: vec![],
            double_jump_squares: vec![],
            promo_vals: vec![],
            attack_sliding_deltas: vec![],
            attack_jump_deltas: vec![(1, 2), (1, -2), (-1, 2), (-1, -2), (2, 1), (2, -1), (-2, 1), (-2, -1)],
            attack_north: false,
            attack_south: false,
            attack_east: false,
            attack_west: false,
            attack_northeast: false,
            attack_northwest: false,
            attack_southeast: false,
            attack_southwest: false,
            translate_jump_deltas: vec![(1, 2), (1, -2), (-1, 2), (-1, -2), (2, 1), (2, -1), (-2, 1), (-2, -1)],
            translate_sliding_deltas: vec![],
            translate_north: false,
            translate_south: false,
            translate_east: false,
            translate_west: false,
            translate_northeast: false,
            translate_northwest: false,
            translate_southeast: false,
            translate_southwest: false,
        }
    }
    
    pub fn make_bishop(&self, id: PieceId) -> PieceDefinition {
        PieceDefinition {
            id,
            char_rep: 'B',
            available_for: vec![0, 1],
            is_leader: false,
            castle_files: None,
            is_castle_rook: false,
            explodes: self.mode == GameMode::Atomic,
            immune_to_explosion: false,
            promotion_squares: vec![],
            double_jump_squares: vec![],
            promo_vals: vec![],
            attack_sliding_deltas: vec![],
            attack_jump_deltas: vec![],
            attack_north: false,
            attack_south: false,
            attack_east: false,
            attack_west: false,
            attack_northeast: true,
            attack_northwest: true,
            attack_southeast: true,
            attack_southwest: true,
            translate_jump_deltas: vec![],
            translate_sliding_deltas: vec![],
            translate_north: false,
            translate_south: false,
            translate_east: false,
            translate_west: false,
            translate_northeast: true,
            translate_northwest: true,
            translate_southeast: true,
            translate_southwest: true,
        }
    }
    
    pub fn make_rook(&self, id: PieceId) -> PieceDefinition {
        PieceDefinition {
            id,
            char_rep: 'R',
            available_for: vec![0, 1],
            is_leader: false,
            castle_files: None,
            is_castle_rook: true,
            explodes: self.mode == GameMode::Atomic,
            immune_to_explosion: false,
            promotion_squares: vec![],
            double_jump_squares: vec![],
            promo_vals: vec![],
            attack_sliding_deltas: vec![],
            attack_jump_deltas: vec![],
            attack_north: true,
            attack_south: true,
            attack_east: true,
            attack_west: true,
            attack_northeast: false,
            attack_northwest: false,
            attack_southeast: false,
            attack_southwest: false,
            translate_jump_deltas: vec![],
            translate_sliding_deltas: vec![],
            translate_north: true,
            translate_south: true,
            translate_east: true,
            translate_west: true,
            translate_northeast: false,
            translate_northwest: false,
            translate_southeast: false,
            translate_southwest: false,
        }
    }
    
    pub fn make_king(&self, id: PieceId) -> PieceDefinition {
        PieceDefinition {
            id,
            char_rep: 'K',
            available_for: if self.mode == GameMode::Horde { vec![1] } else { vec![0, 1] },
            is_leader: true,
            castle_files: Some((2, 6)),
            is_castle_rook: false,
            explodes: self.mode == GameMode::Atomic,
            immune_to_explosion: false,
            promotion_squares: vec![],
            double_jump_squares: vec![],
            promo_vals: vec![],
            attack_sliding_deltas: vec![],
            attack_jump_deltas: vec![(-1, -1), (-1, 0), (-1, 1), (0, -1), (0, 1), (1, -1), (1, 0), (1, 1)],
            attack_north: false,
            attack_south: false,
            attack_east: false,
            attack_west: false,
            attack_northeast: false,
            attack_northwest: false,
            attack_southeast: false,
            attack_southwest: false,
            translate_jump_deltas: vec![(-1, -1), (-1, 0), (-1, 1), (0, -1), (0, 1), (1, -1), (1, 0), (1, 1)],
            translate_sliding_deltas: vec![],
            translate_north: false,
            translate_south: false,
            translate_east: false,
            translate_west: false,
            translate_northeast: false,
            translate_northwest: false,
            translate_southeast: false,
            translate_southwest: false,
        }
    }
    
    pub fn make_queen(&self, id: PieceId) -> PieceDefinition {
        PieceDefinition {
            id,
            char_rep: 'Q',
            available_for: vec![0, 1],
            is_leader: false,
            castle_files: None,
            is_castle_rook: false,
            explodes: self.mode == GameMode::Atomic,
            immune_to_explosion: false,
            promotion_squares: vec![],
            double_jump_squares: vec![],
            promo_vals: vec![],
            attack_sliding_deltas: vec![],
            attack_jump_deltas: vec![],
            attack_north: true,
            attack_south: true,
            attack_east: true,
            attack_west: true,
            attack_northeast: true,
            attack_northwest: true,
            attack_southeast: true,
            attack_southwest: true,
            translate_jump_deltas: vec![],
            translate_sliding_deltas: vec![],
            translate_north: true,
            translate_south: true,
            translate_east: true,
            translate_west: true,
            translate_northeast: true,
            translate_northwest: true,
            translate_southeast: true,
            translate_southwest: true,
        }
    }
}

impl Default for PieceFactory {
    fn default() -> Self {
        PieceFactory { mode: GameMode::Standard }
    }
}
