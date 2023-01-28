#[macro_use]
extern crate lazy_static;
extern crate impl_ops;

pub mod piece;
pub mod move_generator;
pub mod types;
pub mod position;
pub mod searcher;
pub mod utils;

use std::convert::TryFrom;

use types::{BCoord, Centipawns, Depth, Player};
use searcher::{Searcher, eval};
use utils::to_index;

pub use position::Position;
pub use position::game_state::{PiecePlacement, GameState};
pub use position::global_rules::GlobalRules;
pub use move_generator::MoveGen;
pub use piece::{Piece, PieceId, PieceDefinition};
pub use types::MoveInfo;

#[derive(Debug, PartialEq, Eq, Clone, Copy)]
#[must_use]
pub enum MakeMoveResult {
    Ok,
    IllegalMove,
    Checkmate{winner: Player},
    LeaderCaptured{winner: Player},
    PieceInWinSquare{winner: Player},
    CheckLimit{winner: Player},
    Stalemate{winner: Option<Player>},
    Repetition,
}

/// Starting point for the engine
#[derive(Debug, Clone)]
pub struct Engine{
    position: Position,
    num_threads: u32,
}

impl Engine {
    /// Initializes a new engine from a given fen string
    pub fn from_fen(fen: &str) -> wrap_res!(Engine) {
        let state = GameState::from_fen(fen)?;
        let position = Position::try_from(state)?;
        let num_threads = Self::get_max_threads();
        Ok(Engine{ position, num_threads })
    }

    pub fn set_state(&mut self, state: GameState) -> wrap_res!() {
        self.position = Position::try_from(state)?;
        Ok(())
    }
    
    pub fn get_state(&self) -> GameState {
        (&self.position).into()
    }

    /// Returns the zobrist hash key for the current position
    pub fn get_zobrist(&self) -> u64 {
        self.position.get_zobrist()
    }

    /// Returns the score of the current position for the side to move
    pub fn get_score(&self) -> Centipawns {
        eval::evaluate(&self.position)
    }
    
    /// Returns the id of the piece at the given coordinates
    pub fn get_piece_at(&self, position: (BCoord, BCoord)) -> Option<PieceId> {
        self.position.piece_at(to_index(position.0, position.1)).map(Piece::get_piece_id)
    }
    
    /// Returns the character representation of the piece with a given id for the player to move
    pub fn get_piece_char(&self, piece_id: PieceId) -> Option<char> {
        self.position.get_piece_char(self.whos_turn(), piece_id)
    }

    /// Adds a new piece on the board. If the piece is not used for castling, `has_moved` is ignored.
    pub fn add_piece(&mut self, owner: Player, piece_type: PieceId, x: BCoord, y: BCoord, has_moved: bool) -> wrap_res!() {
        self.position.public_add_piece(owner, piece_type, to_index(x,y), !has_moved)?;
        Ok(())
    }

    /// Removes a piece on the board, if it exists
    pub fn remove_piece(&mut self, x: BCoord, y: BCoord) -> wrap_res!() {
        err_assert!(self.position.in_bounds(x, y), "Coordinates ({x}, {y}) are out of bounds");
        self.position.public_remove_piece(to_index(x,y))?;
        Ok(())
    }

    /// Attempts a move on the current board position
    pub fn make_move(&mut self, target_move: &MoveInfo) -> MakeMoveResult {
        let moves = MoveGen::get_pseudo_moves(&mut self.position, true);
        for mv in moves {
            if target_move != &mv {
                continue;
            }
            // Found the move, try to play it
            if !MoveGen::make_move_if_legal(mv, &mut self.position) {
                continue;
            }
            
            // Check if the game is over
            let winner = {
                if self.position.global_rules.invert_win_conditions {
                    self.position.whos_turn
                } else {
                    1 - self.position.whos_turn
                }
            };
            // Leader captured (atomic chess)
            if self.position.leader_is_captured() {
                return MakeMoveResult::LeaderCaptured{winner};
            }
            // Piece moved to winning square (king of the hill, racing kings)
            if self.position.piece_is_on_winning_square() {
                return MakeMoveResult::PieceInWinSquare{winner};
            }
            let in_check = MoveGen::in_check(&mut self.position);
            // No legal moves, check if it's checkmate or stalemate
            if MoveGen::count_legal_moves(&mut self.position) == 0 {
                if in_check {
                    return MakeMoveResult::Checkmate{winner};
                }
                if self.position.global_rules.stalemated_player_loses {
                    return MakeMoveResult::Stalemate{winner: Some(winner)};
                }
                return MakeMoveResult::Stalemate{winner: None};
            }
            if in_check && self.position.increment_num_checks() {
                // Checked N times (N=3 in 3-check)
                return MakeMoveResult::CheckLimit { winner };
            }
            if self.position.draw_by_repetition() {
                // Threefold Repetition
                return MakeMoveResult::Repetition;
            }
            return MakeMoveResult::Ok;
        }
        MakeMoveResult::IllegalMove
    }
    
    pub fn make_move_str(&mut self, target_move: &str) -> wrap_res!(MakeMoveResult) {
        let mv = MoveInfo::try_from(target_move)?;
        Ok(self.make_move(&mv))
    }

    /// Undoes the most recent move on the current board position
    pub fn undo(&mut self) -> wrap_res!() {
        if !self.position.can_unmake_move() {
            return Err("There is no move to undo".to_string());
        }
        self.position.unmake_move();
        Ok(())
    }
    
    pub fn whos_turn(&self) -> Player {
        self.position.whos_turn
    }
    
    pub fn get_width(&self) -> BCoord {
        self.position.dimensions.width
    }
    
    pub fn get_height(&self) -> BCoord {
        self.position.dimensions.height
    }
    
    /// Returns the best move for the current position, along with the evaluation score
    pub fn get_best_move(&mut self, depth: Depth) -> wrap_res!(MoveInfo, Centipawns) {
        self.assert_position_is_valid()?;
        err_assert!(depth != 0, "Depth must be greater than 0");
        let (pv, score, search_depth) = Searcher::get_best_move(&self.position, depth, self.num_threads);
        err_assert!(search_depth == depth, "Search depth ({search_depth}) != requested depth ({depth})");
        err_assert!(pv.len() > 0, "No moves found");
        Ok((pv[0].into(), score))
    }

    /// Returns the best move for the current position, along with the evaluation score and the search depth
    pub fn get_best_move_timeout(&mut self, max_sec: u64) -> wrap_res!(MoveInfo, Centipawns, Depth) {
        self.assert_position_is_valid()?;
        let (pv, score, search_depth) = Searcher::get_best_move_timeout(&self.position, max_sec, self.num_threads);
        err_assert!(pv.len() > 0, "No moves found");
        Ok((pv[0].into(), score, search_depth))
    }
    fn assert_position_is_valid(&mut self) -> wrap_res!() {
        err_assert!(!self.position.leader_is_captured(), "Attempting to get best move but leader is captured");
        err_assert!(MoveGen::count_legal_moves(&mut self.position) != 0, "Attempting to get best move but there are no legal moves");
        Ok(())
    }

    /// Returns a list of all legal moves from the given square
    pub fn moves_from(&mut self, x: BCoord, y: BCoord) -> wrap_res!(Vec<MoveInfo>) {
        err_assert!(self.position.in_bounds(x, y), "Coordinates ({x}, {y}) are out of bounds");
        let from = to_index(x,y);
        let moves = MoveGen::get_legal_moves(&mut self.position)
            .into_iter()
            .filter(|mv| mv.get_from() == from)
            .map(MoveInfo::from)
            .collect();
        Ok(moves)
    }

    /// Returns true if the player to move is in check
    pub fn to_move_in_check(&mut self) -> bool {
        if self.position.leader_is_captured() {
            return false;
        }
        MoveGen::in_check(&mut self.position)
    }
    
    pub fn perft(&mut self, depth: Depth) -> u64 {
        utils::perft::perft(&mut self.position, depth)
    }
    pub fn perft_divide(&mut self, depth: Depth) -> u64 {
        utils::perft::perft_divide(&mut self.position, depth)
    }
    
    
    /// Returns the number of threads that can be used for multithreaded operations.
    /// This corresponds to the size of the global thread pool, which by default is the number of logical cores.
    /// Set the `RAYON_NUM_THREADS` environment variable to change the thread pool size.
    /// 
    /// When compiled to WASM, you must first call `wasm_module.initThreadPool(navigator.hardwareConcurrency)`
    /// from JavaScript (see `protochess-engine-wasm/example-js/wasm-worker.js`).
    pub fn get_max_threads() -> u32 {
        #[cfg(not(feature = "parallel"))] {
            1
        }
        #[cfg(feature = "parallel")] {
            // Return the size of the global thread pool
            rayon::current_num_threads() as u32
        }
    }
    /// Sets the number of threads to use. This does not resize the global thread pool,
    /// but rather changes the number of tasks that will be submitted to the pool.
    /// By default, all available threads are used (see `get_max_threads()`).
    pub fn set_num_threads(&mut self, num_threads: u32) -> wrap_res!() {
        if num_threads > Self::get_max_threads() {
            return Err(format!("The maximum number of threads is {}", Self::get_max_threads()));
        }
        self.num_threads = num_threads;
        Ok(())
    }
}

impl std::fmt::Display for Engine {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        write!(f, "{}", self.position)
    }
}

impl Default for Engine {
    fn default() -> Self {
        let position = Position::try_from(GameState::default()).unwrap();
        let num_threads = Self::get_max_threads();
        Engine{ position, num_threads }
    }
}
