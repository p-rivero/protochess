use crate::position::position_properties::PositionProperties;
use crate::{InitialState, Position, wrap_res, err_assert, MakeMoveResultFlag, MoveInfo, MakeMoveResult};
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
    last_result: Option<MakeMoveResult>,
    move_notation: Vec<String>,
}

impl PositionFactory {    
    
    /// Creates a `Position` from a `GameState`
    /// Returns:
    /// - `Err` If the state is invalid
    /// - `Ok(None)` If the current position can be reused. In this case, `reused_position` parameter
    /// will be updated.
    /// - `Ok(Some(position))` If a new position was created.
    pub fn set_state(&mut self, new_state: GameState, reused_position: Option<&mut Position>) -> wrap_res!(Option<Position>) {
        // No current state, just create a new position
        if self.current_state.is_none() || reused_position.is_none() {
            let position = self.set_state_impl(new_state)?;
            return Ok(Some(position));
        }
        
        // Reuse the current position only if the variant and initial FEN are the same
        let current_state = self.current_state.as_ref().unwrap();
        let reused_position = reused_position.unwrap();
        if current_state.initial_state != new_state.initial_state
        || current_state.initial_fen != new_state.initial_fen {
            let position = self.set_state_impl(new_state)?;
            return Ok(Some(position));
        }
        
        // See for how many moves we can reuse the current position
        let mut reuse_count = 0;
        for (m1, m2) in current_state.move_history.iter().zip(new_state.move_history.iter()) {
            if m1 == m2 { reuse_count += 1; }
            else { break; }
        }
        // Undo the moves that can't be reused
        let undo_count = current_state.move_history.len() - reuse_count;
        for _ in 0..undo_count {
            reused_position.unmake_move();
            self.move_notation.pop();
            self.last_result = None;
        }
        
        // Apply the new moves
        for (i, mv) in new_state.move_history[reuse_count..].iter().enumerate() {
            let result = reused_position.pub_make_move(mv);
            
            // If an illegal move is encountered, roll back and return error
            // In order to roll back, we need to make the moves that were undone earlier
            if result.flag == MakeMoveResultFlag::IllegalMove {
                // Undo i new moves
                for _ in 0..i {
                    reused_position.unmake_move();
                    self.move_notation.pop();
                }
                // Redo the moves that were undone earlier
                for mv2 in &current_state.move_history[reuse_count..] {
                    let result2 = reused_position.pub_make_move(mv2);
                    err_assert!(result2.flag != MakeMoveResultFlag::IllegalMove, 
                        "Invalid move when attempting to rollback: {}", mv2);
                    self.move_notation.push(result2.move_notation.clone().unwrap());
                }
                return Err(format!("Invalid move: {}", mv));
            }
            
            self.move_notation.push(result.move_notation.clone().unwrap());
            self.last_result = Some(result);
        }
        // Incremental update was successful, store the new state
        self.current_state = Some(new_state);
        Ok(None)
    }
    
    /// Creates a `Position` from a fen string, using the previous `GameState`'s variant
    pub fn load_fen(&mut self, fen: &str) -> wrap_res!(Position) {
        if self.current_state.is_none() {
            panic!("No current state, call make_position() first");
        }
        // Make a copy of the current state, so that we can roll back if the FEN is invalid
        let mut new_state = self.current_state.as_ref().unwrap().clone();
        new_state.initial_fen = Some(fen.to_string());
        new_state.move_history.clear();
        // When loading a FEN, always create a new position instead of reusing the current one
        self.set_state_impl(new_state)
    }
    
    fn set_state_impl(&mut self, state: GameState) -> wrap_res!(Position) {
        // Parse the variant's default starting position
        let mut fen_data = FenData::parse_fen(&state.initial_state.fen)?;
        fen_data.player_to_move = state.initial_state.player_to_move;
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
            self.move_notation.push(result.move_notation.clone().unwrap());
            self.last_result = Some(result);
        }
        // Everything went well, store the new state
        self.current_state = Some(state);
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
    
    /// Returns the current move history in algebraic notation
    pub fn get_notation(&self) -> &Vec<String> {
        &self.move_notation
    }
    
    /// Returns the result of the last move in `state.move_history`, or `Ok` if
    /// this information is not known
    pub fn get_last_result(&self) -> MakeMoveResult {
        if let Some(result) = &self.last_result {
            result.clone()
        } else {
            MakeMoveResult::ok(vec![], "??".to_string())
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
        self.last_result = None;
    }
    
    /// Removes the last move from the move history of the current `GameState`
    /// Call this whenever a move is undone to keep the stored `GameState` in sync
    pub fn remove_last_move(&mut self) {
        if let Some(state) = &mut self.current_state {
            state.move_history.pop();
        } else {
            panic!("No current state, call make_position() first");
        }
        self.last_result = None;
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

