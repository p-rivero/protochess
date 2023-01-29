use std::collections::BTreeSet;

#[cfg(feature = "parallel")]
use std::sync::{Arc, Mutex};
#[cfg(feature = "parallel")]
use std::sync::atomic::{Ordering, AtomicBool, AtomicU8};

use instant::{Instant, Duration};

use crate::types::{Move, Depth, Centipawns, SearchTimeout, ZobKey};
use crate::Position;

mod alphabeta;
mod transposition_table;
pub mod eval;

use transposition_table::{TranspositionTable, TranspositionHandle};

#[derive(Debug, Clone)]
pub struct Searcher {
    // The position we are currently searching
    pos: Position,
    //We store two killer moves per ply,
    //indexed by killer_moves[depth][0] or killer_moves[depth][0]
    killer_moves: [[Move;2];64],
    //Indexed by history_moves[side2move][from][to]
    history_moves: [[Centipawns;256];256],
    transposition_table: TranspositionHandle,
    // Stats
    nodes_searched: u64,
    max_searching_depth: Depth,
    end_time: Instant,
    principal_variation: [Move; Depth::MAX as usize + 1],
    known_checks: BTreeSet<ZobKey>,
    
    // Attributes for parallel search
    #[cfg(feature = "parallel")]
    thread_num: u32,
    #[cfg(feature = "parallel")]
    stop_flag: Arc<AtomicBool>,
    #[cfg(feature = "parallel")]
    current_searched_depth: Arc<AtomicU8>,
}

type SearchRes = (Vec<Move>, Centipawns, Depth);

impl Searcher {
    fn new(position: &Position, transposition_table: TranspositionHandle) -> Searcher {
        Searcher{
            pos: position.clone(),
            killer_moves: [[Move::null(); 2];64],
            history_moves: [[0;256];256],
            transposition_table,
            nodes_searched: 0,
            max_searching_depth: 0,
            end_time: Instant::now(),
            principal_variation: [Move::null(); Depth::MAX as usize + 1],
            known_checks: BTreeSet::new(),
            
            #[cfg(feature = "parallel")]
            thread_num: 0,
            #[cfg(feature = "parallel")]
            stop_flag: Default::default(),
            #[cfg(feature = "parallel")]
            current_searched_depth: Default::default(),
        }
    }
    
    pub fn get_best_move(position: &Position, depth: Depth, num_threads: u32) -> SearchRes {
        // Create a new copy of the heuristics for each search
        // Cannot use u64::MAX due to overflow, 1_000_000 seconds is 11.5 days
        Searcher::get_best_move_impl(position, depth, 1_000_000, num_threads)
    }

    pub fn get_best_move_timeout(position: &Position, time_sec: u64, num_threads: u32) -> SearchRes {
        // Create a new copy of the heuristics for each search
        Searcher::get_best_move_impl(position, Depth::MAX, time_sec, num_threads)
    }
    
    // Run for some time, then return the PV, the position score, and the depth
    fn get_best_move_impl(position: &Position, max_depth: Depth, time_sec: u64, num_threads: u32) -> SearchRes {
        // Limit the max depth to 127 to avoid overflow when doubling
        let max_depth = std::cmp::min(max_depth, 127);
        #[cfg(not(feature = "parallel"))] {
            assert!(num_threads == 1);
            let table = TranspositionTable::new();
            Searcher::new(position, table.into()).search(max_depth, time_sec)
        }
        #[cfg(feature = "parallel")] {
            Self::search_multi_thread(position, max_depth, time_sec, num_threads)
        }
    }
    
    #[cfg(feature = "parallel")]
    fn search_multi_thread(position: &Position, max_depth: Depth, time_sec: u64, num_threads: u32) -> SearchRes {
        // Arc pointer to a vector of results
        let res = vec![Default::default(); num_threads as usize];
        let results_arc = Arc::new(Mutex::new(res));
        // Arc pointers to global search state
        let stop_arc = Arc::new(AtomicBool::new(false));
        let depth_arc = Arc::new(AtomicU8::new(0));
        // Global transposition table
        let table = Arc::new(TranspositionTable::new());
        rayon::scope(|scope| {
            for thread_num in 0..num_threads {
                // Clone the pointers on each iteration
                let results_arc = results_arc.clone();
                let stop_arc = stop_arc.clone();
                let depth_arc = depth_arc.clone();
                let table = table.clone();
                // Spawn a new task in the thread pool, take ownership of the pointers
                scope.spawn(move |_scope| {
                    // Create a new searcher (with cloned position) for each thread
                    let mut searcher = Searcher::new(position, table.into());
                    searcher.thread_num = thread_num;
                    searcher.stop_flag = stop_arc;
                    searcher.current_searched_depth = depth_arc;
                    let thread_result = searcher.search(max_depth, time_sec);
                    // When the thread is done, store the result in the results vector
                    let mut results_vec = results_arc.lock().unwrap();
                    results_vec[thread_num as usize] = thread_result;
                });
            }
        });
        let mut best_pv = Vec::new();
        let mut best_score = -Centipawns::MAX;
        let mut best_depth = 0;
        // Consume the results vector, return the best result (prefer higher depth, then higher score, then longer PV)
        let results_mutex = Arc::try_unwrap(results_arc).expect("Arc still has owners");
        let results_vec = results_mutex.into_inner().expect("Mutex is poisoned");
        for (pv, score, depth) in results_vec.into_iter() {
            if depth > best_depth ||
                (depth == best_depth && score > best_score) ||
                (depth == best_depth && score == best_score && pv.len() > best_pv.len())
            {
                best_score = score;
                best_depth = depth;
                best_pv = pv;
            }
        }
        (best_pv, best_score, best_depth)
    }
    
    fn search(&mut self, max_depth: Depth, time_sec: u64) -> SearchRes {
        let mut pv = Vec::with_capacity(max_depth as usize);
        let mut pv_score: Centipawns = 0;
        let mut pv_depth: Depth = 0;
        self.known_checks.clear();
        self.end_time = Instant::now() + Duration::from_secs(time_sec);
        
        let mut search_depth;
        #[cfg(not(feature = "parallel"))] {
            search_depth = 1; // In single-threaded search, start at depth 1
        }
        // When using multiple threads, start threads at different depths
        #[cfg(feature = "parallel")] {
            search_depth = self.thread_num.trailing_ones() as Depth + 1;
            // Limit the depth to the max depth
            search_depth = std::cmp::min(search_depth, max_depth);
            // If another thread has already searched this depth, skip it
            search_depth = std::cmp::max(search_depth, self.current_searched_depth.load(Ordering::Relaxed));
        }
        
        // Iterative deepening search
        loop {
            self.nodes_searched = 0;
            self.max_searching_depth = 2 * search_depth;
            match self.start_alphabeta(search_depth) {
                Ok(score) => {
                    // Update the current searched depth
                    #[cfg(feature = "parallel")] {
                        self.current_searched_depth.fetch_max(search_depth, Ordering::Relaxed);
                    }
                    pv.clear();
                    // Copy the pv into a vector
                    for mv in self.principal_variation {
                        if mv.is_null() {
                            break;
                        }
                        pv.push(mv);
                    }
                    // Clean up the pv
                    for i in 0..self.max_searching_depth {
                        self.principal_variation[i as usize] = Move::null();
                    }
                    pv_depth = search_depth;
                    pv_score = score;
                    // Print PV info
                    println!("{}", self.format_result(score, &pv, search_depth));
                },
                Err(SearchTimeout) => {
                    // Thread timed out, return the best move found so far
                    break;
                },
            }

            #[cfg(feature = "parallel")]
            if self.stop_flag.load(Ordering::Relaxed) {
                // Stop flag set, return the best move found so far
                break;
            }
            
            if Instant::now() >= self.end_time || search_depth == max_depth {
                // Set stop flag to stop other threads
                #[cfg(feature = "parallel")] {
                    self.stop_flag.store(true, Ordering::Relaxed);
                }
                // Return the best move found so far
                break;
            }
            
            #[cfg(not(feature = "parallel"))] {
                search_depth += 1;
            }
            #[cfg(feature = "parallel")] {
                // TODO: Variable increment? (If num_threads == 1, increment by 1)
                let next_depth = self.current_searched_depth.load(Ordering::Relaxed) + 1;
                search_depth = std::cmp::min(next_depth, max_depth);
            }
        }
        (pv, pv_score, pv_depth)
    }
    
    // Format the result as a string in order to print it all at once. This prevents 2 threads from printing at the same time.
    fn format_result(&self, score: i32, pv: &Vec<Move>, depth: Depth) -> String {
        #[cfg(feature = "parallel")]
        let thread_str = format!("T{:<2} ", self.thread_num);
        #[cfg(not(feature = "parallel"))]
        let thread_str = String::new();
        
        let diff = -(score.abs() + alphabeta::GAME_OVER_SCORE);
        let score_str = {
            if diff < 200 {
                let sign = if score > 0 { "" } else { "-" };
                format!("MATE {}{}", sign, (diff+1) / 2)
            } else {
                format!("cp {:<4}", score)
            }
        };
        let mut pv_str = String::new();
        for m in pv {
            pv_str.push_str(&format!("{} ", m));
        }
        
        format!("{thread_str}Depth {depth:<2} Score: {score_str} [nodes: {n}] PV: {pv_str}", n=self.nodes_searched)
    }
}

