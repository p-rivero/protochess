use crate::types::{Move, Depth, Centipawns};
use crate::transposition_table::TranspositionTable;


mod alphabeta;
mod lazy_smp;
use alphabeta::alphabeta;


pub(crate) struct Searcher {
    //We store two killer moves per ply,
    //indexed by killer_moves[depth][0] or killer_moves[depth][0]
    killer_moves: [[Move;2];64],
    //Indexed by history_moves[side2move][from][to]
    history_moves: [[Centipawns;256];256],
    transposition_table: TranspositionTable,
    // Stats
    nodes_searched: u64,
    current_searching_depth: Depth,
}

impl Searcher {
    fn new() -> Searcher {
        Searcher{
            killer_moves: [[Move::null(); 2];64],
            history_moves: [[0;256];256],
            transposition_table: TranspositionTable::new(),
            nodes_searched: 0,
            current_searching_depth: 0,
        }
    }
    // PUBLIC METHODS (defined in lazy_smp.rs):
    // get_best_move(position, eval, movegen, depth) -> SearchResult 
    // get_best_move_timeout(position, eval, movegen, time_sec) -> SearchResult
}
