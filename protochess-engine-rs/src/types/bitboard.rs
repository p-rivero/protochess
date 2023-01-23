use std::ops;
use impl_ops::*;

use crate::utils::to_index;


pub type BIndex = u8; // 256 positions in 16x16 board
pub type BCoord = u8; // Coordinate the board: [0..15]

// Store bounds of the board (bit set to 1 for valid positions) and dimensions (width and height)
#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct BDimensions {
    pub width: BCoord,
    pub height: BCoord,
    pub bounds: Bitboard,
}

impl BDimensions {
    // Create a BDimensions object of a given width and height, with all squares valid
    pub fn new_without_walls(width: BCoord, height: BCoord) -> BDimensions {
        assert!(width <= 16 && height <= 16, "Board dimensions must be <= 16x16");
        let mut bounds = Bitboard::zero();
        for x in 0..width {
            for y in 0..height {
                bounds.set_bit_at(x, y);
            }
        }
        BDimensions { width, height, bounds }
    }
    // Given a list of valid squares (coordinates), return a BDimensions object
    pub fn from_valid_squares(valid_squares: &Vec<(BCoord, BCoord)>) -> BDimensions {
        let mut width = 0;
        let mut height = 0;
        let mut bounds = Bitboard::zero();
        for sq in valid_squares {
            assert!(sq.0 < 16 && sq.1 < 16, "Board dimensions must be <= 16x16");
            if sq.0 >= width { width = sq.0 + 1; }
            if sq.1 >= height { height = sq.1 + 1; }
            bounds.set_bit_at(sq.0, sq.1);
        }
        BDimensions{width, height, bounds}
    }
    // Return true if the given coordinates are within the bounds of the board
    pub fn in_bounds(&self, x: BCoord, y: BCoord) -> bool {
        if x < self.width && y < self.height {
            return self.bounds.get_bit_at(x, y)
        }
        false
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Bitboard {
    board_internal: numext_fixed_uint::U256,
}

impl Bitboard {
    #[inline]
    pub fn zero() -> Bitboard {
        Bitboard { board_internal: numext_fixed_uint::U256::zero() }
    }
    #[inline]
    pub fn one() -> Bitboard {
        Bitboard { board_internal: numext_fixed_uint::U256::one() }
    }
    pub fn from_coord_list(squares: &[(BCoord, BCoord)]) -> Bitboard {
        let mut board = Bitboard::zero();
        squares.iter().for_each(|(x,y)| board.set_bit_at(*x, *y));
        board
    }
    #[inline]
    pub fn set_bit(&mut self, index: BIndex) {
        self.board_internal.set_bit(index as usize, true);
    }
    #[inline]
    pub fn clear_bit(&mut self, index: BIndex) {
        self.board_internal.set_bit(index as usize, false);
    }
    #[inline]
    pub fn set_bit_at(&mut self, x: BCoord, y: BCoord) {
        self.set_bit(to_index(x, y));
    }
    #[inline]
    pub fn clear_bit_at(&mut self, x: BCoord, y: BCoord) {
        self.clear_bit(to_index(x, y));
    }
    #[inline]
    pub fn get_bit(&self, index: BIndex) -> bool {
        self.board_internal.bit(index as usize).unwrap()
    }
    #[inline]
    pub fn get_bit_at(&self, x: BCoord, y: BCoord) -> bool {
        self.get_bit(to_index(x, y))
    }
    #[inline]
    pub fn is_zero(&self) -> bool {
        self.board_internal.is_zero()
    }
    #[inline]
    pub fn lowest_one(&self) -> Option<BIndex> {
        self.board_internal.lowest_one().map(|x| x as BIndex)
    }
    #[inline]
    pub fn highest_one(&self) -> Option<BIndex> {
        self.board_internal.highest_one().map(|x| x as BIndex)
    }
    #[inline]
    pub fn count_ones(&self) -> u32 {
        self.board_internal.count_ones()
    }
    #[inline]
    pub fn overflowing_mul(self, rhs: &Bitboard) -> Bitboard {
        Bitboard { board_internal: self.board_internal.overflowing_mul(&rhs.board_internal).0 }
    }
    #[inline]
    pub fn get_inner(&self) -> &[u64; 4] {
        self.board_internal.get_inner()
    }
    #[inline]
    pub fn get_inner_mut(&mut self) -> &mut [u64; 4] {
        self.board_internal.get_inner_mut()
    }
}
impl std::fmt::Display for Bitboard {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        for y in (0..16).rev() {
            for x in 0..16 {
                if self.get_bit_at(x, y) {
                    write!(f, "1")?;
                } else {
                    write!(f, ".")?;
                }
                write!(f, " ")?;
            }
            writeln!(f)?;
        }
        Ok(())
    }
}
impl std::fmt::LowerHex for Bitboard {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        write!(f, "{:x}", self.board_internal)
    }
}
impl Default for Bitboard {
    fn default() -> Self {
        Bitboard::zero()
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
impl_op_ex!(^= |a: &mut Bitboard, b: &Bitboard| { a.board_internal ^= &b.board_internal });
impl_op_ex!(^= |a: &mut Bitboard, b: u16| { a.board_internal ^= numext_fixed_uint::U256::from(b) });
impl_op_ex!(<<= |a: &mut Bitboard, b: BCoord| { a.board_internal <<= b });
impl_op_ex!(>>= |a: &mut Bitboard, b: BCoord| { a.board_internal >>= b });

trait GetBitboardInner {
    fn get_inner(&self) -> &[u64; 4];
    fn get_inner_mut(&mut self) -> &mut [u64; 4];
}
impl GetBitboardInner for numext_fixed_uint::U256 {
    fn get_inner(&self) -> &[u64; 4] {
        &self.0
    }
    fn get_inner_mut(&mut self) -> &mut [u64; 4] {
        &mut self.0
    }
}
