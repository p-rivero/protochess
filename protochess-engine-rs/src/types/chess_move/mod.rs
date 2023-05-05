use std::fmt;

use crate::piece::PieceId;
use crate::Position;
use crate::utils::{to_rank_file, from_index};

use super::{BIndex, BCoord};

mod move_list;
mod move_info;
mod make_move_result;

pub use move_list::*;
pub use move_info::*;
pub use make_move_result::*;

#[derive(Eq, PartialEq, Debug, Copy, Clone)]
#[must_use]
pub enum MoveType {
    // The least significant bit is used to indicate capture
    Quiet = 0b0000,
    Capture = 0b0001,
    KingsideCastle = 0b0010,
    // Skip 0b0011 because there is no KingsideCastleCapture
    QueensideCastle = 0b0100,
    // Skip 0b0101 because there is no QueensideCastleCapture
    Promotion = 0b0110,
    PromotionCapture = 0b0111,
    DoubleJump = 0b1000,
    // Skip 0b1001 because there is no DoubleJumpCapture
    Null = 0b1010,
}

#[derive(Eq, PartialEq, Copy, Clone)]
#[must_use]
pub struct Move {
    /// Stores a move in a u32
    ///0-7:   from index:u8
    ///8-15:  to index:u8
    ///16-23: target index:u8
    ///24-27 : movetype (see MoveType above)
    /// In captures, target is the index of the captured piece (usually the same as to, except for en passant)
    /// In DoubleJump, target is the index of the generated En Passant square
    move_fields: u32, 
    // Promotion piece
    promotion: PieceId
}

impl Move {
    #[inline]
    pub fn new(from: BIndex, to: BIndex, target: BIndex, move_type: MoveType, promotion: Option<PieceId>) -> Move {
        Move {
            move_fields: (from as u32) | (to as u32) << 8 | (target as u32) << 16 | (move_type as u32) << 24,
            promotion: promotion.unwrap_or('_')
        }
    }

    #[inline]
    pub fn null() -> Move {
        Move::new(0, 0, 0, MoveType::Null, None)
    }
    
    #[inline]
    pub fn is_null(&self) -> bool {
        self.get_move_type() == MoveType::Null
    }
    
    pub fn is_quiet(&self) -> bool {
        self.get_move_type() == MoveType::Quiet || self.get_move_type() == MoveType::DoubleJump
    }

    pub fn get_from(&self) -> BIndex{
        (self.move_fields & 0xFF) as BIndex
    }

    pub fn get_to(&self) -> BIndex{
        ((self.move_fields >> 8) & 0xFF) as BIndex
    }
    
    // Get the index of the victim piece, if any. Usually the same as get_to(), except for en passant
    // In double jump, this is the index of the generated en passant square
    pub fn get_target(&self) -> BIndex {
        ((self.move_fields >> 16) & 0xFF) as BIndex
    }

    pub fn is_capture(&self) -> bool {
        // The least significant bit of the move type is used to indicate capture
        ((self.move_fields >> 24) & 1) != 0
    }
    
    pub fn is_promotion(&self) -> bool {
        let move_type = self.get_move_type();
        move_type == MoveType::Promotion || move_type == MoveType::PromotionCapture
    }
    
    pub fn is_castling(&self) -> bool {
        let move_type = self.get_move_type();
        move_type == MoveType::KingsideCastle || move_type == MoveType::QueensideCastle
    }

    #[inline]
    pub fn get_move_type(&self) -> MoveType {
        // Output a match statement that maps from "x if x == MoveType::XX as u32" to "MoveType::XX"
        macro_rules! match_move_type {
            ($($x:ident),*) => {
                match self.move_fields >> 24 {
                    // For each argument x, generate a line of the match
                    $( x if x == MoveType::$x as u32 => { MoveType::$x } )*
                    _ => { panic!("Invalid move type") }
                }
            }
        }
        match_move_type!(Quiet, Capture, KingsideCastle, QueensideCastle, Promotion, PromotionCapture, DoubleJump, Null)
    }

    pub fn get_promotion_piece(&self) -> Option<PieceId> {
        if self.is_promotion() {
            Some(self.promotion)
        } else {
            None
        }
    }

    /// Returns the squares that would explode if this move was made.
    /// **Important**: Call this function **before** making the move
    pub fn get_potential_explosion(&self, pos: &Position) -> Vec<(BCoord, BCoord)> {
        let piece = pos.piece_at(self.get_from()).unwrap();
        if !piece.explodes_on_capture() || !self.is_capture() {
            return Vec::new();
        }
        let to = self.get_to();
        let mut explosion = piece.get_explosion(to).clone();
        let mut explosion_squares = Vec::new();
        while let Some(index) = explosion.lowest_one() {
            explosion_squares.push(from_index(index));
            explosion.clear_bit(index);
        }
        // Ensure that to square is included in the explosion
        if !explosion_squares.contains(&from_index(to)) {
            explosion_squares.push(from_index(to));
        }
        explosion_squares
    }
}

impl fmt::Display for Move {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        if self.is_null() {
            return write!(f, "[NULL]");
        }
        let (x1, y1) = from_index(self.get_from());
        let (x2, y2) = {
            // Print castling moves as if the king moves to the rook square
            if self.is_castling() { from_index(self.get_target()) }
            else { from_index(self.get_to()) }
        };
        let suffix = {
            if self.is_promotion() { format!("={}", self.promotion) }
            else { "".to_string() }
        };
        write!(f, "{}{}{}", to_rank_file(x1, y1), to_rank_file(x2, y2), suffix)
    }
}
impl fmt::Debug for Move {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        if self.is_null() {
            return write!(f, "[NULL]");
        }
        let (fx, fy) = from_index(self.get_from());
        let from = to_rank_file(fx, fy);
        let (to_x, to_y) = from_index(self.get_to());
        let to = to_rank_file(to_x, to_y);
        let (target_x, target_y) = from_index(self.get_target());
        let target = to_rank_file(target_x, target_y);
        let suffix = {
            if self.is_promotion() { format!("={}", self.promotion) }
            else if self.is_capture() { "x".to_string() }
            else if self.is_castling() { "(Castle)".to_string() }
            else { "".to_string() }
        };
        write!(f, "{from}->{to}{suffix} [t:{target}]")
    }
}

impl Default for Move {
    fn default() -> Self {
        Move::null()
    }
}
