#[macro_use]
extern crate lazy_static;
extern crate impl_ops;

use thread_handler::ThreadHandler;
use utils::custom_position::make_custom_position;

pub use crate::position::Position;
pub use crate::move_generator::MoveGenerator;
use crate::position::parse_fen;
use crate::utils::{to_index, from_index};

//Private modules
mod constants;
pub mod move_generator;
pub mod types;
pub mod position;
pub mod evaluator;
mod searcher;
pub mod utils;
mod transposition_table;
mod thread_handler;
use std::collections::HashMap;
use crate::evaluator::Evaluator;
use crate::types::*;
pub use crate::types::PieceType;
use crate::searcher::Searcher;
pub use crate::position::movement_pattern::MovementPatternExternal;



#[derive(Clone)]
pub struct State {
    pub position: Position,
    pub movegen: MoveGenerator,
    pub(crate) eval: Evaluator,
}

impl State {
    pub fn new(position: Position) -> State {
        State {
            movegen: MoveGenerator::new(),
            eval: Evaluator::new(),
            position,
        }
    }
    
    #[inline(always)]
    pub fn get_score(&mut self) -> Centipawns {
        self.eval.evaluate(&mut self.position, &self.movegen)
    }
    
    #[inline(always)]
    pub fn position_in_check(&mut self) -> bool {
        self.movegen.in_check(&mut self.position)
    }
    
    #[inline(always)]
    fn legal_moves(&mut self) -> Vec<((BCoord, BCoord), (BCoord, BCoord))> {
        self.movegen.get_legal_moves_as_tuples(&mut self.position)
    }
    
    #[inline(always)]
    pub fn is_move_legal(&mut self, mv: &Move) -> bool {
        self.movegen.is_move_legal(mv, &mut self.position)
    }
    
    /// Performs a move from (x1, y1) to (x2, y2) on the current board position
    pub fn attempt_move(position: &mut Position, movegen: &MoveGenerator, x1: BCoord, y1: BCoord, x2: BCoord, y2: BCoord) -> bool {
        let from = to_index(x1, y1);
        let to = to_index(x2, y2);

        let moves = movegen.get_pseudo_moves(position);
        for mv in moves {
            if !movegen.is_move_legal(&mv, position) {
                continue;
            }
            if mv.get_from() == from && mv.get_to() == to {
                position.make_move(mv);
                return true
            }
        }
        false
    }
}


pub struct Game {
    pub current_position: Position,
}

impl Game {
    pub fn default() -> Game {
        Game {
            current_position: Position::default(),
        }
    }

    pub fn set_bounds(&mut self, width: BCoord, height: BCoord, valid_squares:Vec<(BCoord, BCoord)>) {
        let mut bounds = Bitboard::zero();
        for square in valid_squares {
            bounds.set_bit_at(square.0, square.1);
        }
        self.current_position.set_bounds(BDimensions{ width, height }, bounds);
    }

    pub fn set_state(&mut self, movement_patterns: HashMap<char, MovementPatternExternal>,
                     valid_squares: &Vec<(BCoord, BCoord)>, pieces: &Vec<(Player, BCoord, BCoord, char)>) {
        self.current_position = make_custom_position(movement_patterns, valid_squares, pieces);
    }


    pub fn get_width(&self) -> BCoord {
        self.current_position.dimensions.width
    }

    pub fn get_height(&self) -> BCoord {
        self.current_position.dimensions.height
    }

    pub fn to_string(&mut self) -> String {
        self.current_position.to_string()
    }

    pub fn get_zobrist(&self) -> u64 {
        self.current_position.get_zobrist()
    }

    /// Performs a move from (x1, y1) to (x2, y2) on the current board position
    pub fn make_move(&mut self, move_generator: &MoveGenerator, x1: BCoord, y1: BCoord, x2: BCoord, y2: BCoord) -> bool {
        State::attempt_move(&mut self.current_position, move_generator, x1, y1, x2, y2)
    }

    /// Undoes the most recent move on the current board position
    pub fn undo(&mut self) {
        self.current_position.unmake_move();
    }

    pub fn get_whos_turn(&self) -> Player {
        self.current_position.whos_turn
    }

}


/// Starting point for the engine
pub struct Engine{
    pub state: State,
    pub thread_handler: ThreadHandler,
}

impl Engine {
    /// Initializes a new engine
    pub fn default() -> Engine {
        Engine{
            state: State::new(Position::default()),
            thread_handler: ThreadHandler::std_threads(),
        }
    }
    pub fn default_wasm() -> Engine {
        Engine{
            state: State::new(Position::default()),
            thread_handler: ThreadHandler::wasm_threads(),
        }
    }
    pub fn from_fen(fen: String) -> Engine {
        Engine{
            state: State::new(parse_fen(fen)),
            thread_handler: ThreadHandler::std_threads(),
        }
    }
    pub fn set_num_threads(&mut self, num_threads: u32) {
        self.thread_handler.set_num_threads(num_threads);
    }

    /// Returns the zobrist hash key for the current position
    pub fn get_zobrist(&self) -> u64 {
        self.state.position.get_zobrist()
    }

    /// Returns the score of the current position for the side to move
    pub fn get_score(&mut self) -> Centipawns {
        self.state.get_score()
    }

    /// Registers a custom piecetype for the current position
    pub fn register_piecetype(&mut self, char_rep: char, mpe: MovementPatternExternal) {
        self.state.position.register_piecetype(char_rep, mpe);
    }

    /// Adds a new piece on the board
    pub fn add_piece(&mut self, _owner: usize, _piece_type: PieceType, x: BCoord, y: BCoord) {
        self.state.position.public_add_piece(0, PieceType::Custom('a'), to_index(x,y));
    }

    /// Removes a piece on the board, if it exists
    pub fn remove_piece(&mut self, index: BIndex) {
        self.state.position.public_remove_piece(index);
    }

    /// Performs a move from (x1, y1) to (x2, y2) on the current board position
    pub fn make_move(&mut self, x1: BCoord, y1: BCoord, x2: BCoord, y2: BCoord) -> bool {
        State::attempt_move(&mut self.state.position, &self.state.movegen, x1, y1, x2, y2)
    }

    /// Undoes the most recent move on the current board position
    pub fn undo(&mut self) {
        self.state.position.unmake_move();
    }

    pub fn to_string(&mut self) -> String {
        self.state.position.to_string()
    }

    ///Calculates and plays the best move found up to a given depth
    pub fn play_best_move(&mut self, depth: Depth) -> bool {
        let best_move = Searcher::get_best_move(&self, depth);
        match self.process_move(&best_move) {
            Some((x1, y1, x2, y2, _)) => self.make_move(x1, y1, x2, y2),
            None => false
        }
    }

    ///Returns (fromx,fromy,tox,toy)
    pub fn get_best_move(&mut self, depth: Depth) -> Option<(BCoord, BCoord, BCoord, BCoord)> {
        let best_move = Searcher::get_best_move(&self, depth);
        match self.process_move(&best_move) {
            Some((x1, y1, x2, y2, _)) => Some((x1, y1, x2, y2)),
            None => None
        }
    }

    ///Calculates and plays the best move found
    pub fn play_best_move_timeout(&mut self, max_sec:u64) -> (bool, Depth) {
        let best_move = Searcher::get_best_move_timeout(&self, max_sec);
        match self.process_move(&best_move) {
            Some((x1, y1, x2, y2, depth)) => (self.make_move(x1, y1, x2, y2), depth),
            None => return (false, 0)
        }
    }

    ///Returns ((fromX,fromY,toX,toY), depth)
    pub fn get_best_move_timeout(&mut self, max_sec: u64) -> Option<((BCoord, BCoord, BCoord, BCoord), Depth)> {
        let best_move = Searcher::get_best_move_timeout(&self, max_sec);
        match self.process_move(&best_move) {
            Some((x1, y1, x2, y2, depth)) => Some(((x1, y1, x2, y2), depth)),
            None => None
        }
    }
    
    fn process_move(&self, mv: &Result<(Move, Depth), GameResult>) -> Option<(BCoord, BCoord, BCoord, BCoord, Depth)> {
        match mv {
            Ok((best, depth)) => {
                let (x1, y1) = from_index(best.get_from());
                let (x2, y2) = from_index(best.get_to());
                Some((x1, y1, x2, y2, *depth))
            },
            Err(GameResult::Checkmate) => {
                let loser = self.state.position.whos_turn;
                if loser == 0 {
                    println!("CHECKMATE! Black wins!");
                } else {
                    println!("CHECKMATE! White wins!");
                } 
                None
            }
            Err(GameResult::Stalemate) => {
                println!("STALEMATE!");
                None
            }
        }
    }

    pub fn moves_from(&mut self, x: BCoord, y: BCoord) -> Vec<(BCoord, BCoord)>{
        let moves = self.state.legal_moves();
        let mut possible_moves = Vec::new();
        for (from, to) in moves {
            if from == (x, y) {
                possible_moves.push(to);
            }
        }
        possible_moves
    }

    pub fn to_move_in_check(&mut self) -> bool {
        self.state.position_in_check()
    }

    pub fn set_state(&mut self, movement_patterns: HashMap<char, MovementPatternExternal>,
                     valid_squares:Vec<(BCoord, BCoord)>, pieces: Vec<(Player, BCoord, BCoord, char)>) {
        self.state.position = make_custom_position(movement_patterns, &valid_squares, &pieces)
    }
}

