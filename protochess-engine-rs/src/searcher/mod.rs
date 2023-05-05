use std::collections::BTreeSet;

#[cfg(feature = "parallel")]
use std::sync::{Arc, Mutex};
#[cfg(feature = "parallel")]
use std::sync::atomic::{Ordering, AtomicBool, AtomicU8};

use crate::types::{Move, Depth, Centipawns, SearchTimeout, ZobKey};
use crate::Position;

mod alphabeta;
mod search_result;
pub mod transposition_table;
pub mod eval;

use transposition_table::{TranspositionTable, TranspositionHandle};
pub use search_result::SearchResult;

const DEBUG_PRINT: bool = true;

#[derive(Debug, Clone)]
pub struct Searcher<'a> {
    // The position we are currently searching
    pos: Position,
    //We store two killer moves per ply,
    //indexed by killer_moves[depth][0/1]
    killer_moves: [[Move;2];256],
    //Indexed by history_moves[side2move][from][to]
    history_moves: [[Centipawns;256];256],
    transposition_table: TranspositionHandle,
    // Stats
    nodes_searched: u64,
    max_searching_depth: Depth,
    timeout_flag: &'a bool,
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

impl<'a> Searcher<'a> {
    fn new(position: &Position, transposition_table: TranspositionHandle, timeout: &'a bool) -> Searcher<'a> {
        Searcher{
            pos: position.clone(),
            killer_moves: [[Move::null(); 2];256],
            history_moves: [[0;256];256],
            transposition_table,
            nodes_searched: 0,
            max_searching_depth: 0,
            timeout_flag: timeout,
            principal_variation: [Move::null(); Depth::MAX as usize + 1],
            known_checks: BTreeSet::new(),
            
            #[cfg(feature = "parallel")]
            thread_num: 0,
            #[cfg(feature = "parallel")]
            stop_flag: Arc::default(),
            #[cfg(feature = "parallel")]
            current_searched_depth: Arc::default(),
        }
    }
    
    pub fn get_best_move(position: &Position, depth: Depth, num_threads: u32, out: &mut SearchResult) {
        // Create a new copy of the heuristics for each search
        // Cannot use u64::MAX due to overflow, 1_000_000 seconds is 11.5 days
        let dummy_timeout = false;
        Searcher::get_best_move_impl(position, depth, &dummy_timeout, num_threads, out);
    }

    pub fn get_best_move_timeout(position: &Position, timeout: &bool, num_threads: u32, out: &mut SearchResult) {
        // Create a new copy of the heuristics for each search
        Searcher::get_best_move_impl(position, Depth::MAX, timeout, num_threads, out);
    }
    
    // Run for some time, then return the PV, the position score, and the depth
    fn get_best_move_impl(position: &Position, max_depth: Depth, timeout: &'a bool, num_threads: u32, out: &mut SearchResult) {
        // Limit the max depth to 127 to avoid overflow when doubling
        let max_depth = std::cmp::min(max_depth, 127);
        #[cfg(not(feature = "parallel"))] {
            assert!(num_threads == 1);
            let table = TranspositionTable::default();
            let mut on_result = |s: SearchResult| {
                if DEBUG_PRINT { println!("{s}"); };
                *out = s;
            };
            Searcher::new(position, table.into(), timeout).search(max_depth, &mut on_result);
        }
        #[cfg(feature = "parallel")] {
            Self::search_multi_thread(position, max_depth, timeout, num_threads, out);
        }
    }
    
    #[cfg(feature = "parallel")]
    fn search_multi_thread(position: &Position, max_depth: Depth, timeout: &bool, num_threads: u32, out: &mut SearchResult) {
        // Protect the output (mutable ref) with a mutex, and use an Arc to share it
        let out_arc = Arc::new(Mutex::new(out));
        // Arc pointers to global search state
        let stop_arc = Arc::new(AtomicBool::new(false));
        let depth_arc = Arc::new(AtomicU8::new(0));
        // Global transposition table
        let table = Arc::new(TranspositionTable::default());
        rayon::scope(|scope| {
            for thread_num in 0..num_threads {
                // Clone the pointers on each iteration
                let out_arc = out_arc.clone();
                let stop_arc = stop_arc.clone();
                let depth_arc = depth_arc.clone();
                let table = table.clone();
                // Spawn a new task in the thread pool, take ownership of the pointers
                scope.spawn(move |_scope| {
                    // Create a new searcher (with cloned position) for each thread
                    let mut searcher = Searcher::new(position, table.into(), timeout);
                    searcher.thread_num = thread_num;
                    searcher.stop_flag = stop_arc;
                    searcher.current_searched_depth = depth_arc;
                    
                    let mut on_result = |s: SearchResult| {
                        let mut out = out_arc.lock().expect("Mutex is poisoned");
                        if DEBUG_PRINT { println!("{s}"); };
                        // Update best result
                        if s > **out { **out = s; }
                    };
                    searcher.search(max_depth, &mut on_result);
                });
            }
        });
    }
    
    
    /// Start the search, stopping when `max_depth` is reached or `self.timeout_flag` is set to true.
    /// For each depth, call `on_result` with the current PV, score, and depth.
    fn search(&mut self, max_depth: Depth, on_result: &mut dyn FnMut(SearchResult)) {
        let mut pv = Vec::with_capacity(max_depth as usize);
        self.known_checks.clear();
        
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
            match self.start_alphabeta(search_depth, &pv) {
                Ok(score) => {
                    // Update the current searched depth
                    #[cfg(feature = "parallel")] {
                        self.current_searched_depth.fetch_max(search_depth, Ordering::Relaxed);
                    }
                    // If there have been transposition table hits, the new pv won't be complete.
                    // It might be shorter than the previous pv, in which case we can keep the old pv
                    // as long as it's consistent with the new pv.
                    let mut new_pv_len = 0;
                    let mut consistent = true;
                    for mv in self.principal_variation {
                        if mv.is_null() { break; }
                        new_pv_len += 1;
                        if pv.len() >= new_pv_len && pv[new_pv_len-1] != mv {
                            consistent = false;
                            break;
                        }
                    }
                    if new_pv_len >= pv.len() || !consistent {
                        // Copy the new pv into a vector
                        pv.clear();
                        for mv in self.principal_variation {
                            if mv.is_null() { break; }
                            pv.push(mv);
                        }
                    }
                    // Clean up the temporary space for the new pv
                    for i in 0..self.max_searching_depth {
                        self.principal_variation[i as usize] = Move::null();
                    }
                    // Return PV info
                    on_result(SearchResult{
                        pv: pv.clone(),
                        best_move_str: if pv.is_empty() { String::new() } else { pv[0].to_string() },
                        score,
                        depth: search_depth,
                        nodes_searched: self.nodes_searched,
                        #[cfg(feature = "parallel")]
                        thread_num: self.thread_num,
                    });
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
            
            if *self.timeout_flag || search_depth == max_depth {
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
                let next_depth = self.current_searched_depth.load(Ordering::Relaxed) + 1;
                search_depth = std::cmp::min(next_depth, max_depth);
            }
        }
    }
}

