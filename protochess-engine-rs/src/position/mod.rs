use std::fmt;

use crate::{types::*, PieceDefinition, err_assert, wrap_res, err};
use crate::utils::to_index;
use crate::piece::{Piece, PieceId};

mod position_properties;
mod make_move;
pub mod global_rules;
pub mod create;
pub mod piece_set;

use global_rules::GlobalRules;
use position_properties::PositionProperties;
use piece_set::PieceSet;

/// Represents a single position in chess
#[derive(Clone, Debug)]
pub struct Position {
    pub dimensions: BDimensions,
    pub whos_turn: Player,
    pub pieces: [PieceSet; 2], // pieces[0] = white, pieces[1] = black
    // Bitboard squares, that are occupied by a piece or out of bounds
    pub occ_or_out_bounds: Bitboard,
    // Stack of properties relating only to the current position
    // Typically hard-to-recover properties, like castling
    // Similar to state in stockfish
    properties_stack: Vec<PositionProperties>,
    // Full id (piece type + player num) of the captured pieces, if any.
    // Also store whether the captured piece could castle and the index where it was captured.
    // In regular chess, this will be a maximum of 1 piece. In atomic chess, there can be up to 9.
    captures_stack: Vec<(PieceId, Player, bool, BIndex)>,
    // Global rules of the game
    pub global_rules: GlobalRules,
}

impl Position {
    fn new(dimensions: BDimensions, whos_turn: Player, props: PositionProperties, rules: GlobalRules) -> Position {
        let mut properties_stack = Vec::with_capacity(128);
        properties_stack.push(props);
        let occ_or_out_bounds = !&dimensions.bounds;
        
        Position {
            dimensions,
            whos_turn,
            pieces: [PieceSet::new(0), PieceSet::new(1)],
            occ_or_out_bounds,
            properties_stack,
            captures_stack: Vec::with_capacity(128),
            global_rules: rules,
        }
    }

    /// Registers a new piece type for a given player in this position
    pub fn register_piecetype(&mut self, definition: &PieceDefinition) -> wrap_res!() {
        // Insert piece for all players specified in the definition
        for (player, id) in definition.ids.iter().enumerate() {
            if id.is_none() { continue; }
            let id = id.unwrap();
            
            // Make sure that the promotion squares and pieces are specified together
            err_assert!(definition.promotion_squares.is_empty() == definition.promo_vals[player].is_empty(), 
                "Promotion squares and pieces must be specified together");
                
            // Make sure that the piece is uniquely identifiable for this player
            for set in &self.pieces {
                err_assert!(!set.contains_piece(id), "Piece id {id} already exists");
            }
            self.pieces[player].register_piecetype(definition, &self.dimensions)?;
        }
        Ok(())
    }
    pub fn assert_promotion_consistency(&self) -> wrap_res!() {
        for player in 0..self.pieces.len() {
            self.pieces[player].assert_promotion_consistency()?;
        }
        Ok(())
    }

    #[inline]
    pub fn get_zobrist(&self) -> ZobKey {
        self.get_properties().zobrist_key
    }
    
    #[inline]
    pub fn draw_by_repetition(&self) -> bool {
        if self.global_rules.repetitions_draw == 0 {
            return false;
        }
        let mut num_reps = 1;
        let my_zob = self.get_zobrist();
        // Skip the last element, since it's the current position
        let mut i = self.properties_stack.len() - 1;
        while i > 0 {
            let p = &self.properties_stack[i - 1];
            if p.zobrist_key == my_zob {
                num_reps += 1;
            }
            // A capture breaks the repetition
            // We could also break on pawn moves, but the concept of "pawn" doesn't exist in a custom game
            if p.num_captures > 0 {
                break;
            }
            i -= 1;
        }
        num_reps >= self.global_rules.repetitions_draw
    }
    
    #[inline]
    pub fn get_ep_square(&self) -> Option<BIndex> {
        self.get_properties().get_ep_square()
    }
    #[inline]
    pub fn get_ep_victim(&self) -> BIndex {
        self.get_properties().get_ep_victim()
    }
    
    #[inline]
    pub fn get_times_checked(&self) -> &[u8; 2] {
        &self.get_properties().times_in_check
    }
    #[inline]
    pub fn increment_num_checks(&mut self) -> bool {
        if self.global_rules.checks_to_lose == 0 {
            return false;
        }
        let checked_player = self.whos_turn as usize;
        let i = self.properties_stack.len() - 1;
        let old_checks = self.properties_stack[i-1].times_in_check[checked_player];
        let new_checks = old_checks + 1;
        
        self.properties_stack[i].times_in_check[checked_player] = new_checks;
        // Update the zobrist key (use bits 8-9 for white, 10-11 for black)
        self.properties_stack[i].zobrist_key ^= (new_checks as ZobKey) << (8 + 2 * checked_player);
        // Return true if the player has lost
        new_checks >= self.global_rules.checks_to_lose
    }
    
    #[inline]
    pub fn leader_is_captured(&self) -> bool {
        self.get_num_leader_pieces(self.whos_turn) == 0
    }
    #[inline]
    pub fn enemy_leader_is_captured(&self) -> bool {
        self.get_num_leader_pieces(1 - self.whos_turn) == 0
    }
    #[inline]
    fn get_num_leader_pieces(&self, player: Player) -> u32 {
        if let Some(leader) = self.pieces[player as usize].get_leader() {
            leader.get_num_pieces()
        } else {
            self.pieces[player as usize].get_occupied().count_ones()
        }
    }

    pub fn piece_at(&self, index: BIndex) -> Option<&Piece> {
        for ps in &self.pieces {
            if let Some(piece) = ps.piece_at(index) {
                return Some(piece);
            }
        }
        None
    }
    pub fn piece_at_mut(&mut self, index: BIndex) -> Option<&mut Piece> {
        for ps in &mut self.pieces {
            if let Some(piece) = ps.piece_at_mut(index) {
                return Some(piece);
            }
        }
        None
    }
    pub fn player_piece_at(&self, player: Player, index: BIndex) -> Option<&Piece> {
        self.pieces[player as usize].piece_at(index)
    }
    
    /// Returns if the point is in bounds
    pub fn in_bounds(&self, x: BCoord, y: BCoord) -> bool {
        self.dimensions.in_bounds(x, y)
    }

    
    /// Public interface for modifying the position
    pub fn public_add_piece(&mut self, piece_id: PieceId, index: BIndex, can_castle: bool) -> wrap_res!() {
        // Search piece with this id in both players
        let mut owner = None;
        for ps in &mut self.pieces {
            if ps.contains_piece(piece_id) {
                err_assert!(!ps.index_has_piece(index), "Attempted to add piece {piece_id} to square that was already occupied: {index}");
                owner = Some(ps.get_player_num());
            }
        }
        err_assert!(owner.is_some(), "Attempted to add piece with ID={piece_id}, which doesn't exist");
        let owner = owner.unwrap();
        
        let mut zob = self.get_zobrist();
        self.pieces[owner as usize].add_piece(piece_id, index, can_castle);
        let piece = self.player_piece_at(owner, index).unwrap();
        // Update the zobrist key
        zob ^= piece.get_zobrist(index);
        if can_castle && piece.used_in_castling() {
            zob ^= piece.get_castle_zobrist(index);
        }
        self.update_occupied();
        let stack_len = self.properties_stack.len();
        self.properties_stack[stack_len - 1].zobrist_key = zob;
        Ok(())
    }

    /// Removes a piece from the position, assuming the piece is there
    pub fn public_remove_piece(&mut self, index: BIndex) -> wrap_res!() {
        let owner = {
            if self.pieces[0].index_has_piece(index) { 0 }
            else if self.pieces[1].index_has_piece(index) { 1 }
            else { err!("Attempted to remove piece from square that was empty") }
        };
        let mut zob = self.get_zobrist();
        let piece = self.piece_at_mut(index).unwrap();
        // Update the zobrist key
        zob ^= piece.get_zobrist(index);
        let used_in_castling = piece.used_in_castling();
        let castle_zob = piece.get_castle_zobrist(index);
        let could_casle = self.pieces[owner].remove_piece(index);
        if could_casle && used_in_castling {
            zob ^= castle_zob;
        }
        self.update_occupied();
        let stack_len = self.properties_stack.len();
        self.properties_stack[stack_len - 1].zobrist_key = zob;
        Ok(())
    }
    
    /// Returns true if any of the pieces on the board is on a winning square
    pub fn piece_is_on_winning_square(&self) -> bool {
        for piece_set in &self.pieces {
            for p in piece_set.iter() {
                if p.is_in_win_square() {
                    return true;
                }
            }
        }
        false
    }
    
    
    
    /// Get the top of the properties stack
    #[inline]
    fn get_properties(&self) -> &PositionProperties {
        &self.properties_stack[self.properties_stack.len() - 1]
    }

    /// Updates the occupied bitboard
    /// Must be called after every position update/modification
    fn update_occupied(&mut self) {
        self.occ_or_out_bounds = !&self.dimensions.bounds;
        for (_i, ps) in self.pieces.iter_mut().enumerate() {
            ps.update_occupied();
            self.occ_or_out_bounds |= ps.get_occupied();
        }
    }
}

impl fmt::Display for Position {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        for y in (0..self.dimensions.height).rev() {
            write!(f, "{:2} ", y+1)?;
            for x in 0..self.dimensions.width {
                if let Some(piece) = self.piece_at(to_index(x,y)) {
                    write!(f, "{} ", piece)?;
                } else if self.dimensions.in_bounds(x, y) {
                    write!(f, ". ")?;
                } else {
                    write!(f, "X ")?;
                }
            }
            writeln!(f)?;
        }
        write!(f, "   ")?;
        for x in 0..self.dimensions.width {
            write!(f, "{} ", (b'A'+x) as char)?;
        }
        write!(f, "\nZobrist Key: {:x}", self.get_zobrist())?;
        write!(f, "\nPlayer to move: {}", self.whos_turn)
    }
}

impl PartialEq for Position {
    fn eq(&self, other: &Position) -> bool {
        self.dimensions == other.dimensions &&
        self.whos_turn == other.whos_turn &&
        self.pieces == other.pieces &&
        self.occ_or_out_bounds == other.occ_or_out_bounds &&
        // Only compare the top of the stack, since the history may be different
        self.get_properties().zobrist_key == other.get_properties().zobrist_key &&
        // Don't compare captures stack, since the history may be different
        self.global_rules == other.global_rules
    }
}
