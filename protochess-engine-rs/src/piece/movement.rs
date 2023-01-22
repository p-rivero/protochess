use crate::utils::{to_index, from_index};
use crate::{PieceDefinition, MoveGen, Position, PieceId};
use crate::types::{Bitboard, Move, MoveType, BCoord, BIndex};


/// Outputs all pseudo-legal translation (non-capture) moves for a piece at a given index
#[allow(clippy::too_many_arguments)]
pub fn output_translations(
    movement: &PieceDefinition,
    index: BIndex,
    position: &Position,
    enemies: &Bitboard,
    promotion_squares: &Bitboard,
    occ_or_not_in_bounds: &Bitboard,
    can_castle: bool,
    double_jump_squares: &Bitboard,
    jumps_bitboard: &[Bitboard],
    out_moves: &mut Vec<Move>
) {
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
    // Non-attacks (and in bounds) only
    slide_moves &= !&position.occ_or_out_bounds;
    self::flatten_bb_moves(enemies, slide_moves, index, promotion_squares, &movement.promo_vals, out_moves);


    // JUMP MOVES

    let jump_moves = &jumps_bitboard[index as usize] & !&position.occ_or_out_bounds;
    // Output double jump moves
    if double_jump_squares.get_bit(index) {
        let mut jump_moves_copy = jump_moves.clone();
        while let Some(new_index) = jump_moves_copy.lowest_one() {
            let double_jump_moves = &jumps_bitboard[new_index as usize] & !&position.occ_or_out_bounds;
            self::flatten_bb_moves_doublejump(double_jump_moves, index, new_index, promotion_squares, double_jump_squares, &movement.promo_vals, out_moves);
            jump_moves_copy.clear_bit(new_index);
        }
    }
    // Flatten regular jump moves
    self::flatten_bb_moves(enemies, jump_moves, index, promotion_squares, &movement.promo_vals, out_moves);
    
    
    // SLIDING DELTAS
    
    let (x, y) = from_index(index);
    for run in &movement.translate_sliding_deltas {
        for (dx, dy) in run {
            let (x2, y2) = (x as i8 + *dx, y as i8 + *dy);
            if x2 < 0 || y2 < 0 || x2 > 15 || y2 > 15 {
                break;
            }
            let to = to_index(x2 as BCoord, y2 as BCoord);
            //If the point is out of bounds or there is another piece here, we cannot go any farther
            if position.occ_or_out_bounds.get_bit(to) {
                break;
            }
            if promotion_squares.get_bit(to) {
                //Add all the promotion moves
                for c in &movement.promo_vals {
                    out_moves.push(Move::new(index, to, 0, MoveType::Quiet, Some(*c)));
                }
            } else {
                out_moves.push(Move::new(index, to, 0, MoveType::Quiet, None));
            }
        }
    }
    
    // CASTLING
    
    if can_castle {
        // Able to castle, check if there is a rook on the correct square
        let (_, ky) = from_index(index);
        let rank_visibility = attack_tables.get_rank_slide(index, &position.occ_or_out_bounds);
        
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
                    if position.occ_or_out_bounds.get_bit(i as BIndex) {
                        empty = false;
                        break;
                    }
                }
                // Check that the squares between x=5 and x=(king_x - 1) (both included) are empty
                for i in (to_index as i16 - 1)..=(index as i16 - 1) {
                    if position.occ_or_out_bounds.get_bit(i as BIndex) {
                        empty = false;
                        break;
                    }
                }
                if empty {
                    out_moves.push(Move::new(index, to_index, kingside_rook_index, MoveType::KingsideCastle, None));
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
                    if position.occ_or_out_bounds.get_bit(i as BIndex) {
                        empty = false;
                        break;
                    }
                }
                // Check that the squares between x=(king_x + 1) and x=3 (both included) are empty
                for i in (index as i16 + 1)..=(to_index as i16 + 1) {
                    if position.occ_or_out_bounds.get_bit(i as BIndex) {
                        empty = false;
                        break;
                    }
                }
                if empty {
                    out_moves.push(Move::new(index, to_index, queenside_rook_index, MoveType::QueensideCastle, None));
                }
            }
        }
    }
}



/// Outputs all the pseudo-legal capture moves for a piece at a given index
#[allow(clippy::too_many_arguments)]
pub fn output_captures(
    movement: &PieceDefinition,
    index: BIndex,
    position: &Position,
    enemies: &Bitboard,
    promotion_squares: &Bitboard,
    occ_or_not_in_bounds: &Bitboard,
    jumps_bitboard: &Bitboard,
    out_moves: &mut Vec<Move>
) {
    let attack_tables = MoveGen::attack_tables();
    
    // SLIDING MOVES
    
    let mut slide_moves = attack_tables.get_sliding_moves_bb(
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
    slide_moves &= enemies;
    // Keep only in bounds
    slide_moves &= &position.dimensions.bounds;
    self::flatten_bb_moves(enemies, slide_moves, index, promotion_squares, &movement.promo_vals, out_moves);

    
    // JUMP MOVES
    
    let jump_moves = jumps_bitboard & enemies;
    self::flatten_bb_moves(enemies, jump_moves, index, promotion_squares, &movement.promo_vals, out_moves);
    // En passant capture
    if let Some(ep_square) = position.get_ep_square() {
        if movement.can_double_jump() && jumps_bitboard.get_bit(ep_square) {
            let target = position.get_ep_victim();
            out_moves.push(Move::new(index, ep_square, target, MoveType::Capture, None));
        }
    }
    
    
    // SLIDING DELTAS
    let (x, y) = from_index(index);
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
                        out_moves.push(Move::new(index, to, to, MoveType::PromotionCapture, Some(*c)));
                    }
                } else {
                    out_moves.push(Move::new(index, to, to, MoveType::Capture, None));
                }
                break;
            }
            //Occupied by own team
            if position.occ_or_out_bounds.get_bit(to) {
                break;
            }
        }
    }
}

pub fn flatten_bb_moves(
    enemies: &Bitboard,
    mut moves: Bitboard,
    from_index: BIndex,
    promotion_squares: &Bitboard,
    promo_vals: &Vec<PieceId>,
    out_moves: &mut Vec<Move>
) {
    while let Some(to) = moves.lowest_one() {
        let promo_here = promotion_squares.get_bit(to);
        let capture_here = enemies.get_bit(to);
        let move_type = {
            if capture_here {
                if promo_here { MoveType::PromotionCapture }
                else { MoveType::Capture }
            } else {
                if promo_here { MoveType::Promotion }
                else { MoveType::Quiet }
            }
        };
        if promo_here {
            for promo_val in promo_vals {
                out_moves.push(Move::new(from_index, to, to, move_type, Some(*promo_val)));
            }
        } else {
            //No promotion chars left, go to next after this
            out_moves.push(Move::new(from_index, to, to, move_type, None))
        }
        moves.clear_bit(to);
    }
}
pub fn flatten_bb_moves_doublejump(
    mut moves: Bitboard,
    from_index: BIndex,
    ep_square: BIndex,
    promotion_squares: &Bitboard,
    double_jump_squares: &Bitboard,
    promo_vals: &Vec<PieceId>,
    out_moves: &mut Vec<Move>
) {
    while let Some(to) = moves.lowest_one() {
        if promotion_squares.get_bit(to) {
            for promo_val in promo_vals {
                out_moves.push(Move::new(from_index, to, 0, MoveType::Promotion, Some(*promo_val)));
            }
        } else if double_jump_squares.get_bit(ep_square) {
            // In double jump, the first jump index (to) is an en passant square (unless it's also a double jump square)
            out_moves.push(Move::new(from_index, to, 0, MoveType::Quiet, None))
        } else {
            out_moves.push(Move::new(from_index, to, ep_square, MoveType::DoubleJump, None))
        }
        moves.clear_bit(to);
    }
}
