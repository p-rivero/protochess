use instant::{Instant, Duration};

use crate::types::{Move, Depth, Centipawns};
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
    principal_variation: [Move; Depth::MAX as usize + 1],
}

impl Searcher {
    fn new() -> Searcher {
        Searcher{
            killer_moves: [[Move::null(); 2];64],
            history_moves: [[0;256];256],
            transposition_table: TranspositionTable::new(),
            nodes_searched: 0,
            current_searching_depth: 0,
            principal_variation: [Move::null(); Depth::MAX as usize + 1],
        }
    }
    
    pub fn get_best_move(position: &Position, depth: Depth) -> (Vec<Move>, Centipawns, Depth) {
        // Create a new copy of the heuristics for each search
        // Cannot use u64::MAX due to overflow, 1_000_000 seconds is 11.5 days
        Searcher::new().get_best_move_impl(&mut position.clone(), depth, 1_000_000)
    }

    pub fn get_best_move_timeout(position: &Position, time_sec: u64) -> (Vec<Move>, Centipawns, Depth) {
        // Create a new copy of the heuristics for each search
        Searcher::new().get_best_move_impl(&mut position.clone(), Depth::MAX, time_sec)
    }
    
    // Run for some time, then return the PV, the position score, and the depth
    fn get_best_move_impl(&mut self, pos: &mut Position, max_depth: Depth, time_sec: u64) -> (Vec<Move>, Centipawns, Depth) {
        assert!(!pos.leader_is_captured(), "Attempting to get best move but leader is captured");
        assert!(MoveGen::count_legal_moves(pos) != 0, "Attempting to get best move but there are no legal moves");
        
        let end_time = Instant::now() + Duration::from_secs(time_sec);
        let mut pv = Vec::with_capacity(max_depth as usize);
        let mut pv_depth: Depth = 0;
        let mut pv_score: Centipawns = 0;
        
        // Iterative deepening
        for search_depth in 1..=max_depth {
            self.nodes_searched = 0;
            self.current_searching_depth = search_depth;
            match self.search(pos, search_depth, &end_time) {
                Ok(score) => {
                    pv.clear();
                    for i in 0..search_depth {
                        pv.push(self.principal_variation[i as usize]);
                    }
                    pv_depth = search_depth;
                    pv_score = score;
                    // Print PV info
                    let diff = -(score.abs() + alphabeta::GAME_OVER_SCORE);
                    if diff < 200 {
                        print!("[Mate in {}] ", (diff+1) / 2);
                    }
                    println!("Depth {:<2}. Score: {:<4}, nodes: {}", search_depth, score, self.nodes_searched);
                    print!("  PV: ");
                    for m in &pv {
                        print!("{} ", m);
                    }
                    println!();
                },
                Err(_) => {
                    // Thread timed out, return the best move found so far
                    break;
                },
            }

            if Instant::now() >= end_time {
                // Return the best move found so far
                break;
            }
        }
        (pv, pv_score, pv_depth)
    }
}
