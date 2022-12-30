#[macro_use]
extern crate lazy_static;
extern crate impl_ops;

pub mod piece;
pub mod move_generator;
pub mod types;
pub mod position;
pub mod searcher;
pub mod utils;

use types::{BCoord, BDimensions, BIndex, Centipawns, Depth, Player};
use searcher::{Searcher, eval};
use utils::to_index;

pub use position::Position;
pub use move_generator::MoveGen;
pub use piece::{PieceId, PieceDefinition};
pub use types::MoveInfo;

#[derive(Debug, PartialEq, Eq, Clone, Copy)]
pub enum MakeMoveResult {
    Ok,
    IllegalMove,
    Checkmate(Player), // Checkmated player
    LeaderCaptured(Player), // All leader pieces of the player are captured (only in illegal positions or atomic chess)
    Stalemate,
    Repetition,
}

/// Starting point for the engine
pub struct Engine{
    pub position: Position,
}

impl Engine {
    /// Initializes a new engine
    pub fn empty() -> Engine {
        Engine{ position: Position::empty() }
    }
    pub fn default() -> Engine {
        Engine{ position: Position::default() }
    }
    pub fn from_fen(fen: &str) -> Engine {
        Engine{ position: Position::from_fen(fen) }
    }

    /// Returns the zobrist hash key for the current position
    pub fn get_zobrist(&self) -> u64 {
        self.position.get_zobrist()
    }

    /// Returns the score of the current position for the side to move
    pub fn get_score(&mut self) -> Centipawns {
        eval::evaluate(&mut self.position)
    }
    
    /// Returns the id of the piece at the given coordinates
    pub fn get_piece_at(&self, position: (BCoord, BCoord)) -> Option<PieceId> {
        self.position.piece_at(to_index(position.0, position.1)).map(|p| p.get_piece_id())
    }
    
    /// Returns the character representation of the piece with a given id
    pub fn get_piece_char(&self, piece_id: PieceId) -> Option<char> {
        self.position.search_piece_by_id(piece_id).map(|p| p.char_rep())
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

    /// Attempts a move on the current board position
    /// If it's a promotion, the piece type is also specified. Otherwise, promotion char is ignored.
    pub fn make_move(&mut self, target_move: &MoveInfo) -> MakeMoveResult {
        let moves = MoveGen::get_pseudo_moves(&mut self.position);
        for mv in moves {
            if !target_move.matches_move(mv) {
                continue;
            }
            if !MoveGen::is_move_legal(&mv, &mut self.position) {
                continue;
            }
            // Found the move, try to play it
            self.position.make_move(mv, true);
            
            // Check if the game is over
            if self.position.leader_is_captured() {
                return MakeMoveResult::LeaderCaptured(self.position.whos_turn);
            }
            if MoveGen::count_legal_moves(&mut self.position) == 0 {
                if MoveGen::in_check(&mut self.position) {
                    return MakeMoveResult::Checkmate(self.position.whos_turn);
                }
                return MakeMoveResult::Stalemate;
            }
            if self.position.num_repetitions() >= 3 {
                // Threefold Repetition
                return MakeMoveResult::Repetition;
            }
            return MakeMoveResult::Ok;
        }
        MakeMoveResult::IllegalMove
    }

    /// Undoes the most recent move on the current board position
    pub fn undo(&mut self) {
        self.position.unmake_move();
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
    
    /// Returns (fromx,fromy,tox,toy,promotion) if there is a move to be made
    pub fn get_best_move(&mut self, depth: Depth) -> MoveInfo {
        assert!(depth != 0, "Depth must be greater than 0");
        let (mv, search_depth) = Searcher::get_best_move(&self.position, depth);
        assert!(search_depth == depth);
        MoveInfo::from_move(mv)
    }

    ///Returns ((fromX,fromY,toX,toY,promotion), depth)
    pub fn get_best_move_timeout(&mut self, max_sec: u64) -> (MoveInfo, Depth) {
        let (mv, search_depth) = Searcher::get_best_move_timeout(&self.position, max_sec);
        (MoveInfo::from_move(mv), search_depth)
    }

    pub fn moves_from(&mut self, x: BCoord, y: BCoord) -> Vec<MoveInfo>{
        let target_index = to_index(x,y);
        MoveGen::get_legal_moves(&mut self.position)
            .into_iter()
            .filter(|mv| mv.get_from() == target_index)
            .map(MoveInfo::from_move)
            .collect()
    }

    pub fn to_move_in_check(&mut self) -> bool {
        if self.position.leader_is_captured() {
            return false;
        }
        MoveGen::in_check(&mut self.position)
    }

    pub fn set_state(&mut self,
        piece_types: &Vec<PieceDefinition>,
        valid_squares: &Vec<(BCoord, BCoord)>,
        pieces: &[(Player, BCoord, BCoord, PieceId)],
        whos_turn: Player
    ) {
        let dims = BDimensions::from_valid_squares(valid_squares);
        
        // Assert that all pieces are placed on valid squares
        for p in pieces {
            assert!(dims.in_bounds(p.1, p.2));
        }
        
        // For each piece, convert coordnates to index
        let pieces = pieces.iter()
            .map(|(owner, x, y, piece)| (*owner, to_index(*x, *y), *piece))
            .collect();

        self.position = Position::custom(dims, piece_types, pieces, whos_turn);
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

