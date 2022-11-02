// Convert between x-y coordinates and bitboard indices

use crate::types::{BCoord, BIndex};

pub fn to_index(x: BCoord, y: BCoord) -> BIndex{
  (16 * y + x) as BIndex
}

pub fn from_index(index: BIndex) -> (BCoord, BCoord) {
  ((index % 16) as BCoord , (index / 16) as BCoord)
}
