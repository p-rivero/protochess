use crate::types::BCoord;
use crate::utils::debug::eq_anyorder;
use super::PieceId;


/// External representation of a piece

#[derive(Clone, Debug, Hash, PartialEq, Eq, Default)]
#[must_use]
pub struct PieceDefinition {
    // The id of this piece for white and black. None if this piece is not available for that color.
    pub ids: [Option<PieceId>; 2],
    
    pub is_leader: bool,
    // Either None (no castle) or (queenside, kingside) (files where this piece moves when castling)
    pub castle_files: Option<(BCoord, BCoord)>,
    // True if this piece works as a rook for castling purposes
    pub is_castle_rook: bool,
    pub explodes: bool,
    pub explosion_deltas: Vec<(i8, i8)>,
    pub immune_to_explosion: bool,
    
    // Places where this piece can promote, as well as PieceId for the promotion pieces on each side
    pub promotion_squares: Vec<(BCoord, BCoord)>,
    pub promo_vals: [Vec<PieceId>; 2],
    
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
    
    // Used for displaying the piece in the UI
    pub display_name: String,
    pub image_urls: [Option<String>; 2],
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
        !self.promotion_squares.is_empty()
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
    
    pub fn eq_ignore_order(&self, other: &PieceDefinition) -> bool {
        self.ids == other.ids &&
        self.is_leader == other.is_leader &&
        self.castle_files == other.castle_files &&
        self.is_castle_rook == other.is_castle_rook &&
        self.explodes == other.explodes &&
        eq_anyorder(&self.explosion_deltas, &other.explosion_deltas) &&
        self.immune_to_explosion == other.immune_to_explosion &&
        eq_anyorder(&self.promotion_squares, &other.promotion_squares) &&
        eq_anyorder(&self.promo_vals, &other.promo_vals) &&
        eq_anyorder(&self.double_jump_squares, &other.double_jump_squares) &&
        eq_anyorder(&self.attack_sliding_deltas, &other.attack_sliding_deltas) &&
        eq_anyorder(&self.attack_jump_deltas, &other.attack_jump_deltas) &&
        self.attack_north == other.attack_north &&
        self.attack_south == other.attack_south &&
        self.attack_east == other.attack_east &&
        self.attack_west == other.attack_west &&
        self.attack_northeast == other.attack_northeast &&
        self.attack_northwest == other.attack_northwest &&
        self.attack_southeast == other.attack_southeast &&
        self.attack_southwest == other.attack_southwest &&
        eq_anyorder(&self.translate_jump_deltas, &other.translate_jump_deltas) &&
        eq_anyorder(&self.translate_sliding_deltas, &other.translate_sliding_deltas) &&
        self.translate_north == other.translate_north &&
        self.translate_south == other.translate_south &&
        self.translate_east == other.translate_east &&
        self.translate_west == other.translate_west &&
        self.translate_northeast == other.translate_northeast &&
        self.translate_northwest == other.translate_northwest &&
        self.translate_southeast == other.translate_southeast &&
        self.translate_southwest == other.translate_southwest &&
        eq_anyorder(&self.win_squares, &other.win_squares)
    }
}
