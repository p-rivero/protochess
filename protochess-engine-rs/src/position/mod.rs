use std::collections::HashMap;
use std::fmt;
use std::rc::Rc;

use crate::{types::*, PieceDefinition};
use crate::position::piece_set::PieceSet;
use crate::utils::{from_index, to_index};
use crate::piece::{Piece, PieceId};

mod position_properties;
mod parse_fen;
pub mod castled_players;
pub mod piece_set;

use position_properties::PositionProperties;

/// Represents a single position in chess
#[derive(Clone, Debug)]
pub struct Position {
    pub dimensions: BDimensions,
    pub num_players: Player,
    pub whos_turn: Player,
    pub pieces: Vec<PieceSet>, // pieces[0] = white's pieces, pieces[1] black etc
    pub occupied: Bitboard,
    // Properties relating only to the current position
    // Typically hard-to-recover properties, like castling
    // Similar to state in stockfish
    pub properties: Rc<PositionProperties>,
    // Store the positions that have been played to determine repetition
    pub position_repetitions: HashMap<u64, u8, ahash::RandomState>,
}

impl Position {
    pub fn default() -> Position {        
        Position::from_fen("rnbqkbnr/pppppppp/8/8/8/8/PPPPPPPP/RNBQKBNR w KQkq - 0 1")
    }
    pub fn from_fen(fen: &str) -> Position {
        parse_fen::parse_fen(fen)
    }
    pub fn empty() -> Position {
        Position::new(BDimensions::default(), vec![], 0, PositionProperties::default())
    }
    pub fn custom(dims: BDimensions,
                  piece_types: &Vec<PieceDefinition>,
                  pieces: Vec<(Player, BIndex, PieceId)>,
                  whos_turn: Player) -> Position
    {
        let num_players = Position::assert_all_players_have_leader(piece_types, &pieces);
        let mut piece_sets = Vec::with_capacity(num_players as usize);
        for p in 0..num_players {
            piece_sets.push(PieceSet::new(p));
        }
        
        let mut pos = Position::new(dims, piece_sets, whos_turn, PositionProperties::default());
        for definition in piece_types {
            pos.register_piecetype(definition);
        }

        for (owner, index, piece_type) in pieces {
            pos.public_add_piece(owner, piece_type, index);
        }
        pos
    }
    
    fn new(dimensions: BDimensions, pieces: Vec<PieceSet>, whos_turn: Player, props: PositionProperties) -> Position {
        let mut occupied = Bitboard::zero();
        for piece_set in &pieces {
            occupied |= piece_set.get_occupied();
        }
        let mut reps = HashMap::with_capacity_and_hasher(4096, ahash::RandomState::new());
        
        // Add a repetition for the starting position
        reps.insert(props.zobrist_key, 1);
        
        Position {
            dimensions,
            num_players: pieces.len() as Player,
            whos_turn,
            pieces,
            occupied,
            properties: Rc::new(props),
            position_repetitions: reps,
        }
    }

    /// Registers a new piece type for this position
    pub fn register_piecetype(&mut self, definition: &PieceDefinition) {
        // Insert piece for all players
        let dims_copy = self.dimensions.clone();
        for piece_set in &mut self.pieces {
            piece_set.register_piecetype(definition.clone(), &dims_copy);
        }
    }


    /// Modifies the position to make the move
    pub fn make_move(&mut self, mv: Move, update_reps: bool) {
        let my_player_num = self.whos_turn;
        let mut new_props: PositionProperties = (*self.properties).clone();
        new_props.update_reps = update_reps;
        
        // Update the player
        self.whos_turn += 1;
        if self.whos_turn == self.num_players {
            self.whos_turn = 0;
        }
        // Update the player zobrist key
        // For simplicity, use the top bit to represent the player
        new_props.zobrist_key ^= 0x8000000000000000;
        
        // In the special case of the null move, don't do anything except update whos_turn
        // And update props
        if mv.is_null() {
            // Update props
            // Since we're passing, there cannot be an ep square
            new_props.ep_square = None;
            new_props.move_played = Some(mv);
            new_props.prev_properties = Some(Rc::clone(&self.properties));
            // Increment number of repetitions of new position
            if update_reps {
                self.update_repetitions(new_props.zobrist_key, 1);
            }
            
            self.properties = Rc::new(new_props);
            return;
        }

        //Special moves
        match mv.get_move_type() {
            MoveType::Capture | MoveType::PromotionCapture => {
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
                new_props.captured_piece = Some((piece_id, capt_player, could_castle));
            },
            MoveType::KingsideCastle => {
                let rook_from = mv.get_target();
                let rook_to = mv.get_to() - 1;
                let rook_piece = self.player_piece_at(my_player_num, rook_from).unwrap();
                new_props.zobrist_key ^= rook_piece.get_zobrist(rook_from);
                new_props.zobrist_key ^= rook_piece.get_zobrist(rook_to);
                new_props.zobrist_key ^= rook_piece.get_castle_zobrist(rook_from);
                self.move_piece(my_player_num, rook_from, rook_to, false);
                new_props.castled_players.set_player_castled(my_player_num);
            },
            MoveType::QueensideCastle => {
                let rook_from = mv.get_target();
                let rook_to = mv.get_to() + 1;
                let rook_piece = self.player_piece_at(my_player_num, rook_from).unwrap();
                new_props.zobrist_key ^= rook_piece.get_zobrist(rook_from);
                new_props.zobrist_key ^= rook_piece.get_zobrist(rook_to);
                new_props.zobrist_key ^= rook_piece.get_castle_zobrist(rook_from);
                self.move_piece(my_player_num, rook_from, rook_to, false);
                new_props.castled_players.set_player_castled(my_player_num);
            }
            _ => {}
        }

        let from = mv.get_from();
        let to = mv.get_to();
        let moved_piece = self.player_piece_at_mut(my_player_num, from).unwrap();
        let moved_piece_type = moved_piece.get_piece_id();
        let moved_piece_new_pos_zobrist = moved_piece.get_zobrist(to);
        new_props.zobrist_key ^= moved_piece.get_zobrist(from);
        new_props.zobrist_key ^= moved_piece_new_pos_zobrist;
        

        // Move piece to location
        new_props.moved_piece_castle = moved_piece.move_piece(from, to, false);
        if new_props.moved_piece_castle {
            // A castling piece was moved, so it cannot castle anymore
            // Remove the castling ability from the zobrist key
            new_props.zobrist_key ^= moved_piece.get_castle_zobrist(from);
        }
        // Promotion
        match mv.get_move_type() {
            MoveType::PromotionCapture | MoveType::Promotion => {
                // Remove zobrist hash of the old piece
                new_props.zobrist_key ^= moved_piece_new_pos_zobrist;
                new_props.promote_from = Some(moved_piece_type);
                // Remove old piece
                self.player_piece_at_mut(my_player_num, to).unwrap().remove_piece(to);
                // Add new piece
                let promote_to_pt = mv.get_promotion_piece().unwrap();
                let piece = self.add_piece(my_player_num, promote_to_pt, to, false);
                new_props.zobrist_key ^= piece.get_zobrist(to);
            },
            _ => {}
        };

        // Pawn en-passant
        // Check for a pawn double push to set ep square
        // For simplicity, use the ep index as the zobrist key
        if let Some(sq) = self.properties.ep_square {
            // If the last prop had some ep square then we want to clear zob by xoring again
            new_props.zobrist_key ^= sq as u64;
        }

        if mv.get_move_type() == MoveType::DoubleJump {
            new_props.ep_square = Some(mv.get_target());
            new_props.ep_victim = mv.get_to();
            
            new_props.zobrist_key ^= mv.get_target() as u64;
        } else {
            new_props.ep_square = None;
        }
        
        // Increment number of repetitions of new position
        if update_reps {
            self.update_repetitions(new_props.zobrist_key, 1);
        }

        // Update props
        new_props.move_played = Some(mv);
        new_props.prev_properties = Some(Rc::clone(&self.properties));
        self.properties = Rc::new(new_props);
        
        // Update occupied bbs for future calculations
        self.update_occupied();
    }

    /// Undo the most recent move
    pub fn unmake_move(&mut self) {
        
        // Decrement the number of repetitions of old position
        if self.properties.update_reps {
            self.update_repetitions(self.properties.zobrist_key, -1);
        }

        if self.whos_turn == 0 {
            self.whos_turn = self.num_players - 1;
        } else {
            self.whos_turn -= 1;
        }

        let my_player_num = self.whos_turn;
        let mv = self.properties.move_played.expect("No move to undo");
        
        // Undo null moves
        if mv.get_move_type() == MoveType::Null {
            //Update props
            //Consume prev props; never to return again
            self.properties = self.properties.get_prev().unwrap();
            return;
        }
        let from = mv.get_from();
        let to = mv.get_to();

        //Undo move piece to location
        //Remove piece here
        self.move_piece(my_player_num, to, from, self.properties.moved_piece_castle);
        //Undo Promotion
        match mv.get_move_type() {
            MoveType::PromotionCapture | MoveType::Promotion => {
                // Remove old piece
                self.player_piece_at_mut(my_player_num, from).unwrap().remove_piece(from);
                let promoted_from = self.properties.promote_from.unwrap();
                // Assume that the piece that promoted must have moved, so it can't castle
                self.add_piece(my_player_num, promoted_from, from, false);
            },
            _ => {}
        };

        //Undo special moves
        //Special moves
        match mv.get_move_type() {
            MoveType::Capture | MoveType::PromotionCapture => {
                let capt = mv.get_target();
                let (piece_id, owner, captured_can_castle) = self.properties.captured_piece.unwrap();
                self.add_piece(owner, piece_id, capt, captured_can_castle);
            },
            MoveType::KingsideCastle => {
                let rook_from = mv.get_target();
                let (x, y) = from_index(mv.get_to());
                let rook_to = to_index(x - 1, y);
                self.move_piece(self.whos_turn, rook_to, rook_from, true);
            },
            MoveType::QueensideCastle => {
                let rook_from = mv.get_target();
                let (x, y) = from_index(mv.get_to());
                let rook_to = to_index(x + 1, y);
                self.move_piece(self.whos_turn, rook_to, rook_from, true);
            }
            _ => {}
        }

        // Update props
        // Consume prev props; never to return again
        self.properties = self.properties.get_prev().unwrap();

        //Update occupied bbs for future calculations
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

    pub fn get_zobrist(&self) -> u64 {
        self.properties.zobrist_key
    }
    
    pub fn num_repetitions(&self) -> u8 {
        let num_reps = self.position_repetitions.get(&self.properties.zobrist_key);
        *num_reps.unwrap()
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
        // Pieces are registered in all piecesets, just search the first one
        self.pieces[0].search_by_id(piece_id)
    }
    
    /// Returns if the point is in bounds
    pub fn in_bounds(&self, x: BCoord, y: BCoord) -> bool {
        self.dimensions.in_bounds(x, y)
    }

    
    /// Public interface for modifying the position
    pub fn public_add_piece(&mut self, owner: Player, piece_type: PieceId, index: BIndex) {
        // Subtract a repetition for the old position
        self.update_repetitions(self.properties.zobrist_key, -1);
        
        let mut new_props = (*self.properties).clone();
        let piece = self.add_piece(owner, piece_type, index, true);
        new_props.zobrist_key ^= piece.get_zobrist(index);
        self.update_occupied();
        new_props.prev_properties = Some(Rc::clone(&self.properties));
        self.properties = Rc::new(new_props);
        
        // Add a repetition for the new position
        self.update_repetitions(self.properties.zobrist_key, 1);
    }

    /// Removes a piece from the position, assuming the piece is there
    pub fn public_remove_piece(&mut self, index: BIndex) {
        self.piece_at_mut(index).unwrap().remove_piece(index);
        self.update_occupied();
    }
    
    
    
    /// Compute the number of players and assert that each player has a leader. Returns the number of players
    fn assert_all_players_have_leader(piece_types: &Vec<PieceDefinition>, pieces: &Vec<(Player, BIndex, PieceId)>) -> Player {
        assert!(!piece_types.is_empty(), "No piece types defined");
        assert!(!pieces.is_empty(), "No pieces in board");
        let mut leader_pieces = Vec::new();
        for definition in piece_types {
            if definition.is_leader {
                leader_pieces.push(definition.id);
            }
        }
        // The number of players is (max player number) + 1
        let num_players = pieces.iter().map(|(p, _, _)| p).max().unwrap() + 1;
        let mut player_has_leader = vec![false; num_players as usize];
        for (owner, _, piece_type) in pieces {
            if leader_pieces.contains(piece_type) {
                player_has_leader[*owner as usize] = true;
            }
        }
        for (i, has_leader) in player_has_leader.iter().enumerate() {
            assert!(has_leader, "Player {} does not have a leader piece", i);
        }
        num_players
    }
    
    /// Adds a piece to the position, assuming the piecetype already exists
    /// Returns the piece that was added
    fn add_piece(&mut self, owner: Player, piece_id: PieceId, index: BIndex, can_castle: bool) -> &Piece {
        let piece = self.pieces[owner as usize].iter_mut().find(|c| c.get_piece_id() == piece_id).unwrap();
        piece.add_piece(index, can_castle);
        piece
    }
    
    /// Move a piece from one index to another
    /// If set_can_castle is true, set the new index as a castle square.
    /// Returns true if the piece could castle before this move
    fn move_piece(&mut self, player: Player,  from: BIndex, to: BIndex, can_castle: bool) -> bool {
        if let Some(piece) = self.player_piece_at_mut(player, from) {
            piece.move_piece(from, to, can_castle)
        } else {
            panic!("No piece at {} to move to {}", from, to);
        }
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
            write!(f, "{} ", y)?;
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
        write!(f, "  ")?;
        for x in 0..self.dimensions.width {
            write!(f, "{} ", x)?;
        }
        write!(f, "\nZobrist Key: {}", self.properties.zobrist_key)
    }
}
