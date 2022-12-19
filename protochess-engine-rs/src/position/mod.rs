use std::collections::HashMap;
use std::fmt;
use std::sync::Arc;

use crate::constants::piece_scores::*;
use crate::{types::*, PieceDefinition};
use crate::constants::fen;
use crate::position::piece_set::PieceSet;
use crate::utils::{from_index, to_index};

use zobrist_table::ZobristTable;
pub use parse_fen::parse_fen;
use position_properties::PositionProperties;

use crate::piece::{Piece, PieceFactory, PieceId, PieceIdWithPlayer};

mod position_properties;
pub mod castle_rights;
mod zobrist_table;
pub mod parse_fen;
pub mod piece_set;

//No reason to have more than one zobrist table
lazy_static! {
    static ref ZOBRIST_TABLE: ZobristTable = ZobristTable::new();
}


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
    pub properties: Arc<PositionProperties>,
    // Store the positions that have been played to determine repetition
    pub position_repetitions: HashMap<u64, u8, ahash::RandomState>,
}

impl Position {
    pub fn default() -> Position {        
        parse_fen(String::from(fen::STARTING_POS))
    }
    
    pub fn new(dimensions: BDimensions, pieces: Vec<PieceSet>, whos_turn: Player, props: PositionProperties) -> Position {
        let mut occupied = Bitboard::zero();
        for piece_set in pieces.iter() {
            occupied |= &piece_set.occupied;
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
            properties: Arc::new(props),
            position_repetitions: reps,
        }
    }

    /// Registers a new piece type for this position
    pub fn register_piecetype(&mut self, definition: &PieceDefinition) {
        // Insert blank for all players
        let dims_copy = self.dimensions.clone();
        for (i, piece_set) in self.pieces.iter_mut().enumerate() {
            let new_piece = PieceFactory::make_custom(definition.clone(), i as Player, &dims_copy);
            piece_set.custom.push(new_piece);
        }
    }


    /// Modifies the position to make the move
    pub fn make_move(&mut self, mv: Move) {
        let zobrist_table = &ZOBRIST_TABLE;
        let my_player_num = self.whos_turn;
        let mut new_props: PositionProperties = (*self.properties).clone();
        
        // Remove zobrist hash of the old player
        new_props.zobrist_key ^= zobrist_table.get_player_zobrist(self.whos_turn);
        // Update the player
        self.whos_turn += 1;
        if self.whos_turn == self.num_players {
            self.whos_turn = 0;
        }
        // Add zobrist hash of the new player
        new_props.zobrist_key ^= zobrist_table.get_player_zobrist(self.whos_turn);
        
        // In the special case of the null move, don't do anything except update whos_turn
        // And update props
        if mv.is_null() {
            // Update props
            // Since we're passing, there cannot be an ep square
            new_props.ep_square = None;
            new_props.move_played = Some(mv);
            new_props.prev_properties = Some(Arc::clone(&self.properties));
            // Increment number of repetitions of new position
            self.update_repetitions(new_props.zobrist_key, 1);
            
            self.properties = Arc::new(new_props);
            return;
        }

        //Special moves
        match mv.get_move_type() {
            MoveType::Capture | MoveType::PromotionCapture => {
                let capt_index = mv.get_target();
                let captured_piece = self.piece_at(capt_index).unwrap();
                new_props.zobrist_key ^= captured_piece.get_zobrist(capt_index);
                new_props.captured_piece = Some(captured_piece.get_full_id());
                self.remove_piece(capt_index);
            },
            MoveType::KingsideCastle => {
                let rook_from = mv.get_target();
                let (x, y) = from_index(mv.get_to());
                let rook_to = to_index(x - 1, y);
                new_props.zobrist_key ^= Piece::compute_zobrist_at(ID_ROOK, my_player_num, rook_from);
                new_props.zobrist_key ^= Piece::compute_zobrist_at(ID_ROOK, my_player_num, rook_to);
                self.move_piece(rook_from, rook_to);
                new_props.castling_rights.set_player_castled(my_player_num);
            },
            MoveType::QueensideCastle => {
                let rook_from = mv.get_target();
                let (x, y) = from_index(mv.get_to());
                let rook_to = to_index(x + 1, y);
                new_props.zobrist_key ^= Piece::compute_zobrist_at(ID_ROOK, my_player_num, rook_from);
                new_props.zobrist_key ^= Piece::compute_zobrist_at(ID_ROOK, my_player_num, rook_to);
                self.move_piece(rook_from, rook_to);
                new_props.castling_rights.set_player_castled(my_player_num);
            }
            _ => {}
        }

        let from = mv.get_from();
        let to = mv.get_to();
        let from_piece = self.piece_at(from).unwrap();
        let from_piece_type = from_piece.get_piece_id();
        let from_piece_new_pos_zobrist = from_piece.get_zobrist(to);
        new_props.zobrist_key ^= from_piece.get_zobrist(from);
        new_props.zobrist_key ^= from_piece_new_pos_zobrist;
        

        // Move piece to location
        self.move_piece(from, to);
        // Promotion
        match mv.get_move_type() {
            MoveType::PromotionCapture | MoveType::Promotion => {
                // Remove zobrist hash of the old piece
                new_props.zobrist_key ^= from_piece_new_pos_zobrist;
                new_props.promote_from = Some(from_piece_type);
                self.remove_piece(to);
                // Add new piece
                let promote_to_pt = mv.get_promotion_piece().unwrap();
                new_props.zobrist_key ^= Piece::compute_zobrist_at(promote_to_pt, my_player_num, to);
                self.add_piece(my_player_num, promote_to_pt, to);
            },
            _ => {}
        };

        // Pawn en-passant
        // Check for a pawn double push to set ep square
        let (x1, y1) = from_index(from);
        let (x2, y2) = from_index(to);

        if let Some(sq) = self.properties.ep_square {
            // If the last prop had some ep square then we want to clear zob by xoring again
            let (epx, _epy) = from_index(sq);
            new_props.zobrist_key ^= zobrist_table.get_ep_zobrist_file(epx);
        }

        if from_piece_type == ID_PAWN
            && (y2 as i8 - y1 as i8).abs() == 2
            && x1 == x2 {
            new_props.ep_square = Some(
                if y2 > y1 {
                    to_index(x1, y2 - 1)
                } else {
                    to_index(x1, y2 + 1)
                }
            );
            new_props.zobrist_key ^= zobrist_table.get_ep_zobrist_file(x1);
        } else {
            new_props.ep_square = None;
        }

        // Castling
        // Disable rights if applicable
        if new_props.castling_rights.can_player_castle(my_player_num) {
            if from_piece_type == ID_KING {
                new_props.zobrist_key ^= zobrist_table.get_castling_zobrist(my_player_num, true);
                new_props.zobrist_key ^= zobrist_table.get_castling_zobrist(my_player_num, false);
                new_props.castling_rights.disable_kingside_castle(my_player_num);
                new_props.castling_rights.disable_queenside_castle(my_player_num);
            } else if from_piece_type == ID_ROOK {
                // King side
                if x1 >= self.dimensions.width/2 {
                    new_props.castling_rights.disable_kingside_castle(my_player_num);
                    new_props.zobrist_key ^= zobrist_table.get_castling_zobrist(my_player_num, true);
                } else {
                    new_props.castling_rights.disable_queenside_castle(my_player_num);
                    new_props.zobrist_key ^= zobrist_table.get_castling_zobrist(my_player_num, false);
                }
            }
        }
        
        // Increment number of repetitions of new position
        self.update_repetitions(new_props.zobrist_key, 1);

        // Update props
        new_props.move_played = Some(mv);
        new_props.prev_properties = Some(Arc::clone(&self.properties));
        self.properties = Arc::new(new_props);
        
        // Update occupied bbs for future calculations
        self.update_occupied();
    }

    /// Undo the most recent move
    pub fn unmake_move(&mut self) {
        
        // Decrement the number of repetitions of old position
        self.update_repetitions(self.properties.zobrist_key, -1);

        if self.whos_turn == 0 {
            self.whos_turn = self.num_players - 1;
        } else {
            self.whos_turn -= 1;
        }

        let my_player_num = self.whos_turn;
        let mv = self.properties.move_played.unwrap();
        
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
        self.move_piece(to, from);
        //Undo Promotion
        match mv.get_move_type() {
            MoveType::PromotionCapture | MoveType::Promotion => {
                self.remove_piece(from);
                self.add_piece(my_player_num, self.properties.promote_from.as_ref().unwrap().to_owned(), from);
            },
            _ => {}
        };

        //Undo special moves
        //Special moves
        match mv.get_move_type() {
            MoveType::Capture | MoveType::PromotionCapture => {
                let capt = mv.get_target();
                let full_id: PieceIdWithPlayer = self.properties.captured_piece.unwrap();
                let owner = Piece::get_player_from_id(full_id);
                let pt = Piece::get_piecetype_from_id(full_id);
                self.add_piece(owner, pt, capt);
            },
            MoveType::KingsideCastle => {
                let rook_from = mv.get_target();
                let (x, y) = from_index(mv.get_to());
                let rook_to = to_index(x - 1, y);
                self.move_piece(rook_to,rook_from);
            },
            MoveType::QueensideCastle => {
                let rook_from = mv.get_target();
                let (x, y) = from_index(mv.get_to());
                let rook_to = to_index(x + 1, y);
                self.move_piece(rook_to,rook_from);
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
            for piece in ps.get_piece_refs() {
                let mut bb_copy = piece.bitboard.to_owned();
                while !bb_copy.is_zero() {
                    let indx = bb_copy.lowest_one().unwrap();
                    let (x, y) = from_index(indx as BIndex);
                    tuples.push((i as Player, x, y, piece.get_piece_id()));
                    bb_copy.clear_bit(indx);
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

    ///pieces(owner, index, PieceType)
    pub(crate) fn custom(dims: BDimensions,
                  piece_types: &Vec<PieceDefinition>,
                  pieces: Vec<(Player, BIndex, PieceId)>) -> Position
    {
        Position::assert_all_players_have_leader(piece_types, &pieces);
        
        let mut pos = parse_fen(String::from(fen::EMPTY));
        pos.dimensions = dims;
        for definition in piece_types {
            pos.register_piecetype(definition);
        }

        for (owner, index, piece_type) in pieces {
            pos.public_add_piece(owner, piece_type, index);
        }
        pos
    }
    fn assert_all_players_have_leader(piece_types: &Vec<PieceDefinition>, pieces: &Vec<(Player, BIndex, PieceId)>) {
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
            if !has_leader {
                panic!("Player {} does not have a leader piece", i);
            }
        }
    }

    pub fn get_zobrist(&self) -> u64 {
        self.properties.zobrist_key
    }
    
    pub fn num_repetitions(&self) -> u8 {
        let num_reps = self.position_repetitions.get(&self.properties.zobrist_key);
        *num_reps.unwrap()
    }

    /// Returns tuple (player_num, Piece)
    pub fn piece_at(&self, index: BIndex) -> Option<&Piece> {
        for ps in &self.pieces {
            if let Some(piece) = ps.piece_at(index) {
                return Some(piece);
            }
        }
        None
    }
    pub fn piece_at_mut(&mut self, index: BIndex) -> Option<(Player, &mut Piece)> {
        for (i, ps) in self.pieces.iter_mut().enumerate() {
            if let Some(c) = ps.piece_at_mut(index) {
                return Some((i as Player, c));
            }
        }
        None
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
        new_props.zobrist_key ^= Piece::compute_zobrist_at(piece_type, owner, index);
        self.add_piece(owner, piece_type, index);
        self.update_occupied();
        new_props.prev_properties = Some(Arc::clone(&self.properties));
        self.properties = Arc::new(new_props);
        
        // Add a repetition for the new position
        self.update_repetitions(self.properties.zobrist_key, 1);
    }

    pub fn public_remove_piece(&mut self, index: BIndex) {
        self.remove_piece(index);
        self.update_occupied();
    }
    
    
    
    /// Adds a piece to the position, assuming the piecetype already exists
    /// Does nothing if a custom piece isn't registered yet
    fn add_piece(&mut self, owner: Player, piece_id: PieceId, index: BIndex) {
        match piece_id {
            ID_KING => {self.pieces[owner as usize].king.bitboard.set_bit(index);},
            ID_QUEEN => {self.pieces[owner as usize].queen.bitboard.set_bit(index);},
            ID_ROOK => {self.pieces[owner as usize].rook.bitboard.set_bit(index);},
            ID_BISHOP => {self.pieces[owner as usize].bishop.bitboard.set_bit(index);},
            ID_KNIGHT => {self.pieces[owner as usize].knight.bitboard.set_bit(index);},
            ID_PAWN => {self.pieces[owner as usize].pawn.bitboard.set_bit(index);},
            _ => {
                // TODO: Change
                for c in self.pieces[owner as usize].custom.iter_mut() {
                    if piece_id == c.get_piece_id() {
                        c.add_piece(index);
                        break;
                    }
                }
            },
        }
    }
    
    /// Removes a piece from the position, assuming the piece is there
    fn remove_piece(&mut self, index: BIndex) {
        let piece = self.piece_at_mut(index).unwrap().1;
        piece.remove_piece(index);
    }
    
    fn move_piece(&mut self, from: BIndex, to: BIndex) {
        if let Some(piece) = self.piece_at_mut(from) {
            piece.1.move_piece(from, to);
        } else {
            println!("nothing to move??");
            println!("from {} {}", from_index(from).0, from_index(from).1);
            println!("to {} {}", from_index(to).0, from_index(to).1);
            println!("==");
        }
    }

    /// Updates the occupied bitboard
    /// Must be called after every position update/modification
    fn update_occupied(&mut self) {
        self.occupied = Bitboard::zero();
        for (_i, ps) in self.pieces.iter_mut().enumerate() {
            ps.update_occupied();
            self.occupied |= &ps.occupied;
        }
    }
    
    fn update_repetitions(&mut self, key: u64, delta: i16) {
        let num_reps = self.position_repetitions.get(&key);
        if num_reps.is_some() {
            let old_val = *num_reps.unwrap() as i16;
            self.position_repetitions.insert(key, (old_val + delta) as u8);
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
                } else {
                    write!(f, ". ")?;
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
