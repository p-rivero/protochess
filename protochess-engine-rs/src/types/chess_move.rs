use std::fmt;
use crate::piece::PieceId;
use crate::utils::{to_rank_file, from_index};

use super::{BCoord, BIndex};

#[derive(Eq, PartialEq, Debug)]
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

#[derive(Eq, PartialEq, Copy, Clone, Debug)]
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
    promotion: Option<PieceId>
}

impl Move {
    pub fn new(from: BIndex, to: BIndex, target_loc: Option<BIndex>, move_type: MoveType, promotion: Option<PieceId>) -> Move {
        let target = target_loc.unwrap_or(0);
        Move {
            move_fields: (from as u32) | (to as u32) << 8 | (target as u32) << 16 | (move_type as u32) << 24,
            promotion
        }
    }

    pub fn null() -> Move {
        Move::new(0,0,None,MoveType::Null, None)
    }
    
    pub fn is_null(&self) -> bool {
        self.get_move_type() == MoveType::Null
    }
    
    pub fn is_quiet(&self) -> bool {
        self.get_move_type() == MoveType::Quiet || self.get_move_type() == MoveType::DoubleJump
    }

    pub fn get_from(&self) -> BIndex{
        (self.move_fields & (BIndex::MAX as u32)) as BIndex
    }

    pub fn get_to(&self) -> BIndex{
        ((self.move_fields >> 8) & (BIndex::MAX as u32)) as BIndex
    }
    
    // Get the index of the victim piece, if any. Usually the same as get_to(), except for en passant
    // In double jump, this is the index of the generated en passant square
    pub fn get_target(&self) -> BIndex {
        ((self.move_fields >> 16) & (BIndex::MAX as u32)) as BIndex
    }

    pub fn is_capture(&self) -> bool {
        // The least significant bit of the move type is used to indicate capture
        ((self.move_fields >> 24) & 1) != 0
    }

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
        self.promotion
    }
}

impl fmt::Display for Move {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let (x1, y1) = from_index(self.get_from());
        let (x2, y2) = from_index(self.get_to());
        write!(f, "({} -> {})", to_rank_file(x1, y1),to_rank_file(x2, y2))
    }
}


#[derive(Debug, Clone, Copy)]
pub struct MoveInfo {
    pub from: (BCoord, BCoord),
    pub to: (BCoord, BCoord),
    pub promotion: Option<PieceId>,
}

impl MoveInfo {
    pub fn from_move(m: Move) -> MoveInfo {
        let (from_x, from_y) = from_index(m.get_from());
        let (to_x, to_y) = from_index(m.get_to());
        MoveInfo {
            from: (from_x, from_y),
            to: (to_x, to_y),
            promotion: m.get_promotion_piece()
        }
    }
    
    // Create a MoveInfo from a string like "e2e4" or "e7e8=123" (promotion to piece with id 123)
    pub fn from_string(s: &str) -> MoveInfo {
        let mut chars = s.chars();
        let from_x = chars.next().unwrap() as u8 - b'a';
        let from_y = chars.next().unwrap() as u8 - b'1';
        let to_x = chars.next().unwrap() as u8 - b'a';
        let to_y = chars.next().unwrap() as u8 - b'1';
        let promotion = {
            if chars.next() == Some('=') {
                let id = chars.as_str().parse::<PieceId>().unwrap();
                Some(id)
            } else {
                None
            }
        };
        MoveInfo {
            from: (from_x, from_y),
            to: (to_x, to_y),
            promotion
        }
    }
    
    pub fn matches_move(&self, m: Move) -> bool {
        let (from_x, from_y) = from_index(m.get_from());
        let (to_x, to_y) = from_index(m.get_to());
        self.from == (from_x, from_y) && self.to == (to_x, to_y) && self.promotion == m.get_promotion_piece()
    }
}
