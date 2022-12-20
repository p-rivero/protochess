use crate::utils::{to_index, from_index};
use crate::{PieceDefinition, MoveGen};
use crate::types::{Bitboard, Move, MoveType, BCoord, BIndex, BDimensions};
use crate::move_generator::bitboard_moves::BitboardMoves;


#[inline(always)]
pub fn output_moves(movement: &PieceDefinition, index: BIndex,
        dims: &BDimensions, occupied: &Bitboard, enemies: &Bitboard, occ_or_not_in_bounds: &Bitboard,
        out_bb_moves: &mut Vec<BitboardMoves>, out_moves: &mut Vec<Move>)
{
    let attack_tables = MoveGen::attack_tables();
        
    // Sliding moves along ranks or files
    //Attacks!
    let mut raw_attacks = attack_tables.get_sliding_moves_bb(
        index,
        &occ_or_not_in_bounds,
        movement.attack_north,
        movement.attack_east,
        movement.attack_south,
        movement.attack_west,
        movement.attack_northeast,
        movement.attack_northwest,
        movement.attack_southeast,
        movement.attack_southwest
    );
    //Attacks ONLY
    raw_attacks &= enemies;
    //Keep only in bounds
    raw_attacks &= &dims.bounds;
    out_bb_moves.push(BitboardMoves::new(
        enemies.to_owned(),
        raw_attacks,
        index,
        movement.promotion_squares.to_owned(),
        movement.promo_vals.to_owned(),
    ));
    //Movements!
    let mut raw_moves = attack_tables.get_sliding_moves_bb(index,
                                                                &occ_or_not_in_bounds,
                                                                movement.translate_north,
                                                                movement.translate_east,
                                                                movement.translate_south,
                                                                movement.translate_west,
                                                                movement.translate_northeast,
                                                                movement.translate_northwest,
                                                                movement.translate_southeast,
                                                                movement.translate_southwest
    );
    //Non-attacks ONLY
    raw_moves &= !occupied;
    //Keep only in bounds
    raw_moves &= &dims.bounds;
    out_bb_moves.push(BitboardMoves::new(
        enemies.to_owned(),
        raw_moves,
        index,
        movement.promotion_squares.to_owned(),
        movement.promo_vals.to_owned(),
    ));


    // Delta based moves (sliding, non sliding)
    let (x, y) = from_index(index);
    for (dx, dy) in &movement.translate_jump_deltas {
        let (x2, y2) = (x as i8 + *dx, y as i8 + *dy);
        if x2 < 0 || y2 < 0 || x2 > 15 || y2 > 15 {
            continue;
        }
        let to = to_index(x2 as BCoord, y2 as BCoord);
        if dims.in_bounds(x2 as BCoord, y2 as BCoord) && !occupied.get_bit(to) {
            //Promotion here?
            if movement.promotion_at(to) {
                //Add all the promotion moves
                for c in &movement.promo_vals {
                    out_moves.push(Move::new(index, to, None, MoveType::Promotion, Some(*c)));
                }
            } else {
                out_moves.push(Move::new(index, to, None, MoveType::Quiet, None));
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
    }

    for run in &movement.attack_sliding_deltas {
        for (dx, dy) in run {

            let (x2, y2) = (x as i8 + *dx, y as i8 + *dy);
            if x2 < 0 || y2 < 0 || x2 > 15 || y2 > 15 {
                break;
            }

            let to = to_index(x2 as BCoord, y2 as BCoord);
            //Out of bounds, next sliding moves can be ignored
            if !dims.in_bounds(x2 as BCoord, y2 as BCoord) {
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
            if occupied.get_bit(to) {
                break;
            }
        }
    }


    for run in &movement.translate_sliding_deltas {
        for (dx, dy) in run {
            let (x2, y2) = (x as i8 + *dx, y as i8 + *dy);
            if x2 < 0 || y2 < 0 || x2 > 15 || y2 > 15 {
                break;
            }
            let to = to_index(x2 as BCoord, y2 as BCoord);
            //If the point is out of bounds or there is another piece here, we cannot go any
            //farther
            if !dims.in_bounds(x2 as BCoord, y2 as BCoord) || occupied.get_bit(to) {
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
}
