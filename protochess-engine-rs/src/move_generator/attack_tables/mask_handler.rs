use arrayvec::ArrayVec;
use crate::types::{Bitboard, BIndex, BCoord};
use crate::utils::to_index;

/// General bitboard masks for use with attack tables
#[derive(Clone, Debug)]
pub struct MaskHandler {
    north: ArrayVec<[Bitboard;256]>,
    east: ArrayVec<[Bitboard;256]>,
    south: ArrayVec<[Bitboard;256]>,
    west: ArrayVec<[Bitboard;256]>,
    northeast: ArrayVec<[Bitboard;256]>,
    northwest: ArrayVec<[Bitboard;256]>,
    southeast: ArrayVec<[Bitboard;256]>,
    southwest: ArrayVec<[Bitboard;256]>,
    // Precomputed file attacks for a given 16-bit occupancy and column, 32 MB
    file_attacks: Vec<Vec<Bitboard>>,
    // Precomputed diagonal attacks for a given 16-bit occupancy and diagonal number, 62 MB
    diagonal_attacks: Vec<Vec<Bitboard>>,
    // Precomputed antidiagonal attacks for a given 16-bit occupancy and diagonal number, 62 MB
    antidiagonal_attacks: Vec<Vec<Bitboard>>,
}

impl MaskHandler {
    pub fn new() -> MaskHandler {
        let mut north = ArrayVec::<[Bitboard;256]>::new();
        let mut east = ArrayVec::<[Bitboard;256]>::new();
        let mut west = ArrayVec::<[Bitboard;256]>::new();
        let mut south = ArrayVec::<[Bitboard;256]>::new();
        let mut northeast = ArrayVec::<[Bitboard;256]>::new();
        let mut northwest = ArrayVec::<[Bitboard;256]>::new();
        let mut southeast = ArrayVec::<[Bitboard;256]>::new();
        let mut southwest = ArrayVec::<[Bitboard;256]>::new();
        let mut file_attacks = Vec::with_capacity(16);
        let mut diagonal_attacks = Vec::with_capacity(31);
        let mut antidiagonal_attacks = Vec::with_capacity(31);
        
        for _ in 0..256 {
            north.push(Bitboard::zero());
            east.push(Bitboard::zero());
            west.push(Bitboard::zero());
            south.push(Bitboard::zero());
            northeast.push(Bitboard::zero());
            northwest.push(Bitboard::zero());
            southeast.push(Bitboard::zero());
            southwest.push(Bitboard::zero());
        }

        for x in 0..16_i8 {
            for y in 0..16_i8 {
                let index: usize = to_index(x as BCoord, y as BCoord) as usize;

                //NORTH LOOKUP TABLE
                for j in y + 1..16 {
                    north[index].set_bit_at(x as BCoord, j as BCoord);
                }
                //SOUTH LOOKUP TABLE
                for j in 0..y {
                    south[index].set_bit_at(x as BCoord, j as BCoord);
                }
                //EAST LOOKUP TABLE
                for j in x + 1..16 {
                    east[index].set_bit_at(j as BCoord, y as BCoord);
                }
                //WEST LOOKUP TABLE
                for j in 0..x {
                    west[index].set_bit_at(j as BCoord, y as BCoord);
                }
                //NORTHEAST LOOKUP TABLE
                let mut x2 = x + 1;
                let mut y2 = y + 1;
                while x2 < 16 && y2 < 16 {
                    northeast[index].set_bit_at(x2 as BCoord, y2 as BCoord);
                    x2 +=1;
                    y2 +=1;
                }

                //NORTHWEST LOOKUP TABLE
                x2 = x - 1;
                y2 = y + 1;
                while x2 >= 0 && y2 < 16 {
                    northwest[index].set_bit_at(x2 as BCoord, y2 as BCoord);
                    x2 -= 1;
                    y2 += 1;
                }
                //SOUTHEAST LOOKUP TABLE
                x2 = x + 1;
                y2 = y - 1;
                while x2 < 16 && y2 >= 0 {
                    southeast[index].set_bit_at(x2 as BCoord, y2 as BCoord);
                    x2 += 1;
                    y2 -= 1;
                }
                //SOUTHWEST LOOKUP TABLE
                x2 = x - 1;
                y2 = y - 1;
                while x2 >= 0 && y2 >= 0 {
                    southwest[index].set_bit_at(x2 as BCoord, y2 as BCoord);
                    x2 -= 1;
                    y2 -= 1;
                }
            }
        }
        
        let mut leftmost_file = Bitboard::zero();
        for y in 0..16 {
            leftmost_file.set_bit_at(0, y);
        }
        
        // File attacks
        for x in 0..16 {
            let mut v = Vec::with_capacity(65536);
            for attack in 0..65536 {
                let mut file = Bitboard::zero();
                for y in 0..16 {
                    if attack & (1 << (15 - y)) != 0 {
                        file.set_bit_at(x, y);
                    }
                }
                v.push(file);
            }
            file_attacks.push(v);
        }
        
        // Diagonal attacks
        for d in 0..31 {
            let mut v = Vec::with_capacity(65536);
            let mut diagonal_bb = Bitboard::zero();
            for x in 0..16 {
                let y: i8 = d + x - 15;
                if y >= 0 && y < 16 {
                    diagonal_bb.set_bit_at(x as BCoord, y as BCoord);
                }
            }
            for attack in 0..65536 {
                let mut attack_bb = Bitboard::zero();
                attack_bb ^= attack as u16;
                let result = attack_bb.overflowing_mul(&leftmost_file) & &diagonal_bb;
                v.push(result);
            }
            diagonal_attacks.push(v);
        }
        
        // Antidiagonal attacks
        for d in 0..31 {
            let mut v = Vec::with_capacity(65536);
            let mut diagonal_bb = Bitboard::zero();
            for x in 0..16 {
                let y: i8 = d - x;
                if y >= 0 && y < 16 {
                    diagonal_bb.set_bit_at(x as BCoord, y as BCoord);
                }
            }
            for attack in 0..65536 {
                let mut attack_bb = Bitboard::zero();
                attack_bb ^= attack as u16;
                let result = attack_bb.overflowing_mul(&leftmost_file) & &diagonal_bb;
                v.push(result);
            }
            antidiagonal_attacks.push(v);
        }

        MaskHandler {
            north,
            east,
            south,
            west,
            northeast,
            northwest,
            southeast,
            southwest,
            file_attacks,
            diagonal_attacks,
            antidiagonal_attacks,
        }
    }

    pub fn get_north(&self, index: BIndex) -> &Bitboard{
        &self.north[index as usize]
    }

    pub fn get_south(&self, index: BIndex) -> &Bitboard{
        &self.south[index as usize]
    }

    pub fn get_east(&self, index: BIndex) -> &Bitboard{
        &self.east[index as usize]
    }

    pub fn get_west(&self, index: BIndex) -> &Bitboard{
        &self.west[index as usize]
    }

    pub fn get_northwest(&self, index: BIndex) -> &Bitboard{
        &self.northwest[index as usize]
    }

    pub fn get_northeast(&self, index: BIndex) -> &Bitboard{
        &self.northeast[index as usize]
    }

    pub fn get_southeast(&self, index: BIndex) -> &Bitboard{
        &self.southeast[index as usize]
    }

    pub fn get_southwest(&self, index: BIndex) -> &Bitboard{
        &self.southwest[index as usize]
    }
    
    pub fn get_file_attack(&self, x: BCoord, attack: u16) -> &Bitboard {
        &self.file_attacks[x as usize][attack as usize]
    }
    
    /// diagonal = y - x + 15
    pub fn get_diagonal_attack(&self, diagonal: u8, attack: u16) -> &Bitboard {
        &self.diagonal_attacks[diagonal as usize][attack as usize]
    }
    
    /// antidiagonal = y + x
    pub fn get_antidiagonal_attack(&self, antidiagonal: u8, attack: u16) -> &Bitboard {
        &self.antidiagonal_attacks[antidiagonal as usize][attack as usize]
    }
}

impl Default for MaskHandler {
    fn default() -> Self {
        Self::new()
    }
}
