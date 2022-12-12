use std::fmt;
use crate::piece::PieceId;
use crate::utils::{to_rank_file, from_index};
use crate::types::bitboard::BIndex;

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
    Null = 0b1000,
}

#[derive(Eq, PartialEq, Copy, Clone, Debug)]
pub struct Move {
    /// Stores a move in a u32
    ///0-7:   from index:u8
    ///8-15:  to index:u8
    ///16-23: target index:u8
    ///24-27 : movetype (see MoveType above)
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

    pub fn get_from(&self) -> BIndex{
        (self.move_fields & (BIndex::MAX as u32)) as BIndex
    }

    pub fn get_to(&self) -> BIndex{
        ((self.move_fields >> 8) & (BIndex::MAX as u32)) as BIndex
    }
    
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
        match_move_type!(Quiet, Capture, KingsideCastle, QueensideCastle, Promotion, PromotionCapture, Null)
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
