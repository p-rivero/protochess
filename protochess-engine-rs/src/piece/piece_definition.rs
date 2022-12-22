use crate::types::{Bitboard, BIndex};
use super::PieceId;


/// External representation of a piece

#[derive(Clone, Debug)]
pub struct PieceDefinition {
    pub id: PieceId,
    pub char_rep: char,
    
    pub is_leader: bool,
    pub can_castle: bool,
    pub is_castle_rook: bool,
    
    // Places where this piece can promote, as well as PieceId for the promotion pieces
    pub promotion_squares: Bitboard,
    pub promo_vals: Vec<PieceId>,
    
    // Places where this piece can double move
    pub double_move_squares: Bitboard,

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

impl PieceDefinition {
    pub fn promotion_at(&self, index: BIndex) -> bool {
        self.promotion_squares.get_bit(index)
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
        let has_promotion_squares = !self.promotion_squares.is_zero();
        let has_promo_vals = !self.promo_vals.is_empty();
        assert!(has_promotion_squares == has_promo_vals);
        has_promotion_squares
    }
    pub fn can_jump(&self) -> bool {
        !self.translate_jump_deltas.is_empty() || !self.attack_jump_deltas.is_empty()
    }
    pub fn can_double_jump(&self) -> bool {
        !self.double_move_squares.is_zero()
    }
    pub fn has_sliding_deltas(&self) -> bool {
        !self.translate_sliding_deltas.is_empty() || !self.attack_sliding_deltas.is_empty()
    }
}
