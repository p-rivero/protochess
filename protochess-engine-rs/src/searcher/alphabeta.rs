use instant::Instant;

use crate::types::{Move, MoveType, Depth, SearcherError, Centipawns};
use crate::position::Position;
use crate::move_generator::MoveGenerator;
use crate::evaluator::Evaluator;
use crate::transposition_table::{Entry, EntryFlag};

use super::{Searcher, transposition_table};


// This file contains the single-threaded alpha-beta search algorithm, with its extensions and heuristics


// Interface between lazy SMP algorithm and alphabeta
pub(crate) fn alphabeta(searcher: &mut Searcher, position: &mut Position, eval: &mut Evaluator, movegen: &MoveGenerator, depth: Depth, end_time: &Instant) -> Result<Centipawns, SearcherError> {
    // Use -MAX instead of MIN to avoid overflow when negating
    searcher.alphabeta(position, eval, movegen, depth, -Centipawns::MAX, Centipawns::MAX, true, end_time)
}

impl Searcher {
    fn alphabeta(&mut self, position: &mut Position, eval: &mut Evaluator, movegen: &MoveGenerator, depth: Depth, mut alpha: Centipawns, beta: Centipawns, do_null: bool, end_time: &Instant) -> Result<Centipawns, SearcherError> {

        if depth <= 0 {
            return self.quiesce(position, eval, movegen, 0, alpha, beta);
        }
        
        self.nodes_searched += 1;
        if self.nodes_searched & 0x7FFFF == 0 {
            // Check for timeout periodically (~500k nodes)
            if Instant::now() >= *end_time {
                return Err(SearcherError::Timeout);
            }
        }

        let is_pv = alpha != beta - 1;
        if let Some(entry) = transposition_table().retrieve(position.get_zobrist()) {
            if entry.depth >= depth {
                match entry.flag {
                    EntryFlag::EXACT => {
                        if entry.value < alpha {
                            return Ok(alpha);
                        }
                        if entry.value >= beta {
                            return Ok(beta);
                        }
                        return Ok(entry.value);
                    }
                    EntryFlag::BETA => {
                        if !is_pv && beta <= entry.value {
                            return Ok(beta);
                        }
                    }
                    EntryFlag::ALPHA => {
                        if !is_pv && alpha >= entry.value {
                            return Ok(alpha);
                        }
                    }
                    EntryFlag::NULL => {}
                }
            }
        }

        //Null move pruning
        if !is_pv && do_null {
            if let Some(beta) = self.try_null_move(position, eval, movegen, depth, alpha, beta, end_time)? {
                return Ok(beta);
            }
        }

        let mut best_move = Move::null();
        let mut num_legal_moves = 0;
        let old_alpha = alpha;
        let mut best_score = -Centipawns::MAX; // Use -MAX instead of MIN to avoid overflow when negating
        let in_check = movegen.in_check(position);
        
        // Get potential moves, sorted by move ordering heuristics (try the most promising moves first)
        for (_move_score, mv) in self.sort_moves_by_score(eval, movegen.get_pseudo_moves(position), position, depth) {
            
            if !movegen.is_move_legal(&mv, position) {
                continue;
            }

            num_legal_moves += 1;
            position.make_move((&mv).to_owned());
            let mut score: Centipawns;
            if num_legal_moves == 1 {
                score = -self.alphabeta(position, eval, movegen, depth - 1, -beta, -alpha, true, end_time)?;
            } else {
                //Try late move reduction
                if num_legal_moves > 4
                    && mv.get_move_type() == MoveType::Quiet
                    && !is_pv
                    && depth >= 5
                    && !in_check {
                    //Null window search with depth - 2
                    let mut reduced_depth = depth - 2;
                    if num_legal_moves > 10 {
                        reduced_depth = depth - 3;
                    }
                    score = -self.alphabeta(position, eval, movegen, reduced_depth, -alpha - 1, -alpha, true, end_time)?;
                } else {
                    //Cannot reduce, proceed with standard PVS
                    score = alpha + 1;
                }

                if score > alpha {
                    //PVS
                    //Null window search
                    score = -self.alphabeta(position, eval, movegen, depth - 1, -alpha - 1, -alpha, true, end_time)?;
                    //Re-search if necessary
                    if score > alpha && score < beta {
                        score = -self.alphabeta(position, eval, movegen, depth - 1, -beta, -alpha, true, end_time)?;
                    }
                }

            }

            position.unmake_move();

            if score > best_score {
                best_score = score;
                best_move = mv;

                if score > alpha {
                    if score >= beta {
                        // Record new killer moves
                        self.update_killers(depth, (&mv).to_owned());
                        // Beta cutoff, store in transpositon table
                        transposition_table().insert(position.get_zobrist(), Entry{
                            key: position.get_zobrist(),
                            flag: EntryFlag::BETA,
                            value: beta,
                            mv,
                            depth,
                            ancient: false
                        });
                        return Ok(beta);
                    }
                    alpha = score;

                    // History heuristic
                    self.update_history_heuristic(depth, &mv);
                }
            }
        }

        if num_legal_moves == 0 {
            return if in_check {
                // No legal moves and in check: Checkmate
                if self.nodes_searched == 1 {
                    // We are at the root: the game is over
                    Err(SearcherError::Checkmate)
                } else {
                    // Keep playing until checkmate
                    // A checkmate is effectively -inf, but if we are losing we prefer the longest sequence
                    // Add 1 centipawn per ply to the score to prefer shorter checkmates (or longer when losing)
                    let current_depth = self.current_searching_depth - depth;
                    Ok(-99999 + current_depth as Centipawns)
                }
            } else {
                // No legal moves but also not in check: Stalemate
                if self.nodes_searched == 1 {
                    // We are at the root: it's a draw
                    Err(SearcherError::Stalemate)
                } else {
                    // Keep playing until stalemate
                    Ok(0)
                }
            };
        }

        if alpha != old_alpha {
            //Alpha improvement, record PV
            transposition_table().insert(position.get_zobrist(), Entry{
                key: position.get_zobrist(),
                flag: EntryFlag::EXACT,
                value: best_score,
                mv: (&best_move).to_owned(),
                depth,
                ancient: false
            })
        } else {
            transposition_table().insert(position.get_zobrist(), Entry{
                key: position.get_zobrist(),
                flag: EntryFlag::ALPHA,
                value: alpha,
                mv: best_move,
                depth,
                ancient: false
            })
        }
        Ok(alpha)
    }


    fn quiesce(&mut self, position: &mut Position, eval: &mut Evaluator, movegen: &MoveGenerator, depth: Depth, mut alpha: Centipawns, beta: Centipawns) -> Result<Centipawns, SearcherError> {
        let score = eval.evaluate(position, movegen);
        if score >= beta{
            return Ok(beta);
        }
        if score > alpha {
            alpha = score;
        }

        // Get only captures, sorted by move ordering heuristics (try the most promising moves first)
        for (_move_score, mv) in self.sort_moves_by_score(eval, movegen.get_capture_moves(position), position, depth) {
            if !movegen.is_move_legal(&mv, position) {
                continue;
            }

            position.make_move((&mv).to_owned());
            let score = -self.quiesce(position, eval, movegen, depth, -beta, -alpha)?;
            position.unmake_move();

            if score >= beta {
                return Ok(beta);
            }
            if score > alpha {
                alpha = score;
                // best_move = mv;
            }
        }
        Ok(alpha)
    }

    #[inline]
    fn update_killers(&mut self, depth: Depth, mv: Move) {
        if !mv.get_is_capture() {
            if mv != self.killer_moves[depth as usize][0] && mv != self.killer_moves[depth as usize][1] {
                self.killer_moves[depth as usize][1] = self.killer_moves[depth as usize][0];
                self.killer_moves[depth as usize][0] = mv;
            }
        }
    }

    #[inline]
    fn update_history_heuristic(&mut self, depth: Depth, mv:&Move) {
        if !mv.get_is_capture() {
            self.history_moves
                [mv.get_from() as usize]
                [mv.get_to() as usize] += depth as u16;
        }
    }

    #[inline]
    fn sort_moves_by_score<I: Iterator<Item=Move>>(&mut self, eval: &mut Evaluator, moves: I, position: &mut Position, depth: Depth) -> Vec<(Centipawns, Move)> {
        let mut moves_and_score: Vec<(Centipawns, Move)> = moves.map(|mv| {
            (eval.score_move(&self.history_moves, &self.killer_moves[depth as usize], position, &mv), mv)
        }).collect();

        // Assign PV/hash moves to Centipawns::MAX (search first in the PV)
        if let Some(entry) = transposition_table().retrieve(position.get_zobrist()) {
            let best_move = &entry.mv;
            for (score, mv) in &mut moves_and_score {
                if mv == best_move {
                    *score = Centipawns::MAX;
                }
            }
        }
        
        // Sort moves by decreasing score
        moves_and_score.sort_unstable_by(|a, b| b.0.cmp(&a.0));
        
        moves_and_score
    }

    #[inline]
    fn try_null_move(&mut self, position: &mut Position, eval: &mut Evaluator, movegen: &MoveGenerator, depth: Depth, _alpha: Centipawns, beta: Centipawns, end_time: &Instant) -> Result<Option<Centipawns>, SearcherError> {
        
        if depth > 3 && eval.can_do_null_move(position) && !movegen.in_check(position) {
            position.make_move(Move::null());
            let nscore = -self.alphabeta(position,eval, movegen, depth - 3, -beta, -beta + 1, false, end_time)?;
            position.unmake_move();
            if nscore >= beta {
                return Ok(Some(beta));
            }
        }
        Ok(None)
    }

}