use std::convert::TryFrom;

use ahash::AHashSet;

use crate::{PieceDefinition, PieceId, err_assert, wrap_res};
use crate::utils::{to_index, from_index};

use super::Position;
use super::global_rules::GlobalRules;
use super::position_properties::PositionProperties;
use super::{BCoord, Player, BDimensions};

#[must_use]
pub struct PiecePlacement {
    pub owner: Player,
    pub piece_id: PieceId,
    pub x: BCoord,
    pub y: BCoord,
    // True if it has not moved. This is an option so that JS can leave it as undefined
    pub can_castle: Option<bool>,
}
impl PiecePlacement {
    pub fn new(owner: Player, piece_id: PieceId, x: BCoord, y: BCoord, can_castle: bool) -> Self {
        PiecePlacement { owner, piece_id, x, y, can_castle: Some(can_castle), }
    }
}
#[must_use]
pub struct GameState {
    pub piece_types: Vec<PieceDefinition>,
    pub valid_squares: Vec<(BCoord, BCoord)>,
    pub pieces: Vec<PiecePlacement>,
    pub whos_turn: Player,
    pub ep_square_and_victim: Option<((BCoord, BCoord), (BCoord, BCoord))>,
    pub times_in_check: Option<[u8; 2]>,
    pub global_rules: GlobalRules,
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
                    pieces.push(PiecePlacement::new(piece.get_player(), piece.get_piece_id(), x, y, piece.has_not_moved(index)));
                }
                if pos.dimensions.in_bounds(x, y) {
                    valid_squares.push((x, y));
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
            valid_squares,
            pieces,
            whos_turn: pos.whos_turn,
            ep_square_and_victim,
            times_in_check: Some(*pos.get_times_checked()),
            global_rules: pos.global_rules.clone(),
        }
    }
}

impl TryFrom<GameState> for Position {
    type Error = String;
    fn try_from(state: GameState) -> wrap_res!(Self) {
        let dims = BDimensions::from_valid_squares(&state.valid_squares)?;
    
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
        if state.whos_turn == 1 {
            // Use the top bit as player zobrist key
            props.zobrist_key ^= 0x8000_0000_0000_0000;
        }
        if let Some(times_in_check) = state.times_in_check {
            props.times_in_check = times_in_check;
        }

        // Instantiate position and register piecetypes
        let mut pos = Position::new(dims, state.whos_turn, props, state.global_rules);
        for definition in &state.piece_types {
            pos.register_piecetype(definition)?;
        }
        
        // Add pieces
        for p in state.pieces {
            // By default, assume pieces have not moved
            let can_castle = p.can_castle.unwrap_or(true);
            pos.public_add_piece(p.owner, p.piece_id, to_index(p.x, p.y), can_castle)?;
        }
        Ok(pos)
    }
}

impl Default for GameState {
    fn default() -> Self {
        GameState::from_fen("rnbqkbnr/pppppppp/8/8/8/8/PPPPPPPP/RNBQKBNR w KQkq - 0 1").unwrap()
    }
}
