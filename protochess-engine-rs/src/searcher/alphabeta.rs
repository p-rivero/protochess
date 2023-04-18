#[cfg(feature = "parallel")]
use std::sync::atomic::Ordering;

use crate::MoveGen;
use crate::types::{Move, Depth, Centipawns, SearchTimeout, ZobKey};

use super::Searcher;
use super::eval;
use super::transposition_table::{Entry, EntryFlag};

pub const GAME_OVER_SCORE: Centipawns = -1_000_000;

impl Searcher<'_> {
    /// Search for the best move to play at the current position.
    /// Populates the principal variation vector and returns the score of the position.
    /// Use the previous PV as a hint: begin the search by playing the PV, and the rest of the search attempts to refute it.
    /// # Errors
    /// Returns `Err(SearchTimeout)` if the search timed out.
    pub fn start_alphabeta(&mut self, depth: Depth, hint: &Vec<Move>) -> Result<Centipawns, SearchTimeout> {
        // Use -MAX instead of MIN to avoid overflow when negating
        self.alphabeta::<true,true>(depth, depth, 0, -Centipawns::MAX, Centipawns::MAX, true, Some(hint))
    }
    
    // alpha is the best score that I can currently guarantee at this level or above.
    // beta is the worst score for me that the opponent can currently guarantee at this level or above.
    #[allow(clippy::too_many_lines)]
    #[allow(clippy::too_many_arguments)]
    fn alphabeta<const IS_PV: bool, const IS_ROOT: bool>(&mut self,
            mut depth: Depth,
            mut search_depth: Depth,
            pv_index: usize,
            mut alpha: Centipawns,
            mut beta: Centipawns,
            do_null: bool,
            hint: Option<&Vec<Move>>
        ) -> Result<Centipawns, SearchTimeout>
    {
        let mut known_check = false;
        if IS_PV {
            // If in check, extend search by 1 ply. Limit the extension to 2x the original depth.
            known_check = self.known_checks.contains(&self.zobrist());
            if known_check && search_depth < self.max_searching_depth {
                depth += 1;
                search_depth += 1;
            }
        }
        
        // Skip position if a mating move at this depth is already available
        if !IS_ROOT && !self.pos.global_rules.invert_win_conditions {
            alpha = std::cmp::max(alpha, self.checkmate_score(pv_index));
            beta = std::cmp::min(beta, -self.checkmate_score(pv_index+1));
            if alpha >= beta {
                self.end_pv::<IS_PV>(pv_index-1);
                return Ok(alpha);
            }
        }

        // Probe transposition table
        if let Some(entry) = self.transposition_table.retrieve(self.zobrist()) {
            if entry.depth >= depth {
                match entry.flag {
                    EntryFlag::Exact => {
                        let val = entry.value;
                        if val <= alpha {
                            return Ok(alpha);
                        }
                        if val >= beta {
                            return Ok(beta);
                        }
                        // It's not feasible to store the rest of the PV in the transposition table,
                        // so we don't know what the next moves in the PV are.
                        self.end_pv::<IS_PV>(pv_index);
                        return Ok(val);
                    }
                    EntryFlag::Beta => {
                        if !IS_PV && beta <= entry.value {
                            return Ok(beta);
                        }
                    }
                    EntryFlag::Alpha => {
                        if !IS_PV && alpha >= entry.value {
                            return Ok(alpha);
                        }
                    }
                    EntryFlag::Null => {}
                }
            }
        }
        
        if depth == 0 {
            let quiesce_score = self.quiesce(alpha, beta, pv_index)?;
            let flag = {
                if quiesce_score <= alpha { EntryFlag::Alpha }
                else if quiesce_score >= beta { EntryFlag::Beta }
                else { EntryFlag::Exact }
            };
            self.transposition_table.insert(Entry::new(
                self.zobrist(),
                flag,
                quiesce_score,
                Move::null(),
                depth,
            ));
            self.end_pv::<IS_PV>(pv_index);
            return Ok(quiesce_score); 
        }
        
        self.increment_num_nodes()?;

        // Null move pruning
        if  !IS_PV && depth > 3 && // Don't skip a turn in PV nodes or close to the leaves
            do_null && // Don't do 2 null moves in a row
            !self.pos.global_rules.capturing_is_forced && // Don't skip a turn if capturing is forced
            eval::can_do_null_move(&self.pos) && // Don't skip a turn in endgame
            !MoveGen::in_check(&mut self.pos) // Don't skip a turn in check
        {
            self.pos.make_move(Move::null());
            let nscore = -self.alphabeta::<false,false>(depth-3, search_depth, pv_index+1, -beta, -beta+1, false, None)?;
            self.pos.unmake_move();
            if nscore >= beta {
                return Ok(beta);
            }
        }
        
        let mut best_move = Move::null();
        let mut num_legal_moves = 0;
        let old_alpha = alpha;
        let mut best_score = -Centipawns::MAX; // Use -MAX instead of MIN to avoid overflow when negating
        let in_check = known_check || MoveGen::in_check(&mut self.pos);
        if !IS_ROOT && in_check && self.pos.increment_num_checks() {
            // If the player has been checked N times, the game is over.
            // Don't increment the check counter in the root node
            self.end_pv::<IS_PV>(pv_index);
            return Ok(self.checkmate_score(pv_index));
        }
        if IS_PV && in_check && !known_check && search_depth < self.max_searching_depth {
            // If in check, extend search by 1 ply. Limit the extension to 2x the original depth.
            depth += 1;
            search_depth += 1;
            self.known_checks.insert(self.zobrist());
        }
        
        // If a hint is available, try the hinted move as the leftmost child
        if let Some(pv_hint) = hint {
            if IS_PV && pv_hint.len() > pv_index {
                let mv = pv_hint[pv_index];
                // Since this move was in the PV, it must be legal
                self.pos.make_move(mv);
                num_legal_moves += 1;
                let score: Centipawns;
                if let Some(end_score) = self.is_game_over(mv, pv_index+1) {
                    self.end_pv::<IS_PV>(pv_index);
                    score = -end_score;
                } else {
                    score = -self.alphabeta::<IS_PV,false>(depth-1, search_depth, pv_index+1, -beta, -alpha, true, hint)?;
                }
                self.pos.unmake_move();
                // This is the leftmost branch, we know that best_score = -INF && alpha = -INF
                assert!(score > best_score);
                assert!(score > alpha);
                best_score = score;
                best_move = mv;
                alpha = score;
                self.update_history_heuristic(depth, mv);
            }
        }
        
        // Get potential moves, sorted by move ordering heuristics (try the most promising moves first)
        let moves = MoveGen::get_pseudo_moves(&mut self.pos, true);
        for (_move_score, mv) in self.sort_moves_by_score(moves, depth) {
            
            if !MoveGen::make_move_if_legal(mv, &mut self.pos) {
                continue;
            }

            num_legal_moves += 1;
            let mut score: Centipawns;
            if let Some(end_score) = self.is_game_over(mv, pv_index+1) {
                self.end_pv::<IS_PV>(pv_index);
                score = -end_score;
            } else if num_legal_moves == 1 {
                // Leftmost child when the hint is not available
                score = -self.alphabeta::<IS_PV,false>(depth-1, search_depth, pv_index+1, -beta, -alpha, true, None)?;
            } else {
                // Try late move reduction
                if !IS_PV && num_legal_moves > 4 && mv.is_quiet() && depth >= 5 && !in_check {
                    // Null window search
                    let reduced_depth = {
                        if num_legal_moves > 10 { depth - 4 }
                        else { depth - 3 }
                    };
                    score = -self.alphabeta::<false,false>(reduced_depth, search_depth, pv_index+1, -alpha-1, -alpha, true, None)?;
                } else {
                    // Cannot reduce, proceed with standard PVS
                    score = alpha + 1;
                }

                if IS_PV || score > alpha {
                    // PVS
                    // Null window search
                    score = -self.alphabeta::<false,false>(depth-1, search_depth, pv_index+1, -alpha-1, -alpha, true, None)?;
                    // Re-search if necessary
                    if score > alpha && score < beta {
                        score = -self.alphabeta::<IS_PV,false>(depth-1, search_depth, pv_index+1, -beta, -alpha, true, None)?;
                    }
                }
            }

            self.pos.unmake_move();

            if score > best_score {
                best_score = score;
                best_move = mv;

                if score > alpha {
                    if score >= beta {
                        // Record new killer moves
                        self.update_killers(depth, mv);
                        // Beta cutoff, store in transpositon table
                        self.transposition_table.insert(Entry::new(
                            self.zobrist(),
                            EntryFlag::Beta,
                            beta,
                            mv,
                            depth,
                        ));
                        self.end_pv::<IS_PV>(pv_index);
                        return Ok(beta);
                    }
                    alpha = score;

                    // History heuristic
                    self.update_history_heuristic(depth, mv);
                }
            }
        }

        if num_legal_moves == 0 {
            return if in_check || self.pos.global_rules.stalemated_player_loses {
                // No legal moves and in check: Checkmate
                self.end_pv::<IS_PV>(pv_index);
                Ok(self.checkmate_score(pv_index))
            } else {
                // No legal moves but also not in check: Stalemate
                self.end_pv::<IS_PV>(pv_index);
                Ok(0)
            };
        }

        if IS_PV && alpha != old_alpha {
            //Alpha improvement, record PV
            self.transposition_table.insert(Entry::new(
                self.zobrist(),
                EntryFlag::Exact,
                best_score,
                best_move,
                depth,
            ));
            self.principal_variation[pv_index] = best_move;
            
        } else {
            self.transposition_table.insert(Entry::new(
                self.zobrist(),
                EntryFlag::Alpha,
                alpha,
                best_move,
                depth,
            ));
        }
        
        Ok(alpha)
    }


    // Keep seaching, but only consider capture moves (avoid horizon effect)
    fn quiesce(&mut self, mut alpha: Centipawns, beta: Centipawns, pv_index: usize) -> Result<Centipawns, SearchTimeout> {
        
        if self.pos.leader_is_captured() {
            return Ok(self.checkmate_score(pv_index));
        }
        self.increment_num_nodes()?;

        let score = eval::evaluate(&self.pos);
        
        if score >= beta {
            return Ok(beta);
        }
        if score > alpha {
            alpha = score;
        }

        // Get only captures, sorted by move ordering heuristics (try the most promising moves first)
        let moves = MoveGen::get_pseudo_moves(&mut self.pos, false);
        for (_move_score, mv) in self.sort_moves_by_score(moves, 0) {
            // This is a capture move, so there is no need to check for repetition
            if !MoveGen::make_move_if_legal(mv, &mut self.pos) {
                continue;
            }
            let score = -self.quiesce(-beta, -alpha, pv_index+1)?;
            self.pos.unmake_move();

            if score >= beta {
                return Ok(beta);
            }
            if score > alpha {
                alpha = score;
            }
        }
        Ok(alpha)
    }
    
    #[inline]
    fn zobrist(&self) -> ZobKey {
        self.pos.get_zobrist()
    } 
    
    #[inline]
    fn increment_num_nodes(&mut self) -> Result<(), SearchTimeout> {
        self.nodes_searched += 1;
        // Check for timeout periodically (every 2^16 nodes)
        #[allow(clippy::collapsible_if)]
        if self.nodes_searched.trailing_zeros() >= 16 {
            #[cfg(feature = "parallel")]
            if self.stop_flag.load(Ordering::Relaxed) {
                return Err(SearchTimeout);
            }
            // If this is the first search (depth 1, max_searching_depth 2), don't time out
            if *self.timeout_flag && self.max_searching_depth > 2 {
                // Signal other threads to stop
                #[cfg(feature = "parallel")]
                self.stop_flag.store(true, Ordering::Relaxed);
                return Err(SearchTimeout);
            }
        }
        Ok(())
    }
    
    #[inline]
    // Check for instant game over conditions (does not check for checkmate or stalemate)
    fn is_game_over(&mut self, mv: Move, pv_index: usize) -> Option<Centipawns> {
        // There is repetition, the result is always a draw
        if self.pos.draw_by_repetition() {
            return Some(0);
        }
        // The leader is captured
        if self.pos.leader_is_captured() {
            return Some(self.checkmate_score(pv_index));
        }
        // The opponent has moved the leader to a winning position
        let opponent = 1 - self.pos.whos_turn;
        let to = mv.get_to();
        // Piece could have exploded when capturing in the win square, in that case the square is empty
        if let Some(enemy_piece) = self.pos.player_piece_at(opponent, to) {
            if enemy_piece.wins_at(to) {
                return Some(self.checkmate_score(pv_index));
            }
        }
        None
    }

    #[inline]
    fn update_killers(&mut self, depth: Depth, mv: Move) {
        if !mv.is_capture() && mv != self.killer_moves[depth as usize][0] && mv != self.killer_moves[depth as usize][1] {
            self.killer_moves[depth as usize][1] = self.killer_moves[depth as usize][0];
            self.killer_moves[depth as usize][0] = mv;
        }
    }

    #[inline]
    fn update_history_heuristic(&mut self, depth: Depth, mv: Move) {
        if !mv.is_capture() {
            self.history_moves
                [mv.get_from() as usize]
                [mv.get_to() as usize] += depth as Centipawns;
        }
    }

    #[inline]
    fn sort_moves_by_score(&self, moves: Vec<Move>, depth: Depth) -> Vec<(Centipawns, Move)> {
        // Limit depth to the size of the killer moves array
        let depth = std::cmp::min(self.killer_moves.len() - 1, depth as usize);
        let mut moves_and_score = Vec::with_capacity(moves.len());
        for mv in moves {
            let score = eval::score_move(self, depth, mv);
            moves_and_score.push((score, mv));
        }

        // Assign PV/hash moves to Centipawns::MAX (search first in the PV)
        if let Some(entry) = self.transposition_table.retrieve(self.zobrist()) {
            let best_move = &entry.mv;
            for (score, mv) in &mut moves_and_score {
                if mv == best_move {
                    *score = Centipawns::MAX;
                }
            }
        }
        
        // Sort moves by decreasing score
        moves_and_score.sort_unstable_by(|a, b| b.0.cmp(&a.0));
        
        if self.pos.global_rules.invert_win_conditions {
            moves_and_score.reverse();
        }
        
        moves_and_score
    }
    
    #[inline]
    fn checkmate_score(&self, pv_index: usize) -> Centipawns {
        // A checkmate is effectively -inf, but if we are losing we prefer the longest sequence
        // Add 1 centipawn per ply to the score to prefer shorter checkmates (or longer when losing)
        let score = GAME_OVER_SCORE + pv_index as Centipawns;
        if self.pos.global_rules.invert_win_conditions { -score } else { score }
    }
    
    #[inline]
    fn end_pv<const IS_PV: bool>(&mut self, index: usize) {
        if IS_PV {
            // Clear the next position, use null move as a sentinel
            self.principal_variation[index] = Move::null();
        }
    }
    

}
