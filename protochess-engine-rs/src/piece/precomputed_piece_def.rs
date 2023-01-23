use crate::types::{Bitboard, BDimensions, BCoord};
use crate::PieceDefinition;
use crate::utils::from_index;


#[derive(Debug, Clone)]
pub struct PrecomputedPieceDef {
    // Positions at which this piece can promote
    pub promotion_squares: Bitboard,
    // Positions at which this piece can double jump
    pub double_jump_squares: Bitboard,
    // Positions at which the game is won instantly
    pub instant_win_squares: Bitboard,
    
    // Jump bitboards for this piece
    pub jump_bitboards_translate: Vec<Bitboard>,
    pub jump_bitboards_capture: Vec<Bitboard>,
    
    // Explosion bitboards for this piece
    pub explosion_bitboards: Vec<Bitboard>,
}

impl From<(&PieceDefinition, &BDimensions)> for PrecomputedPieceDef {
    fn from((definition, dims): (&PieceDefinition, &BDimensions)) -> Self {
        PrecomputedPieceDef { 
            promotion_squares: Bitboard::from_coord_list(&definition.promotion_squares) & &dims.bounds,
            double_jump_squares: Bitboard::from_coord_list(&definition.double_jump_squares) & &dims.bounds,
            instant_win_squares: Bitboard::from_coord_list(&definition.win_squares) & &dims.bounds,
            jump_bitboards_translate: Self::precompute_jumps(&definition.translate_jump_deltas, dims),
            jump_bitboards_capture: Self::precompute_jumps(&definition.attack_jump_deltas, dims),
            explosion_bitboards: Self::precompute_jumps(&definition.explosion_deltas, dims),
        }
    }
}

impl PrecomputedPieceDef {
    fn precompute_jumps(deltas: &Vec<(i8, i8)>, dims: &BDimensions) -> Vec<Bitboard> {
        let mut jumps = Vec::with_capacity(256);
        for index in 0..=255 {
            let mut jump = Bitboard::zero();
            let (x, y) = from_index(index);
            for (dx, dy) in deltas {
                let (x2, y2) = (x as i8 + *dx, y as i8 + *dy);
                if x2 < 0 || y2 < 0 {
                    continue;
                }
                if dims.in_bounds(x2 as BCoord, y2 as BCoord) {
                    jump.set_bit_at(x2 as BCoord, y2 as BCoord);
                }
            }
            jumps.push(jump);
        }
        jumps
    }
}
