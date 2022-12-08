use crate::types::{Bitboard, BIndex, BCoord};
use crate::utils::from_index;

// TODO: Make promotion_squares not an Option

/// External variant of MovementPattern for public
#[derive(Clone, Debug )]
pub struct MovementPatternExternal {
    // Places where this piece can promote, as well as char codes for the promotion pieces
    pub promotion_squares: Option<Vec<(BCoord, BCoord)>>,
    pub promo_vals: Option<Vec<char>>,

    // Ways the piece can capture (but not move without capturing)
    pub attack_sliding_deltas: Vec<Vec<(i8, i8)>>,
    pub attack_jump_deltas: Vec<(i8, i8)>,
    pub attack_north: bool,
    pub attack_south: bool,
    pub attack_east: bool,
    pub attack_west: bool,
    pub attack_northeast: bool,
    pub attack_northwest: bool,
    pub attack_southeast: bool,
    pub attack_southwest: bool,

    //Ways the piece can move (but not capture)
    pub translate_jump_deltas: Vec<(i8, i8)>,
    pub translate_sliding_deltas: Vec<Vec<(i8, i8)>>,
    pub translate_north: bool,
    pub translate_south: bool,
    pub translate_east: bool,
    pub translate_west: bool,
    pub translate_northeast: bool,
    pub translate_northwest: bool,
    pub translate_southeast: bool,
    pub translate_southwest: bool,
}

impl MovementPatternExternal {
    pub fn to_internal(self) -> MovementPattern{
        let promotion_squares = {
            if let Some(vec) = self.promotion_squares {
                let mut bb = Bitboard::zero();
                for (x, y) in vec {
                    bb.set_bit_at(x, y);
                }
                Some(bb)
            } else {
                None
            }
        };
        MovementPattern{
            promotion_squares,
            promo_vals: self.promo_vals,
            attack_sliding_deltas: self.attack_sliding_deltas,
            attack_jump_deltas: self.attack_jump_deltas,
            attack_north: self.attack_north,
            attack_south: self.attack_south,
            attack_east: self.attack_east,
            attack_west: self.attack_west,
            attack_northeast: self.attack_northeast,
            attack_northwest: self.attack_northwest,
            attack_southeast: self.attack_southeast,
            attack_southwest: self.attack_southwest,
            translate_jump_deltas: self.translate_jump_deltas,
            translate_sliding_deltas: self.translate_sliding_deltas,
            translate_north: self.translate_north,
            translate_south: self.translate_south,
            translate_east: self.translate_east,
            translate_west: self.translate_west,
            translate_northeast: self.translate_northeast,
            translate_northwest: self.translate_northwest,
            translate_southeast: self.translate_southeast,
            translate_southwest: self.translate_southwest
        }
    }
}

/// MovementPattern describes how a custom piece can move
/// Each sliding field expects a vec of vec, with the inner vec represnting a new "run"
#[derive(Clone, Debug )]
pub struct MovementPattern {
    // Places where this piece can promote, as well as char codes for the promotion pieces
    pub promotion_squares: Option<Bitboard>,
    pub promo_vals: Option<Vec<char>>,

    // Ways the piece can capture (but not move without capturing)
    pub attack_sliding_deltas: Vec<Vec<(i8, i8)>>,
    pub attack_jump_deltas: Vec<(i8, i8)>,
    pub attack_north: bool,
    pub attack_south: bool,
    pub attack_east: bool,
    pub attack_west: bool,
    pub attack_northeast: bool,
    pub attack_northwest: bool,
    pub attack_southeast: bool,
    pub attack_southwest: bool,

    //Ways the piece can move (but not capture)
    pub translate_jump_deltas: Vec<(i8, i8)>,
    pub translate_sliding_deltas: Vec<Vec<(i8, i8)>>,
    pub translate_north: bool,
    pub translate_south: bool,
    pub translate_east: bool,
    pub translate_west: bool,
    pub translate_northeast: bool,
    pub translate_northwest: bool,
    pub translate_southeast: bool,
    pub translate_southwest: bool,
}

impl MovementPattern {
    pub fn promotion_at(&self, index: BIndex) -> bool {
        if let Some(bb) = &self.promotion_squares {
            return bb.get_bit(index)
        }
        false
    }
    pub fn can_slide_north(&self) -> bool {
        self.translate_north || self.attack_north
    }
    pub fn can_slide_south(&self) -> bool {
        self.translate_south || self.attack_south
    }
    pub fn can_slide_east(&self) -> bool {
        self.translate_east || self.attack_east
    }
    pub fn can_slide_west(&self) -> bool {
        self.translate_west || self.attack_west
    }
    pub fn can_slide_northeast(&self) -> bool {
        self.translate_northeast || self.attack_northeast
    }
    pub fn can_slide_northwest(&self) -> bool {
        self.translate_northwest || self.attack_northwest
    }
    pub fn can_slide_southeast(&self) -> bool {
        self.translate_southeast || self.attack_southeast
    }
    pub fn can_slide_southwest(&self) -> bool {
        self.translate_southwest || self.attack_southwest
    }
    pub fn can_slide_main_direction(&self) -> bool {
        self.can_slide_north() || self.can_slide_south() || self.can_slide_east() || self.can_slide_west()
    }
    pub fn can_slide_north_indirectly(&self) -> bool {
        self.can_slide_north() || self.can_slide_northeast() || self.can_slide_northwest()
    }
    pub fn can_slide_south_indirectly(&self) -> bool {
        self.can_slide_south() || self.can_slide_southeast() || self.can_slide_southwest()
    }
    pub fn can_slide_east_indirectly(&self) -> bool {
        self.can_slide_east() || self.can_slide_northeast() || self.can_slide_southeast()
    }
    pub fn can_slide_west_indirectly(&self) -> bool {
        self.can_slide_west() || self.can_slide_northwest() || self.can_slide_southwest()
    }
    pub fn can_promote(&self) -> bool {
        self.promotion_squares.is_some()
    }
}

impl MovementPattern {
    pub fn to_external(self) -> MovementPatternExternal {
        let promotion_squares = {
            let mut sq = Vec::new();
            if let Some(mut bb) = self.promotion_squares {
                while bb.is_zero() {
                    let index = bb.lowest_one().unwrap();
                    sq.push(from_index(index as BIndex));
                    bb.clear_bit(index);
                }
                if sq.len() != 0 {
                    Some(sq)
                } else {
                    None
                }
            } else {
                None
            }
        };
        MovementPatternExternal {
            promotion_squares,
            promo_vals: self.promo_vals,
            attack_sliding_deltas: self.attack_sliding_deltas,
            attack_jump_deltas: self.attack_jump_deltas,
            attack_north: self.attack_north,
            attack_south: self.attack_south,
            attack_east: self.attack_east,
            attack_west: self.attack_west,
            attack_northeast: self.attack_northeast,
            attack_northwest: self.attack_northwest,
            attack_southeast: self.attack_southeast,
            attack_southwest: self.attack_southwest,
            translate_jump_deltas: self.translate_jump_deltas,
            translate_sliding_deltas: self.translate_sliding_deltas,
            translate_north: self.translate_north,
            translate_south: self.translate_south,
            translate_east: self.translate_east,
            translate_west: self.translate_west,
            translate_northeast: self.translate_northeast,
            translate_northwest: self.translate_northwest,
            translate_southeast: self.translate_southeast,
            translate_southwest: self.translate_southwest
        }
    }
}
