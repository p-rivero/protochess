use super::{Piece, PieceId, MovementPattern, MovementPatternExternal};
use crate::types::Player;
use crate::constants::piece_scores::*;

// TODO: Remove this
static NULL_MOVEMENT_PATTERN: MovementPattern = MovementPattern{
    promotion_squares: None,
    promo_vals: None,
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
};

pub struct PieceFactory { }

impl PieceFactory {
    
    // TODO: Remove this
    pub fn blank_custom(player_num: Player, char_rep: char, moves: MovementPatternExternal) -> Piece {
        let move_pattern = moves.to_internal();
        Piece::new(
            BASE_ID_CUSTOM + char_rep as PieceId,
            char_rep,
            player_num,
            move_pattern,
            false,
            false,
            false,
        )
    }
    pub fn blank_pawn(player_num: Player) -> Piece{
        let ch = if player_num == 0 { 'P' } else { 'p' };
        Piece::new(
            ID_PAWN,
            ch,
            player_num,
            NULL_MOVEMENT_PATTERN.clone(),
            false,
            true,
            false,
        )
    }
    pub fn blank_knight(player_num: Player) -> Piece{
        let ch = if player_num == 0 { 'N' } else { 'n' };
        Piece::new(
            ID_KNIGHT,
            ch,
            player_num,
            NULL_MOVEMENT_PATTERN.clone(),
            false,
            false,
            false,
        )
    }
    pub fn blank_king(player_num: Player) -> Piece{
        let ch = if player_num == 0 { 'K' } else { 'k' };
        Piece::new(
            ID_KING,
            ch,
            player_num,
            NULL_MOVEMENT_PATTERN.clone(),
            true,
            false,
            false,
        )
    }
    pub fn blank_rook(player_num: Player) -> Piece{
        let ch = if player_num == 0 { 'R' } else { 'r' };
        Piece::new(
            ID_ROOK,
            ch,
            player_num,
            NULL_MOVEMENT_PATTERN.clone(),
            false,
            false,
            true,
        )
    }
    pub fn blank_bishop(player_num: Player) -> Piece{
        let ch = if player_num == 0 { 'B' } else { 'b' };
        Piece::new(
            ID_BISHOP,
            ch,
            player_num,
            NULL_MOVEMENT_PATTERN.clone(),
            false,
            false,
            false,
        )
    }
    pub fn blank_queen(player_num: Player) -> Piece{
        let ch = if player_num == 0 { 'Q' } else { 'q' };
        Piece::new(
            ID_QUEEN,
            ch,
            player_num,
            NULL_MOVEMENT_PATTERN.clone(),
            false,
            false,
            false,
        )
    }
}