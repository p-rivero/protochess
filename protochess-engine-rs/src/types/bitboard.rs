
pub fn to_index(x:u8, y:u8) -> BoardIndex{
    (16 * y + x) as BoardIndex
}

pub fn from_index(index:BoardIndex) -> (u8, u8) {
    (index % 16 , index / 16 )
}

pub type BoardIndex = u8; // 256 positions in 16x16 board

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Bitboard {
    board_internal: numext_fixed_uint::U256,
}

impl Bitboard {
    pub fn zero() -> Bitboard {
        Bitboard { board_internal: numext_fixed_uint::U256::zero() }
    }
    pub fn one() -> Bitboard {
        Bitboard { board_internal: numext_fixed_uint::U256::one() }
    }
    pub fn set_bit(&mut self, index: BoardIndex) {
        self.board_internal.set_bit(index as usize, true);
    }
    pub fn clear_bit(&mut self, index: BoardIndex) {
        self.board_internal.set_bit(index as usize, false);
    }
    pub fn set_bit_at(&mut self, x: u8, y: u8) {
        self.set_bit(to_index(x, y));
    }
    pub fn clear_bit_at(&mut self, x: u8, y: u8) {
        self.clear_bit(to_index(x, y));
    }
    pub fn get_bit(&self, index: BoardIndex) -> bool {
        self.board_internal.bit(index as usize).unwrap()
    }
    pub fn get_bit_at(&self, x: u8, y: u8) -> bool {
        self.get_bit(to_index(x, y))
    }
    pub fn get_byte(&self, index: usize) -> u8 {
        self.board_internal.byte(index).unwrap()
    }
    pub fn is_zero(&self) -> bool {
        self.board_internal.is_zero()
    }
    pub fn lowest_one(&self) -> Option<BoardIndex> {
        self.board_internal.lowest_one().map(|x| x as BoardIndex)
    }
    pub fn count_ones(&self) -> u32 {
        self.board_internal.count_ones()
    }
    pub fn overflowing_mul(self, rhs: &Bitboard) -> Bitboard {
        Bitboard { board_internal: self.board_internal.overflowing_mul(&rhs.board_internal).0 }
    }
}

use std::ops;
use impl_ops::*;

impl_op_ex!(+ |a: &Bitboard, b: &Bitboard| -> Bitboard { Bitboard{board_internal: &a.board_internal + &b.board_internal} });
impl_op_ex!(- |a: &Bitboard, b: &Bitboard| -> Bitboard { Bitboard{board_internal: &a.board_internal - &b.board_internal} });
impl_op_ex!(& |a: &Bitboard, b: &Bitboard| -> Bitboard { Bitboard{board_internal: &a.board_internal & &b.board_internal} });
impl_op_ex!(| |a: &Bitboard, b: &Bitboard| -> Bitboard { Bitboard{board_internal: &a.board_internal | &b.board_internal} });
impl_op_ex!(^ |a: &Bitboard, b: &Bitboard| -> Bitboard { Bitboard{board_internal: &a.board_internal ^ &b.board_internal} });
impl_op_ex!(<< |a: &Bitboard, b: u8| -> Bitboard { Bitboard{board_internal: &a.board_internal << b} });
impl_op_ex!(>> |a: &Bitboard, b: u8| -> Bitboard { Bitboard{board_internal: &a.board_internal >> b} });
impl_op_ex!(! |a: &Bitboard| -> Bitboard { Bitboard{board_internal: ! &a.board_internal} });

impl_op_ex!(+= |a: &mut Bitboard, b: &Bitboard| { a.board_internal += &b.board_internal });
impl_op_ex!(-= |a: &mut Bitboard, b: &Bitboard| { a.board_internal -= &b.board_internal });
impl_op_ex!(&= |a: &mut Bitboard, b: &Bitboard| { a.board_internal &= &b.board_internal });
impl_op_ex!(|= |a: &mut Bitboard, b: &Bitboard| { a.board_internal |= &b.board_internal });
impl_op_ex!(|= |a: &mut Bitboard, b: u64| { a.board_internal |= numext_fixed_uint::U256::from(b) });
impl_op_ex!(^= |a: &mut Bitboard, b: &Bitboard| { a.board_internal ^= &b.board_internal });
impl_op_ex!(^= |a: &mut Bitboard, b: u16| { a.board_internal ^= numext_fixed_uint::U256::from(b) });
impl_op_ex!(<<= |a: &mut Bitboard, b: u8| { a.board_internal <<= b });
impl_op_ex!(>>= |a: &mut Bitboard, b: u8| { a.board_internal >>= b });


// pub fn to_string(bitboard:&Bitboard) -> String {
//     let mut return_str = String::new();
//     for y in (0..16).rev() {
//         for x in 0..16 {
//             if bitboard.bit(to_index(x, y)).unwrap() {
//                 return_str.push('1');
//             } else {
//                 return_str.push('.');
//             }
//             return_str.push(' ');
//         }
//         return_str.push('\n');
//     }
//     return_str
// }


