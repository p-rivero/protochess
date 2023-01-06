use ahash::AHashSet;

use crate::{PieceDefinition, PieceId};
use crate::utils::to_index;

use super::Position;
use super::position_properties::PositionProperties;
use super::{BCoord, Player, BDimensions};

pub struct GameState {
    pub piece_types: Vec<PieceDefinition>,
    pub valid_squares: Vec<(BCoord, BCoord)>,
    pub pieces: Vec<(Player, BCoord, BCoord, PieceId)>,
    pub whos_turn: Player,
}

impl From<&Position> for GameState {
    fn from(pos: &Position) -> Self {
        let mut piece_types_set = AHashSet::<&PieceDefinition>::new();
        let mut pieces = Vec::new();
        let mut valid_squares = Vec::new();
        for x in 0..pos.dimensions.width {
            for y in 0..pos.dimensions.height {
                let index = to_index(x, y);
                if let Some(piece) = pos.piece_at(index) {
                    piece_types_set.insert(piece.get_movement());
                    pieces.push((piece.get_player(), x, y, piece.get_piece_id()));
                }
                if pos.dimensions.in_bounds(x, y) {
                    valid_squares.push((x, y));
                }
            }
        }
        // Convert set of &PieceDefinition to Vec of PieceDefinition
        let piece_types = piece_types_set.into_iter().cloned().collect::<Vec<_>>(); 
        GameState {
            piece_types,
            valid_squares,
            pieces,
            whos_turn: pos.whos_turn,
        }
    }
}

impl From<GameState> for Position {
    fn from(state: GameState) -> Self {
        let dims = BDimensions::from_valid_squares(&state.valid_squares);
    
        // Assert that all pieces are placed on valid squares
        for p in &state.pieces {
            assert!(dims.in_bounds(p.1, p.2));
        }

        let mut pos = Position::new_2(dims, state.whos_turn, PositionProperties::default());
        for definition in &state.piece_types {
            pos.register_piecetype(definition);
        }

        for (owner, x, y, piece_type) in state.pieces {
            pos.public_add_piece(owner, piece_type, to_index(x, y));
        }
        pos
    }
}
