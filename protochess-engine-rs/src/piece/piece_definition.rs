use crate::types::{Bitboard, BCoord, Player};
use super::PieceId;


/// External representation of a piece

#[derive(Clone, Debug)]
pub struct PieceDefinition {
    pub id: PieceId,
    pub char_rep: char,
    pub available_for: Vec<Player>,
    
    pub is_leader: bool,
    pub can_castle: bool,
    pub is_castle_rook: bool,
    pub explodes: bool,
    pub immune_to_explosion: bool,
    
    // Places where this piece can promote, as well as PieceId for the promotion pieces
    pub promotion_squares: Vec<(BCoord, BCoord)>,
    pub promo_vals: Vec<PieceId>,
    
    // Places where this piece can double move
    pub double_jump_squares: Vec<(BCoord, BCoord)>,

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
    pub fn promotion_squares_bb(&self) -> Bitboard {
        Bitboard::from_coord_list(&self.promotion_squares)
    }
    pub fn double_jump_squares_bb(&self) -> Bitboard {
        Bitboard::from_coord_list(&self.double_jump_squares)
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
        let has_promotion_squares = !self.promotion_squares.is_empty();
        let has_promo_vals = !self.promo_vals.is_empty();
        assert!(has_promotion_squares == has_promo_vals);
        has_promotion_squares
    }
    pub fn can_jump(&self) -> bool {
        !self.translate_jump_deltas.is_empty() || !self.attack_jump_deltas.is_empty()
    }
    pub fn can_double_jump(&self) -> bool {
        !self.double_jump_squares.is_empty()
    }
    pub fn has_sliding_deltas(&self) -> bool {
        !self.translate_sliding_deltas.is_empty() || !self.attack_sliding_deltas.is_empty()
    }
    
    pub fn update_inverse_attack(&mut self, other: &PieceDefinition) {
        self.attack_north |= other.attack_south;
        self.attack_south |= other.attack_north;
        self.attack_east |= other.attack_west;
        self.attack_west |= other.attack_east;
        self.attack_northeast |= other.attack_southwest;
        self.attack_northwest |= other.attack_southeast;
        self.attack_southeast |= other.attack_northwest;
        self.attack_southwest |= other.attack_northeast;
        
        for delta in &other.attack_jump_deltas {
            self.attack_jump_deltas.push((-delta.0, -delta.1));
        }
        
        for delta in &other.attack_sliding_deltas {
            let mut new_delta = Vec::new();
            for (x, y) in delta {
                new_delta.push((-x, -y));
            }
            self.attack_sliding_deltas.push(new_delta);
        }
    }
}

impl Default for PieceDefinition {
    fn default() -> Self {
        PieceDefinition {
            id: 0,
            char_rep: '?',
            available_for: vec![],
            is_leader: false,
            can_castle: false,
            is_castle_rook: false,
            explodes: false,
            immune_to_explosion: false,
            promotion_squares: vec![],
            promo_vals: Vec::new(),
            double_jump_squares: vec![],
            attack_sliding_deltas: Vec::new(),
            attack_jump_deltas: Vec::new(),
            attack_north: false,
            attack_south: false,
            attack_east: false,
            attack_west: false,
            attack_northeast: false,
            attack_northwest: false,
            attack_southeast: false,
            attack_southwest: false,
            translate_jump_deltas: Vec::new(),
            translate_sliding_deltas: Vec::new(),
            translate_north: false,
            translate_south: false,
            translate_east: false,
            translate_west: false,
            translate_northeast: false,
            translate_northwest: false,
            translate_southeast: false,
            translate_southwest: false,
        }
    }
}
