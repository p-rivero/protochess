use std::convert::TryFrom;

use ahash::AHashSet;

use crate::{PieceDefinition, PieceId, err_assert, wrap_res};
use crate::utils::{to_index, from_index};
use crate::utils::debug::eq_anyorder;

use super::Position;
use super::global_rules::GlobalRules;
use super::position_properties::PositionProperties;
use super::{BCoord, Player, BDimensions};

#[must_use]
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct PiecePlacement {
    // Id of the piece (can be upper or lower case)
    pub piece_id: PieceId,
    pub x: BCoord,
    pub y: BCoord,
    // True if it has not moved. This is an option so that JS can leave it as undefined
    pub can_castle: Option<bool>,
}
impl PiecePlacement {
    pub fn new(piece_id: PieceId, x: BCoord, y: BCoord, can_castle: bool) -> Self {
        PiecePlacement { piece_id, x, y, can_castle: Some(can_castle), }
    }
}
#[must_use]
#[derive(Debug, Clone)]
pub struct GameState {
    pub piece_types: Vec<PieceDefinition>,
    pub board_width: BCoord,
    pub board_height: BCoord,
    pub invalid_squares: Vec<(BCoord, BCoord)>,
    pub pieces: Vec<PiecePlacement>,
    pub player_to_move: Player,
    pub ep_square_and_victim: Option<((BCoord, BCoord), (BCoord, BCoord))>,
    pub times_in_check: Option<[u8; 2]>,
    pub global_rules: GlobalRules,
}
#[derive(Debug, Clone)]
pub struct GameStateGui {
    pub state: GameState,
    pub fen: String,
    pub in_check: bool,
}

impl PartialEq<GameState> for GameState {
    fn eq(&self, other: &GameState) -> bool {
        eq_anyorder(&self.piece_types, &other.piece_types) &&
        self.board_width == other.board_width &&
        self.board_height == other.board_height &&
        eq_anyorder(&self.invalid_squares, &other.invalid_squares) &&
        eq_anyorder(&self.pieces, &other.pieces) &&
        self.player_to_move == other.player_to_move &&
        self.ep_square_and_victim == other.ep_square_and_victim &&
        self.times_in_check == other.times_in_check &&
        self.global_rules == other.global_rules
    }
}


impl From<&Position> for GameState {
    fn from(pos: &Position) -> Self {
        let mut piece_types_set = AHashSet::<&PieceDefinition>::new();
        let mut pieces = Vec::new();
        let board_width = pos.dimensions.width;
        let board_height = pos.dimensions.height;
        let mut invalid_squares = Vec::new();
        for x in 0..board_width {
            for y in 0..board_height {
                let index = to_index(x, y);
                if let Some(piece) = pos.piece_at(index) {
                    piece_types_set.insert(piece.get_movement());
                    pieces.push(PiecePlacement::new(piece.get_piece_id(), x, y, piece.has_not_moved(index)));
                }
                if !pos.dimensions.in_bounds(x, y) {
                    invalid_squares.push((x, y));
                }
            }
        }
        // Convert set of &PieceDefinition to Vec of PieceDefinition
        let piece_types = piece_types_set.into_iter().cloned().collect::<Vec<_>>(); 
        // Extract EP square
        let ep_square_and_victim = {
            if let Some(ep_square) = pos.get_ep_square() {
                let ep_victim = pos.get_ep_victim();
                Some((from_index(ep_square), from_index(ep_victim)))
            } else {
                None
            }
        };
        GameState {
            piece_types,
            board_width,
            board_height,
            invalid_squares,
            pieces,
            player_to_move: pos.whos_turn,
            ep_square_and_victim,
            times_in_check: Some(*pos.get_times_checked()),
            global_rules: pos.global_rules.clone(),
        }
    }
}

impl TryFrom<GameState> for Position {
    type Error = String;
    fn try_from(state: GameState) -> wrap_res!(Self) {
        let dims = BDimensions::from_invalid_squares(state.board_width, state.board_height, &state.invalid_squares)?;
    
        // Assert that all pieces are placed on valid squares
        for p in &state.pieces {
            err_assert!(dims.in_bounds(p.x, p.y), "Invalid piece placement: ({}, {})", p.x, p.y);
        }
        
        // Update props
        let mut props = PositionProperties::default();
        if let Some(((sx,sy),(vx,vy))) = state.ep_square_and_victim {
            err_assert!(dims.in_bounds(sx, sy), "Invalid EP square: ({sx}, {sy})");
            err_assert!(dims.in_bounds(vx, vy), "Invalid EP victim: ({vx}, {vy})");
            props.set_ep_square(to_index(sx, sy), to_index(vx, vy));
        }
        if state.player_to_move == 1 {
            // Use the lowest bit as player zobrist key
            props.zobrist_key ^= 1;
        }
        if let Some(times_in_check) = state.times_in_check {
            props.times_in_check = times_in_check;
        }

        // Instantiate position and register piecetypes
        let mut pos = Position::new(dims, state.player_to_move, props, state.global_rules);
        for definition in &state.piece_types {
            pos.register_piecetype(definition)?;
        }
        pos.assert_promotion_consistency()?;
        
        // Add pieces
        for p in state.pieces {
            // By default, assume pieces have not moved
            let can_castle = p.can_castle.unwrap_or(true);
            pos.public_add_piece(p.piece_id, to_index(p.x, p.y), can_castle)?;
        }
        Ok(pos)
    }
}

impl Default for GameState {
    fn default() -> Self {
        GameState::from_fen("rnbqkbnr/pppppppp/8/8/8/8/PPPPPPPP/RNBQKBNR w KQkq - 0 1").unwrap()
    }
}
