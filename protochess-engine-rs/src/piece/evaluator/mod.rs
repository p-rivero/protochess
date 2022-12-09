use crate::piece::Piece;
use crate::position::piece_set::PieceSet;
use crate::position::Position;
use crate::types::{Move, Centipawns};
use crate::constants::piece_scores::*;


pub mod material_score;
pub mod positional_score;

/// Assigns a score to a given position
#[derive(Clone, Debug)]
pub struct Evaluator { }

impl Evaluator {
    /// Retrieves the score for the player to move (position.whos_turn)
    pub fn evaluate(position: &mut Position) -> Centipawns {
        let player_num = position.whos_turn;
        // Material score (black pieces are negative)
        let mut score: Centipawns = 0;
        //Material score of both players (black pieces are positive)
        let mut total_material_score: Centipawns = 0;
        
        for ps in position.pieces.iter() {
            let material_score = Evaluator::get_material_score_for_pieceset(ps);
            
            if ps.player_num == player_num {
                score += material_score;
            } else {
                score -= material_score;
            }
            
            total_material_score += material_score;
        }

        //Positional score
        let is_endgame = total_material_score < 2 * KING_SCORE + 2 * QUEEN_SCORE + 2 * ROOK_SCORE;
        for ps in position.pieces.iter() {
            let positional_score = Evaluator::get_positional_score(is_endgame, ps);
            //Castling bonus
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

    fn get_material_score_for_pieceset(piece_set: &PieceSet) -> Centipawns {
        let mut material_score = 0;
        material_score += piece_set.king.bitboard.count_ones() as Centipawns * KING_SCORE;
        material_score += piece_set.queen.bitboard.count_ones() as Centipawns * QUEEN_SCORE;
        material_score += piece_set.rook.bitboard.count_ones() as Centipawns * ROOK_SCORE;
        material_score += piece_set.knight.bitboard.count_ones() as Centipawns * KNIGHT_SCORE;
        material_score += piece_set.bishop.bitboard.count_ones() as Centipawns * BISHOP_SCORE;
        material_score += piece_set.pawn.bitboard.count_ones() as Centipawns * PAWN_SCORE;

        for piece in &piece_set.custom {
            let score = Evaluator::get_material_score(piece);
            material_score += piece.bitboard.count_ones() as Centipawns * score;
        }
        material_score
    }

    /// Scores a move on a position
    /// This is used for move ordering in order to search the moves with the most potential first
    pub fn score_move(history_moves: &[[Centipawns;256];256], killer_moves: &[Move;2], position: &mut Position, mv: &Move) -> Centipawns {
        const CAPTURE_BASE_SCORE: Centipawns = 10000;
        const KILLERMOVE_SCORE: Centipawns = 9000;
        if mv.is_capture() {
            let attacker = position.piece_at(mv.get_from()).unwrap();
            let victim = position.piece_at(mv.get_target()).unwrap();

            let attack_score = Evaluator::get_material_score(attacker);
            let victim_score = Evaluator::get_material_score(victim);

            return CAPTURE_BASE_SCORE + victim_score - attack_score
        }
        if mv == &killer_moves[0] || mv == &killer_moves[1] {
            KILLERMOVE_SCORE
        } else {
            history_moves[mv.get_from() as usize][mv.get_to() as usize]
        }
    }

    /// Returns the current material score of a piece
    fn get_material_score(piece: &Piece) -> Centipawns {
        let piece_type = piece.get_piece_id();
        match piece_type {
            ID_PAWN => { PAWN_SCORE }
            ID_KNIGHT => { KNIGHT_SCORE }
            ID_BISHOP => { BISHOP_SCORE }
            ID_ROOK => { ROOK_SCORE }
            ID_QUEEN => { QUEEN_SCORE }
            ID_KING => { KING_SCORE }
            _ => { piece.get_material_score() }
        }
    }

    /// Determines whether or not null move pruning can be performed for a Position
    pub fn can_do_null_move(position: &Position) -> bool {
        Evaluator::get_material_score_for_pieceset(&position.pieces[position.whos_turn as usize])
            > KING_SCORE + ROOK_SCORE
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

}

