use crate::position::piece_set::PieceSet;
use crate::position::Position;
use crate::types::{Move, Centipawns};
use crate::constants::piece_scores::*;

/// Retrieves the score for the player to move (position.whos_turn)
pub fn evaluate(position: &mut Position) -> Centipawns {
    // Material score (without leaders) of both players combined, below which the game is considered to be in the endgame
    // Arbitrary threshold of roughly 2 queens and 2 rooks, feel free to experiment
    const ENDGAME_THRESHOLD: Centipawns = 3000;
    
    let player_num = position.whos_turn;
    // Material score (opponent pieces are negative)
    let mut score = 0;
    //Material score of both players (opponent pieces are positive), without the leaders
    let mut total_leaderless_score = 0;
    
    for ps in position.pieces.iter() {
        let (material_score, leaders_score) = ps.get_material_score();
        
        if ps.player_num == player_num {
            score += material_score;
        } else {
            score -= material_score;
        }
        
        total_leaderless_score += material_score - leaders_score;
    }

    // Positional score
    let is_endgame = total_leaderless_score < ENDGAME_THRESHOLD;
    for ps in position.pieces.iter() {
        let positional_score = get_positional_score(is_endgame, ps);
        //Castling bonus
        // TODO: Keep castling bonus also in the endgame?
        if position.properties.castling_rights.did_player_castle(ps.player_num) && !is_endgame {
            if ps.player_num == player_num {
                score += CASTLING_BONUS;
            } else {
                score -= CASTLING_BONUS;
            }
        }
        
        if ps.player_num == player_num {
            score += positional_score;
        } else {
            score -= positional_score;
        }
    }

    score
}


/// Scores a move on a position
/// This is used for move ordering in order to search the moves with the most potential first
pub fn score_move(history_moves: &[[Centipawns;256];256], killer_moves: &[Move;2], position: &mut Position, mv: &Move) -> Centipawns {
    const CAPTURE_BASE_SCORE: Centipawns = 10000;
    const KILLERMOVE_SCORE: Centipawns = 9000;
    if mv.is_capture() {
        let attacker = position.piece_at(mv.get_from()).unwrap();
        let victim = position.piece_at(mv.get_target()).unwrap();

        let attack_score = attacker.get_material_score();
        let victim_score = victim.get_material_score();

        return CAPTURE_BASE_SCORE + victim_score - attack_score
    }
    if mv == &killer_moves[0] || mv == &killer_moves[1] {
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

fn get_positional_score(is_endgame: bool, piece_set: &PieceSet) -> Centipawns {
    let mut score = 0;
    for piece in piece_set.get_piece_refs() {
        // TODO: Inverting the leader score may not always be the best option
        // For each piece, get the positional score.
        // Invert the leader so that it stays away from the center, except in the endgame
        if piece.is_leader() && !is_endgame {
            score -= piece.get_positional_score_all();
        } else {
            score += piece.get_positional_score_all();
        }
    }
    score
}
