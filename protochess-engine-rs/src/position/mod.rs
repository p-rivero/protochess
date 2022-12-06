use arrayvec::ArrayVec;
use std::sync::Arc;
use std::collections::HashMap;

use crate::constants::piece_scores::{ID_ROOK, ID_PAWN, ID_KING, ID_QUEEN, ID_BISHOP, ID_KNIGHT, BASE_ID_CUSTOM};
use crate::types::*;
use crate::constants::fen;
use crate::position::piece_set::PieceSet;
use crate::utils::{from_index, to_index};

use movement_pattern::{MovementPattern, MovementPatternExternal, external_mp_to_internal, internal_mp_to_external};
use piece::Piece;
use zobrist_table::ZobristTable;
pub use parse_fen::parse_fen;
use position_properties::PositionProperties;

use self::piece::{PieceId, PieceIdWithPlayer};

mod position_properties;
pub mod castle_rights;
mod zobrist_table;
pub mod parse_fen;
pub mod piece;
pub mod piece_set;
pub mod movement_pattern;

//No reason to have more than one zobrist table
lazy_static! {
    static ref ZOBRIST_TABLE: ZobristTable = {
        let mut zob = ZobristTable::new();
        // Iterate for entire alphabet
        for c in ('a' as u8)..=('z' as u8) {
            zob.register_piecetype(Piece::blank_custom(0, c as char));
            zob.register_piecetype(Piece::blank_custom(1, c as char));
        }
        for i in 0..=1 {
            zob.register_piecetype(Piece::blank_king(i));
            zob.register_piecetype(Piece::blank_queen(i));
            zob.register_piecetype(Piece::blank_rook(i));
            zob.register_piecetype(Piece::blank_bishop(i));
            zob.register_piecetype(Piece::blank_knight(i));
            zob.register_piecetype(Piece::blank_pawn(i));
        }
        zob
    };
}


/// Represents a single position in chess
#[derive(Clone, Debug)]
pub struct Position {
    pub dimensions: BDimensions,
    pub bounds: Bitboard, //Bitboard representing the boundaries
    pub num_players: Player,
    pub whos_turn: Player,
    //Map of custom piece types to movement patterns
    pub movement_rules: HashMap<PieceId, MovementPattern>,
    pub pieces:ArrayVec<[PieceSet;4]>, //pieces[0] = white's pieces, pieces[1] black etc
    pub occupied: Bitboard,
    //Properties relating only to the current position
    // Typically hard-to-recover properties, like castling
    //Similar to state in stockfish
    pub properties: Arc<PositionProperties>,
}

impl Position {
    pub fn default() -> Position {
        // Ensure that the zobrist table is initialized
        let _ = &*ZOBRIST_TABLE;
        
        parse_fen(String::from(fen::STARTING_POS))
    }

    /// Registers a new piece type for this position
    pub fn register_piecetype(&mut self, char_rep: char, mpe: MovementPatternExternal) {
        let mp = external_mp_to_internal(mpe);
        //Store the movement rule
        let id = BASE_ID_CUSTOM + char_rep as PieceId;
        self.movement_rules.insert(id, mp);
        //Insert blank for all players
        for (i, p) in self.pieces.iter_mut().enumerate() {
                //ZOBRIST_TABLE.register_piecetype(0, &PieceType::Custom(char_rep));
            p.custom.push(Piece::blank_custom(i as Player, char_rep));
        }
    }

    // TODO: Remove
    pub fn get_char_movementpattern_map(&self) -> HashMap<char, MovementPatternExternal> {
        let mut return_map = HashMap::new();
        for (piece_type, movement_pattern) in self.movement_rules.iter() {
            if *piece_type >= BASE_ID_CUSTOM {
                let c = (*piece_type - BASE_ID_CUSTOM) as u8 as char;
                return_map.insert(c, internal_mp_to_external(movement_pattern.to_owned()));
            }
        }
        return_map
    }

    pub(crate) fn get_movement_pattern(&self, piece_type: PieceId) -> Option<&MovementPattern> {
       self.movement_rules.get(&piece_type)
    }

    pub(crate) fn set_bounds(&mut self, dims: BDimensions, bounds: Bitboard) {
        self.dimensions = dims;
        self.bounds = bounds;
    }


    /// Modifies the position to make the move
    pub fn make_move(&mut self, mv: Move) {
        let zobrist_table = &ZOBRIST_TABLE;
        let my_player_num = self.whos_turn;
        let mut new_props:PositionProperties = (*self.properties).clone();
        
        // Remove zobrist hash of the old player
        new_props.zobrist_key ^= zobrist_table.get_player_zobrist(self.whos_turn);
        // Update the player
        self.whos_turn = self.whos_turn + 1;
        if self.whos_turn == self.num_players {
            self.whos_turn = 0;
        }
        // Add zobrist hash of the new player
        new_props.zobrist_key ^= zobrist_table.get_player_zobrist(self.whos_turn);
        
        //In the special case of the null move, don't do anything except update whos_turn
        //And update props
        if mv.get_move_type() == MoveType::Null {
            //Update props
            //Since we're passing, there cannot be an ep square
            new_props.ep_square = None;
            new_props.move_played = Some(mv);
            new_props.prev_properties = Some(Arc::clone(&self.properties));
            self.properties = Arc::new(new_props);
            return;
        }

        //Special moves
        match mv.get_move_type() {
            MoveType::Capture | MoveType::PromotionCapture => {
                let capt_index = mv.get_target();
                let (_owner, captured_piece) = self.piece_at(capt_index).unwrap();
                new_props.zobrist_key ^= zobrist_table.get_zobrist_sq(captured_piece, capt_index);
                new_props.captured_piece = Some(captured_piece.get_full_id());
                self.remove_piece(capt_index);
            },
            MoveType::KingsideCastle => {
                let rook_from = mv.get_target();
                let (x, y) = from_index(mv.get_to());
                let rook_to = to_index(x - 1, y);
                new_props.zobrist_key ^= zobrist_table.get_zobrist_sq_from(ID_ROOK, my_player_num, rook_from);
                new_props.zobrist_key ^= zobrist_table.get_zobrist_sq_from(ID_ROOK, my_player_num, rook_to);
                self.move_piece(rook_from, rook_to);
                new_props.castling_rights.set_player_castled(my_player_num);
            },
            MoveType::QueensideCastle => {
                let rook_from = mv.get_target();
                let (x, y) = from_index(mv.get_to());
                let rook_to = to_index(x + 1, y);
                new_props.zobrist_key ^= zobrist_table.get_zobrist_sq_from(ID_ROOK, my_player_num, rook_from);
                new_props.zobrist_key ^= zobrist_table.get_zobrist_sq_from(ID_ROOK, my_player_num, rook_to);
                self.move_piece(rook_from, rook_to);
                new_props.castling_rights.set_player_castled(my_player_num);
            }
            _ => {}
        }

        let from = mv.get_from();
        let to = mv.get_to();
        let from_piece = self.piece_at(from).unwrap().1;
        let from_piece_type = from_piece.get_piece_id();
        new_props.zobrist_key ^= zobrist_table.get_zobrist_sq(&from_piece, from);
        new_props.zobrist_key ^= zobrist_table.get_zobrist_sq(&from_piece, to);
        

        //Move piece to location
        self.move_piece(from, to);
        //Promotion
        match mv.get_move_type() {
            MoveType::PromotionCapture | MoveType::Promotion => {
                // Remove zobrist hash of the old piece
                new_props.zobrist_key ^= zobrist_table.get_zobrist_sq_from(from_piece_type, my_player_num, to);
                new_props.promote_from = Some(from_piece_type);
                self.remove_piece(to);
                // Add new piece
                let promote_to_pt = char_to_pieceid(mv.get_promotion_char().unwrap());
                new_props.zobrist_key ^= zobrist_table.get_zobrist_sq_from(promote_to_pt, my_player_num, to);
                self.add_piece(my_player_num, promote_to_pt, to);
            },
            _ => {}
        };

        // Pawn en-passant
        // Check for a pawn double push to set ep square
        let (x1, y1) = from_index(from);
        let (x2, y2) = from_index(to);

        if let Some(sq) = self.properties.ep_square {
            //If the last prop had some ep square then we want to clear zob by xoring again
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

        //Castling
        //Disable rights if applicable
        if new_props.castling_rights.can_player_castle(my_player_num) {
            if from_piece_type == ID_KING {
                new_props.zobrist_key ^= zobrist_table.get_castling_zobrist(my_player_num, true);
                new_props.zobrist_key ^= zobrist_table.get_castling_zobrist(my_player_num, false);
                new_props.castling_rights.disable_kingside_castle(my_player_num);
                new_props.castling_rights.disable_queenside_castle(my_player_num);
            } else if from_piece_type == ID_ROOK {
                //King side
                if x1 >= self.dimensions.width/2 {
                    new_props.castling_rights.disable_kingside_castle(my_player_num);
                    new_props.zobrist_key ^= zobrist_table.get_castling_zobrist(my_player_num, true);
                } else {
                    new_props.castling_rights.disable_queenside_castle(my_player_num);
                    new_props.zobrist_key ^= zobrist_table.get_castling_zobrist(my_player_num, false);
                }
            }
        }

        //Update props
        new_props.move_played = Some(mv);
        new_props.prev_properties = Some(Arc::clone(&self.properties));
        self.properties = Arc::new(new_props);
        //Update occupied bbs for future calculations
        self.update_occupied();
    }

    /// Undo the most recent move
    pub fn unmake_move(&mut self) {

        if self.whos_turn == 0 {
            self.whos_turn = self.num_players - 1;
        } else {
            self.whos_turn -= 1;
        }

        let my_player_num = self.whos_turn;
        let mv = self.properties.move_played.unwrap();
        //Undo null moves
        if mv.get_move_type() == MoveType::Null {
            //Update props
            //Consume prev props; never to return again
            self.properties = self.properties.get_prev().unwrap();
            return;
        }
        let from = mv.get_from();
        let to= mv.get_to();

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

        //Update props
        //Consume prev props; never to return again
        self.properties = self.properties.get_prev().unwrap();

        //Update occupied bbs for future calculations
        self.update_occupied();
    }

    pub fn to_string(&mut self) -> String {
        let mut return_str= String::new();
        for y in (0..self.dimensions.height).rev() {
            return_str = format!("{} {} ", return_str, y);
            for x in 0..self.dimensions.width {
                if let Some((player_num, piece)) = self.piece_at(to_index(x,y)) {
                    // TODO: no need to check player_num
                    if player_num == 0 {
                        return_str.push(piece.char_rep().to_ascii_uppercase());
                    } else {
                        return_str.push(piece.char_rep().to_ascii_lowercase());
                    }
                } else {
                    return_str.push('.');
                }
                return_str.push(' ');
            }
            return_str.push('\n');
        }
        return_str = format!("{}  ", return_str);
        for x in 0..self.dimensions.width {
            return_str = format!("{} {}", return_str, x);
        }

        format!("{} \nZobrist Key: {}", return_str, self.properties.zobrist_key)
    }

    /// Return piece for (owner, x, y, char)
    pub fn pieces_as_tuples(&self) -> Vec<(Player, BCoord, BCoord, char)>{
        let mut tuples = Vec::new();
        for (i, ps) in self.pieces.iter().enumerate() {
            for piece in ps.get_piece_refs() {
                let mut bb_copy = (&piece.bitboard).to_owned();
                while !bb_copy.is_zero() {
                    let indx = bb_copy.lowest_one().unwrap();
                    let (x, y) = from_index(indx as BIndex);
                    tuples.push((i as Player, x, y, piece.char_rep()));
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
                if self.xy_in_bounds(x, y) {
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
    pub(crate) fn custom(dims: BDimensions, bounds: Bitboard,
                  movement_patterns: HashMap<char, MovementPatternExternal>,
                  pieces: Vec<(Player, BIndex, PieceId)>) -> Position
    {
        let mut pos = parse_fen(String::from(fen::EMPTY));
        pos.dimensions = dims;
        pos.bounds = bounds;
        for (chr, mpe) in movement_patterns {
            pos.register_piecetype(chr, mpe);
        }

        for (owner, index, piece_type) in pieces {
            pos.public_add_piece(owner, piece_type, index);
        }
        pos
    }

    pub fn get_zobrist(&self) -> u64 {
        self.properties.zobrist_key
    }

    /// Returns tuple (player_num, Piece)
    pub fn piece_at(&self, index: BIndex) -> Option<(Player, &Piece)> {
        for (i, ps) in self.pieces.iter().enumerate() {
            if let Some(c) = ps.piece_at(index) {
                return Some((i as Player, c));
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

    /// Returns bitoard of piece at index
    pub fn piece_bb_at(&self, index: BIndex) -> Option<&Bitboard> {
        if let Some((_num, piece)) = self.piece_at(index) {
            return Some(&piece.bitboard)
        }
        None
    }
    pub fn piece_bb_at_mut(&mut self, index: BIndex) -> Option<&mut Bitboard> {
        if let Some((_num, piece)) = self.piece_at_mut(index) {
            return Some(&mut piece.bitboard)
        }
        None
    }

    /// Returns if the point is in bounds
    pub fn xy_in_bounds(&self, x: BCoord, y: BCoord) -> bool {
        if x < self.dimensions.width && y < self.dimensions.height {
            return self.bounds.get_bit_at(x, y)
        }
        false
    }

    
    /// Public interface for modifying the position
    pub fn public_add_piece(&mut self, owner: Player, piece_type: PieceId, index: BIndex) {
        let mut new_props = (*self.properties).clone();
        let zobrist_table = &ZOBRIST_TABLE;
        new_props.zobrist_key ^= zobrist_table.get_zobrist_sq_from(piece_type, owner, index);
        self.add_piece(owner, piece_type, index);
        self.update_occupied();
        new_props.prev_properties = Some(Arc::clone(&self.properties));
        self.properties = Arc::new(new_props);
    }

    pub fn public_remove_piece(&mut self, index: BIndex) {
        self.remove_piece(index);
        self.update_occupied();
    }
    
    
    
    /// Adds a piece to the position, assuming the piecetype already exists
    /// Does nothing if a custom piece isn't registered yet
    fn add_piece(&mut self, owner: Player, pt: PieceId, index: BIndex) {
        match pt {
            ID_KING => {self.pieces[owner as usize].king.bitboard.set_bit(index);},
            ID_QUEEN => {self.pieces[owner as usize].queen.bitboard.set_bit(index);},
            ID_ROOK => {self.pieces[owner as usize].rook.bitboard.set_bit(index);},
            ID_BISHOP => {self.pieces[owner as usize].bishop.bitboard.set_bit(index);},
            ID_KNIGHT => {self.pieces[owner as usize].knight.bitboard.set_bit(index);},
            ID_PAWN => {self.pieces[owner as usize].pawn.bitboard.set_bit(index);},
            _ => {
                // TODO: Change
                for c in self.pieces[owner as usize].custom.iter_mut() {
                    if pt == c.get_piece_id() {
                        c.bitboard.set_bit(index);
                        break;
                    }
                }
            },
        }
    }
    
    /// Removes a piece from the position, assuming the piece is there
    fn remove_piece(&mut self, index: BIndex) {
        let capd_bb = self.piece_bb_at_mut(index).unwrap();
        capd_bb.clear_bit(index);
    }
    
    fn move_piece(&mut self, from: BIndex, to: BIndex) {
        if let Some(source_bb) = self.piece_bb_at_mut(from) {
            source_bb.clear_bit(from);
            source_bb.set_bit(to);
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
}
