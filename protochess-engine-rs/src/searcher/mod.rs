use std::sync::atomic::{AtomicU8, AtomicU64, Ordering::Relaxed};
use std::thread;

use crate::types::chess_move::{Move, MoveType};
use crate::position::Position;
use crate::move_generator::MoveGenerator;
use crate::evaluator::Evaluator;
use crate::transposition_table::{TranspositionTable, Entry, EntryFlag};

// Global structures, shared between threads
static mut TRANSPOSITION_TABLE: Option<TranspositionTable> = None;
static mut GLOBAL_DEPTH: AtomicU8 = AtomicU8::new(0);
static mut THREAD_ID: AtomicU8 = AtomicU8::new(1);
static mut SEARCH_ID: AtomicU64 = AtomicU64::new(1);
static NUM_THREADS: usize = 4;

pub(crate) struct Searcher {
    //We store two killer moves per ply,
    //indexed by killer_moves[depth][0] or killer_moves[depth][0]
    killer_moves: [[Move;2];64],
    //Indexed by history_moves[side2move][from][to]
    history_moves: [[u16;256];256],
}

impl Searcher {
    pub fn get_best_move(position: &Position, eval: &Evaluator, movegen: &MoveGenerator, depth: u8) -> Option<(Move, u8)> {
        Searcher::get_best_move_impl(position, eval, movegen, depth, u64::MAX)
    }

    pub fn get_best_move_timeout(position: &Position, eval: &Evaluator, movegen: &MoveGenerator, time_sec: u64) -> Option<(Move, u8)> {
        Searcher::get_best_move_impl(position, eval, movegen, u8::MAX, time_sec)
    }
    
    fn new() -> Searcher {
        Searcher{
            killer_moves: [[Move::null(); 2];64],
            history_moves: [[0;256];256],
        }
    }
    
    fn get_best_move_impl(position: &Position, eval: &Evaluator, movegen: &MoveGenerator, max_depth: u8, time_sec: u64) -> Option<(Move, u8)> {
        
        // Initialize the global structures
        unsafe {
            if TRANSPOSITION_TABLE.is_none() {
                TRANSPOSITION_TABLE = Some(TranspositionTable::new());
            } else {
                transposition_table().set_ancient();
            }
            GLOBAL_DEPTH.store(0, Relaxed);
            THREAD_ID.store(1, Relaxed);
            SEARCH_ID.store(1, Relaxed);
        }
        
        // Init threads, store handles
        let mut handles = Vec::with_capacity(NUM_THREADS);
        for _ in 0..NUM_THREADS {
            let mut pos_copy = position.clone();
            let mut eval_copy = eval.clone();
            let movegen_copy = (*movegen).clone();
            let h = thread::spawn(move || {
                let mut searcher = Searcher::new();
                searcher.search_thread(&mut pos_copy, &mut eval_copy, &movegen_copy, max_depth, time_sec)
            });
            handles.push(h);
        }
        
        // Wait for threads to finish
        let mut best_move = None;
        let mut best_score = 0;
        let mut best_depth = 0;
        for h in handles {
            let (mv, score, depth) = h.join().unwrap();
            if depth > best_depth || (depth == best_depth && score > best_score) {
                best_move = Some(mv);
                best_score = score;
                best_depth = depth;
            }
        }
        match best_move {
            Some(mv) => { Some((mv, best_depth)) },
            None => { None }
        }
    }
    
    // Run for some time, then return the best move, its score, and the depth
    fn search_thread(&mut self, position: &mut Position, eval: &mut Evaluator, movegen: &MoveGenerator, max_depth: u8, time_sec: u64) -> (Move, isize, u8) {
        let start = instant::Instant::now();
        let max_time = instant::Duration::from_secs(time_sec);
        
        let mut best_move: Move;
        let mut best_score: isize;
        let mut best_depth: u8;
        
        let thread_id = unsafe { THREAD_ID.fetch_add(1, Relaxed) };
        // At the start, each thread should search a different depth (between 1 and max_depth, inclusive)
        let mut local_depth = (thread_id % max_depth) + 1;
        // 1/2 threads search 1 ply deeper, 1/4 threads search 2 ply deeper, etc.
        let depth_increment = 1 + thread_id.trailing_zeros() as u8;
        println!("Thread {} starting at depth {} with increment {}", thread_id, local_depth, depth_increment);
        
        //Iterative deepening
        loop {
            best_score = self.alphabeta(position, eval, movegen, local_depth, -isize::MAX, isize::MAX, true);
            best_move = transposition_table().retrieve(position.get_zobrist()).unwrap().mv;
            best_depth = local_depth;

            //Print PV info
            println!("thread {}, score:{} depth: {}", thread_id, best_score, best_depth);
            
            // Set the global depth to max(local_depth, global_depth)
            // GLOBAL_DEPTH contains the maximum depth searched by any thread
            let old_global_depth = unsafe { GLOBAL_DEPTH.fetch_max(local_depth, Relaxed) };
            
            // If time is up or any thread has searched to max_depth, return
            if start.elapsed() >= max_time || local_depth == max_depth || old_global_depth == max_depth {
                // Signal to other threads that they can stop
                unsafe { GLOBAL_DEPTH.store(max_depth, Relaxed); }
                break;
            }
                        
            // Update local_depth: set to GLOBAL_DEPTH + offset
            local_depth = unsafe { GLOBAL_DEPTH.load(Relaxed) } + depth_increment;
            // Limit local_depth to max_depth
            local_depth = std::cmp::min(local_depth, max_depth);
        }

        (best_move.to_owned(), best_score, best_depth)
    }

    fn alphabeta(&mut self, position: &mut Position, eval: &mut Evaluator, movegen: &MoveGenerator, depth: u8, mut alpha: isize, beta: isize, do_null: bool) -> isize {

        if depth <= 0 {
            return self.quiesce(position, eval, movegen, 0, alpha, beta);
        }

        let is_pv = alpha != beta - 1;
        if let Some(entry) = transposition_table().retrieve(position.get_zobrist()) {
            if entry.depth >= depth {
                match entry.flag {
                    EntryFlag::EXACT => {
                        if entry.value < alpha {
                            return alpha;
                        }
                        if entry.value >= beta{
                            return beta;
                        }
                        return entry.value;
                    }
                    EntryFlag::BETA => {
                        if !is_pv && beta <= entry.value {
                            return beta;
                        }
                    }
                    EntryFlag::ALPHA => {
                        if !is_pv && alpha >= entry.value {
                            return alpha;
                        }
                    }
                    EntryFlag::NULL => {}
                }
            }
        }

        //Null move pruning
        if !is_pv && do_null {
            if let Some(beta) = self.try_null_move(position, eval, movegen, depth, alpha, beta) {
                return beta;
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
                score = -self.alphabeta(position, eval, movegen,
                                        depth - 1, -beta, -alpha, true);
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
                    score = -self.alphabeta(position, eval, movegen,
                                            reduced_depth, -alpha - 1, -alpha, true);
                } else {
                    //Cannot reduce, proceed with standard PVS
                    score = alpha + 1;
                }

                if score > alpha {
                    //PVS
                    //Null window search
                    score = -self.alphabeta(position, eval, movegen,
                                            depth - 1, -alpha - 1, -alpha, true);
                    //Re-search if necessary
                    if score > alpha && score < beta {
                        score = -self.alphabeta(position, eval, movegen,
                                                depth - 1, -beta, -alpha, true);
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
                        return beta;
                    }
                    alpha = score;

                    //History heuristic
                    self.update_history_heuristic(depth, &mv);
                }
            }
        }

        if num_legal_moves == 0 {
            return if in_check {
                //Checkmate
                -99999
            } else {
                //Stalemate
                0
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
        alpha
    }


    fn quiesce(&mut self, position: &mut Position, eval: &mut Evaluator, movegen: &MoveGenerator, depth:u8, mut alpha: isize, beta: isize) -> isize {
        let score = eval.evaluate(position, movegen);
        if score >= beta{
            return beta;
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
            let score = -self.quiesce(position, eval, movegen, depth, -beta, -alpha);
            position.unmake_move();

            if score >= beta {
                return beta;
            }
            if score > alpha {
                alpha = score;
                // best_move = mv;
            }
        }

        alpha
    }

    #[inline]
    fn update_killers(&mut self, depth: u8, mv: Move) {
        if !mv.get_is_capture() {
            if mv != self.killer_moves[depth as usize][0] && mv != self.killer_moves[depth as usize][1] {
                self.killer_moves[depth as usize][1] = self.killer_moves[depth as usize][0];
                self.killer_moves[depth as usize][0] = mv;
            }
        }
    }

    #[inline]
    fn update_history_heuristic(&mut self, depth: u8, mv:&Move) {
        if !mv.get_is_capture() {
            self.history_moves
                [mv.get_from() as usize]
                [mv.get_to() as usize] += depth as u16;
        }
    }

    #[inline]
    fn sort_moves_by_score<I: Iterator<Item=Move>>(&mut self, eval: &mut Evaluator, moves: I, position: &mut Position, depth: u8) -> Vec<(usize, Move)> {
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
    fn try_null_move(&mut self, position: &mut Position, eval: &mut Evaluator, movegen: &MoveGenerator, depth: u8, _alpha: isize, beta: isize) -> Option<isize> {
        
        if depth > 3 && eval.can_do_null_move(position) && !movegen.in_check(position) {
            position.make_move(Move::null());
            let nscore = -self.alphabeta(position,eval, movegen, depth - 3, -beta, -beta + 1, false);
            position.unmake_move();
            if nscore >= beta {
                return Some(beta);
            }
        }
        None
    }

}

#[inline]
fn transposition_table() -> &'static mut TranspositionTable {
    unsafe {
        // All threads can access the transposition table. Each row is protected by a lock.
        TRANSPOSITION_TABLE.as_mut().unwrap()
    }
}