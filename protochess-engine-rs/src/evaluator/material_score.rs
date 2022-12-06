use crate::position::movement_pattern::MovementPattern;
use crate::types::Centipawns;

/// Returns a score value for a piece, given its movement pattern
pub fn score_movement_pattern(mp: &MovementPattern) -> Centipawns {
    // https://www.chessprogramming.org/Point_Value
    
    // This function is called only once, so it's worth it to implement a more complex scoring system
    
    let mut score: Centipawns = 0;
    
    // 130 centipawns for each direction (Rook is 4*130 = 520 centipawns, Queen is 8*130 = 1040 centipawns)
    if mp.attack_north {score += 80}
    if mp.translate_north {score += 50}
    if mp.attack_east {score += 80}
    if mp.translate_east {score += 50}
    if mp.attack_south {score += 80}
    if mp.translate_south {score += 50}
    if mp.attack_west {score += 80}
    if mp.translate_west {score += 50}
    
    if mp.attack_northeast {score += 80}
    if mp.translate_northeast {score += 50}
    if mp.attack_northwest {score += 80}
    if mp.translate_northwest {score += 50}
    if mp.attack_southeast {score += 80}
    if mp.translate_southeast {score += 50}
    if mp.attack_southwest {score += 80}
    if mp.translate_southwest {score += 50}
    
    // Debuff for being limited to a single color of squares
    if !mp.can_slide_main_direction() && !mp.can_promote() {
        // Bishop is 4*130 - 150 = 370 centipawns
        score -= 150;
    }
    
    // Debuff for being limited to a single direction
    if mp.can_slide_north_indirectly() && !mp.can_slide_south_indirectly() && !mp.can_promote() {
        score -= 200;
    }
    if mp.can_slide_south_indirectly() && !mp.can_slide_north_indirectly() && !mp.can_promote() {
        score -= 200;
    }
    if mp.can_slide_east_indirectly() && !mp.can_slide_west_indirectly() && !mp.can_promote() {
        score -= 200;
    }
    if mp.can_slide_west_indirectly() && !mp.can_slide_east_indirectly() && !mp.can_promote() {
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
    
    // Minimum score is 10
    std::cmp::max(score, 10)
}
