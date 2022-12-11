use crate::constants::piece_scores::*;
use crate::types::{Centipawns, BIndex, Bitboard, BCoord, BDimensions};
use crate::utils::{from_index, to_index, distance_to_one};
use crate::{MoveGen, PieceDefinition};
use crate::piece::Piece;

const POSITION_BASE_MULT: Centipawns = 5;
const POSITION_EDGE_DIST_MULT: Centipawns = 5;
const POSITION_PROMOTION_DIST_MULT: Centipawns = 7;

/// Returns Vec of size 256, each with an integer representing # of moves possible at that location
// TODO: Once the hardcoded pieces are removed, delete the Piece parameter
pub fn compute_piece_square_table(mp: &PieceDefinition, dims: &BDimensions, piece: &Piece) -> Vec<Centipawns> {
    let mut return_vec = Vec::with_capacity(256);
    let center_squares_bb = get_center_squares(dims.width, dims.height);
    let promotion_squares_bb = get_promotion_squares(mp, dims, piece);
    
    for index in 0..=BIndex::MAX {
        let (x, y) = from_index(index);
        if !dims.in_bounds(x, y) {
            return_vec.push(0);
            continue;
        }
        let mut moves = get_moves_on_empty_board(mp, index, dims, piece);
        moves = moves & center_squares_bb.to_owned();
        
        // 1 point for each move that lands on a center square
        let mut score = moves.count_ones() as Centipawns * POSITION_BASE_MULT;
        
        // 1 point for being 1 square away from the edge (prefer to occupy the center)
        let delta_x = std::cmp::min(x, dims.width - x - 1);
        let delta_y = std::cmp::min(y, dims.height - y - 1);
        
        let distance_from_edge = std::cmp::min(delta_x, delta_y);
        score += distance_from_edge as Centipawns * POSITION_EDGE_DIST_MULT;
        
        // Extra points for being close to promotion
        if !promotion_squares_bb.is_zero() {
            let distance = distance_to_one(x, y, &promotion_squares_bb) as isize;
            let promotion_points = std::cmp::max(0, 4 - distance) as Centipawns;
            score += promotion_points * POSITION_PROMOTION_DIST_MULT;
        }
        
        return_vec.push(score);
    }
    
    return_vec
}

// Construct a bitboard with the squares at the middle set to 1
fn get_center_squares(width: BCoord, height: BCoord) -> Bitboard {
    let mut center_squares = Bitboard::zero();
    // For even dimensions, the center is the 4 squares in the middle
    // For odd dimensions, the center is the 9 squares in the middle
    let x1 = width / 2 - 1;
    let x2 = width / 2;
    let x3 = (width + 1) / 2;
    let y1 = height / 2 - 1;
    let y2 = height / 2;
    let y3 = (height + 1) / 2;
    
    center_squares.set_bit_at(x1, y1);
    center_squares.set_bit_at(x1, y2);
    center_squares.set_bit_at(x1, y3);
    center_squares.set_bit_at(x2, y1);
    center_squares.set_bit_at(x2, y2);
    center_squares.set_bit_at(x2, y3);
    center_squares.set_bit_at(x3, y1);
    center_squares.set_bit_at(x3, y2);
    center_squares.set_bit_at(x3, y3);
    center_squares
}


/// Returns the number of moves of a piecetype on an otherwise empty board
// TODO: Once the hardcoded pieces are removed, delete the Piece parameter
fn get_moves_on_empty_board(mp: &PieceDefinition, index: BIndex, dims: &BDimensions, piece: &Piece) -> Bitboard {
    let (x, y) = from_index(index);
    if !dims.in_bounds(x, y) {
        return Bitboard::zero();
    }
    let zero = Bitboard::zero();
    let walls = !&dims.bounds;
    let mut moves = match piece.get_piece_id() {
        ID_QUEEN => {MoveGen::attack_tables().get_queen_attack(index, &walls, &zero)}
        ID_BISHOP => {MoveGen::attack_tables().get_bishop_attack(index, &walls, &zero)}
        ID_ROOK => {MoveGen::attack_tables().get_rook_attack(index, &walls, &zero)}
        ID_KNIGHT => {MoveGen::attack_tables().get_knight_attack(index, &walls, &zero)}
        ID_KING => {MoveGen::attack_tables().get_king_attack(index, &walls, &zero)}
        ID_PAWN => {
            if piece.player_num() == 0 {
                MoveGen::attack_tables().get_north_pawn_attack(index, &walls, &Bitboard::all_ones())
            } else {
                MoveGen::attack_tables().get_south_pawn_attack(index, &walls, &Bitboard::all_ones())
            }
        }
        _ => {

            let mut slides = MoveGen::attack_tables().get_sliding_moves_bb(
                index,
                &walls,
                mp.translate_north || mp.attack_north,
                mp.translate_east || mp.attack_east,
                mp.translate_south || mp.attack_south,
                mp.translate_west || mp.attack_west,
                mp.translate_northeast || mp.attack_northeast,
                mp.translate_northwest || mp.attack_northwest,
                mp.translate_southeast || mp.attack_southeast,
                mp.translate_southwest || mp.attack_southwest
            );

            // Delta based moves (sliding, non sliding)
            let (x, y) = from_index(index);
            for (dx, dy) in mp.translate_jump_deltas.iter().chain(mp.attack_jump_deltas.iter()) {
                let (x2, y2) = (x as i8 + *dx, y as i8 + *dy);
                if x2 < 0 || y2 < 0 || x2 > 15 || y2 > 15 {
                    continue;
                }

                let to = to_index(x2 as BCoord, y2 as BCoord);
                if dims.bounds.get_bit(to) {
                    slides.set_bit(to);
                }
            }
            for run in mp.attack_sliding_deltas.iter().chain(mp.translate_sliding_deltas.iter()) {
                for (dx, dy) in run {
                    let (x2, y2) = (x as i8 + *dx, y as i8 + *dy);
                    if x2 < 0 || y2 < 0 || x2 > 15 || y2 > 15 {
                        break;
                    }
                    let to = to_index(x2 as BCoord, y2 as BCoord);
                    //Out of bounds, next sliding moves can be ignored
                    if !dims.bounds.get_bit(to) {
                        break;
                    }
                    slides.set_bit(to);
                }
            }
            slides
        }
    };
    //Keep only in bounds
    moves &= &dims.bounds;
    moves
}

// TODO: Once the hardcoded pieces are removed, delete the Piece parameter
fn get_promotion_squares(mp: &PieceDefinition, dims: &BDimensions, piece: &Piece) -> Bitboard {
    let piece_id = piece.get_piece_id();
    if piece_id == ID_PAWN {
        let mut promotion_squares = Bitboard::zero();
        let y = if piece.player_num() == 0 { dims.height - 1 } else { 0 };
        for x in 0..dims.width {
            promotion_squares.set_bit_at(x, y);
        }
        promotion_squares
    } else if piece_id >= BASE_ID_CUSTOM {
        // Keep promotion squares in bounds
        (&mp.promotion_squares) & (&dims.bounds)
    } else {
        Bitboard::zero()
    }
}

