use crate::types::{Bitboard, BIndex, Move, MoveType};
use crate::piece::PieceId;

/// Iterator that converts a Bitboard of move possibilities to Moves
pub struct BitboardMoves {
    pub(crate) enemies: Bitboard,    // Enemies
    pub(crate) moves: Bitboard,      // moveset for source piece
    pub(crate) source_index: BIndex, // Source piece index
    pub(crate) promotion_squares: Bitboard,   // Promotion squares for this piece
    pub(crate) promo_vals: Vec<PieceId>,      // Promotable values for this piece
    current_promo_vals: Option<Vec<PieceId>>, // Internal; used as a copy of promovals for each sq
}

impl BitboardMoves {
    pub fn new(enemies: Bitboard,
               moves: Bitboard,
               source_index: BIndex,
               promotion_squares: Bitboard,
               promo_vals: Vec<PieceId>) -> BitboardMoves{
        BitboardMoves{
            enemies,
            moves,
            source_index,
            promotion_squares,
            promo_vals,
            current_promo_vals: None,
        }
    }
}

impl Iterator for BitboardMoves {
    type Item = Move;

    fn next(&mut self) -> Option<Self::Item> {
        if let Some(to) = self.moves.lowest_one() {
            let promo_here = self.promotion_squares.get_bit(to);
            let capture_here = { self.enemies.get_bit(to) };
            let move_type = {
                match (capture_here, promo_here) {
                    (true, true) => { MoveType::PromotionCapture },
                    (true, false) => { MoveType::Capture },
                    (false, true) => { MoveType::Promotion },
                    (false, false) => { MoveType::Quiet },
                }
            };
            let target = {if capture_here {to} else {0}};
            let promotion = {
                if promo_here {
                    //promotion, do not go next until we run out of promo options
                    let promo_options = {
                        if let Some(pv) = &mut self.current_promo_vals {
                            pv
                        } else {
                            self.current_promo_vals = Some(self.promo_vals.clone());
                            self.current_promo_vals.as_mut().unwrap()
                        }
                    };

                    //Unwrap intentionally here; want to panic if this goes wrong
                    let next_promo = promo_options.pop().unwrap();
                    //If we run out of promo options, we can go to the next square
                    if promo_options.is_empty() {
                        //Reset current_promo_vals for the next time
                        self.current_promo_vals = None;
                        self.moves.clear_bit(to);
                    }
                    Some(next_promo)
                } else {
                    //No promotion chars left, go to next after this
                    self.moves.clear_bit(to);
                    None
                }
            };
            Some(Move::new(self.source_index, to, Some(target), move_type, promotion))
        } else {
            None
        }
    }
}


