use std::fmt;

use crate::{types::*, PieceDefinition};
use crate::utils::{from_index, to_index};
use crate::piece::{Piece, PieceId};

mod position_properties;
pub mod global_rules;
pub mod game_state;
pub mod castled_players;
pub mod piece_set;

use global_rules::{GlobalRules, GlobalRulesInternal};
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
    pub global_rules: GlobalRulesInternal,
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
            global_rules: rules.into(),
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
    pub fn make_move(&mut self, mv: Move) {
        let my_player_num = self.whos_turn;
        let mut new_props = *self.get_properties(); // Copy the current properties
        new_props.num_captures = 0;
        let move_type = mv.get_move_type();
        
        // Update the player
        self.whos_turn = 1 - self.whos_turn;
        // Update the player zobrist key
        // For simplicity, use the top bit to represent the player
        new_props.zobrist_key ^= 0x8000000000000000;
        
        // In the special case of the null move, don't do anything except update whos_turn
        // And update props
        if move_type == MoveType::Null {
            // Update props
            // Since we're passing, there cannot be an ep square
            new_props.clear_ep_square();
            new_props.move_played = Some(mv);
            self.properties_stack.push(new_props);
            return;
        }

        // If this move is a capture, remove the captured piece before moving
        if move_type == MoveType::Capture || move_type == MoveType::PromotionCapture {
            let capt_index = mv.get_target();
            let captured_piece = self.player_piece_at(self.whos_turn, capt_index).unwrap();
            let piece_id = captured_piece.get_piece_id();
            let capt_player = captured_piece.get_player();
            let castling_zob = captured_piece.get_castle_zobrist(capt_index);
            new_props.zobrist_key ^= captured_piece.get_zobrist(capt_index);
    
            let could_castle = self.pieces[capt_player as usize].remove_piece(capt_index);
            if could_castle {
                new_props.zobrist_key ^= castling_zob
            }
            self.captures_stack.push((piece_id, capt_player, could_castle, capt_index));
            new_props.num_captures += 1;
    
            // Check if the capturing piece explodes
            self.explode_piece(mv, my_player_num, &mut new_props);
        }
        
        // If this move is a castle, first remove the rook (in chess960 the king could move to the rook's square
        // and the rook would be overwritten)
        let mut rook_id = None;
        if move_type == MoveType::KingsideCastle || move_type == MoveType::QueensideCastle {
            let rook_from = mv.get_target();
            let rook_piece = self.player_piece_at(my_player_num, rook_from).unwrap();
            new_props.zobrist_key ^= rook_piece.get_zobrist(rook_from);
            new_props.zobrist_key ^= rook_piece.get_castle_zobrist(rook_from);
            rook_id = Some(rook_piece.get_piece_id());
            self.pieces[my_player_num as usize].remove_piece(rook_from);
        }

        let from = mv.get_from();
        let to = mv.get_to();
        // Move the piece (only if it hasn't exploded)
        if self.pieces[my_player_num as usize].index_has_piece(from) {
            // Move piece to location
            new_props.moved_piece_castle = self.pieces[my_player_num as usize].move_piece(from, to, false);
            let moved_piece = self.player_piece_at(my_player_num, to).unwrap();
            new_props.zobrist_key ^= moved_piece.get_zobrist(from);
            new_props.zobrist_key ^= moved_piece.get_zobrist(to);
            if new_props.moved_piece_castle {
                // A castling piece was moved, so it cannot castle anymore
                // Remove the castling ability from the zobrist key
                new_props.zobrist_key ^= moved_piece.get_castle_zobrist(from);
            }
            
            // Promotion
            if let Some(promo) = mv.get_promotion_piece() {
                // Remove zobrist hash of the old piece
                new_props.zobrist_key ^= moved_piece.get_zobrist(to);
                new_props.promote_from = Some(moved_piece.get_piece_id());
                // Remove old piece
                self.pieces[my_player_num as usize].remove_piece(to);
                // Add new piece
                self.pieces[my_player_num as usize].add_piece(promo, to, false);
                let piece = self.player_piece_at(my_player_num, to).unwrap();
                new_props.zobrist_key ^= piece.get_zobrist(to);
            }
        }
        
        // If this move is a castle, add the rook back
        if move_type == MoveType::KingsideCastle || move_type == MoveType::QueensideCastle {
            let rook_to = {
                if move_type == MoveType::KingsideCastle { to - 1 }
                else { to + 1 }
            };
            self.pieces[my_player_num as usize].add_piece(rook_id.unwrap(), rook_to, false);
            let rook_piece = self.player_piece_at(my_player_num, rook_to).unwrap();
            new_props.zobrist_key ^= rook_piece.get_zobrist(rook_to);
            new_props.castled_players.set_player_castled(my_player_num);
        }

        // Pawn en-passant
        // Check for a pawn double push to set ep square
        if move_type == MoveType::DoubleJump {
            new_props.set_ep_square(mv.get_target(), mv.get_to())
        } else {
            new_props.clear_ep_square();
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
        let moved_piece = self.pieces[my_player_num as usize].piece_at_mut(from).unwrap();
        if !moved_piece.explodes() {
            return;
        }
        new_props.zobrist_key ^= moved_piece.get_zobrist(from);
        let moved_piece_castle_zob = moved_piece.get_castle_zobrist(from);
        let moved_piece_id = moved_piece.get_piece_id();
        // Remove the capturing piece
        let capturing_could_castle = self.pieces[my_player_num as usize].remove_piece(from);
        if capturing_could_castle {
            new_props.zobrist_key ^= moved_piece_castle_zob;
        }
        self.captures_stack.push((moved_piece_id, my_player_num, capturing_could_castle, from));
        new_props.num_captures += 1;
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
                if let Some(exploded_piece) = self.piece_at_mut(nindex) {
                    if exploded_piece.immune_to_explosion() {
                        continue;
                    }
                    new_props.zobrist_key ^= exploded_piece.get_zobrist(nindex);
                    let exploded_id = exploded_piece.get_piece_id();
                    let exploded_player = exploded_piece.get_player();
                    let exploded_castle_zob = exploded_piece.get_castle_zobrist(nindex);
                    let could_castle = self.pieces[exploded_player as usize].remove_piece(nindex);
                    if could_castle {
                        new_props.zobrist_key ^= exploded_castle_zob;
                    }
                    self.captures_stack.push((exploded_id, exploded_player, could_castle, nindex));
                    new_props.num_captures += 1;
                }
            }
        }
    }

    /// Undo the most recent move
    pub fn unmake_move(&mut self) {
        // Update props
        // Consume prev props; never to return again
        let props = self.properties_stack.pop().unwrap();
        
        // Update player turn
        self.whos_turn = 1 - self.whos_turn;

        let my_player_num = self.whos_turn;
        let mv = props.move_played.expect("No move to undo");
        let move_type = mv.get_move_type();
        
        // Undo null moves
        if move_type == MoveType::Null {
            return;
        }
        let from = mv.get_from();
        let to = mv.get_to();
        
        // If this move is a castle, remove the rook
        let mut rook_id = None;
        if move_type == MoveType::KingsideCastle || move_type == MoveType::QueensideCastle {
            let rook_to = {
                if move_type == MoveType::KingsideCastle { to - 1 }
                else { to + 1 }
            };
            let rook_piece = self.pieces[my_player_num as usize].piece_at_mut(rook_to).unwrap();
            rook_id = Some(rook_piece.get_piece_id());
            self.pieces[my_player_num as usize].remove_piece(rook_to);
        }

        // Undo move piece to location
        if self.pieces[my_player_num as usize].index_has_piece(to) {
            self.pieces[my_player_num as usize].move_piece(to, from, props.moved_piece_castle);
            
            // Undo Promotion
            if move_type == MoveType::Promotion || move_type == MoveType::PromotionCapture {
                // Remove old piece
                self.pieces[my_player_num as usize].remove_piece(from);
                let promoted_from = props.promote_from.unwrap();
                // Assume that the piece that promoted must have moved, so it can't castle
                self.pieces[my_player_num as usize].add_piece(promoted_from, from, false);
            }
        }

        // Undo special moves
        // Special moves
        match move_type {
            MoveType::Capture | MoveType::PromotionCapture => {
                for _ in 0..props.num_captures {
                    let (piece_id, owner, captured_can_castle, capt_index) = self.captures_stack.pop().unwrap();
                    self.pieces[owner as usize].add_piece(piece_id, capt_index, captured_can_castle);
                }
            },
            MoveType::KingsideCastle | MoveType::QueensideCastle => {
                // Add back the rook
                let rook_from = mv.get_target();
                self.pieces[my_player_num as usize].add_piece(rook_id.unwrap(), rook_from, true);
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
        let mut num_reps = 1;
        let my_zob = self.get_zobrist();
        for p in &self.properties_stack {
            if p.zobrist_key == my_zob {
                num_reps += 1;
            }
        }
        num_reps
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
        if let Some(leader) = self.pieces[self.whos_turn as usize].get_leader() {
            leader.get_num_pieces() == 0
        } else {
            self.pieces[self.whos_turn as usize].get_occupied().count_ones() == 0
        }
    }
    #[inline]
    pub fn enemy_leader_is_captured(&self) -> bool {
        if let Some(leader) = self.pieces[1 - self.whos_turn as usize].get_leader() {
            leader.get_num_pieces() == 0
        } else {
            self.pieces[1 - self.whos_turn as usize].get_occupied().count_ones() == 0
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
    
    pub fn get_piece_char(&self, player: Player, piece_id: PieceId) -> Option<char> {
        self.pieces[player as usize].get_piece_char(piece_id)
    }
    
    /// Returns if the point is in bounds
    pub fn in_bounds(&self, x: BCoord, y: BCoord) -> bool {
        self.dimensions.in_bounds(x, y)
    }

    
    /// Public interface for modifying the position
    pub fn public_add_piece(&mut self, owner: Player, piece_type: PieceId, index: BIndex, can_castle: bool) {
        let mut zob = self.get_zobrist();
        self.pieces[owner as usize].add_piece(piece_type, index, can_castle);
        let piece = self.player_piece_at(owner, index).unwrap();
        // Update the zobrist key
        zob ^= piece.get_zobrist(index);
        if can_castle && piece.used_in_castling() {
            zob ^= piece.get_castle_zobrist(index);
        }
        self.update_occupied();
        let stack_len = self.properties_stack.len();
        self.properties_stack[stack_len - 1].zobrist_key = zob;
    }

    /// Removes a piece from the position, assuming the piece is there
    pub fn public_remove_piece(&mut self, index: BIndex) {
        let owner = {
            if self.pieces[0].index_has_piece(index) { 0 }
            else { 1 }
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
        write!(f, "\nZobrist Key: {:x}", self.get_zobrist())?;
        write!(f, "\nPlayer to move: {}", self.whos_turn)
    }
}
