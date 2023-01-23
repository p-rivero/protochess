use crate::types::{BCoord, Player};
use super::PieceId;


/// External representation of a piece

#[derive(Clone, Debug, Hash, PartialEq, Eq)]
pub struct PieceDefinition {
    pub id: PieceId,
    pub char_rep: char,
    pub available_for: Vec<Player>,
    
    pub is_leader: bool,
    // Either None (no castle) or (queenside, kingside) (files where this piece moves when castling)
    pub castle_files: Option<(BCoord, BCoord)>,
    // True if this piece works as a rook for castling purposes
    pub is_castle_rook: bool,
    pub explodes: bool,
    pub explosion_deltas: Vec<(i8, i8)>,
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
    
    // Successfully moving the piece to a win square is an instant win
    pub win_squares: Vec<(BCoord, BCoord)>,
}

impl PieceDefinition {
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
    pub fn can_castle(&self) -> bool {
        self.castle_files.is_some()
    }
}

impl Default for PieceDefinition {
    fn default() -> Self {
        PieceDefinition {
            id: 0,
            char_rep: '?',
            available_for: Vec::new(),
            is_leader: false,
            castle_files: None,
            is_castle_rook: false,
            explodes: false,
            explosion_deltas: Vec::new(),
            immune_to_explosion: false,
            promotion_squares: Vec::new(),
            promo_vals: Vec::new(),
            double_jump_squares: Vec::new(),
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
            win_squares: Vec::new(),
        }
    }
}
