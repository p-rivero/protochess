use instant::Instant;

use crate::State;
use crate::types::{Move, MoveType, Depth, SearcherError, Centipawns};
use crate::transposition_table::{Entry, EntryFlag};

use super::{Searcher, transposition_table};


// This file contains the single-threaded alpha-beta search algorithm, with its extensions and heuristics


// Interface between lazy SMP algorithm and alphabeta
pub(crate) fn alphabeta(searcher: &mut Searcher, state: &mut State, depth: Depth, end_time: &Instant) -> Result<Centipawns, SearcherError> {
    // Use -MAX instead of MIN to avoid overflow when negating
    searcher.alphabeta(state, depth, -Centipawns::MAX, Centipawns::MAX, true, end_time)
}

impl Searcher {
    fn alphabeta(&mut self, state: &mut State, depth: Depth, mut alpha: Centipawns, beta: Centipawns, do_null: bool, end_time: &Instant) -> Result<Centipawns, SearcherError> {
        
        if depth <= 0 {
            return self.quiesce(state, 0, alpha, beta);
        }
        
        self.nodes_searched += 1;
        if self.nodes_searched & 0x7FFFF == 0 {
            // Check for timeout periodically (~500k nodes)
            if Instant::now() >= *end_time {
                return Err(SearcherError::Timeout);
            }
        }

        let is_pv = alpha != beta - 1;
        if let Some(entry) = transposition_table().retrieve(state.position.get_zobrist()) {
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
            if let Some(beta) = self.try_null_move(state, depth, alpha, beta, end_time)? {
                return Ok(beta);
            }
        }

        let mut best_move = Move::null();
        let mut num_legal_moves = 0;
        let old_alpha = alpha;
        let mut best_score = -Centipawns::MAX; // Use -MAX instead of MIN to avoid overflow when negating
        let in_check = state.position_in_check();
        
        // Get potential moves, sorted by move ordering heuristics (try the most promising moves first)
        let moves = state.movegen.get_pseudo_moves(&mut state.position);
        for (_move_score, mv) in self.sort_moves_by_score(state, moves, depth) {
            
            if !state.is_move_legal(&mv) {
                continue;
            }

            num_legal_moves += 1;
            state.position.make_move((&mv).to_owned());
            let mut score: Centipawns;
            if num_legal_moves == 1 {
                score = -self.alphabeta(state, depth - 1, -beta, -alpha, true, end_time)?;
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
                    score = -self.alphabeta(state, reduced_depth, -alpha - 1, -alpha, true, end_time)?;
                } else {
                    //Cannot reduce, proceed with standard PVS
                    score = alpha + 1;
                }

                if score > alpha {
                    //PVS
                    //Null window search
                    score = -self.alphabeta(state, depth - 1, -alpha - 1, -alpha, true, end_time)?;
                    //Re-search if necessary
                    if score > alpha && score < beta {
                        score = -self.alphabeta(state, depth - 1, -beta, -alpha, true, end_time)?;
                    }
                }

            }

            state.position.unmake_move();

            if score > best_score {
                best_score = score;
                best_move = mv;

                if score > alpha {
                    if score >= beta {
                        // Record new killer moves
                        self.update_killers(depth, (&mv).to_owned());
                        // Beta cutoff, store in transpositon table
                        transposition_table().insert(state.position.get_zobrist(), Entry{
                            key: state.position.get_zobrist(),
                            flag: EntryFlag::BETA,
                            value: beta,
                            mv,
                            depth,
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
            transposition_table().insert(state.position.get_zobrist(), Entry{
                key: state.position.get_zobrist(),
                flag: EntryFlag::EXACT,
                value: best_score,
                mv: (&best_move).to_owned(),
                depth,
            })
        } else {
            transposition_table().insert(state.position.get_zobrist(), Entry{
                key: state.position.get_zobrist(),
                flag: EntryFlag::ALPHA,
                value: alpha,
                mv: best_move,
                depth,
            })
        }
        Ok(alpha)
    }


    fn quiesce(&mut self, state: &mut State, depth: Depth, mut alpha: Centipawns, beta: Centipawns) -> Result<Centipawns, SearcherError> {
        let score = state.get_score();
        
        if score >= beta{
            return Ok(beta);
        }
        if score > alpha {
            alpha = score;
        }

        // Get only captures, sorted by move ordering heuristics (try the most promising moves first)
        let moves = state.movegen.get_capture_moves(&mut state.position);
        for (_move_score, mv) in self.sort_moves_by_score(state, moves, depth) {
            if !state.is_move_legal(&mv) {
                continue;
            }

            state.position.make_move((&mv).to_owned());
            let score = -self.quiesce(state, depth, -beta, -alpha)?;
            state.position.unmake_move();

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
                [mv.get_to() as usize] += depth as Centipawns;
        }
    }

    #[inline]
    fn sort_moves_by_score<I: Iterator<Item=Move>>(&mut self, state: &mut State, moves: I, depth: Depth) -> Vec<(Centipawns, Move)> {
        let killer_moves_at_depth = &self.killer_moves[depth as usize];
        let mut moves_and_score: Vec<(Centipawns, Move)> = moves.map(|mv| 
            (state.eval.score_move(&self.history_moves, killer_moves_at_depth, &mut state.position, &mv), mv))
            .collect();

        // Assign PV/hash moves to Centipawns::MAX (search first in the PV)
        if let Some(entry) = transposition_table().retrieve(state.position.get_zobrist()) {
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
    fn try_null_move(&mut self, state: &mut State, depth: Depth, _alpha: Centipawns, beta: Centipawns, end_time: &Instant) -> Result<Option<Centipawns>, SearcherError> {
        
        if depth > 3 && state.eval.can_do_null_move(&mut state.position) && !state.position_in_check() {
            state.position.make_move(Move::null());
            let nscore = -self.alphabeta(state, depth - 3, -beta, -beta + 1, false, end_time)?;
            state.position.unmake_move();
            if nscore >= beta {
                return Ok(Some(beta));
            }
        }
        Ok(None)
    }

}