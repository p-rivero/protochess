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
    diagonals: ArrayVec<[Bitboard;256]>,
    antidiagonals: ArrayVec<[Bitboard;256]>,
    files: ArrayVec<[Bitboard;16]>,
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
        let mut diagonals = ArrayVec::<[Bitboard;256]>::new();
        let mut antidiagonals = ArrayVec::<[Bitboard;256]>::new();
        for _i in 0..256 {
            north.push(Bitboard::zero());
            east.push(Bitboard::zero());
            west.push(Bitboard::zero());
            south.push(Bitboard::zero());
            northeast.push(Bitboard::zero());
            northwest.push(Bitboard::zero());
            southeast.push(Bitboard::zero());
            southwest.push(Bitboard::zero());
            diagonals.push(Bitboard::zero());
            antidiagonals.push(Bitboard::zero());
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

                diagonals[index] = &northeast[index] ^ &southwest[index];
                antidiagonals[index] = &northwest[index] ^ &southeast[index];
            }
        }
        
        let mut files = ArrayVec::<[Bitboard;16]>::new();
        for i in 0..16 {
            let mut file = Bitboard::zero();
            for y in 0..16 {
                file.set_bit_at(i, y);
            }
            files.push(file);
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
            diagonals,
            antidiagonals,
            files,
        }
    }

    pub fn get_diagonal(&self, index: BIndex) -> &Bitboard{
        &self.diagonals[index as usize]
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

    pub fn get_antidiagonal(&self, index: BIndex) -> &Bitboard{
        &self.antidiagonals[index as usize]
    }

    pub fn get_file(&self, n: BCoord) -> &Bitboard {
        &self.files[n as usize]
    }
}

impl Default for MaskHandler {
    fn default() -> Self {
        Self::new()
    }
}
