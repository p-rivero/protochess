use arrayvec::ArrayVec;
use crate::searcher::types::Player;
use crate::types::*;
use crate::constants::{fen, DEFAULT_WIDTH, DEFAULT_HEIGHT};
use crate::position::piece_set::PieceSet;
use crate::types::bitboard::{Bitboard, to_index, from_index, BIndex, BCoord, BDimensions};
use std::sync::Arc;

use position_properties::PositionProperties;
use crate::types::chess_move::{Move, MoveType};
use crate::position::movement_pattern::{MovementPattern, MovementPatternExternal, external_mp_to_internal, internal_mp_to_external};
use crate::position::piece::Piece;
use std::collections::HashMap;
use crate::position::zobrist_table::ZobristTable;
use crate::constants::fen::EMPTY;

mod position_properties;
mod castle_rights;
mod zobrist_table;
pub mod piece;
pub mod piece_set;
pub mod movement_pattern;

//No reason to have more than one zobrist table
lazy_static! {
    static ref ZOBRIST_TABLE: ZobristTable = {
        let mut zob = ZobristTable::new();
        for c in "acdefghijlmostuvwxyz".chars() {
            zob.register_piecetype(0, &PieceType::Custom(c));
            zob.register_piecetype(1, &PieceType::Custom(c));
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
    pub movement_rules: HashMap<PieceType, MovementPattern>,
    pub pieces:ArrayVec<[PieceSet;4]>, //pieces[0] = white's pieces, pieces[1] black etc
    pub occupied: Bitboard,
    //Properties relating only to the current position
    // Typically hard-to-recover properties, like castling
    //Similar to state in stockfish
    pub properties: Arc<PositionProperties>,
}

impl Position {
    pub fn default() -> Position{
        Position::from_fen(String::from(fen::STARTING_POS))
    }

    /// Registers a new piece type for this position
    pub fn register_piecetype(&mut self, char_rep: char, mpe: MovementPatternExternal) {
        let mp = external_mp_to_internal(mpe);
        //Store the movement rule
        self.movement_rules.insert(PieceType::Custom(char_rep), mp);
        //Insert blank for all players
        for (i, p) in self.pieces.iter_mut().enumerate() {
                //ZOBRIST_TABLE.register_piecetype(0, &PieceType::Custom(char_rep));
            p.custom.push(Piece::blank_custom(i as Player, char_rep));
        }
    }

    pub fn get_char_movementpattern_map(&self) -> HashMap<char, MovementPatternExternal> {
        let mut return_map = HashMap::new();
        for (piece_type, movement_pattern) in self.movement_rules.iter() {
            match piece_type {
                PieceType::Custom(c) => {
                    return_map.insert(*c, internal_mp_to_external(movement_pattern.to_owned()));
                }
                _ => {}
            }
        }
        return_map
    }

    pub(crate) fn get_movement_pattern(&self, piece_type: &PieceType) -> Option<&MovementPattern> {
       self.movement_rules.get(piece_type)
    }

    pub(crate) fn set_bounds(&mut self, dims: BDimensions, bounds: Bitboard) {
        self.dimensions = dims;
        self.bounds = bounds;
    }


    /// Modifies the position to make the move
    pub fn make_move(&mut self, mv: Move) {
        let zobrist_table = &ZOBRIST_TABLE;
        let my_player_num = self.whos_turn;
        self.whos_turn = (self.whos_turn + 1) % self.num_players;

        let mut new_props:PositionProperties = (*self.properties).clone();
        new_props.zobrist_key ^= zobrist_table.get_to_move_zobrist(self.whos_turn);
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
                let (owner, captd) = self.piece_at(capt_index).unwrap();
                let captd_piece_type = (&captd.piece_type).to_owned();
                let captd_owner = captd.player_num;
                new_props.zobrist_key ^= zobrist_table.get_zobrist_sq_from_pt(&captd_piece_type,captd_owner , capt_index);
                new_props.captured_piece = Some((owner, captd_piece_type));
                self._remove_piece(capt_index);
            },
            MoveType::KingsideCastle => {
                let rook_from = mv.get_target();
                let (x, y) = from_index(mv.get_to());
                let rook_to = to_index(x - 1, y);
                new_props.zobrist_key ^= zobrist_table.get_zobrist_sq_from_pt(&PieceType::Rook, my_player_num, rook_from);
                new_props.zobrist_key ^= zobrist_table.get_zobrist_sq_from_pt(&PieceType::Rook, my_player_num, rook_to);
                self.move_piece(rook_from, rook_to);
                new_props.castling_rights.set_player_castled(my_player_num);
            },
            MoveType::QueensideCastle => {
                let rook_from = mv.get_target();
                let (x, y) = from_index(mv.get_to());
                let rook_to = to_index(x + 1, y);
                new_props.zobrist_key ^= zobrist_table.get_zobrist_sq_from_pt(&PieceType::Rook, my_player_num, rook_from);
                new_props.zobrist_key ^= zobrist_table.get_zobrist_sq_from_pt(&PieceType::Rook, my_player_num, rook_to);
                self.move_piece(rook_from, rook_to);
                new_props.castling_rights.set_player_castled(my_player_num);
            }
            _ => {}
        }

        let from= mv.get_from();
        let to = mv.get_to();
        let from_piece = self.piece_at(from).unwrap().1;
        let from_piece_type = from_piece.piece_type.to_owned();
        new_props.zobrist_key ^= zobrist_table.get_zobrist_sq_from_pt(&from_piece_type, my_player_num, from);
        new_props.zobrist_key ^= zobrist_table.get_zobrist_sq_from_pt(&from_piece_type, my_player_num, to);

        //Move piece to location
        self.move_piece(from, to);
        //Promotion
        match mv.get_move_type() {
            MoveType::PromotionCapture | MoveType::Promotion => {
                new_props.promote_from = Some(from_piece_type.to_owned());
                new_props.zobrist_key ^= zobrist_table.get_zobrist_sq_from_pt(&from_piece_type, my_player_num, to);
                self._remove_piece(to);
                let promote_to_pt = PieceType::from_char(mv.get_promotion_char().unwrap());
                new_props.zobrist_key ^= zobrist_table.get_zobrist_sq_from_pt(&promote_to_pt, my_player_num, to);
                self.add_piece_impl(my_player_num, promote_to_pt, to);
            },
            _ => {}
        };

        //Pawn en-passant
        //Check for a pawn double push to set ep square
        let (x1, y1) = from_index(from);
        let (x2, y2) = from_index(to);

        if let Some(sq) = self.properties.ep_square {
            //If the last prop had some ep square then we want to clear zob by xoring again
            let (epx, _epy) = from_index(sq);
            new_props.zobrist_key ^= zobrist_table.get_ep_zobrist_file(epx);
        }

        if from_piece_type == PieceType::Pawn
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
            if from_piece_type == PieceType::King {
                new_props.zobrist_key ^= zobrist_table.get_castling_zobrist(my_player_num, true);
                new_props.zobrist_key ^= zobrist_table.get_castling_zobrist(my_player_num, false);
                new_props.castling_rights.disable_kingside_castle(my_player_num);
                new_props.castling_rights.disable_queenside_castle(my_player_num);
            } else if from_piece_type == PieceType::Rook {
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
            self.whos_turn = self.num_players -1;
        } else {
            self.whos_turn = (self.whos_turn - 1) % self.num_players;
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
                self._remove_piece(from);
                self.add_piece_impl(my_player_num, self.properties.promote_from.as_ref().unwrap().to_owned(), from);
            },
            _ => {}
        };

        //Undo special moves
        //Special moves
        match mv.get_move_type() {
            MoveType::Capture | MoveType::PromotionCapture => {
                let capt = mv.get_target();
                let (owner, pt) = self.properties.captured_piece.as_ref().unwrap();
                self.add_piece_impl(*owner, pt.to_owned(), capt);
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
                if let Some((player_num, piece)) = self.piece_at(bitboard::to_index(x,y)) {
                    if player_num == 0 {
                        return_str.push(piece.char_rep.to_ascii_uppercase());
                    } else {
                        return_str.push(piece.char_rep.to_ascii_lowercase());
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
                    tuples.push((i as Player, x, y, piece.char_rep));
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
                  pieces: Vec<(Player, BIndex, PieceType)>) -> Position
    {
        let mut pos = Position::from_fen(String::from(EMPTY));
        pos.dimensions = dims;
        pos.bounds = bounds;
        for (chr, mpe) in movement_patterns {
            pos.register_piecetype(chr, mpe);
        }

        for (owner, index, piece_type) in pieces {
            pos.add_piece(owner, piece_type, index);
        }
        pos
    }

    pub fn from_fen(fen: String) -> Position {
        let dims = BDimensions{width:DEFAULT_WIDTH, height:DEFAULT_HEIGHT};

        let mut wb_pieces = ArrayVec::<[_;4]>::new();
        let mut w_pieces = PieceSet::new(0);
        let mut b_pieces = PieceSet::new(1);

        let mut x: BCoord = 0;
        let mut y: BCoord = 7;
        let mut field = 0;

        let mut whos_turn = 0;
        let mut can_w_castle_k = false;
        let mut can_b_castle_k = false;
        let mut can_w_castle_q = false;
        let mut can_b_castle_q = false;
        
        let mut ep_x: i8 = -1;
        let mut ep_y: i8 = -1;
        
        for c in fen.chars() {
            if c == ' ' {
                field += 1;
                continue;
            }
            match field{
                //position
                0 => {
                    if c == '/' {
                        x = 0;
                        y -= 1;
                        continue;
                    } else if c.is_numeric() {
                        x += c.to_digit(10).expect("Not a digit!") as BCoord;
                        continue;
                    }

                    let index = bitboard::to_index(x, y);
                    let pieces = if c.is_ascii_uppercase() {
                        &mut w_pieces
                    } else {
                        &mut b_pieces
                    };
                    let bitboard: &mut Bitboard = match c.to_ascii_lowercase() {
                        'k' => { &mut pieces.king.bitboard },
                        'q' => { &mut pieces.queen.bitboard },
                        'r' => { &mut pieces.rook.bitboard },
                        'b' => { &mut pieces.bishop.bitboard },
                        'n' => { &mut pieces.knight.bitboard },
                        'p' => { &mut pieces.pawn.bitboard },
                        _ => continue,
                    };

                    bitboard.set_bit(index);
                    if c.is_uppercase() {
                        w_pieces.occupied.set_bit(index)
                    } else {
                        b_pieces.occupied.set_bit(index)
                    };
                    x += 1;
                }
                //next to move
                1 => {
                    if c == 'w' {
                        whos_turn = 0;
                    } else {
                        whos_turn = 1;
                    }
                }
                //Castling rights
                2 => {
                    match c {
                        'K' => {can_w_castle_k = true;}
                        'Q' => {can_w_castle_q = true;}
                        'k' => {can_b_castle_k = true;}
                        'q' => {can_b_castle_q = true;}
                        _ => {}
                    }
                }
                //En Passant square
                3 => {
                    // This field can be either '-' or a square in the form of a letter followed by a number
                    if c == '-' {
                        continue;
                    } else if c.is_numeric() {
                        ep_y = c.to_digit(10).expect("Not a digit!") as i8 - 1;
                    } else {
                        ep_x = c as i8 - 'a' as i8;
                    }
                }
                _ => continue,
            }
        }

        let mut occupied = Bitboard::zero();
        occupied |= &w_pieces.occupied;
        occupied |= &b_pieces.occupied;
        let zobrist_table = ZobristTable::new();
        let mut zobrist_key = 0;

        let mut properties = PositionProperties::default();
        zobrist_key ^= zobrist_table.get_castling_zobrist(0, true);
        zobrist_key ^= zobrist_table.get_castling_zobrist(0, false);
        zobrist_key ^= zobrist_table.get_castling_zobrist(1, true);
        zobrist_key ^= zobrist_table.get_castling_zobrist(1, false);
        if !can_w_castle_k {
            properties.castling_rights.disable_kingside_castle(0);
            zobrist_key ^= zobrist_table.get_castling_zobrist(0, true);
        }

        if !can_b_castle_k {
            properties.castling_rights.disable_kingside_castle(1);
            zobrist_key ^= zobrist_table.get_castling_zobrist(1, true);
        }

        if !can_w_castle_q {
            properties.castling_rights.disable_queenside_castle(0);
            zobrist_key ^= zobrist_table.get_castling_zobrist(0, false);
        }

        if !can_b_castle_q {
            properties.castling_rights.disable_queenside_castle(1);
            zobrist_key ^= zobrist_table.get_castling_zobrist(1, false);
        }
        
        if ep_x != -1 {
            if ep_y == -1 || (ep_y != 2 && ep_y != 5) {
                panic!("Invalid en passant square: {}", fen);
            }
            properties.ep_square = Some(to_index(ep_x as BCoord, ep_y as BCoord));
            zobrist_key ^= zobrist_table.get_ep_zobrist_file(ep_x as BCoord);
        }


        for piece in w_pieces.get_piece_refs().into_iter().chain(b_pieces.get_piece_refs().into_iter()) {
            let mut bb_copy = (&piece.bitboard).to_owned();
            while !bb_copy.is_zero() {
                let indx = bb_copy.lowest_one().unwrap();
                zobrist_key ^= zobrist_table.get_zobrist_sq(piece, indx);
                bb_copy.clear_bit(indx);
            }
        }

        properties.zobrist_key = zobrist_key;

        wb_pieces.push(w_pieces);
        wb_pieces.push(b_pieces);

        let mut bounds = Bitboard::zero();
        for x in 0..8 {
            for y in 0..8 {
                bounds.set_bit_at(x,y);
            }
        }

        let pos = Position{
            whos_turn,
            num_players: 2,
            dimensions: dims,
            pieces: wb_pieces,
            occupied,
            bounds,
            properties: Arc::new(properties),
            movement_rules: Default::default()
        };

        pos
    }

    pub fn get_zobrist(&self) -> u64 {
        self.properties.zobrist_key
    }

    /// Returns tuple (player_num, Piece)
    pub fn piece_at(&mut self, index: BIndex) -> Option<(Player, &mut Piece)> {
        for (i, ps) in self.pieces.iter_mut().enumerate() {
            if let Some(c) = ps.piece_at(index) {
                return Some((i as Player, c));
            }
        }
        None
    }

    /// Returns bitoard of piece at index
    pub fn piece_bb_at(&mut self,index: BIndex) -> Option<&mut Bitboard> {
        if let Some((_num, piece)) = self.piece_at(index) {
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

    pub fn move_piece(&mut self, from: BIndex, to: BIndex) {
        if let Some(source_bb) = self.piece_bb_at(from) {
            source_bb.clear_bit(from);
            source_bb.set_bit(to);
        } else {
            println!("nothing to move??");
            println!("from {} {}", from_index(from).0, from_index(from).1);
            println!("to {} {}", from_index(to).0, from_index(to).1);
            println!("==");
        }
    }

    /// Removes a piece from the position, assuming the piece is there
    fn _remove_piece(&mut self, index: BIndex) {
        let capd_bb:&mut Bitboard = self.piece_bb_at(index).unwrap();
        capd_bb.clear_bit(index);
    }

    /// Adds a piece to the position, assuming the piecetype already exists
    /// Does nothing if a custom piece isn't registered yet
    fn add_piece_impl(&mut self, owner: Player, pt: PieceType, index: BIndex) {
        match pt {
            PieceType::King => {self.pieces[owner as usize].king.bitboard.set_bit(index);},
            PieceType::Queen => {self.pieces[owner as usize].queen.bitboard.set_bit(index);},
            PieceType::Rook => {self.pieces[owner as usize].rook.bitboard.set_bit(index);},
            PieceType::Bishop => {self.pieces[owner as usize].bishop.bitboard.set_bit(index);},
            PieceType::Knight => {self.pieces[owner as usize].knight.bitboard.set_bit(index);},
            PieceType::Pawn => {self.pieces[owner as usize].pawn.bitboard.set_bit(index);},
            PieceType::Custom(ptc) => {
                for c in self.pieces[owner as usize].custom.iter_mut() {
                    if ptc == c.char_rep {
                        c.bitboard.set_bit(index);
                        break;
                    }
                }
            },
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

    /// Public interface for modifying the position
    pub fn add_piece(&mut self, owner: Player, pt: PieceType, index: BIndex) {
        let mut new_props:PositionProperties = (*self.properties).clone();
        let zobrist_table = &ZOBRIST_TABLE;
        new_props.zobrist_key ^= zobrist_table.get_zobrist_sq_from_pt(&pt, owner, index);
        self.add_piece_impl(owner, pt, index);
        self.update_occupied();
        new_props.prev_properties = Some(Arc::clone(&self.properties));
        self.properties = Arc::new(new_props);
    }

    pub fn remove_piece(&mut self, index: BIndex) {
        self._remove_piece(index);
        self.update_occupied();
    }
}

#[cfg(test)]
mod pos_test {
    use crate::position::Position;
    use crate::move_generator::MoveGenerator;
    use crate::types::chess_move::Move;

    #[test]
    fn print_pieces() {
        let pos = Position::default();
        for pce in pos.pieces_as_tuples() {
            println!("{:?}", pce);
        }

        for pce in pos.tiles_as_tuples() {
            println!("{:?}", pce);
        }

    }


    #[test]
    fn null_move_eq() {
        let mut pos = Position::default();
        //let movegen = MoveGenerator::new();
        let zob_0 = pos.get_zobrist();
        pos.make_move(Move::null());
        pos.make_move(Move::null());
        pos.make_move(Move::null());
        pos.make_move(Move::null());
        pos.unmake_move();
        pos.unmake_move();
        pos.unmake_move();
        pos.unmake_move();
        assert_eq!(zob_0, pos.get_zobrist());
    }
    #[test]
    fn zobrist_equality() {
        let mut pos = Position::default();
        let movegen = MoveGenerator::new();
        let zob_0 = pos.get_zobrist();
        for mv in movegen.get_pseudo_moves(&mut pos) {
            pos.make_move(mv);
            for mv in movegen.get_pseudo_moves(&mut pos) {
                pos.make_move(mv);
                for mv in movegen.get_pseudo_moves(&mut pos) {
                    pos.make_move(mv);
                    pos.unmake_move();
                }
                pos.unmake_move();
            }
            pos.unmake_move();
        };
        assert_eq!(zob_0, pos.get_zobrist())
    }
}

