mod mask_handler;

use crate::types::{Bitboard, BIndex};
use crate::move_generator::attack_tables::mask_handler::MaskHandler;
use crate::utils::{from_index};

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
        //First map the file to a rank
        let (x, y) = from_index(loc_index);
        //mask rank only and shift to A file
        let a_shifted = self.masks.shift_west(x, occ) & self.masks.get_file(0);
        let first_rank = self.masks.shift_south(15, &a_shifted.overflowing_mul(self.masks.get_main_diagonal()));
        //Lookup the attack in our table
        let occ_index = (first_rank.get_byte(0) as u16) ^ ((first_rank.get_byte(1) as u16) << 8);
        let rank_index = 15 - y;
        let attack = self.slider_attacks[rank_index as usize][occ_index as usize];
        //Map the attable back into the file
        let mut return_bb = Bitboard::zero();
        return_bb ^= attack;
        //Shift the rank back into the corresponding file
        let last_file = self.masks.get_right_mask(1) & return_bb.overflowing_mul(self.masks.get_main_diagonal());
        self.masks.shift_west(15 - x, &last_file)

    }

    pub fn get_diagonal_attack(&self, loc_index: BIndex, occ: &Bitboard) -> Bitboard {
        let (x, _y) = from_index(loc_index);
        //Map the diagonal to the first rank
        let masked_diag = occ & self.masks.get_diagonal(loc_index);
        let last_rank_with_garbage = masked_diag.overflowing_mul(self.masks.get_file(0));
        let first_rank = self.masks.shift_south(15,&last_rank_with_garbage);
        //Lookup the attack for the first rank
        let occ_index = (first_rank.get_byte(0) as u16) ^ ((first_rank.get_byte(1) as u16) << 8);
        let attack = self.slider_attacks[x as usize][occ_index as usize];
        //Map attack back to diagonal
        let mut return_bb = Bitboard::zero();
        return_bb ^= attack;
        return_bb.overflowing_mul(self.masks.get_file(0)) & self.masks.get_diagonal(loc_index)
    }

    pub fn get_antidiagonal_attack(&self, loc_index: BIndex, occ: &Bitboard) -> Bitboard {
        let (x, _y) = from_index(loc_index);
        //Map the diagonal to the first rank
        let masked_diag = occ & self.masks.get_antidiagonal(loc_index);
        let last_rank_with_garbage = masked_diag.overflowing_mul(self.masks.get_file(0));
        let first_rank = self.masks.shift_south(15,&last_rank_with_garbage);
        //Lookup the attack for the first rank
        let occ_index = (first_rank.get_byte(0) as u16) ^ ((first_rank.get_byte(1) as u16) << 8);
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
