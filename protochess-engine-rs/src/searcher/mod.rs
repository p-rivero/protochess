use instant::{Instant, Duration};

use crate::types::{Move, Depth, Centipawns, SearchResult, SearchError};
use crate::Position;

mod alphabeta;
mod transposition_table;

use transposition_table::TranspositionTable;

pub struct Searcher {
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
    
    pub fn get_best_move(position: &Position, depth: Depth) -> SearchResult {
        // Create a new copy of the heuristics for each search
        // Cannot use u64::MAX due to overflow, 1_000_000 seconds is 11.5 days
        Searcher::new().get_best_move_impl(&mut position.to_owned(), depth, 1_000_000)
    }

    pub fn get_best_move_timeout(position: &Position, time_sec: u64) -> SearchResult {
        // Create a new copy of the heuristics for each search
        Searcher::new().get_best_move_impl(&mut position.to_owned(), Depth::MAX, time_sec)
    }
    
    // Run for some time, then return the best move, its score, and the depth
    fn get_best_move_impl(&mut self, pos: &mut Position, max_depth: Depth, time_sec: u64) -> SearchResult {
        let end_time = Instant::now() + Duration::from_secs(time_sec);
        
        let mut best_move: Move = Move::null();
        let mut backup_move: Option<Move> = None;
        let mut best_depth: Depth = 0;
        
        // Iterative deepening
        let mut search_depth = 1;
        loop {
            self.nodes_searched = 0;
            self.current_searching_depth = search_depth;
            match self.search(pos, search_depth, &end_time) {
                Ok(_) => {
                    // This should not happen, scores are only passad between inner nodes
                    panic!("Root call to alphabeta() returned a score instead of a move");
                },
                Err(SearchError::BestMove(mv, score, backup)) => {
                    // This is not an error, but a signal to return the best move
                    best_move = mv;
                    best_depth = search_depth;
                    if backup.is_some() {
                        backup_move = backup;
                    }
                    // Print PV info
                    println!("Depth {:<2} {}. Score: {:<5}, nodes: {}", search_depth, mv, score, self.nodes_searched);
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
            
            if Instant::now() >= end_time || search_depth == max_depth {
                // Return the best move found so far
                break;
            }
            search_depth += 1;
        }

        SearchResult::BestMove(best_move, best_depth, backup_move)
    }
}
