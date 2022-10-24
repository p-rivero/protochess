pub mod types;

use std::sync::atomic::{AtomicU8, AtomicU64, Ordering::Relaxed};
use std::thread;
use instant::{Instant, Duration};

use crate::types::chess_move::{Move, MoveType};
use crate::position::Position;
use crate::move_generator::MoveGenerator;
use crate::evaluator::Evaluator;
use crate::transposition_table::{TranspositionTable, Entry, EntryFlag};

use self::types::*;

// Global structures, shared between threads
static mut TRANSPOSITION_TABLE: Option<TranspositionTable> = None;
static mut GLOBAL_DEPTH: AtomicU8 = AtomicU8::new(0);
static mut SEARCH_ID: AtomicU64 = AtomicU64::new(1);

static NUM_THREADS: u32 = 4;


pub(crate) struct Searcher {
    //We store two killer moves per ply,
    //indexed by killer_moves[depth][0] or killer_moves[depth][0]
    killer_moves: [[Move;2];64],
    //Indexed by history_moves[side2move][from][to]
    history_moves: [[u16;256];256],
    // Stats
    nodes_searched: u64,
    current_searching_depth: Depth,
}

impl Searcher {
    pub fn get_best_move(position: &Position, eval: &Evaluator, movegen: &MoveGenerator, depth: Depth) -> Result<(Move, Depth), GameResult> {
        // Cannot use u64::MAX due to overflow, 1_000_000 seconds is 11.5 days
        Searcher::get_best_move_impl(position, eval, movegen, depth, 1_000_000)
    }

    pub fn get_best_move_timeout(position: &Position, eval: &Evaluator, movegen: &MoveGenerator, time_sec: u64) -> Result<(Move, Depth), GameResult> {
        Searcher::get_best_move_impl(position, eval, movegen, Depth::MAX, time_sec)
    }
    
    fn new() -> Searcher {
        Searcher{
            killer_moves: [[Move::null(); 2];64],
            history_moves: [[0;256];256],
            nodes_searched: 0,
            current_searching_depth: 0,
        }
    }
    
    fn get_best_move_impl(position: &Position, eval: &Evaluator, movegen: &MoveGenerator, max_depth: Depth, time_sec: u64) -> Result<(Move, Depth), GameResult> {
        
        // Initialize the global structures
        unsafe {
            if TRANSPOSITION_TABLE.is_none() {
                TRANSPOSITION_TABLE = Some(TranspositionTable::new());
            } else {
                transposition_table().set_ancient();
            }
            GLOBAL_DEPTH.store(0, Relaxed);
            SEARCH_ID.store(1, Relaxed);
        }
        
        // Init threads, store handles
        let mut handles = Vec::with_capacity(NUM_THREADS as usize);
        for thread_id in 0..NUM_THREADS {
            // For each thread, create a local copy of the heuristics
            let mut pos_copy = position.clone();
            let mut eval_copy = eval.clone();
            let movegen_copy = (*movegen).clone();
            let h = thread::spawn(move || {
                let mut searcher = Searcher::new();
                searcher.search_thread(thread_id, &mut pos_copy, &mut eval_copy, &movegen_copy, max_depth, time_sec)
            });
            handles.push(h);
        }
        
        // Wait for threads to finish
        let mut best_move = Move::null();
        let mut best_score = isize::MIN;
        let mut best_depth = 0;
        for h in handles {
            // If any thread returns a GameResult, drop the other threads and return the result
            let (mv, score, depth) = h.join().unwrap()?;
            // If any thread reaches the target depth, drop the other threads and return the move
            if depth == max_depth {
                return Ok((mv, depth));
            }
            if depth > best_depth || (depth == best_depth && score > best_score) {
                best_move = mv;
                best_score = score;
                best_depth = depth;
            }
        }
        Ok((best_move, best_depth))
    }
    
    // Run for some time, then return the best move, its score, and the depth
    fn search_thread(&mut self, thread_id: u32, position: &mut Position, eval: &mut Evaluator, movegen: &MoveGenerator, max_depth: Depth, time_sec: u64) -> Result<(Move, isize, Depth), GameResult> {
        let end_time = Instant::now() + Duration::from_secs(time_sec);
        
        let mut best_move: Move = Move::null();
        let mut best_score: isize = 0;
        let mut best_depth: Depth = 0;
        
        // At the start, each thread should search a different depth (between 1 and max_depth, inclusive)
        let mut local_depth = (thread_id as Depth % max_depth) + 1;
        
        loop {
            self.nodes_searched = 0;
            self.current_searching_depth = local_depth;
            match self.alphabeta(position, eval, movegen, local_depth, -isize::MAX, isize::MAX, true, &end_time) {
                Ok(score) => {
                    best_score = score;
                    best_move = transposition_table().retrieve(position.get_zobrist()).unwrap().mv;
                    best_depth = local_depth;
                },
                Err(SearcherError::Timeout) => {
                    // Thread timed out, return the best move found so far
                    break;
                },
                Err(SearcherError::Checkmate) => {
                    assert!(best_depth == 0);
                    return Err(GameResult::Checkmate);
                },
                Err(SearcherError::Stalemate) => {
                    assert!(best_depth == 0);
                    return Err(GameResult::Stalemate);
                }
            }
            //Print PV info
            println!("Thread {} score: {}, depth: {}, nodes: {}", thread_id, best_score, best_depth, self.nodes_searched);
            
            // Set the global depth to max(local_depth, global_depth)
            // GLOBAL_DEPTH contains the maximum depth searched by any thread
            let old_global_depth = unsafe { GLOBAL_DEPTH.fetch_max(local_depth, Relaxed) };
            
            // If time is up or any thread has searched to max_depth, return
            if Instant::now() >= end_time || local_depth == max_depth || old_global_depth == max_depth {
                // Signal to other threads that they can stop
                unsafe { GLOBAL_DEPTH.store(max_depth, Relaxed); }
                break;
            }
                        
            if NUM_THREADS == 1 {
                // Iterative deepening
                local_depth += 1;
            } else {
                // Update local_depth: set to GLOBAL_DEPTH + increment
                let search_id = unsafe { SEARCH_ID.fetch_add(1, Relaxed) };
                // 1/2 threads search 1 ply deeper, 1/4 threads search 2 ply deeper, etc.
                let increment = 1 + search_id.trailing_zeros() as Depth;
                local_depth = unsafe { GLOBAL_DEPTH.load(Relaxed) } + increment;
            }
            // Limit local_depth to max_depth
            local_depth = std::cmp::min(local_depth, max_depth);
        }

        Ok((best_move.to_owned(), best_score, best_depth))
    }

    fn alphabeta(&mut self, position: &mut Position, eval: &mut Evaluator, movegen: &MoveGenerator, depth: Depth, mut alpha: isize, beta: isize, do_null: bool, end_time: &Instant) -> Result<isize, SearcherError> {

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
                        if entry.value >= beta{
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
        let mut best_score = -isize::MAX;
        let in_check = movegen.in_check(position);
        
        // Get potential moves, sorted by move ordering heuristics (try the most promising moves first)
        for (_move_score, mv) in self.sort_moves_by_score(eval, movegen.get_pseudo_moves(position), position, depth) {
            
            if !movegen.is_move_legal(&mv, position) {
                continue;
            }

            num_legal_moves += 1;
            position.make_move((&mv).to_owned());
            let mut score: isize;
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
                        //Record new killer moves
                        self.update_killers(depth, (&mv).to_owned());
                        //Beta cutoff, store in transpositon table
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

                    //History heuristic
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
                    let current_depth = self.current_searching_depth - depth;
                    Ok(-99999 + current_depth as isize)
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


    fn quiesce(&mut self, position: &mut Position, eval: &mut Evaluator, movegen: &MoveGenerator, depth: Depth, mut alpha: isize, beta: isize) -> Result<isize, SearcherError> {
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
    fn sort_moves_by_score<I: Iterator<Item=Move>>(&mut self, eval: &mut Evaluator, moves: I, position: &mut Position, depth: Depth) -> Vec<(usize, Move)> {
        let mut moves_and_score:Vec<(usize, Move)> = moves.map(|mv| {
            (eval.score_move(&self.history_moves, &self.killer_moves[depth as usize], position, &mv), mv)
        }).collect();

        //Assign PV/hash moves to usize::MAX
        if let Some(entry) = transposition_table().retrieve(position.get_zobrist()) {
            let best_move = &entry.mv;
            for i in 0..moves_and_score.len() {
                if moves_and_score[i].1 == *best_move {
                    moves_and_score[i] = (usize::MAX, moves_and_score[i].1);
                    break;
                }
            }
        }
        
        // Sort moves by decreasing score
        moves_and_score.sort_unstable_by(|a, b| b.0.cmp(&a.0));
        
        moves_and_score
    }

    #[inline]
    fn try_null_move(&mut self, position: &mut Position, eval: &mut Evaluator, movegen: &MoveGenerator, depth: Depth, _alpha: isize, beta: isize, end_time: &Instant) -> Result<Option<isize>, SearcherError> {
        
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

#[inline]
fn transposition_table() -> &'static mut TranspositionTable {
    unsafe {
        // All threads can access the transposition table. Each row is protected by a lock.
        TRANSPOSITION_TABLE.as_mut().unwrap()
    }
}