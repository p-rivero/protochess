use std::collections::HashMap;

use crate::piece::{Piece, PieceId, PieceIdWithPlayer};
use crate::position::piece_set::PieceSet;
use crate::position::Position;
use crate::move_generator::MoveGenerator;
use crate::types::{Move, Centipawns};
use crate::constants::piece_scores::*;


mod material_score;
use material_score::score_movement_pattern;
mod positional_score;
use positional_score::compute_piece_square_table;

/// Assigns a score to a given position
#[derive(Clone, Debug)]
pub struct Evaluator {
    //Piece values for pieces,
    //Hard coded for builtin pieces,
    //generated dynamically based on the piece's movement pattern
    custom_piece_value_table: HashMap<PieceId, Centipawns, ahash::RandomState>,
    //Piece-square values for all pieces, done as a function of movement possibilities
    //Generated dynamically for all pieces
    piece_square_table: HashMap<PieceIdWithPlayer, Vec<Centipawns>, ahash::RandomState>
}

impl Evaluator {
    pub fn new() -> Evaluator {
        Evaluator {
            custom_piece_value_table: HashMap::with_hasher(ahash::RandomState::new()),
            piece_square_table:HashMap::with_hasher(ahash::RandomState::new())
        }
    }
    /// Retrieves the score for the player to move (position.whos_turn)
    pub fn evaluate(&mut self, position: &mut Position, movegen: &MoveGenerator) -> Centipawns {
        let player_num = position.whos_turn;
        // Material score (black pieces are negative)
        let mut score: Centipawns = 0;
        //Material score of both players (black pieces are positive)
        let mut total_material_score: Centipawns = 0;
        
        for ps in position.pieces.iter() {
            let material_score = self.get_material_score_for_pieceset(ps);
            
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
            let positional_score = self.get_positional_score(is_endgame, position, ps,movegen);
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

    fn get_material_score_for_pieceset(&mut self, piece_set: &PieceSet) -> Centipawns {
        let mut material_score = 0;
        material_score += piece_set.king.bitboard.count_ones() as Centipawns * KING_SCORE;
        material_score += piece_set.queen.bitboard.count_ones() as Centipawns * QUEEN_SCORE;
        material_score += piece_set.rook.bitboard.count_ones() as Centipawns * ROOK_SCORE;
        material_score += piece_set.knight.bitboard.count_ones() as Centipawns * KNIGHT_SCORE;
        material_score += piece_set.bishop.bitboard.count_ones() as Centipawns * BISHOP_SCORE;
        material_score += piece_set.pawn.bitboard.count_ones() as Centipawns * PAWN_SCORE;

        for piece in &piece_set.custom {
            let score = self.get_material_score(piece);
            material_score += piece.bitboard.count_ones() as Centipawns * score;
        }
        material_score
    }

    /// Scores a move on a position
    /// This is used for move ordering in order to search the moves with the most potential first
    pub fn score_move(&mut self, history_moves: &[[Centipawns;256];256], killer_moves: &[Move;2], position: &mut Position, mv: &Move) -> Centipawns {
        const CAPTURE_BASE_SCORE: Centipawns = 10000;
        const KILLERMOVE_SCORE: Centipawns = 9000;
        if mv.is_capture() {
            let attacker = position.piece_at(mv.get_from()).unwrap().1;
            let victim = position.piece_at(mv.get_target()).unwrap().1;

            let attack_score = self.get_material_score(attacker);
            let victim_score = self.get_material_score(victim);

            return CAPTURE_BASE_SCORE + victim_score - attack_score
        }
        if mv == &killer_moves[0] || mv == &killer_moves[1] {
            KILLERMOVE_SCORE
        } else {
            history_moves[mv.get_from() as usize][mv.get_to() as usize]
        }
    }

    /// Returns the current material score of a piece
    fn get_material_score(&mut self, piece: &Piece) -> Centipawns {
        let piece_type = piece.get_piece_id();
        match piece_type {
            ID_PAWN => { PAWN_SCORE }
            ID_KNIGHT => { KNIGHT_SCORE }
            ID_BISHOP => { BISHOP_SCORE }
            ID_ROOK => { ROOK_SCORE }
            ID_QUEEN => { QUEEN_SCORE }
            ID_KING => { KING_SCORE }
            _ => {
                if self.custom_piece_value_table.contains_key(&piece_type) {
                    *self.custom_piece_value_table.get(&piece_type).unwrap()
                } else {
                    let score = score_movement_pattern(piece.get_movement());
                    self.custom_piece_value_table.insert(piece_type, score);
                    score
                }
            }
        }
    }

    /// Determines whether or not null move pruning can be performed for a Position
    pub fn can_do_null_move(&mut self, position: &Position) -> bool {
        self.get_material_score_for_pieceset(&position.pieces[position.whos_turn as usize])
            > KING_SCORE + ROOK_SCORE
    }

    fn get_positional_score(&mut self, is_endgame: bool, position: &Position, piece_set: &PieceSet, movegen: &MoveGenerator) -> Centipawns {
        let mut score = 0;

        for p in piece_set.get_piece_refs() {
            let key = p.get_full_id();
            let score_table =
                match self.piece_square_table.get(&key) {
                    Some(score_table) => score_table,
                    None => {
                        let score_vec = compute_piece_square_table(position, p, movegen);
                        self.piece_square_table.insert(key, score_vec);
                        self.piece_square_table.get(&key).unwrap()
                    }
                };
            
            let mut bb_copy = (&p.bitboard).to_owned();
            while !bb_copy.is_zero() {
                let index = bb_copy.lowest_one().unwrap();
                //If it is the king then limit moves (encourage moving away from the center)
                if  p.get_piece_id() == ID_KING && !is_endgame {
                    score -= score_table[index as usize];
                } else {
                    score += score_table[index as usize];
                }

                bb_copy.clear_bit(index);
            }
        }
        score
    }

}

