use crate::utils::{to_index, from_index};
use crate::{PieceDefinition, MoveGen, Position};
use crate::types::{Bitboard, Move, MoveType, BCoord, BIndex};
use crate::move_generator::bitboard_moves::BitboardMoves;


/// Outputs all pseudo-legal translation (non-capture) moves for a piece at a given index
#[allow(clippy::too_many_arguments)]
#[inline]
pub fn output_translations(movement: &PieceDefinition, index: BIndex,
        position: &Position, enemies: &Bitboard, promotion_squares: &Bitboard,
        occ_or_not_in_bounds: &Bitboard, can_castle: bool, double_jump_squares: &Bitboard,
        out_bb_moves: &mut Vec<BitboardMoves>, out_moves: &mut Vec<Move>)
{
    let attack_tables = MoveGen::attack_tables();
    
    // SLIDING MOVES
    
    let mut slide_moves = attack_tables.get_sliding_moves_bb(
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
    slide_moves &= !&position.occupied;
    // Keep only in bounds
    slide_moves &= &position.dimensions.bounds;
    out_bb_moves.push(BitboardMoves::new(
        enemies.clone(),
        slide_moves,
        index,
        promotion_squares.clone(),
        movement.promo_vals.clone(),
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
            if promotion_squares.get_bit(to) {
                // Add all the promotion moves
                for c in &movement.promo_vals {
                    out_moves.push(Move::new(index, to, None, MoveType::Promotion, Some(*c)));
                }
            } else {
                out_moves.push(Move::new(index, to, None, MoveType::Quiet, None));
            }
            
            if double_jump_squares.get_bit(index) {
                // Jump again
                for (dx2, dy2) in &movement.translate_jump_deltas {
                    let (x3, y3) = (x2 as i8 + *dx2, y2 as i8 + *dy2);
                    if x3 < 0 || y3 < 0 || x3 > 15 || y3 > 15 {
                        continue;
                    }
                    let to2 = to_index(x3 as BCoord, y3 as BCoord);
                    if position.in_bounds(x3 as BCoord, y3 as BCoord) && !position.occupied.get_bit(to2) {
                        // Promotion here?
                        if promotion_squares.get_bit(to2) {
                            // Add all the promotion moves
                            for c in &movement.promo_vals {
                                out_moves.push(Move::new(index, to2, None, MoveType::Promotion, Some(*c)));
                            }
                        } else if double_jump_squares.get_bit(to) {
                            // In double jump, the first jump index (to) is an en passant square (unless it's also a double jump square)
                            out_moves.push(Move::new(index, to2, None, MoveType::Quiet, None));
                        } else {
                            out_moves.push(Move::new(index, to2, Some(to), MoveType::DoubleJump, None));
                        }
                    }
                }
            }
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
            if promotion_squares.get_bit(to) {
                //Add all the promotion moves
                for c in &movement.promo_vals {
                    out_moves.push(Move::new(index, to, None, MoveType::Quiet, Some(*c)));
                }
            } else {
                out_moves.push(Move::new(index, to, None, MoveType::Quiet, None));
            }
        }
    }
    
    // CASTLING
    
    if can_castle {
        // Able to castle, check if there is a rook on the correct square
        let (_, ky) = from_index(index);
        let rank_visibility = attack_tables.get_rank_attack(index, &position.occupied);
        
        // King side
        // Index of the closest piece to the east
        let kingside_rook_index = rank_visibility.highest_one().unwrap();
        if let Some(rook) = position.player_piece_at(position.whos_turn, kingside_rook_index) {
            // There is a piece of the same player
            if rook.is_rook() && rook.has_not_moved(kingside_rook_index) {
                // Rook is in direct line of sight of the king and hasn't moved
                let kingside_file = movement.castle_files.unwrap().1;
                let to_index = to_index(kingside_file, ky);
                // Check that the squares between x=(rook_x + 1) and x=6 (both included) are empty
                let mut empty = true;
                for i in (kingside_rook_index as i16 + 1)..=(to_index as i16) {
                    if position.occupied.get_bit(i as BIndex) {
                        empty = false;
                        break;
                    }
                }
                // Check that the squares between x=5 and x=(king_x - 1) (both included) are empty
                for i in (to_index as i16 - 1)..=(index as i16 - 1) {
                    if position.occupied.get_bit(i as BIndex) {
                        empty = false;
                        break;
                    }
                }
                if empty {
                    out_moves.push(Move::new(index, to_index, Some(kingside_rook_index), MoveType::KingsideCastle, None));
                }
            }
        }
        // Queen side (same logic as kingside)
        let queenside_rook_index = rank_visibility.lowest_one().unwrap();
        if let Some(rook) = position.player_piece_at(position.whos_turn, queenside_rook_index) {
            if rook.is_rook() && rook.has_not_moved(queenside_rook_index) {
                let queenside_file = movement.castle_files.unwrap().0;
                let to_index = to_index(queenside_file, ky);
                // Check that the squares between x=2 and x=(rook_x - 1) (both included) are empty
                let mut empty = true;
                for i in (to_index as i16)..=(queenside_rook_index as i16 - 1) {
                    if position.occupied.get_bit(i as BIndex) {
                        empty = false;
                        break;
                    }
                }
                // Check that the squares between x=(king_x + 1) and x=3 (both included) are empty
                for i in (index as i16 + 1)..=(to_index as i16 + 1) {
                    if position.occupied.get_bit(i as BIndex) {
                        empty = false;
                        break;
                    }
                }
                if empty {
                    out_moves.push(Move::new(index, to_index, Some(queenside_rook_index), MoveType::QueensideCastle, None));
                }
            }
        }
    }
}



/// Outputs all the pseudo-legal capture moves for a piece at a given index
#[allow(clippy::too_many_arguments)]
#[inline]
pub fn output_captures(movement: &PieceDefinition, index: BIndex,
        position: &Position, enemies: &Bitboard, promotion_squares: &Bitboard,
        occ_or_not_in_bounds: &Bitboard, out_bb_moves: &mut Vec<BitboardMoves>, out_moves: &mut Vec<Move>)
{
    let attack_tables = MoveGen::attack_tables();
    
    // SLIDING MOVES
    
    let mut slide_attacks = attack_tables.get_sliding_moves_bb(
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
    slide_attacks &= enemies;
    // Keep only in bounds
    slide_attacks &= &position.dimensions.bounds;
    out_bb_moves.push(BitboardMoves::new(
        enemies.clone(),
        slide_attacks,
        index,
        promotion_squares.clone(),
        movement.promo_vals.clone(),
    ));

    
    // JUMP MOVES

    let (x, y) = from_index(index);
    for (dx, dy) in &movement.attack_jump_deltas {
        let (x2, y2) = (x as i8 + *dx, y as i8 + *dy);
        if x2 < 0 || y2 < 0 || x2 > 15 || y2 > 15 {
            continue;
        }
        let to = to_index(x2 as BCoord, y2 as BCoord);
        if enemies.get_bit(to) {
            //Promotion here?
            if promotion_squares.get_bit(to) {
                //Add all the promotion moves
                for c in &movement.promo_vals {
                    out_moves.push(Move::new(index, to, Some(to), MoveType::PromotionCapture, Some(*c)));
                }
            } else {
                out_moves.push(Move::new(index, to, Some(to), MoveType::Capture, None));
            }
        }
        // En passant capture
        if movement.can_double_jump() && position.get_ep_square() == Some(to) {
            let target = position.get_ep_victim();
            out_moves.push(Move::new(index, to, Some(target), MoveType::Capture, None));
        }
    }
    
    
    // SLIDING DELTAS
    
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
                if promotion_squares.get_bit(to) {
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
}
