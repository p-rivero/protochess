use std::sync::atomic::{AtomicU8, AtomicU64, Ordering::Relaxed};

use crate::types::{Move, Depth};
use crate::transposition_table::TranspositionTable;


mod alphabeta;
mod lazy_smp;
use alphabeta::alphabeta;


// Global structures, shared between threads
// Transposition table, accessed concurrently by all threads (lazy SMP)
static mut TRANSPOSITION_TABLE: Option<TranspositionTable> = None;
// Depth of the deepest search of the current threadpool
static mut GLOBAL_DEPTH: AtomicU8 = AtomicU8::new(0);
// Counter for the variable deepening search
static mut SEARCH_ID: AtomicU64 = AtomicU64::new(1);
// Threadpool id, so that threads know if their search is outdated
static mut CURRENT_POOL_ID: u32 = 0;



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
    fn new() -> Searcher {
        Searcher{
            killer_moves: [[Move::null(); 2];64],
            history_moves: [[0;256];256],
            nodes_searched: 0,
            current_searching_depth: 0,
        }
    }
    // PUBLIC METHODS (defined in lazy_smp.rs):
    // get_best_move(position, eval, movegen, depth) -> Result<(Move, Depth), GameResult> 
    // get_best_move_timeout(position, eval, movegen, time_sec) -> Result<(Move, Depth), GameResult>
}


#[inline]
fn init_globals() {
    unsafe {
        if TRANSPOSITION_TABLE.is_none() {
            TRANSPOSITION_TABLE = Some(TranspositionTable::new());
        } else {
            transposition_table().set_ancient();
        }
        GLOBAL_DEPTH.store(0, Relaxed);
        SEARCH_ID.store(1, Relaxed);
    }
}

#[inline]
fn transposition_table() -> &'static mut TranspositionTable {
    unsafe {
        // All threads can access the transposition table. Each row is protected by a lock.
        TRANSPOSITION_TABLE.as_mut().unwrap()
    }
}
