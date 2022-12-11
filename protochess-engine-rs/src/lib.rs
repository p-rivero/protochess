#[macro_use]
extern crate lazy_static;
extern crate impl_ops;

use crate::piece::PieceId;
use utils::custom_position::make_custom_position;

pub use crate::position::Position;
pub use crate::move_generator::MoveGen;
use crate::position::parse_fen;
use crate::utils::{to_index, from_index};

//Private modules
mod constants;
pub mod piece;
pub mod move_generator;
pub mod types;
pub mod position;
pub mod searcher;
pub mod utils;
use crate::types::*;
use crate::searcher::{Searcher, eval};
pub use crate::piece::PieceDefinition;


/// Starting point for the engine
pub struct Engine{
    position: Position,
}

impl Engine {
    /// Initializes a new engine
    pub fn default() -> Engine {
        Engine{
            position: Position::default(),
        }
    }
    pub fn from_fen(fen: String) -> Engine {
        Engine{
            position: parse_fen(fen),
        }
    }

    /// Returns the zobrist hash key for the current position
    pub fn get_zobrist(&self) -> u64 {
        self.position.get_zobrist()
    }

    /// Returns the score of the current position for the side to move
    pub fn get_score(&mut self) -> Centipawns {
        eval::evaluate(&mut self.position)
    }
    
    /// Returns the character representation of the piece at the given coordinates
    pub fn get_piece_at(&mut self, x: BCoord, y: BCoord) -> Option<PieceId> {
        self.position.piece_at(to_index(x, y)).map(|p| p.get_piece_id())
    }

    /// Registers a custom piecetype for the current position
    pub fn register_piecetype(&mut self, definition: &PieceDefinition) {
        self.position.register_piecetype(definition);
    }

    /// Adds a new piece on the board
    pub fn add_piece(&mut self, owner: Player, piece_type: PieceId, x: BCoord, y: BCoord) {
        self.position.public_add_piece(owner, piece_type, to_index(x,y));
    }

    /// Removes a piece on the board, if it exists
    pub fn remove_piece(&mut self, index: BIndex) {
        self.position.public_remove_piece(index);
    }

    /// Performs a move from (x1, y1) to (x2, y2) on the current board position
    /// If it's a promotion, the piece type is also specified. Otherwise, promotion char is ignored.
    pub fn make_move(&mut self, x1: BCoord, y1: BCoord, x2: BCoord, y2: BCoord, promotion: Option<PieceId>) -> bool {
        let from = to_index(x1, y1);
        let to = to_index(x2, y2);

        let moves = MoveGen::get_pseudo_moves(&mut self.position);
        for mv in moves {
            if !MoveGen::is_move_legal(&mv, &mut self.position) {
                continue;
            }
            if mv.get_promotion_piece() != promotion {
                continue;
            }
            if mv.get_from() == from && mv.get_to() == to {
                self.position.make_move(mv);
                return true
            }
        }
        false
    }

    /// Undoes the most recent move on the current board position
    pub fn undo(&mut self) {
        self.position.unmake_move();
    }

    pub fn to_string(&mut self) -> String {
        self.position.to_string()
    }
    
    pub fn get_whos_turn(&self) -> Player {
        self.position.whos_turn
    }
    
    pub fn get_width(&self) -> BCoord {
        self.position.dimensions.width
    }
    
    pub fn get_height(&self) -> BCoord {
        self.position.dimensions.height
    }
    
    ///Calculates and plays the best move found up to a given depth
    pub fn play_best_move(&mut self, depth: Depth) -> bool {
        let best_move = Searcher::get_best_move(&self.position, depth);
        match self.process_move(&best_move) {
            Some((x1, y1, x2, y2, prom, _)) => self.make_move(x1, y1, x2, y2, prom),
            None => false
        }
    }

    /// Returns (fromx,fromy,tox,toy,promotion) if there is a move to be made
    pub fn get_best_move(&mut self, depth: Depth) -> Option<(BCoord, BCoord, BCoord, BCoord, Option<PieceId>)> {
        let best_move = Searcher::get_best_move(&self.position, depth);
        match self.process_move(&best_move) {
            Some((x1, y1, x2, y2, prom, _)) => Some((x1, y1, x2, y2, prom)),
            None => None
        }
    }

    ///Calculates and plays the best move found
    pub fn play_best_move_timeout(&mut self, max_sec:u64) -> (bool, Depth) {
        let best_move = Searcher::get_best_move_timeout(&self.position, max_sec);
        match self.process_move(&best_move) {
            Some((x1, y1, x2, y2, prom, depth)) => (self.make_move(x1, y1, x2, y2, prom), depth),
            None => return (false, 0)
        }
    }

    ///Returns ((fromX,fromY,toX,toY,promotion), depth)
    pub fn get_best_move_timeout(&mut self, max_sec: u64) -> Option<((BCoord, BCoord, BCoord, BCoord, Option<PieceId>), Depth)> {
        let best_move = Searcher::get_best_move_timeout(&self.position, max_sec);
        match self.process_move(&best_move) {
            Some((x1, y1, x2, y2, prom, depth)) => Some(((x1, y1, x2, y2, prom), depth)),
            None => None
        }
    }
    
    // Unwraps a SearchResult into basic data types
    fn process_move(&self, mv: &SearchResult) -> Option<(BCoord, BCoord, BCoord, BCoord, Option<PieceId>, Depth)> {
        match mv {
            // TODO: Use backup move
            SearchResult::BestMove(best, depth, _backup) => {
                let (x1, y1) = from_index(best.get_from());
                let (x2, y2) = from_index(best.get_to());
                let prom = best.get_promotion_piece();
                Some((x1, y1, x2, y2, prom, *depth))
            },
            SearchResult::Checkmate(losing_player) => {
                if *losing_player == 0 {
                    println!("CHECKMATE! Black wins!");
                } else {
                    println!("CHECKMATE! White wins!");
                } 
                None
            }
            SearchResult::Stalemate => {
                println!("STALEMATE!");
                None
            }
        }
    }

    pub fn moves_from(&mut self, x: BCoord, y: BCoord) -> Vec<(BCoord, BCoord)>{
        let moves = MoveGen::get_legal_moves_as_tuples(&mut self.position);
        let mut possible_moves = Vec::new();
        for (from, to) in moves {
            if from == (x, y) {
                possible_moves.push(to);
            }
        }
        possible_moves
    }

    pub fn to_move_in_check(&mut self) -> bool {
        MoveGen::in_check(&mut self.position)
    }

    pub fn set_state(&mut self, piece_types: &Vec<PieceDefinition>,
                     valid_squares:Vec<(BCoord, BCoord)>, pieces: Vec<(Player, BCoord, BCoord, PieceId)>) {
        self.position = make_custom_position(piece_types, &valid_squares, &pieces)
    }
    
    pub fn perft(&mut self, depth: Depth) -> u64 {
        utils::perft::perft(&mut self.position, depth)
    }
}

