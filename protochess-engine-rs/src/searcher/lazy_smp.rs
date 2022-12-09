use std::collections::VecDeque;
use std::sync::atomic::{Ordering::Relaxed};
use instant::{Instant, Duration};

use crate::{Engine, Position};
use crate::types::{Move, Depth, SearchResult, SearchError};

use super::{Searcher, init_globals, GLOBAL_DEPTH, SEARCH_ID, CURRENT_POOL_ID};

// This file contains the multi-threaded search using Lazy SMP, which uses the alphabeta() function from alphabeta.rs

impl Searcher {
    pub fn get_best_move(engine: &Engine, depth: Depth) -> SearchResult {
        // Cannot use u64::MAX due to overflow, 1_000_000 seconds is 11.5 days
        Searcher::get_best_move_impl(engine, depth, 1_000_000)
    }

    pub fn get_best_move_timeout(engine: &Engine, time_sec: u64) -> SearchResult {
        Searcher::get_best_move_impl(engine, Depth::MAX, time_sec)
    }
    
    fn get_best_move_impl(engine: &Engine, max_depth: Depth, time_sec: u64) -> SearchResult {
        // Initialize the global structures
        init_globals();
        
        // Init threads, store handles in a queue
        let num_threads = engine.thread_handler.num_threads();
        let mut handles = VecDeque::with_capacity(num_threads as usize);
        for thread_id in 0..num_threads {
            // For each thread, create a local copy of the heuristics
            let mut pos_copy = engine.position.clone();
            let pool_id = unsafe { CURRENT_POOL_ID };
            let h = engine.thread_handler.spawn(move || {
                let mut searcher = Searcher::new();
                searcher.search_thread(thread_id, pool_id, num_threads, &mut pos_copy, max_depth, time_sec)
            });
            handles.push_back(h);
        }
        
        // Wait for threads to finish
        let mut best_move = Move::null();
        let mut best_depth = 0;
        let mut best_backup = None;
        loop {
            let h = handles.pop_front().unwrap();
            // Poll all threads for results
            if !h.is_finished() {
                handles.push_back(h);
                engine.thread_handler.sleep(Duration::from_millis(50));
                continue;
            }
            match h.join().unwrap() {
                // If any thread returns a checkmate or stalemate, drop the other threads and return the result
                SearchResult::Checkmate(p) => {
                    return SearchResult::Checkmate(p);
                },
                SearchResult::Stalemate => {
                    return SearchResult::Stalemate;
                },
                // Else, update the global best move with this thread's best move
                SearchResult::BestMove(mv, depth, backup) => {
                    // If any thread reaches the target depth, drop the other threads and return the move
                    if depth == max_depth {
                        best_move = mv;
                        best_depth = depth;
                        best_backup = backup;
                        break;
                    }
                    // Prefer deepest search
                    if depth > best_depth {
                        best_move = mv;
                        best_depth = depth;
                        best_backup = backup;
                    }
                },
            }
            if handles.is_empty() {
                break;
            }
        }
        // Before returning the best move, signal to remaining threads that they are invalid
        unsafe { CURRENT_POOL_ID += 1; }
        if num_threads > 1 {
            println!("Best move {} at depth {}", best_move, best_depth);
        }
        SearchResult::BestMove(best_move, best_depth, best_backup)
    }
    
    // Run for some time, then return the best move, its score, and the depth
    fn search_thread(&mut self, thread_id: u32, pool_id: u32, num_threads: u32, pos: &mut Position, max_depth: Depth, time_sec: u64) -> SearchResult {
        let end_time = Instant::now() + Duration::from_secs(time_sec);
        
        let mut best_move: Move = Move::null();
        let mut backup_move: Option<Move> = None;
        let mut best_depth: Depth = 0;
        
        // At the start, each thread should search a different depth (between 1 and max_depth, inclusive)
        let mut local_depth = (thread_id as Depth % max_depth) + 1;
        
        loop {
            self.nodes_searched = 0;
            self.current_searching_depth = local_depth;
            match super::alphabeta(self, pos, local_depth, &end_time) {
                Ok(_) => {
                    // This should not happen, scores are only passad between inner nodes
                    panic!("Search thread returned a score instead of a move");
                },
                Err(SearchError::BestMove(mv, score, backup)) => {
                    // This is not an error, but a signal to return the best move
                    best_move = mv;
                    best_depth = local_depth;
                    if backup.is_some() {
                        backup_move = backup;
                    }
                    //Print PV info
                    println!("Depth {:<2} {}. Score: {:<5}, nodes: {}", local_depth, mv, score, self.nodes_searched);
                    if backup.is_some() {
                        println!("Backup move: {}", backup.unwrap());
                    }
                },
                Err(SearchError::Timeout) => {
                    // Thread timed out, return the best move found so far
                    break;
                },
                Err(SearchError::Checkmate) => {
                    assert!(best_depth == 0);
                    let losing_player = pos.whos_turn;
                    return SearchResult::Checkmate(losing_player);
                },
                Err(SearchError::Stalemate) => {
                    assert!(best_depth == 0);
                    return SearchResult::Stalemate;
                }
            }
            if unsafe { CURRENT_POOL_ID } != pool_id {
                // This thread has been invalidated by a new search
                break;
            }
            
            // Set the global depth to max(local_depth, global_depth)
            // GLOBAL_DEPTH contains the maximum depth searched by any thread
            let old_global_depth = unsafe { GLOBAL_DEPTH.fetch_max(local_depth, Relaxed) };
            
            // If time is up or any thread has searched to max_depth, return
            if Instant::now() >= end_time || local_depth == max_depth || old_global_depth == max_depth {
                // Signal to other threads that they can stop
                unsafe { GLOBAL_DEPTH.store(max_depth, Relaxed); }
                break;
            }
                        
            if num_threads == 1 {
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

        SearchResult::BestMove(best_move, best_depth, backup_move)
    }
}