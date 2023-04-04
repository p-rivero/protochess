#[macro_use]
extern crate lazy_static;
extern crate impl_ops;

pub mod piece;
pub mod move_generator;
pub mod types;
pub mod position;
pub mod searcher;
pub mod utils;

use std::collections::HashMap;
use std::convert::TryFrom;

use position::create::position_factory::PositionFactory;
use types::{BCoord, Centipawns, Depth, Player, ZobKey};
use searcher::Searcher;
use utils::{to_index, from_index};

pub use position::Position;
pub use position::create::game_state::*;
pub use position::global_rules::GlobalRules;
pub use move_generator::MoveGen;
pub use piece::{Piece, PieceId, PieceDefinition};
pub use types::{MoveInfo, MoveList, MakeMoveResult, MakeMoveResultFlag, MakeMoveResultWinner};

/// Starting point for the engine
#[derive(Debug, Clone)]
pub struct Engine{
    position: Position,
    factory: PositionFactory,
    num_threads: u32,
}

impl Engine {
    /// Sets up the engine with a given game state
    pub fn set_state(&mut self, state: GameState) -> wrap_res!() {
        self.position = self.factory.set_state(state)?;
        Ok(())
    }
    /// Updates the engine by loading a fen string. The variant is unchanged.
    pub fn load_fen(&mut self, fen: &str) -> wrap_res!() {
        self.position = self.factory.load_fen(fen)?;
        Ok(())
    }
    /// Returns the current GameState, which can later be used in `set_state()`
    pub fn get_state(&mut self) -> &GameState {
        self.factory.get_state()
    }
    /// Returns the current state, but only the fields that can change during a game
    pub fn get_state_diff(&mut self) -> StateDiff {
        StateDiff::from(&mut self.position)
    }
    
    /// Returns the id (can be uppercase or lowercase) of the piece at the given coordinates
    pub fn get_piece_at(&self, position: (BCoord, BCoord)) -> wrap_res!(PieceId) {
        let piece = self.position.piece_at(to_index(position.0, position.1));
        err_assert!(piece.is_some(), "No piece at the given coordinates");
        Ok(piece.unwrap().get_piece_id())
    }

    /// Adds a new piece on the board. If the piece is not used for castling, `has_moved` is ignored.
    pub fn add_piece(&mut self, piece_id: PieceId, x: BCoord, y: BCoord, has_moved: bool) -> wrap_res!() {
        self.position.public_add_piece(piece_id, to_index(x,y), !has_moved)?;
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
        self.factory.add_move(target_move);
        self.position.pub_make_move(target_move)
    }
    
    /// Attempts a move on the current board position, given a string in the format "e2e4"
    pub fn make_move_str(&mut self, target_move: &str) -> wrap_res!(MakeMoveResult) {
        let mv = MoveInfo::try_from(target_move)?;
        Ok(self.make_move(&mv))
    }

    /// Undoes the most recent move on the current board position
    pub fn undo(&mut self) -> wrap_res!() {
        if !self.position.can_unmake_move() {
            return Err("There is no move to undo".to_string());
        }
        self.factory.remove_last_move();
        self.position.unmake_move();
        Ok(())
    }
    
    /// Returns `0` if it's white's turn, `1` if it's black's turn
    pub fn player_to_move(&self) -> Player {
        self.position.whos_turn
    }
    
    /// Returns the best move for the current position, along with the evaluation score
    pub fn get_best_move(&mut self, depth: Depth) -> wrap_res!(MoveInfo, Centipawns) {
        self.validate_position()?;
        err_assert!(depth != 0, "Depth must be greater than 0");
        let (pv, score, search_depth) = Searcher::get_best_move(&self.position, depth, self.num_threads);
        err_assert!(search_depth == depth, "Search depth ({search_depth}) != requested depth ({depth})");
        err_assert!(!pv.is_empty(), "No moves found");
        Ok((pv[0].into(), score))
    }

    /// Returns the best move for the current position, along with the evaluation score and the search depth
    pub fn get_best_move_timeout(&mut self, max_sec: u64) -> wrap_res!(MoveInfo, Centipawns, Depth) {
        self.validate_position()?;
        let (pv, score, search_depth) = Searcher::get_best_move_timeout(&self.position, max_sec, self.num_threads);
        err_assert!(!pv.is_empty(), "No moves found");
        Ok((pv[0].into(), score, search_depth))
    }
    
    /// Returns an error if the current position is invalid
    pub fn validate_position(&mut self) -> wrap_res!() {
        let player = self.position.whos_turn;
        let player_str = if player == 0 { "White" } else { "Black" };
        if self.position.leader_is_captured() {
            let has_leader = self.position.pieces[player as usize].get_leader().is_some();
            let piece_str = if has_leader { "leaders" } else { "pieces" };
            err!("All the {piece_str} of the player to move ({player_str}) have already been captured");
        }
        err_assert!(MoveGen::count_legal_moves(&mut self.position) != 0, "The player to move ({player_str}) has no legal moves");
        Ok(())
    }
    
    /// Returns a list of all squares (x,y) from which the given piece can move, along with the moves themselves
    pub fn legal_moves(&mut self) -> Vec<MoveList> {
        let all_moves = MoveGen::get_legal_moves(&mut self.position);
        let mut moves_from_map = HashMap::new();
        for mv in all_moves {
            let from = mv.get_from();
            let coords = from_index(from);
            let from_moves = moves_from_map.entry(coords).or_insert_with(Vec::new);
            from_moves.push(MoveInfo::from(mv));
        }
        let mut output = Vec::new();
        for ((x,y), moves) in moves_from_map {
            output.push(MoveList{x, y, moves});
        }
        output
    }
    
    /// Returns a list of all possible promotions for the given move
    pub fn possible_promotions(&mut self, from: (BCoord, BCoord), to: (BCoord, BCoord)) -> Vec<PieceId> {
        MoveGen::get_legal_moves(&mut self.position)
            .into_iter()
            .filter(|mv| {
                let mv_from = from_index(mv.get_from());
                let mv_to = from_index(mv.get_to());
                mv_from == from && mv_to == to && mv.is_promotion()
            })
            .map(|mv| mv.get_promotion_piece().unwrap())
            .collect()
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
    
    
    
    // Debugging functions
    pub fn get_zobrist(&self) -> ZobKey {
        self.position.get_zobrist()
    }
    pub fn perft(&mut self, depth: Depth) -> u64 {
        utils::perft::perft(&mut self.position, depth)
    }
    pub fn perft_divide(&mut self, depth: Depth) -> u64 {
        utils::perft::perft_divide(&mut self.position, depth)
    }
}

impl std::fmt::Display for Engine {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        write!(f, "{}", self.position)
    }
}

impl Default for Engine {
    fn default() -> Self {
        // Set up position using default GameState
        let state = GameState::default();
        let mut factory = PositionFactory::default();
        let position = factory.set_state(state).unwrap();
        // Use maximum number of threads (usually this is too many, the user should change this later)
        let num_threads = Self::get_max_threads();
        Engine { position, factory, num_threads }
    }
}
