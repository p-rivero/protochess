use super::{PieceDefinition, PieceId};
use crate::types::{Player, Bitboard, BDimensions};

pub struct PieceFactory { }

impl PieceFactory {
    
    pub fn make_pawn(id: PieceId, player_num: Player, dims: &BDimensions, promotions: Vec<PieceId>) -> PieceDefinition {
        let is_white = player_num == 0;
        let promotion_rank = { if is_white { dims.height - 1 } else { 0 } };
        let double_move_rank = { if is_white { 1 } else { dims.height - 2 } };
        let mut promotion_squares = Bitboard::zero();
        let mut double_move_squares = Bitboard::zero();
        for i in 0..dims.width {
            promotion_squares.set_bit_at(i, promotion_rank);
            double_move_squares.set_bit_at(i, double_move_rank);
        }
        let move_dir = { if is_white { 1 } else { -1 } };
        
        PieceDefinition {
            id,
            char_rep: if is_white { 'P' } else { 'p' },
            is_leader: false,
            can_castle: false,
            is_castle_rook: false,
            promotion_squares,
            double_move_squares,
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
    
    pub fn make_knight(id: PieceId, player_num: Player) -> PieceDefinition {
        PieceDefinition {
            id,
            char_rep: if player_num == 0 { 'N' } else { 'n' },
            is_leader: false,
            can_castle: false,
            is_castle_rook: false,
            promotion_squares: Bitboard::zero(),
            double_move_squares: Bitboard::zero(),
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
    
    pub fn make_bishop(id: PieceId, player_num: Player) -> PieceDefinition {
        PieceDefinition {
            id,
            char_rep: if player_num == 0 { 'B' } else { 'b' },
            is_leader: false,
            can_castle: false,
            is_castle_rook: false,
            promotion_squares: Bitboard::zero(),
            double_move_squares: Bitboard::zero(),
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
    
    pub fn make_rook(id: PieceId, player_num: Player) -> PieceDefinition {
        PieceDefinition {
            id,
            char_rep: if player_num == 0 { 'R' } else { 'r' },
            is_leader: false,
            can_castle: false,
            is_castle_rook: true,
            promotion_squares: Bitboard::zero(),
            double_move_squares: Bitboard::zero(),
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
    
    pub fn make_king(id: PieceId, player_num: Player) -> PieceDefinition {
        PieceDefinition {
            id,
            char_rep: if player_num == 0 { 'K' } else { 'k' },
            is_leader: true,
            can_castle: true,
            is_castle_rook: false,
            promotion_squares: Bitboard::zero(),
            double_move_squares: Bitboard::zero(),
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
    
    pub fn make_queen(id: PieceId, player_num: Player) -> PieceDefinition {
        PieceDefinition {
            id,
            char_rep: if player_num == 0 { 'Q' } else { 'q' },
            is_leader: false,
            can_castle: false,
            is_castle_rook: false,
            promotion_squares: Bitboard::zero(),
            double_move_squares: Bitboard::zero(),
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