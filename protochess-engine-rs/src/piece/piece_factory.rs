use super::{PieceDefinition, PieceId};
use crate::types::GameMode;

pub struct PieceFactory {
    mode: GameMode
}

impl PieceFactory {
    pub fn new(mode: GameMode) -> PieceFactory {
        PieceFactory { mode }
    }
    
    pub fn make_pawn(&self, id: PieceId, is_white: bool, promotions: Vec<PieceId>) -> PieceDefinition {
        let promotion_rank = { if is_white { 7 } else { 0 } };
        let double_move_rank1 = { if is_white { 1 } else { 6 } };
        let double_move_rank2 = { if is_white { 0 } else { 7 } }; // Needed for horde
        let mut promotion_squares = vec![];
        let mut double_jump_squares = vec![];
        for i in 0..8 {
            promotion_squares.push((i, promotion_rank));
            double_jump_squares.push((i, double_move_rank1));
            double_jump_squares.push((i, double_move_rank2));
        }
        let move_dir = { if is_white { 1 } else { -1 } };
        let available_for = {
            if self.mode == GameMode::RacingKings {
                vec![]
            } else if is_white {
                vec![0]
            } else {
                vec![1]
            }
        };
        
        PieceDefinition {
            id,
            char_rep: 'P',
            available_for,
            is_leader: false,
            castle_files: None,
            is_castle_rook: false,
            explodes: self.mode == GameMode::Atomic,
            explosion_deltas: vec![(-1, -1), (-1, 0), (-1, 1), (0, -1), (0, 1), (1, -1), (1, 0), (1, 1)],
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
            win_squares: vec![],
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
            explosion_deltas: vec![(-1, -1), (-1, 0), (-1, 1), (0, -1), (0, 1), (1, -1), (1, 0), (1, 1)],
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
            win_squares: vec![],
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
            explosion_deltas: vec![(-1, -1), (-1, 0), (-1, 1), (0, -1), (0, 1), (1, -1), (1, 0), (1, 1)],
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
            win_squares: vec![],
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
            explosion_deltas: vec![(-1, -1), (-1, 0), (-1, 1), (0, -1), (0, 1), (1, -1), (1, 0), (1, 1)],
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
            win_squares: vec![],
        }
    }
    
    pub fn make_king(&self, id: PieceId) -> PieceDefinition {
        let win_squares = {
            if self.mode == GameMode::KingOfTheHill {
                vec![(3,3), (3,4), (4,3), (4,4)]
            } else if self.mode == GameMode::RacingKings {
                vec![(0,7), (1,7), (2,7), (3,7), (4,7), (5,7), (6,7), (7,7)]
            } else {
                vec![]
            }
        };
        let can_castle = self.mode != GameMode::Antichess && self.mode != GameMode::RacingKings;
        PieceDefinition {
            id,
            char_rep: 'K',
            available_for: if self.mode == GameMode::Horde { vec![1] } else { vec![0, 1] },
            is_leader: self.mode != GameMode::Antichess,
            castle_files: if can_castle { Some((2, 6)) } else { None },
            is_castle_rook: false,
            explodes: self.mode == GameMode::Atomic,
            explosion_deltas: vec![(-1, -1), (-1, 0), (-1, 1), (0, -1), (0, 1), (1, -1), (1, 0), (1, 1)],
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
            win_squares
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
            explosion_deltas: vec![(-1, -1), (-1, 0), (-1, 1), (0, -1), (0, 1), (1, -1), (1, 0), (1, 1)],
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
            win_squares: vec![],
        }
    }
}

impl Default for PieceFactory {
    fn default() -> Self {
        PieceFactory { mode: GameMode::Standard }
    }
}
