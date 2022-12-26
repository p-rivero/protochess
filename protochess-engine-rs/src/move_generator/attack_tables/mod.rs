mod mask_handler;

use crate::types::{Bitboard, BIndex};
use crate::move_generator::attack_tables::mask_handler::MaskHandler;
use crate::utils::{from_index, to_index};

/// Holds pre-calculated attack tables for the pieces, assuming a 16x16 size board
/// Only for classical set of pieces
#[derive(Clone, Debug)]
pub struct AttackTables {
    slider_attacks: Vec<Vec<u16>>,
    pub masks: MaskHandler
}

impl AttackTables {
    pub fn new() -> AttackTables {
        //16 * 2^16 possible states; 16 squares in 1 rank, 2^16 possible occupancies per rank
        let mut slider_attacks = vec![vec![0; 65536]; 16];
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
        }

        AttackTables{
            slider_attacks,
            masks: MaskHandler::new()
        }
    }

    pub fn get_rank_attack(&self, loc_index: BIndex, occ: &Bitboard) -> Bitboard {
        let (x, y) = from_index(loc_index);
        //Isolate the rank
        let rank_only = self.masks.shift_south(y, occ);
        let first_byte = rank_only.get_byte(0);
        let second_byte = rank_only.get_byte(1);
        //Looup the occupancy rank in our table
        let occ_index = (second_byte as u16) << 8 ^ (first_byte as u16);
        let attack = self.slider_attacks[x as usize][occ_index as usize];
        //Shift attack back to rank
        let mut return_bb = Bitboard::zero();
        return_bb ^= attack;
        self.masks.shift_north(y, &return_bb)
    }

    pub fn get_file_attack(&self, loc_index: BIndex, occ: &Bitboard) -> Bitboard {
        let mut occ_index = 0;
        let (x, y_loc) = from_index(loc_index);
        for y in 0..16 {
            occ_index <<= 1;
            if occ.get_bit(to_index(x, y)) {
                occ_index |= 1;
            }
        }
        let rank_index = (15 - y_loc) as usize;
        let attack = self.slider_attacks[rank_index][occ_index];
        //Map the attable back into the file
        let mut return_bb = Bitboard::zero();
        for y in 0..16 {
            if attack & (1 << (15 - y)) != 0 {
                return_bb.set_bit(to_index(x, y));
            }
        }
        return_bb
    }

    pub fn get_diagonal_attack(&self, loc_index: BIndex, occ: &Bitboard) -> Bitboard {
        let (x, y) = from_index(loc_index);
        // Start at the top right corner of the diagonal
        let mut index = {
            if x >= y {
                // Bottom right triangle
                to_index(15, 15 + y - x) as i16
            } else {
                // Top left triangle
                to_index(15 + x - y, 15) as i16
            }
        };
        
        // Move down-left until we hit the end
        let mut occ_index = 0;
        while index >= 0 {
            occ_index <<= 1;
            if occ.get_bit(index as BIndex) {
                occ_index |= 1;
            }
            index -= 17;
        }
        if x >= y {
            occ_index <<= x - y;
        } else {
            occ_index >>= y - x - 1;
        }

        // Lookup the attack for the first rank
        let attack = self.slider_attacks[x as usize][occ_index as usize];
        //Map attack back to diagonal
        let mut return_bb = Bitboard::zero();
        return_bb ^= attack;
        return_bb.overflowing_mul(self.masks.get_file(0)) & self.masks.get_diagonal(loc_index)
    }

    pub fn get_antidiagonal_attack(&self, loc_index: BIndex, occ: &Bitboard) -> Bitboard {
        let (x, y) = from_index(loc_index);
        // Start at the bottom right corner of the diagonal
        let (mut index, num_iters) = {
            if y + x <= 15 {
                // Bottom left triangle
                (to_index(y + x, 0) as i16, y + x + 1)
            } else {
                // Top right triangle
                (to_index(15, y + x - 15) as i16, 16 + 15 - y - x)
            }
        };
        
        // Move up-left until we hit the end
        let mut occ_index = 0;
        for _ in 0..num_iters {
            occ_index <<= 1;
            if occ.get_bit(index as BIndex) {
                occ_index |= 1;
            }
            index += 15 as i16;
        }
        if y + x > 15 {
            occ_index <<= 16 - num_iters;
        }
        
        //Lookup the attack for the first rank
        let attack = self.slider_attacks[x as usize][occ_index as usize];
        //Map attack back to diagonal
        let mut return_bb = Bitboard::zero();
        return_bb ^= attack;
        return_bb.overflowing_mul(self.masks.get_file(0)) & self.masks.get_antidiagonal(loc_index)
    }

    /// Returns a bitboard of the sliding piece moves
    #[allow(clippy::too_many_arguments)]
    pub fn get_sliding_moves_bb(&self,
                                loc_index: BIndex,
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
        let mut raw_attacks = Bitboard::zero();
        if north || south {
            raw_attacks |= self.get_file_attack(loc_index, occ);
            if !north {
                raw_attacks &= !self.masks.get_north(loc_index);
            } else if !south {
                raw_attacks &= !self.masks.get_south(loc_index);
            }
        }

        if east || west {
            raw_attacks |= self.get_rank_attack(loc_index, occ);
            if !east {
                raw_attacks &= !self.masks.get_east(loc_index);
            } else if !west {
                raw_attacks &= !self.masks.get_west(loc_index);
            }
        }

        if northeast || southwest {
            raw_attacks |= self.get_diagonal_attack(loc_index, occ);
            if !northeast {
                raw_attacks &= !self.masks.get_northeast(loc_index);
            } else if !southwest {
                raw_attacks &= !self.masks.get_southwest(loc_index);
            }
        }

        if northwest || southeast {
            raw_attacks |= self.get_antidiagonal_attack(loc_index, occ);
            if !northwest {
                raw_attacks &= !self.masks.get_northwest(loc_index);
            } else if !southeast {
                raw_attacks &= !self.masks.get_southeast(loc_index);
            }
        }

        raw_attacks
    }

}

impl Default for AttackTables {
    fn default() -> Self {
        Self::new()
    }
}
