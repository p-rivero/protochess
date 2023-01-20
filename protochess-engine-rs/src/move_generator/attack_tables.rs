use crate::types::{Bitboard, BIndex};
use crate::utils::{from_index, to_index};

/// Holds pre-calculated attack tables for the pieces, assuming a 16x16 size board
/// Only for classical set of pieces
#[derive(Clone, Debug)]
pub struct AttackTables {
    // Precomputed slider moves for a given 16-bit row and index
    rank_slides: Vec<Vec<u16>>,
    // Precomputed masks for bits at the left and right of a given square
    left_masks: Vec<Bitboard>,
    right_masks: Vec<Bitboard>,
    // L-shaped masks for edges of the board
    top_right: Bitboard,
    bottom_left: Bitboard,
    top_left: Bitboard,
    bottom_right: Bitboard,
    // Masks for edges of the board
    horizontal_edges: Bitboard,
    vertical_edges: Bitboard,
}

impl AttackTables {
    pub fn new() -> AttackTables {
        //16 * 2^16 possible states; 16 squares in 1 rank, 2^16 possible occupancies per rank
        let mut slider_attacks = vec![vec![0; 65536]; 16];
        let mut top_bits = Bitboard::zero();
        let mut bottom_bits = Bitboard::zero();
        let mut left_bits = Bitboard::zero();
        let mut right_bits = Bitboard::zero();
        let mut left_masks = Vec::with_capacity(256);
        let mut right_masks = Vec::with_capacity(256);
        //16 squares in 1 rank
        for i in 0..16 {
            //2^16 = 65536 possible occupancies
            for occ in 0..=65535 {
                //Classical approach to generate the table
                fn get_left_attack(src:u16) -> u16 {
                    if src == 0 {
                        0
                    } else {
                        src - 1u16
                    }
                }
                fn get_right_attack(src:u16) -> u16 {
                    !src & !get_left_attack(src)
                }
                //Square from index
                let sq = 1u16 << i;

                let mut left_attack = get_left_attack(sq);
                let left_blockers = occ & left_attack;

                if left_blockers != 0 {
                    let msb_blockers = 1u16 << (15 - left_blockers.leading_zeros());
                    left_attack ^= get_left_attack(msb_blockers);
                }

                let mut right_attack = get_right_attack(sq);
                let right_blockers = occ & right_attack;
                if right_blockers != 0 {
                    let lsb_blockers = 1u16 << right_blockers.trailing_zeros();
                    right_attack ^= get_right_attack(lsb_blockers);
                }
                slider_attacks[i as usize][occ as usize] = right_attack ^ left_attack;
            }
            top_bits.set_bit_at(i, 15);
            bottom_bits.set_bit_at(i, 0);
            left_bits.set_bit_at(0, i);
            right_bits.set_bit_at(15, i);
        }
        for _ in 0..256 {
            left_masks.push(Bitboard::zero());
            right_masks.push(Bitboard::zero());
        }
        for x in 0..16 {
            for y in 0..16 {
                let index: usize = to_index(x, y) as usize;
                for j in 0..x {
                    left_masks[index].set_bit_at(j, y);
                }
                for j in (x + 1)..16 {
                    right_masks[index].set_bit_at(j, y);
                }
            }
        }
        
        AttackTables{
            rank_slides: slider_attacks,
            left_masks,
            right_masks,
            top_right: &top_bits | &right_bits,
            bottom_left: &bottom_bits | &left_bits,
            top_left: &top_bits | &left_bits,
            bottom_right: &bottom_bits | &right_bits,
            horizontal_edges: &left_bits | &right_bits,
            vertical_edges: &top_bits | &bottom_bits,
        }
    }

    pub fn get_rank_slide(&self, loc_index: BIndex, occ: &Bitboard) -> Bitboard {
        let (x, y) = from_index(loc_index);
        //Isolate the rank
        let word_index = y / 4;
        let word = occ.get_inner()[word_index as usize];
        let line_index = y % 4;
        let occ_index = (word >> (line_index * 16)) as u16;
        //Looup the occupancy rank in our table
        let attack = self.rank_slides[x as usize][occ_index as usize];
        //Shift attack back to rank
        let mut return_bb = Bitboard::zero();
        let new_word = (attack as u64) << (line_index * 16);
        return_bb.get_inner_mut()[word_index as usize] = new_word;
        return_bb
    }
    
    #[inline]
    pub fn add_slide_top_right(&self, out: &mut Bitboard, start_index: BIndex, occ: &Bitboard, step: i16) {
        let mut index = start_index as i16 + step;
        loop {
            out.set_bit(index as BIndex);
            if occ.get_bit(index as BIndex) {
                break;
            }
            index += step;
        }
    }
    
    #[inline]
    pub fn add_slide_bottom_left(&self, out: &mut Bitboard, start_index: BIndex, occ: &Bitboard, step: i16) {
        let mut index = start_index as i16 - step;
        loop {
            out.set_bit(index as BIndex);
            if occ.get_bit(index as BIndex) {
                break;
            }
            index -= step;
        }
    }
    
    /// Returns a bitboard of the sliding piece moves
    #[allow(clippy::too_many_arguments)]
    pub fn get_sliding_moves_bb(&self,
                                index: BIndex,
                                occ: &Bitboard,
                                north: bool,
                                east: bool,
                                south: bool,
                                west: bool,
                                northeast: bool,
                                northwest: bool,
                                southeast:bool,
                                southwest:bool,
    ) -> Bitboard {
        let mut moves = {
            if east || west {
                let mut m = self.get_rank_slide(index, occ);
                if !east {
                    m &= &self.left_masks[index as usize];
                } else if !west {
                    m &= &self.right_masks[index as usize];
                }
                m
            } else {
                Bitboard::zero()
            }
        };
        
        // For north/south, add top and bottom edges to the occupied bitboard to handle end condition
        let mut occ = occ | &self.vertical_edges;
        
        if north && index < 240 {
            self.add_slide_top_right(&mut moves, index, &occ, 16);
        }
        if south && index > 15 {
            self.add_slide_bottom_left(&mut moves, index, &occ, 16);
        }
        
        // For diagonals, add remaining edges to the occupied bitboard to to handle end condition
        occ |= &self.horizontal_edges;
        
        if northeast && !self.top_right.get_bit(index) {
            self.add_slide_top_right(&mut moves, index, &occ, 17);
        }
        if southwest && !self.bottom_left.get_bit(index) {
            self.add_slide_bottom_left(&mut moves, index, &occ, 17);
        }
        if northwest && !self.top_left.get_bit(index) {
            self.add_slide_top_right(&mut moves, index, &occ, 15);
        }
        if southeast && !self.bottom_right.get_bit(index) {
            self.add_slide_bottom_left(&mut moves, index, &occ, 15);
        }
        
        moves
    }

}

impl Default for AttackTables {
    fn default() -> Self {
        Self::new()
    }
}
