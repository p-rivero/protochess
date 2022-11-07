use std::collections::HashMap;

use crate::position::piece_set::PieceSet;
use crate::position::Position;
use crate::move_generator::MoveGenerator;
use crate::position::piece::Piece;
use crate::MovementPattern;
use crate::types::{BIndex, Move, PieceType, Centipawns};
use crate::utils::from_index;
use crate::constants::piece_scores::*;

/// Assigns a score to a given position
#[derive(Clone, Debug)]
pub struct Evaluator {
    //Piece values for pieces,
    //Hard coded for builtin pieces,
    //generated dynamically based on the piece's movement pattern
    custom_piece_value_table: HashMap<PieceType, Centipawns, ahash::RandomState>,
    //Piece-square values for all pieces, done as a function of movement possibilities
    //Generated dynamically for all pieces
    piece_square_table: HashMap<PieceType, Vec<Centipawns>, ahash::RandomState>
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
            let material_score = self.get_material_score_for_pieceset(position, ps);
            
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
                score += CASTLING_BONUS;
            }
            
            if ps.player_num == player_num {
                score += positional_score;
            } else {
                score -= positional_score;
            }
        }

        score
    }

    fn get_material_score_for_pieceset(&mut self, position: &Position, piece_set: &PieceSet) -> Centipawns {
        let mut material_score = 0;
        material_score += piece_set.king.bitboard.count_ones() as Centipawns * KING_SCORE;
        material_score += piece_set.queen.bitboard.count_ones() as Centipawns * QUEEN_SCORE;
        material_score += piece_set.rook.bitboard.count_ones() as Centipawns * ROOK_SCORE;
        material_score += piece_set.knight.bitboard.count_ones() as Centipawns * KNIGHT_SCORE;
        material_score += piece_set.bishop.bitboard.count_ones() as Centipawns * BISHOP_SCORE;
        material_score += piece_set.pawn.bitboard.count_ones() as Centipawns * PAWN_SCORE;

        for custom in &piece_set.custom {
            let score = 
                if self.custom_piece_value_table.contains_key(&custom.piece_type) {
                    *self.custom_piece_value_table.get(&custom.piece_type).unwrap()
                } else {
                    let piece_score = 
                        if let Some(mp) = position.get_movement_pattern(&custom.piece_type) {
                            Evaluator::score_movement_pattern(mp)
                        } else {
                            0
                        };
                    self.custom_piece_value_table.insert(custom.piece_type.to_owned(), piece_score);
                    piece_score
                };
            material_score += custom.bitboard.count_ones() as Centipawns * score;
        }
        material_score
    }

    /// Scores a move on a position
    /// This is used for move ordering in order to search the moves with the most potential first
    pub fn score_move(&mut self, history_moves: &[[u16;256];256], killer_moves: &[Move;2], position: &mut Position, mv: &Move) -> Centipawns {
        if !mv.get_is_capture() {
            return if mv == &killer_moves[0] || mv == &killer_moves[1] {
                9000
            } else {
                history_moves[mv.get_from() as usize][mv.get_to() as usize] as Centipawns
            }
        }
        let attacker:PieceType = (&position.piece_at(mv.get_from()).unwrap().1.piece_type).to_owned();
        let victim:PieceType = (&position.piece_at(mv.get_target()).unwrap().1.piece_type).to_owned();

        let attack_score = self.get_material_score(attacker, position);
        let victim_score = self.get_material_score(victim, position);

        (KING_SCORE + (victim_score - attack_score)) as Centipawns
    }

    /// Returns the current material score for a given Position
    pub fn get_material_score(&mut self, piece_type:PieceType, position:&Position) -> Centipawns {
        match piece_type {
            PieceType::Pawn => { PAWN_SCORE }
            PieceType::Knight => { KNIGHT_SCORE }
            PieceType::Bishop => { BISHOP_SCORE }
            PieceType::Rook => { ROOK_SCORE }
            PieceType::Queen => { QUEEN_SCORE }
            PieceType::King => { KING_SCORE }
            PieceType::Custom(_c) => {
                if self.custom_piece_value_table.contains_key(&piece_type) {
                    *self.custom_piece_value_table.get(&piece_type).unwrap()
                } else {
                    let option_mp = position.get_movement_pattern(&piece_type);
                    let score = {
                        if let Some(mp) = option_mp {
                            Evaluator::score_movement_pattern(mp)
                        } else {
                            0
                        }
                    };
                    self.custom_piece_value_table.insert((&piece_type).to_owned(), score);
                    score
                }
            }
        }
    }

    /// Determines whether or not null move pruning can be performed for a Position
    pub fn can_do_null_move(&mut self, position:&Position) -> bool {
        self.get_material_score_for_pieceset(&position, &position.pieces[position.whos_turn as usize])
            > KING_SCORE + ROOK_SCORE
    }

    /// Returns a score value for a movement pattern
    fn score_movement_pattern(mp: &MovementPattern) -> Centipawns {
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
        
        println!("SCORE FOR MOVEMENT PATTERN: {}", score);

        // Minimum score is 10
        std::cmp::max(score, 10)
    }

    fn get_positional_score(&mut self, is_endgame: bool, position: &Position, piece_set: &PieceSet, movegen: &MoveGenerator) -> Centipawns {
        let mut score = 0;

        for p in piece_set.get_piece_refs() {

            //Add piecetype if not in table
            if !self.piece_square_table.contains_key(&p.piece_type) {
                //New entry
                let score_vec = Evaluator::get_positional_score_vec(position, p, movegen);

                self.piece_square_table.insert((&p.piece_type).to_owned(), score_vec);
            }
            //Calculate score for these pieces
            let mut bb_copy = (&p.bitboard).to_owned();
            let score_table = self.piece_square_table.get(&p.piece_type).unwrap();
            while !bb_copy.is_zero() {
                let index = bb_copy.lowest_one().unwrap();
                //If it is the king then limit moves (encourage moving away from the center)
                if  p.piece_type == PieceType::King && !is_endgame {
                    score -= score_table[index as usize] * PST_MULTIPLIER;
                } else {
                    score += score_table[index as usize] * PST_MULTIPLIER;
                }

                bb_copy.clear_bit(index);
            }
        }
        score
    }

    /// Returns Vec of size 256, each with an integer representing # of moves possible at that
    /// location
    fn get_positional_score_vec(position: &Position, piece:&Piece, movegen: &MoveGenerator) -> Vec<Centipawns> {
        let mut return_vec = Vec::with_capacity(256);
        let mut total_entries = 0;
        let mut sum = 0;
        for i in 0..=BIndex::MAX {
            let (x, y) = from_index(i);
            let num_moves = movegen.get_num_moves_on_empty_board(i, position, piece, &position.bounds) as Centipawns;
            if position.xy_in_bounds(x, y) {
                total_entries += 1;
                sum += num_moves;
            }
            return_vec.push(num_moves);
        }

        let mean = sum / total_entries;
        //Center the dataset to give a bonus towards squares with higher moves
        //And a penalty for squares with fewer moves
        for i in 0..=BIndex::MAX {
            return_vec[i as usize] -= mean;
        }

        return_vec
    }

}

