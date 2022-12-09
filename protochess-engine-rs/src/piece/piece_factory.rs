use super::PieceDefinition;
use super::{Piece, PieceId};
use crate::types::{Player, Bitboard};

// TODO: Remove this
fn null_piece_def(id: PieceId, char_rep: char) -> PieceDefinition {
    PieceDefinition {
        id,
        char_rep,
        is_leader: false,
        can_double_move: false,
        can_castle: false,
        promotion_squares: Bitboard::zero(),
        promo_vals: vec![],
        attack_sliding_deltas: vec![],
        attack_jump_deltas: vec![],
        attack_north: false,
        attack_south: false,
        attack_east: false,
        attack_west: false,
        attack_northeast: false,
        attack_northwest: false,
        attack_southeast: false,
        attack_southwest: false,
        translate_jump_deltas: vec![],
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


pub struct PieceFactory { }

impl PieceFactory {
    
    // TODO: Remove this
    pub fn make_custom(definition: PieceDefinition, player_num: Player) -> Piece {
        Piece::new(definition, player_num)
    }
    pub fn make_pawn(id: PieceId, player_num: Player) -> Piece{
        let ch = { if player_num == 0 { 'P' } else { 'p' } };
        Piece::new(null_piece_def(id, ch), player_num)
    }
    pub fn make_knight(id: PieceId, player_num: Player) -> Piece{
        let ch = { if player_num == 0 { 'N' } else { 'n' } };
        Piece::new(null_piece_def(id, ch), player_num)
    }
    pub fn make_king(id: PieceId, player_num: Player) -> Piece{
        let ch = { if player_num == 0 { 'K' } else { 'k' } };
        Piece::new(null_piece_def(id, ch), player_num)
    }
    pub fn make_rook(id: PieceId, player_num: Player) -> Piece{
        let ch = { if player_num == 0 { 'R' } else { 'r' } };
        Piece::new(null_piece_def(id, ch), player_num)
    }
    pub fn make_bishop(id: PieceId, player_num: Player) -> Piece{
        let ch = { if player_num == 0 { 'B' } else { 'b' } };
        Piece::new(null_piece_def(id, ch), player_num)
    }
    pub fn make_queen(id: PieceId, player_num: Player) -> Piece{
        let ch = { if player_num == 0 { 'Q' } else { 'q' } };
        Piece::new(null_piece_def(id, ch), player_num)
    }
}