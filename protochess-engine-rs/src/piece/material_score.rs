use super::super::PieceDefinition;
use crate::utils::to_index;
use crate::MoveGen;
use crate::types::{Centipawns, BDimensions};

/// Returns a score value for a piece, given its movement pattern
pub fn compute_material_score(mp: &PieceDefinition, dims: &BDimensions) -> Centipawns {
    // This function is called only once, so it's worth it to implement a more complex scoring system
    // https://www.chessprogramming.org/Point_Value
    
    const ATTACK_MUL: f32 = 10.0;
    const TRANSLATE_MUL: f32 = 6.5;
    
    let mut score = 0;
    
    let width = average_dimension(dims, true, false, false, false);
    let height = average_dimension(dims, false, true, false, false);
    let diag = 1.4 * average_dimension(dims, false, false, true, false);
    let antidiag = 1.4 * average_dimension(dims, false, false, false, true);
    
    // 130 centipawns for each direction (Rook is 4*130 = 520 centipawns, Queen is 8*130 = 1040 centipawns)
    if mp.attack_north { score += (ATTACK_MUL * height) as Centipawns }
    if mp.attack_south { score += (ATTACK_MUL * height) as Centipawns }
    if mp.attack_east  { score += (ATTACK_MUL * width) as Centipawns }
    if mp.attack_west  { score += (ATTACK_MUL * width) as Centipawns }
    if mp.translate_north { score += (TRANSLATE_MUL * height) as Centipawns }
    if mp.translate_south { score += (TRANSLATE_MUL * height) as Centipawns }
    if mp.translate_east  { score += (TRANSLATE_MUL * width) as Centipawns }
    if mp.translate_west  { score += (TRANSLATE_MUL * width) as Centipawns }
    
    if mp.attack_northeast { score += (ATTACK_MUL * diag) as Centipawns }
    if mp.attack_southwest { score += (ATTACK_MUL * diag) as Centipawns }
    if mp.attack_northwest { score += (ATTACK_MUL * antidiag) as Centipawns }
    if mp.attack_southeast { score += (ATTACK_MUL * antidiag) as Centipawns }
    if mp.translate_northeast { score += (TRANSLATE_MUL * diag) as Centipawns }
    if mp.translate_southwest { score += (TRANSLATE_MUL * diag) as Centipawns }
    if mp.translate_northwest { score += (TRANSLATE_MUL * antidiag) as Centipawns }
    if mp.translate_southeast { score += (TRANSLATE_MUL * antidiag) as Centipawns }
    
    let only_able_to_slide = !mp.can_promote() && !mp.can_jump() && !mp.has_sliding_deltas();
    
    // Debuff for being limited to a single color of squares
    if !mp.can_slide_main_direction() && only_able_to_slide {
        // Bishop is 4*130 - 150 = 370 centipawns
        score -= 150;
    }
    
    // Debuff for being limited to a single direction
    if mp.can_slide_north_indirectly() && !mp.can_slide_south_indirectly() && only_able_to_slide {
        score -= 200;
    }
    if mp.can_slide_south_indirectly() && !mp.can_slide_north_indirectly() && only_able_to_slide {
        score -= 200;
    }
    if mp.can_slide_east_indirectly() && !mp.can_slide_west_indirectly() && only_able_to_slide {
        score -= 200;
    }
    if mp.can_slide_west_indirectly() && !mp.can_slide_east_indirectly() && only_able_to_slide {
        score -= 200;
    }
    
    // 40 centipawns for each jump (Knight is 8*40 = 320 centipawns)
    score += (mp.translate_jump_deltas.len() * 20) as Centipawns;
    score += (mp.attack_jump_deltas.len() * 20) as Centipawns;
    // 40 centipawns for each delta-based slide group
    for d in mp.translate_sliding_deltas.iter().chain(mp.attack_sliding_deltas.iter()) {
        score += (d.len() * 20) as Centipawns;
    }
    
    // 40 centipawns for being able to promote
    if mp.can_promote() {
        // Pawn is 20*3 + 40 = 100 centipawns
        score += 40;
    }
    
    if mp.is_leader {
        // Leader piece is 4x the value of the regular piece
        score *= 4;
    }
    
    // Minimum score is 10
    std::cmp::max(score, 10)
}

/// Returns the average dimension (width, height, diagonals) of the board, from all legal indexes
/// Gets a callback function that returns the desired dimension for a given index
fn average_dimension(dims: &BDimensions, x_dir: bool, y_dir: bool, diag: bool, antidiag: bool) -> f32 {
    let walls = !&dims.bounds;
    let mut total = 0.0;
    let mut count = 0.0;
    for x in 0..dims.width {
        for y in 0..dims.height {
            let index = to_index(x, y);
            if !dims.bounds.get_bit(index) {
                continue; // Skip illegal indexes
            }
            let sliding_moves = MoveGen::attack_tables().get_sliding_moves_bb(
                index,
                &walls,
                y_dir,
                x_dir,
                y_dir,
                x_dir,
                diag,
                antidiag,
                antidiag,
                diag
            );
            total += sliding_moves.count_ones() as f32;
            count += 1.0;
        }
    }
    total / count
}
