use crate::utils::{to_index, from_index};
use crate::{PieceDefinition, MoveGen, Position};
use crate::types::{Bitboard, Move, MoveType, BCoord, BIndex};
use crate::move_generator::bitboard_moves::BitboardMoves;


#[inline(always)]
pub fn output_moves(movement: &PieceDefinition, index: BIndex,
        position: &Position, enemies: &Bitboard,
        occ_or_not_in_bounds: &Bitboard, can_castle: bool, can_double_jump: bool,
        out_bb_moves: &mut Vec<BitboardMoves>, out_moves: &mut Vec<Move>)
{
    let attack_tables = MoveGen::attack_tables();
    
    // SLIDING MOVES
    
    // Attacks!
    let mut raw_attacks = attack_tables.get_sliding_moves_bb(
        index,
        occ_or_not_in_bounds,
        movement.attack_north,
        movement.attack_east,
        movement.attack_south,
        movement.attack_west,
        movement.attack_northeast,
        movement.attack_northwest,
        movement.attack_southeast,
        movement.attack_southwest
    );
    // Attacks only
    raw_attacks &= enemies;
    // Keep only in bounds
    raw_attacks &= &position.dimensions.bounds;
    out_bb_moves.push(BitboardMoves::new(
        enemies.to_owned(),
        raw_attacks,
        index,
        movement.promotion_squares.to_owned(),
        movement.promo_vals.to_owned(),
    ));
    
    // Movements!
    let mut raw_moves = attack_tables.get_sliding_moves_bb(
        index,
        occ_or_not_in_bounds,
        movement.translate_north,
        movement.translate_east,
        movement.translate_south,
        movement.translate_west,
        movement.translate_northeast,
        movement.translate_northwest,
        movement.translate_southeast,
        movement.translate_southwest
    );
    // Non-attacks only
    raw_moves &= !&position.occupied;
    // Keep only in bounds
    raw_moves &= &position.dimensions.bounds;
    out_bb_moves.push(BitboardMoves::new(
        enemies.to_owned(),
        raw_moves,
        index,
        movement.promotion_squares.to_owned(),
        movement.promo_vals.to_owned(),
    ));


    // JUMP MOVES
    
    let (x, y) = from_index(index);
    for (dx, dy) in &movement.translate_jump_deltas {
        let (x2, y2) = (x as i8 + *dx, y as i8 + *dy);
        if x2 < 0 || y2 < 0 || x2 > 15 || y2 > 15 {
            continue;
        }
        let to = to_index(x2 as BCoord, y2 as BCoord);
        if position.in_bounds(x2 as BCoord, y2 as BCoord) && !position.occupied.get_bit(to) {
            // Promotion here?
            if movement.promotion_at(to) {
                // Add all the promotion moves
                for c in &movement.promo_vals {
                    out_moves.push(Move::new(index, to, None, MoveType::Promotion, Some(*c)));
                }
            } else {
                out_moves.push(Move::new(index, to, None, MoveType::Quiet, None));
            }
            
            if can_double_jump {
                // Jump again
                for (dx2, dy2) in &movement.translate_jump_deltas {
                    let (x3, y3) = (x2 as i8 + *dx2, y2 as i8 + *dy2);
                    if x3 < 0 || y3 < 0 || x3 > 15 || y3 > 15 {
                        continue;
                    }
                    let to2 = to_index(x3 as BCoord, y3 as BCoord);
                    if position.in_bounds(x3 as BCoord, y3 as BCoord) && !position.occupied.get_bit(to2) {
                        // Promotion here?
                        if movement.promotion_at(to2) {
                            // Add all the promotion moves
                            for c in &movement.promo_vals {
                                out_moves.push(Move::new(index, to2, None, MoveType::Promotion, Some(*c)));
                            }
                        } else {
                            // In double jump, the first jump index (to) is an en passant square
                            out_moves.push(Move::new(index, to2, Some(to), MoveType::DoubleJump, None));
                        }
                    }
                }
            }
        }
    }

    for (dx, dy) in &movement.attack_jump_deltas {
        let (x2, y2) = (x as i8 + *dx, y as i8 + *dy);
        if x2 < 0 || y2 < 0 || x2 > 15 || y2 > 15 {
            continue;
        }
        let to = to_index(x2 as BCoord, y2 as BCoord);
        if enemies.get_bit(to) {
            //Promotion here?
            if movement.promotion_at(to) {
                //Add all the promotion moves
                for c in &movement.promo_vals {
                    out_moves.push(Move::new(index, to, Some(to), MoveType::PromotionCapture, Some(*c)));
                }
            } else {
                out_moves.push(Move::new(index, to, Some(to), MoveType::Capture, None));
            }
        }
        // En passant capture
        if position.properties.ep_square == Some(to) && movement.can_double_jump() {
            let target = position.properties.ep_victim;
            out_moves.push(Move::new(index, to, Some(target), MoveType::Capture, None));
        }
    }
    
    
    // SLIDING DELTAS
    
    for run in &movement.translate_sliding_deltas {
        for (dx, dy) in run {
            let (x2, y2) = (x as i8 + *dx, y as i8 + *dy);
            if x2 < 0 || y2 < 0 || x2 > 15 || y2 > 15 {
                break;
            }
            let to = to_index(x2 as BCoord, y2 as BCoord);
            //If the point is out of bounds or there is another piece here, we cannot go any
            //farther
            if !position.in_bounds(x2 as BCoord, y2 as BCoord) || position.occupied.get_bit(to) {
                break;
            }
            if movement.promotion_at(to) {
                //Add all the promotion moves
                for c in &movement.promo_vals {
                    out_moves.push(Move::new(index, to, None, MoveType::Quiet, Some(*c)));
                }
            } else {
                out_moves.push(Move::new(index, to, None, MoveType::Quiet, None));
            }
        }
    }

    for run in &movement.attack_sliding_deltas {
        for (dx, dy) in run {

            let (x2, y2) = (x as i8 + *dx, y as i8 + *dy);
            if x2 < 0 || y2 < 0 || x2 > 15 || y2 > 15 {
                break;
            }

            let to = to_index(x2 as BCoord, y2 as BCoord);
            //Out of bounds, next sliding moves can be ignored
            if !position.in_bounds(x2 as BCoord, y2 as BCoord) {
                break;
            }
            //If there is an enemy here, we can add an attack move
            if enemies.get_bit(to) {
                if movement.promotion_at(to) {
                    //Add all the promotion moves
                    for c in &movement.promo_vals {
                        out_moves.push(Move::new(index, to, Some(to), MoveType::PromotionCapture, Some(*c)));
                    }
                } else {
                    out_moves.push(Move::new(index, to, Some(to), MoveType::Capture, None));
                }
                break;
            }
            //Occupied by own team
            if position.occupied.get_bit(to) {
                break;
            }
        }
    }
    
    // CASTLING
    
    if can_castle {
        // Able to castle, check if there is a rook on the correct square
        let (kx, ky) = from_index(index);
        
        // King side
        let kingside_rook_index = to_index(position.dimensions.width-1, ky);
        if let Some(rook) = position.piece_at(kingside_rook_index) {
            // There is a rook (that has not moved) of the same player
            if rook.is_castle_rook(kingside_rook_index) && rook.get_player() == position.whos_turn {
                // Rook is on the correct square and can castle
                // Check if the squares between the king and the rook are empty
                let east = attack_tables.masks.get_east(index);
                let mut occ = east & &position.occupied;
                occ.clear_bit(kingside_rook_index);
                if occ.is_zero() {
                    // See if we can move the king one step east without stepping into check
                    let to_index = to_index(kx + 2, ky);
                    out_moves.push(Move::new(index, to_index, Some(kingside_rook_index), MoveType::KingsideCastle, None));
                }
            }
        }
        // Queen side (same as kingside)
        let queenside_rook_index = to_index(0, ky);
        if let Some(rook) = position.piece_at(queenside_rook_index) {
            if rook.is_castle_rook(queenside_rook_index) && rook.get_player() == position.whos_turn {
                let west = attack_tables.masks.get_west(index);
                let mut occ = west & &position.occupied;
                occ.clear_bit(queenside_rook_index);
                if occ.is_zero() {
                    let to_index = to_index(kx - 2, ky);
                    out_moves.push(Move::new(index, to_index, Some(queenside_rook_index), MoveType::QueensideCastle, None));
                }
            }
        }
    }
}
