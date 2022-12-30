use crate::types::{Centipawns, BIndex, Bitboard, BCoord, BDimensions};
use crate::utils::{from_index, to_index, distance_to_one};
use crate::{MoveGen, PieceDefinition};

const POSITION_BASE_MULT: Centipawns = 5;
const POSITION_EDGE_DIST_MULT: Centipawns = 5;
const POSITION_PROMOTION_DIST_MULT: Centipawns = 7;

/// Returns Vec of size 256, each with an integer representing # of moves possible at that location
pub fn compute_piece_square_table(mp: &PieceDefinition, dims: &BDimensions) -> Vec<Centipawns> {
    let mut return_vec = Vec::with_capacity(256);
    let center_squares_bb = get_center_squares(dims.width, dims.height);
    let promotion_squares_bb = mp.promotion_squares_bb() & &dims.bounds; // Keep promotion squares in bounds
    
    
    for index in 0..=BIndex::MAX {
        let (x, y) = from_index(index);
        if !dims.in_bounds(x, y) {
            return_vec.push(0);
            continue;
        }
        let mut moves = get_moves_on_empty_board(mp, index, dims, true);
        moves &= &center_squares_bb;
        
        // 1 point for each move that lands on a center square
        let mut score = moves.count_ones() as Centipawns * POSITION_BASE_MULT;
        
        // 1 point for being 1 square away from the edge (prefer to occupy the center)
        let delta_x = std::cmp::min(x, dims.width - x - 1);
        let delta_y = std::cmp::min(y, dims.height - y - 1);
        
        let distance_from_edge = std::cmp::min(delta_x, delta_y);
        score += distance_from_edge as Centipawns * POSITION_EDGE_DIST_MULT;
        
        // Extra points for being close to promotion
        if !promotion_squares_bb.is_zero() {
            let get_neighbors = |x: BCoord, y: BCoord| {
                let mut neighbors = Vec::new();
                let mut moves = get_moves_on_empty_board(mp, to_index(x, y), dims, false);
                // Get the coordinates of all the 1s in the bitboard
                while let Some(index) = moves.lowest_one() {
                    neighbors.push(from_index(index));
                    moves.clear_bit(index);
                }
                neighbors
            };
            let distance = distance_to_one(x, y, &promotion_squares_bb, get_neighbors);
            // Extend promotion bonus until half of the average board length
            let half_board = ((dims.width + dims.height) / 4) as isize; 
            let promotion_points = std::cmp::max(0, half_board - distance) as Centipawns;
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
fn get_moves_on_empty_board(mp: &PieceDefinition, index: BIndex, dims: &BDimensions, include_attacks: bool) -> Bitboard {
    let (x, y) = from_index(index);
    if !dims.in_bounds(x, y) {
        return Bitboard::zero();
    }
    let walls = !&dims.bounds;
    let mut moves = MoveGen::attack_tables().get_sliding_moves_bb(
        index,
        &walls,
        mp.translate_north || (mp.attack_north && include_attacks),
        mp.translate_east || (mp.attack_east && include_attacks),
        mp.translate_south || (mp.attack_south && include_attacks),
        mp.translate_west || (mp.attack_west && include_attacks),
        mp.translate_northeast || (mp.attack_northeast && include_attacks),
        mp.translate_northwest || (mp.attack_northwest && include_attacks),
        mp.translate_southeast || (mp.attack_southeast && include_attacks),
        mp.translate_southwest || (mp.attack_southwest && include_attacks),
    );

    // Delta based moves (sliding, non sliding)
    let (x, y) = from_index(index);
    let jumps = {
        if include_attacks {
            mp.translate_jump_deltas.iter().chain(mp.attack_jump_deltas.iter()).collect()
        } else {
            mp.translate_jump_deltas.iter().collect::<Vec<_>>()
        }
    };
    for (dx, dy) in jumps {
        let (x2, y2) = (x as i8 + *dx, y as i8 + *dy);
        if x2 < 0 || y2 < 0 || x2 > 15 || y2 > 15 {
            continue;
        }

        let to = to_index(x2 as BCoord, y2 as BCoord);
        if dims.bounds.get_bit(to) {
            moves.set_bit(to);
        }
    }
    
    let sliding_delta_groups = {
        if include_attacks {
            mp.translate_sliding_deltas.iter().chain(mp.attack_sliding_deltas.iter()).collect()
        } else {
            mp.translate_sliding_deltas.iter().collect::<Vec<_>>()
        }
    };
    for run in sliding_delta_groups {
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
            moves.set_bit(to);
        }
    }
    
    //Keep only in bounds
    moves &= &dims.bounds;
    moves
}
