use ahash::AHashMap;
use std::fmt;

use crate::{types::*, PieceDefinition};
use crate::utils::{from_index, to_index};
use crate::piece::{Piece, PieceId};

mod position_properties;
pub mod game_state;
pub mod castled_players;
pub mod piece_set;

use position_properties::PositionProperties;
use piece_set::PieceSet;

/// Represents a single position in chess
#[derive(Clone, Debug)]
pub struct Position {
    pub dimensions: BDimensions,
    pub whos_turn: Player,
    pub pieces: [PieceSet; 2], // pieces[0] = white, pieces[1] = black
    pub occupied: Bitboard,
    // Stack of properties relating only to the current position
    // Typically hard-to-recover properties, like castling
    // Similar to state in stockfish
    properties_stack: Vec<PositionProperties>,
    // Store the positions that have been played to determine repetition
    position_repetitions: AHashMap<u64, u8>,
}

impl Position {
    fn new(dimensions: BDimensions, whos_turn: Player, props: PositionProperties) -> Position {
        // Add a repetition for the starting position
        let mut position_repetitions = AHashMap::with_capacity(4096);
        position_repetitions.insert(props.zobrist_key, 1);
        
        let mut properties_stack = Vec::with_capacity(128);
        properties_stack.push(props);
        
        Position {
            dimensions,
            whos_turn,
            pieces: [PieceSet::new(0), PieceSet::new(1)],
            occupied: Bitboard::zero(),
            properties_stack,
            position_repetitions,
        }
    }

    /// Registers a new piece type for a given player in this position
    pub fn register_piecetype(&mut self, definition: &PieceDefinition) {
        // Insert piece for all players specified in the definition
        for player in &definition.available_for {
            assert!(*player < self.pieces.len() as Player, "Player {} does not exist", player);
            self.pieces[*player as usize].register_piecetype(definition.clone(), &self.dimensions);
        }
    }


    /// Modifies the position to make the move
    pub fn make_move(&mut self, mv: Move, update_reps: bool) {
        let my_player_num = self.whos_turn;
        let mut new_props: PositionProperties = self.get_properties().cheap_clone();
        
        // Update the player
        self.whos_turn = 1 - self.whos_turn;
        // Update the player zobrist key
        // For simplicity, use the top bit to represent the player
        new_props.zobrist_key ^= 0x8000000000000000;
        
        // In the special case of the null move, don't do anything except update whos_turn
        // And update props
        if mv.is_null() {
            // Update props
            // Since we're passing, there cannot be an ep square
            new_props.clear_ep_square();
            new_props.move_played = Some(mv);
            self.properties_stack.push(new_props);
            return;
        }

        // If this move is a capture, remove the captured piece before moving
        let move_type = mv.get_move_type();
        if move_type == MoveType::Capture || move_type == MoveType::PromotionCapture {
            let capt_index = mv.get_target();
            let captured_piece = self.player_piece_at(self.whos_turn, capt_index).unwrap();
            let piece_id = captured_piece.get_piece_id();
            let capt_player = captured_piece.get_player();
            let castling_zob = captured_piece.get_castle_zobrist(capt_index);
            new_props.zobrist_key ^= captured_piece.get_zobrist(capt_index);
    
            let could_castle = self.player_piece_at_mut(capt_player, capt_index).unwrap().remove_piece(capt_index);
            if could_castle {
                new_props.zobrist_key ^= castling_zob
            }
            new_props.captured_pieces.push((piece_id, capt_player, could_castle, capt_index));
    
            // Check if the capturing piece explodes
            self.explode_piece(mv, my_player_num, &mut new_props);
        }

        let from = mv.get_from();
        let to = mv.get_to();
        // Move the piece (only if it hasn't exploded)
        if let Some(moved_piece) = self.player_piece_at_mut(my_player_num, from) {
            // Move piece to location
            new_props.moved_piece_castle = moved_piece.move_piece(from, to, false);
            new_props.zobrist_key ^= moved_piece.get_zobrist(from);
            new_props.zobrist_key ^= moved_piece.get_zobrist(to);
            if new_props.moved_piece_castle {
                // A castling piece was moved, so it cannot castle anymore
                // Remove the castling ability from the zobrist key
                new_props.zobrist_key ^= moved_piece.get_castle_zobrist(from);
            }
            
            // Promotion
            match mv.get_move_type() {
                MoveType::PromotionCapture | MoveType::Promotion => {
                    // Remove zobrist hash of the old piece
                    new_props.zobrist_key ^= moved_piece.get_zobrist(to);
                    new_props.promote_from = Some(moved_piece.get_piece_id());
                    // Remove old piece
                    self.player_piece_at_mut(my_player_num, to).unwrap().remove_piece(to);
                    // Add new piece
                    let promote_to_pt = mv.get_promotion_piece().unwrap();
                    let piece = self.add_piece(my_player_num, promote_to_pt, to, false);
                    new_props.zobrist_key ^= piece.get_zobrist(to);
                },
                _ => {}
            };
        }
        
        // If this move is a castle, also move the rook
        if move_type == MoveType::KingsideCastle || move_type == MoveType::QueensideCastle {
            let rook_from = mv.get_target();
            let rook_to = {
                if mv.get_move_type() == MoveType::KingsideCastle { to - 1 }
                else { to + 1 }
            };
            // At this point, it's possible that in "rook_from" there are 2 pieces at the same time
            // (the moved king and the rook waiting to be moved). This only happens in chess960.
            // Use rook_at_mut() to make sure that we are moving the rook and not the king
            let rook_piece = self.pieces[my_player_num as usize].rook_at_mut(rook_from).unwrap();
            new_props.zobrist_key ^= rook_piece.get_zobrist(rook_from);
            new_props.zobrist_key ^= rook_piece.get_zobrist(rook_to);
            new_props.zobrist_key ^= rook_piece.get_castle_zobrist(rook_from);
            rook_piece.move_piece(rook_from, rook_to, false);
            new_props.castled_players.set_player_castled(my_player_num);
        }

        // Pawn en-passant
        // Check for a pawn double push to set ep square
        if mv.get_move_type() == MoveType::DoubleJump {
            new_props.set_ep_square(mv.get_target(), mv.get_to())
        } else {
            new_props.clear_ep_square();
        }
        
        // Increment number of repetitions of new position
        if update_reps {
            new_props.update_reps = true;
            self.update_repetitions(new_props.zobrist_key, 1);
        }

        // Update props
        new_props.move_played = Some(mv);
        self.properties_stack.push(new_props);
        
        // Update occupied bbs for future calculations
        self.update_occupied();
    }

    #[inline]
    fn explode_piece(&mut self, mv: Move, my_player_num: u8, new_props: &mut PositionProperties) {
        let from = mv.get_from();
        let moved_piece = self.player_piece_at_mut(my_player_num, from).unwrap();
        if !moved_piece.explodes() {
            return;
        }
        // Remove the capturing piece
        let capturing_could_castle = moved_piece.remove_piece(from);
        new_props.zobrist_key ^= moved_piece.get_zobrist(from);
        if capturing_could_castle {
            new_props.zobrist_key ^= moved_piece.get_castle_zobrist(from);
        }
        new_props.captured_pieces.push((moved_piece.get_piece_id(), my_player_num, capturing_could_castle, from));
        // Remove all pieces in the explosion radius
        let (x, y) = from_index(mv.get_to());
        
        for dx in -1..=1 {
            for dy in -1..=1 {
                if dx == 0 && dy == 0 {
                    continue;
                }
                let (nx, ny) = (x as i8 + dx, y as i8 + dy);
                if nx < 0 || ny < 0 || !self.dimensions.in_bounds(nx as BCoord, ny as BCoord) {
                    continue;
                }
                let nindex = to_index(nx as BCoord, ny as BCoord);
                if let Some(piece) = self.piece_at_mut(nindex) {
                    if piece.immune_to_explosion() {
                        continue;
                    }
                    let could_castle = piece.remove_piece(nindex);
                    new_props.zobrist_key ^= piece.get_zobrist(nindex);
                    if could_castle {
                        new_props.zobrist_key ^= piece.get_castle_zobrist(nindex);
                    }
                    new_props.captured_pieces.push((piece.get_piece_id(), piece.get_player(), could_castle, nindex));
                }
            }
        }
    }

    /// Undo the most recent move
    pub fn unmake_move(&mut self) {
        // Update props
        // Consume prev props; never to return again
        let props = self.properties_stack.pop().unwrap();
        
        // Decrement the number of repetitions of old position
        if props.update_reps {
            self.update_repetitions(props.zobrist_key, -1);
        }

        // Update player turn
        self.whos_turn = 1 - self.whos_turn;

        let my_player_num = self.whos_turn;
        let mv = props.move_played.expect("No move to undo");
        
        // Undo null moves
        if mv.is_null() {
            return;
        }
        let from = mv.get_from();
        let to = mv.get_to();

        // Undo move piece to location
        let moved_piece_castle = props.moved_piece_castle;
        if let Some(moved_piece) = self.player_piece_at_mut(my_player_num, to) {
            moved_piece.move_piece(to, from, moved_piece_castle);
            
            // Undo Promotion
            match mv.get_move_type() {
                MoveType::PromotionCapture | MoveType::Promotion => {
                    // Remove old piece
                    moved_piece.remove_piece(from);
                    let promoted_from = props.promote_from.unwrap();
                    // Assume that the piece that promoted must have moved, so it can't castle
                    self.add_piece(my_player_num, promoted_from, from, false);
                },
                _ => {}
            };
        }

        // Undo special moves
        // Special moves
        match mv.get_move_type() {
            MoveType::Capture | MoveType::PromotionCapture => {
                let num_captures = props.captured_pieces.len();
                for i in 0..num_captures {
                    let (piece_id, owner, captured_can_castle, capt_index) = props.captured_pieces[i];
                    self.add_piece(owner, piece_id, capt_index, captured_can_castle);
                }
            },
            MoveType::KingsideCastle | MoveType::QueensideCastle => {
                let rook_from = mv.get_target();
                let (x, y) = from_index(mv.get_to());
                let rook_x = if mv.get_move_type() == MoveType::KingsideCastle { x - 1 } else { x + 1 };
                let rook_to = to_index(rook_x, y);
                // At this point, it's possible that in "rook_to" there are 2 pieces at the same time
                // (the moved king and the rook waiting to be moved). This only happens in chess960.
                // Use rook_at_mut() to make sure that we are moving the rook and not the king
                let rook = self.pieces[self.whos_turn as usize].rook_at_mut(rook_to).unwrap();
                rook.move_piece(rook_to, rook_from, true);
            }
            _ => {}
        }
        
        // Update occupied bbs for future calculations
        self.update_occupied();
    }

    /// Return piece for (owner, x, y, char)
    pub fn pieces_as_tuples(&self) -> Vec<(Player, BCoord, BCoord, PieceId)>{
        let mut tuples = Vec::new();
        for (i, ps) in self.pieces.iter().enumerate() {
            for piece in ps.iter() {
                for index in piece.get_indexes() {
                    let (x, y) = from_index(index as BIndex);
                    tuples.push((i as Player, x, y, piece.get_piece_id()));
                }
            }
        }
        tuples
    }

    pub fn tiles_as_tuples(&self) -> Vec<(BCoord, BCoord, char)> {
        let mut squares = Vec::new();
        for x in 0..self.dimensions.width {
            for y in 0..self.dimensions.height {
                if self.in_bounds(x, y) {
                    let char_rep = if (x + y) % 2 == 0 {'b'} else {'w'};
                    squares.push((x, y, char_rep));
                } else {
                    squares.push((x, y, 'x'));
                }
            }
        }
        squares
    }

    #[inline]
    pub fn get_zobrist(&self) -> u64 {
        self.get_properties().zobrist_key
    }
    
    #[inline]
    pub fn num_repetitions(&self) -> u8 {
        let num_reps = self.position_repetitions.get(&self.get_zobrist());
        *num_reps.unwrap_or(&0)
    }
    
    #[inline]
    pub fn has_player_castled(&self, player: Player) -> bool {
        self.get_properties().castled_players.did_player_castle(player)
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
    pub fn leader_is_captured(&self) -> bool {
        self.pieces[self.whos_turn as usize].get_leader().get_num_pieces() == 0
    }
    #[inline]
    pub fn enemy_leader_is_captured(&self) -> bool {
        self.pieces[1 - self.whos_turn as usize].get_leader().get_num_pieces() == 0
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
    pub fn player_piece_at_mut(&mut self, player: Player, index: BIndex) -> Option<&mut Piece> {
        self.pieces[player as usize].piece_at_mut(index)
    }
    
    pub fn search_piece_by_id(&self, piece_id: PieceId) -> Option<&Piece> {
        for ps in &self.pieces {
            if let Some(piece) = ps.search_by_id(piece_id) {
                return Some(piece);
            }
        }
        None
    }
    
    /// Returns if the point is in bounds
    pub fn in_bounds(&self, x: BCoord, y: BCoord) -> bool {
        self.dimensions.in_bounds(x, y)
    }

    
    /// Public interface for modifying the position
    pub fn public_add_piece(&mut self, owner: Player, piece_type: PieceId, index: BIndex, can_castle: bool) {
        // Subtract a repetition for the old position
        let mut zob = self.get_zobrist();
        self.update_repetitions(zob, -1);
        
        let piece = self.add_piece(owner, piece_type, index, can_castle);
        // Update the zobrist key
        zob ^= piece.get_zobrist(index);
        if can_castle && piece.used_in_castling() {
            zob ^= piece.get_castle_zobrist(index);
        }
        self.update_occupied();
        // Add a repetition for the new position
        self.update_repetitions(zob, 1);
        
        let stack_len = self.properties_stack.len();
        self.properties_stack[stack_len - 1].zobrist_key = zob;
    }

    /// Removes a piece from the position, assuming the piece is there
    pub fn public_remove_piece(&mut self, index: BIndex) {
        // Subtract a repetition for the old position
        let mut zob = self.get_zobrist();
        self.update_repetitions(zob, -1);
        
        let piece = self.piece_at_mut(index).unwrap();
        let could_casle = piece.remove_piece(index);
        // Update the zobrist key
        zob ^= piece.get_zobrist(index);
        if could_casle && piece.used_in_castling() {
            zob ^= piece.get_castle_zobrist(index);
        }
        self.update_occupied();
        // Add a repetition for the new position
        self.update_repetitions(zob, 1);
        
        let stack_len = self.properties_stack.len();
        self.properties_stack[stack_len - 1].zobrist_key = zob;
    }
    
    
    
    /// Get the top of the properties stack
    #[inline]
    fn get_properties(&self) -> &PositionProperties {
        &self.properties_stack[self.properties_stack.len() - 1]
    }
    
    /// Adds a piece to the position, assuming the piecetype already exists
    /// Returns the piece that was added
    fn add_piece(&mut self, owner: Player, piece_id: PieceId, index: BIndex, can_castle: bool) -> &Piece {
        let piece = self.pieces[owner as usize].iter_mut().find(|c| c.get_piece_id() == piece_id).unwrap();
        piece.add_piece(index, can_castle);
        piece
    }

    /// Updates the occupied bitboard
    /// Must be called after every position update/modification
    fn update_occupied(&mut self) {
        self.occupied = Bitboard::zero();
        for (_i, ps) in self.pieces.iter_mut().enumerate() {
            ps.update_occupied();
            self.occupied |= ps.get_occupied();
        }
    }
    
    #[inline]
    fn update_repetitions(&mut self, key: u64, delta: i16) {
        let num_reps = self.position_repetitions.get(&key);
        if let Some(old_val) = num_reps {
            let new_val = (*old_val as i16 + delta) as u8;
            if new_val == 0 {
                self.position_repetitions.remove(&key);
            } else {
                self.position_repetitions.insert(key, new_val);
            }
        } else {
            self.position_repetitions.insert(key, delta as u8);
        }
    }
}

impl fmt::Display for Position {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        for y in (0..self.dimensions.height).rev() {
            write!(f, "{:2} ", y+1)?;
            for x in 0..self.dimensions.width {
                if let Some(piece) = self.piece_at(to_index(x,y)) {
                    write!(f, "{} ", piece.char_rep())?;
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
        write!(f, "\nZobrist Key: {:x}", self.get_zobrist())
    }
}
