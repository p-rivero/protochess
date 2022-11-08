use std::ops;
use impl_ops::*;

use crate::utils::to_index;


pub type BIndex = u8; // 256 positions in 16x16 board
pub type BCoord = u8; // Coordinate the board: [0..15]

#[derive(Clone, Debug)]
pub struct BDimensions {
    pub width: BCoord,
    pub height: BCoord,
}

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
    pub fn all_ones() -> Bitboard {
        Bitboard { board_internal: numext_fixed_uint::U256::max_value() }
    }
    pub fn set_bit(&mut self, index: BIndex) {
        self.board_internal.set_bit(index as usize, true);
    }
    pub fn clear_bit(&mut self, index: BIndex) {
        self.board_internal.set_bit(index as usize, false);
    }
    pub fn set_bit_at(&mut self, x: BCoord, y: BCoord) {
        self.set_bit(to_index(x, y));
    }
    pub fn clear_bit_at(&mut self, x: BCoord, y: BCoord) {
        self.clear_bit(to_index(x, y));
    }
    pub fn get_bit(&self, index: BIndex) -> bool {
        self.board_internal.bit(index as usize).unwrap()
    }
    pub fn get_bit_at(&self, x: BCoord, y: BCoord) -> bool {
        self.get_bit(to_index(x, y))
    }
    pub fn get_byte(&self, index: usize) -> u8 {
        self.board_internal.byte(index).unwrap()
    }
    pub fn is_zero(&self) -> bool {
        self.board_internal.is_zero()
    }
    pub fn lowest_one(&self) -> Option<BIndex> {
        self.board_internal.lowest_one().map(|x| x as BIndex)
    }
    pub fn count_ones(&self) -> u32 {
        self.board_internal.count_ones()
    }
    pub fn overflowing_mul(&self, rhs: &Bitboard) -> Bitboard {
        Bitboard { board_internal: self.board_internal.overflowing_mul(&rhs.board_internal).0 }
    }
    pub fn to_string(&self, bitboard:&Bitboard) -> String {
        let mut return_str = String::new();
        for y in (0..16).rev() {
            for x in 0..16 {
                if bitboard.get_bit_at(x, y) {
                    return_str.push('1');
                } else {
                    return_str.push('.');
                }
                return_str.push(' ');
            }
            return_str.push('\n');
        }
        return_str
    }
}

impl_op_ex!(+ |a: &Bitboard, b: &Bitboard| -> Bitboard { Bitboard{board_internal: &a.board_internal + &b.board_internal} });
impl_op_ex!(- |a: &Bitboard, b: &Bitboard| -> Bitboard { Bitboard{board_internal: &a.board_internal - &b.board_internal} });
impl_op_ex!(& |a: &Bitboard, b: &Bitboard| -> Bitboard { Bitboard{board_internal: &a.board_internal & &b.board_internal} });
impl_op_ex!(| |a: &Bitboard, b: &Bitboard| -> Bitboard { Bitboard{board_internal: &a.board_internal | &b.board_internal} });
impl_op_ex!(^ |a: &Bitboard, b: &Bitboard| -> Bitboard { Bitboard{board_internal: &a.board_internal ^ &b.board_internal} });
impl_op_ex!(<< |a: &Bitboard, b: BCoord| -> Bitboard { Bitboard{board_internal: &a.board_internal << b} });
impl_op_ex!(>> |a: &Bitboard, b: BCoord| -> Bitboard { Bitboard{board_internal: &a.board_internal >> b} });
impl_op_ex!(! |a: &Bitboard| -> Bitboard { Bitboard{board_internal: ! &a.board_internal} });

impl_op_ex!(+= |a: &mut Bitboard, b: &Bitboard| { a.board_internal += &b.board_internal });
impl_op_ex!(-= |a: &mut Bitboard, b: &Bitboard| { a.board_internal -= &b.board_internal });
impl_op_ex!(&= |a: &mut Bitboard, b: &Bitboard| { a.board_internal &= &b.board_internal });
impl_op_ex!(|= |a: &mut Bitboard, b: &Bitboard| { a.board_internal |= &b.board_internal });
impl_op_ex!(|= |a: &mut Bitboard, b: u64| { a.board_internal |= numext_fixed_uint::U256::from(b) });
impl_op_ex!(^= |a: &mut Bitboard, b: &Bitboard| { a.board_internal ^= &b.board_internal });
impl_op_ex!(^= |a: &mut Bitboard, b: u16| { a.board_internal ^= numext_fixed_uint::U256::from(b) });
impl_op_ex!(<<= |a: &mut Bitboard, b: BCoord| { a.board_internal <<= b });
impl_op_ex!(>>= |a: &mut Bitboard, b: BCoord| { a.board_internal >>= b });
