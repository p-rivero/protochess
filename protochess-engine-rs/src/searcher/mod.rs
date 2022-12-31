use instant::{Instant, Duration};

use crate::types::{Move, Depth, Centipawns, SearchError};
use crate::{Position, MoveGen};

mod alphabeta;
mod transposition_table;
pub mod eval;

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
    
    pub fn get_best_move(position: &Position, depth: Depth) -> (Move, Depth) {
        // Create a new copy of the heuristics for each search
        // Cannot use u64::MAX due to overflow, 1_000_000 seconds is 11.5 days
        Searcher::new().get_best_move_impl(&mut position.clone(), depth, 1_000_000)
    }

    pub fn get_best_move_timeout(position: &Position, time_sec: u64) -> (Move, Depth) {
        // Create a new copy of the heuristics for each search
        Searcher::new().get_best_move_impl(&mut position.clone(), Depth::MAX, time_sec)
    }
    
    // Run for some time, then return the best move, its score, and the depth
    fn get_best_move_impl(&mut self, pos: &mut Position, max_depth: Depth, time_sec: u64) -> (Move, Depth) {
        assert!(!pos.leader_is_captured(), "Attempting to get best move but leader is captured");
        assert!(MoveGen::count_legal_moves(pos) != 0, "Attempting to get best move but there are no legal moves");
        
        let end_time = Instant::now() + Duration::from_secs(time_sec);
        let mut best_move: Move = Move::null();
        let mut best_depth: Depth = 0;
        
        // Iterative deepening
        for search_depth in 1..=max_depth {
            self.nodes_searched = 0;
            self.current_searching_depth = search_depth;
            match self.search(pos, search_depth, &end_time) {
                Ok(_) => {
                    // This should not happen, scores are only passad between inner nodes
                    panic!("Root call to alphabeta() returned a score instead of a move");
                },
                Err(SearchError::BestMove(mv, score)) => {
                    // This is not an error, but a signal to return the best move
                    best_move = mv;
                    best_depth = search_depth;
                    // Print PV info
                    let diff = -(score.abs() + alphabeta::GAME_OVER_SCORE);
                    if diff < 200 {
                        print!("[Mate in {}] ", (diff+1) / 2);
                    }
                    println!("Depth {:<2} {}. Score: {:<5}, nodes: {}", search_depth, mv, score, self.nodes_searched);
                },
                Err(SearchError::Timeout) => {
                    // Thread timed out, return the best move found so far
                    break;
                },
            }

            if Instant::now() >= end_time {
                // Return the best move found so far
                break;
            }
        }

        (best_move, best_depth)
    }
}
