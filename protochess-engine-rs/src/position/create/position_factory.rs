use crate::position::position_properties::PositionProperties;
use crate::{InitialState, Position, wrap_res, err_assert, MakeMoveResultFlag, MoveInfo};
use crate::utils::to_index;
use crate::types::BDimensions;

use super::fen::FenData;
use super::game_state::GameState;

/// This struct is responsible for creating Position objects from `GameState` objects.
/// Do not create Position objects directly, use this factory instead.
/// When calling `make_position()`, it checks if the current Position can be reused, and if so, it
/// performs an incremental update instead of creating a new Position object.
#[derive(Debug, Clone, Default)]
pub struct PositionFactory {
    current_state: Option<GameState>,
}

impl PositionFactory {    
    
    /// Creates a `Position` from a `GameState`
    pub fn set_state(&mut self, state: GameState) -> wrap_res!(Position) {
        let pos = Self::set_state_impl(&state)?;
        self.current_state = Some(state);
        Ok(pos)
    }
    
    /// Creates a `Position` from a fen string, using the previous `GameState`'s variant
    pub fn load_fen(&mut self, fen: &str) -> wrap_res!(Position) {
        if self.current_state.is_none() {
            panic!("No current state, call make_position() first");
        }
        // Update the initial fen of the stored GameState
        let state = self.current_state.as_mut().unwrap();
        state.initial_fen = Some(fen.to_string());
        Self::set_state_impl(state)
    }
    
    fn set_state_impl(state: &GameState) -> wrap_res!(Position) {
        // Parse the variant's default starting position
        let mut fen_data = FenData::parse_fen(&state.initial_state.fen)?;
        // Apply the user-proveded initial fen, if any
        if let Some(initial_fen) = &state.initial_fen {
            let old_fen = fen_data;
            fen_data = FenData::parse_fen(initial_fen)?;
            // Don't allow the user to override the walls
            fen_data.walls = old_fen.walls;
        }
        let mut pos = Self::create_new_position(&state.initial_state, fen_data)?;
        // Apply the move history
        for m in &state.move_history {
            let result = pos.pub_make_move(m);
            err_assert!(result.flag != MakeMoveResultFlag::IllegalMove, "Invalid move: {}", m);
        }
        Ok(pos)
    }
    
    /// Returns the current `GameState`
    pub fn get_state(&self) -> &GameState {
        if let Some(state) = &self.current_state {
            state
        } else {
            panic!("No current state, call make_position() first");
        }
    }
    
    /// Adds a new move to the move history of the current `GameState`
    /// Call this whenever a move is made to keep the stored `GameState` in sync
    pub fn add_move(&mut self, m: &MoveInfo) {
        if let Some(state) = &mut self.current_state {
            state.move_history.push(*m);
        } else {
            panic!("No current state, call make_position() first");
        }
    }
    
    /// Removes the last move from the move history of the current `GameState`
    /// Call this whenever a move is undone to keep the stored `GameState` in sync
    pub fn remove_last_move(&mut self) {
        if let Some(state) = &mut self.current_state {
            state.move_history.pop();
        } else {
            panic!("No current state, call make_position() first");
        }
    }
    
    /// Creates a new position from scratch, using the following data:
    /// - **Board height and width:** From `InitialState`
    /// - **Piece definitions:** From `InitialState`
    /// - **Global rules:** From `InitialState`
    /// - **Piece placements and Walls:** From `FenData`
    /// - **Player to move:** From `FenData`
    /// - **Castling availability:** From `FenData`
    /// - **EP square and victim:** From `FenData`
    /// - **Times in check:** From `FenData`
    fn create_new_position(state: &InitialState, fen: FenData) -> wrap_res!(Position) {
        
        let dims = BDimensions::from_walls(state.board_width, state.board_height, &fen.walls)?;
    
        // Assert that all pieces are placed on valid squares
        for p in &fen.piece_placements {
            err_assert!(dims.in_bounds(p.x, p.y), "The piece at ({}, {}) is inside a wall", p.x, p.y);
        }
        
        // Update props
        let mut props = PositionProperties::default();
        if let Some(((sx,sy),(vx,vy))) = fen.ep_square_and_victim {
            err_assert!(dims.in_bounds(sx, sy), "Invalid EP square: ({sx}, {sy})");
            err_assert!(dims.in_bounds(vx, vy), "Invalid EP victim: ({vx}, {vy})");
            props.set_ep_square(to_index(sx, sy), to_index(vx, vy));
        }
        if fen.player_to_move == 1 {
            // Use the lowest bit as player zobrist key
            props.zobrist_key ^= 1;
        }
        props.times_in_check = fen.times_in_check.unwrap_or([0,0]);

        // Instantiate position and register piecetypes
        let mut pos = Position::new(dims, fen.player_to_move, props, state.global_rules.clone());
        for definition in &state.piece_types {
            pos.register_piecetype(definition)?;
        }
        pos.assert_promotion_consistency()?;
        
        // Add pieces
        for p in fen.piece_placements {
            let can_castle = {
                if fen.castling_availability.is_none() { true }
                else { fen.castling_availability.as_ref().unwrap().contains(&(p.x, p.y)) }
            };
            pos.public_add_piece(p.piece_id, to_index(p.x, p.y), can_castle)?;
        }
        Ok(pos)
    }
}

