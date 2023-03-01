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

use types::{BCoord, Centipawns, Depth, Player, ZobKey};
use searcher::{Searcher, eval};
use utils::{to_index, from_index};

pub use position::Position;
pub use position::game_state::{PiecePlacement, GameState, GameStateGui};
pub use position::global_rules::GlobalRules;
pub use move_generator::MoveGen;
pub use piece::{Piece, PieceId, PieceDefinition};
pub use types::{MoveInfo, MoveList, MakeMoveResult, MakeMoveResultFlag, MakeMoveResultWinner};

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
    
    pub fn get_state(&mut self) -> GameStateGui {
        let state = GameState::from(&self.position);
        let fen = state.create_fen();
        GameStateGui {
            state,
            fen,
            in_check: self.to_move_in_check(),
        }
    }

    /// Returns the zobrist hash key for the current position
    pub fn get_zobrist(&self) -> ZobKey {
        self.position.get_zobrist()
    }

    /// Returns the score of the current position for the side to move
    pub fn get_score(&self) -> Centipawns {
        eval::evaluate(&self.position)
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
        let moves = MoveGen::get_pseudo_moves(&mut self.position, true);
        for mv in moves {
            if target_move != &mv {
                continue;
            }
            // Found the move, try to play it
            let exploded = mv.get_potential_explosion(&self.position);
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
                return MakeMoveResult::leader_captured(winner, exploded);
            }
            // Piece moved to winning square (king of the hill, racing kings)
            if self.position.piece_is_on_winning_square() {
                return MakeMoveResult::piece_in_win_square(winner, exploded);
            }
            let in_check = MoveGen::in_check(&mut self.position);
            // No legal moves, check if it's checkmate or stalemate
            if MoveGen::count_legal_moves(&mut self.position) == 0 {
                if in_check {
                    return MakeMoveResult::checkmate(winner, exploded);
                }
                if self.position.global_rules.stalemated_player_loses {
                    return MakeMoveResult::stalemate(Some(winner), exploded);
                }
                return MakeMoveResult::stalemate(None, exploded);
            }
            if in_check && self.position.increment_num_checks() {
                // Checked N times (N=3 in 3-check)
                return MakeMoveResult::check_limit(winner, exploded);
            }
            if self.position.draw_by_repetition() {
                // Threefold Repetition
                return MakeMoveResult::repetition();
            }
            return MakeMoveResult::ok(exploded);
        }
        MakeMoveResult::illegal_move()
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
    
    pub fn player_to_move(&self) -> Player {
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
        err_assert!(!pv.is_empty(), "No moves found");
        Ok((pv[0].into(), score))
    }

    /// Returns the best move for the current position, along with the evaluation score and the search depth
    pub fn get_best_move_timeout(&mut self, max_sec: u64) -> wrap_res!(MoveInfo, Centipawns, Depth) {
        self.assert_position_is_valid()?;
        let (pv, score, search_depth) = Searcher::get_best_move_timeout(&self.position, max_sec, self.num_threads);
        err_assert!(!pv.is_empty(), "No moves found");
        Ok((pv[0].into(), score, search_depth))
    }
    fn assert_position_is_valid(&mut self) -> wrap_res!() {
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
