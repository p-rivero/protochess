use crate::position::Position;
use crate::types::{Move, Centipawns};

/// Retrieves the score for the player to move (`position.whos_turn`)
pub fn evaluate(position: &mut Position) -> Centipawns {
    // Material score (without leaders) of both players combined, below which the game is considered to be in the endgame
    // Arbitrary threshold of roughly 2 queens and 2 rooks, feel free to experiment
    const ENDGAME_THRESHOLD: Centipawns = 3000;
    
    let player_num = position.whos_turn;
    // Material score (opponent pieces are negative)
    let mut score = 0;
    //Material score of both players (opponent pieces are positive), without the leaders
    let mut total_leaderless_score = 0;
    
    for ps in &position.pieces {
        let (material_score, leaders_score) = ps.get_material_score();
        
        if ps.get_player_num() == player_num {
            score += material_score;
        } else {
            score -= material_score;
        }
        
        total_leaderless_score += material_score - leaders_score;
    }

    // Positional score
    let is_endgame = total_leaderless_score < ENDGAME_THRESHOLD;
    for ps in &position.pieces {
        let ps_score = {
            if is_endgame {
                ps.get_positional_score::<true>()
            } else {
                ps.get_positional_score::<false>()
            }
        };
        if ps.get_player_num() == player_num {
            score += ps_score;
        } else {
            score -= ps_score;
        }
    }

    // When trying to lose, minimize own score
    if position.global_rules.invert_win_conditions {
        score = -score;
    }
    
    if position.global_rules.checks_to_lose != 0 {
        const CHECK_PENALTY: Centipawns = 512;
        let times_checked = position.get_times_checked();
        score -= CHECK_PENALTY * times_checked[player_num as usize] as Centipawns;
        score += CHECK_PENALTY * times_checked[1-player_num as usize] as Centipawns;
    }
    
    score
}


/// Scores a move on a position
/// This is used for move ordering in order to search the moves with the most potential first
pub fn score_move(history_moves: &[[Centipawns;256];256], killer_moves: &[Move;2], position: &mut Position, mv: Move) -> Centipawns {
    const CAPTURE_BASE_SCORE: Centipawns = 10000;
    const KILLERMOVE_SCORE: Centipawns = 9000;
    if mv.is_capture() {
        let current_player = position.whos_turn;
        let attacker = position.player_piece_at(current_player, mv.get_from()).unwrap();
        let victim = position.player_piece_at(1-current_player, mv.get_target()).unwrap();

        let attack_score = attacker.get_material_score();
        let victim_score = victim.get_material_score();

        return CAPTURE_BASE_SCORE + victim_score - attack_score
    }
    if mv == killer_moves[0] || mv == killer_moves[1] {
        KILLERMOVE_SCORE
    } else {
        history_moves[mv.get_from() as usize][mv.get_to() as usize]
    }
}

/// Determines whether or not null move pruning can be performed for a Position
pub fn can_do_null_move(position: &Position) -> bool {
    // Material score (without leaders) of current player, below which null move pruning is NOT performed
    // Arbitrary threshold of roughly 1 rook, feel free to experiment
    const NULL_MOVE_THRESHOLD: Centipawns = 500;
    
    let piece_set = &position.pieces[position.whos_turn as usize];
    let (total_score, leader_score) = piece_set.get_material_score();
    total_score - leader_score > NULL_MOVE_THRESHOLD
}
