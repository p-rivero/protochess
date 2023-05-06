use std::convert::TryFrom;

use regex::Regex;
use scan_fmt::scan_fmt;

use crate::{PieceId, err_assert, wrap_res, err};
use crate::types::BCoord;
use crate::utils::from_index;
use crate::utils::notation::tuple_to_rank_file;

use super::Move;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[must_use]
pub struct MoveInfo {
    pub from: (BCoord, BCoord),
    pub to: (BCoord, BCoord),
    pub promotion: Option<PieceId>,
}

impl From<Move> for MoveInfo {
    fn from(m: Move) -> Self {
        let from = from_index(m.get_from());
        let to = {
            if m.is_castling() {
                // Castling moves are stored as if the king moves to the rook's square
                from_index(m.get_target())
            } else {
                from_index(m.get_to())
            }
        };
        MoveInfo { from, to, promotion: m.get_promotion_piece() }
    }
}

// Create a MoveInfo from a string like "e2e4" or "e7e8=Q"
impl TryFrom<&str> for MoveInfo {
    type Error = String;
    fn try_from(s: &str) -> wrap_res!(Self) {
        const EXPECTED_REGEX: &str = r"^[a-p][0-9]+[a-p][0-9]+(=.)?$";
        let s = s.trim();
        err_assert!(Regex::new(EXPECTED_REGEX).unwrap().is_match(s), "Invalid move format: '{s}' (expected 'e2e4', 'e7e8=Q')");
        let (from_x, from_y, to_x, to_y) = match scan_fmt!(s, "{[a-p]}{d}{[a-p]}{d}", char, isize, char, isize) {
            Ok(parts) => parts,
            Err(_) => err!("Invalid move format: '{s}'"),
        };
        let promotion = match scan_fmt!(s, "{*[a-p]}{*d}{*[a-p]}{*d}={}", PieceId) {
            Ok(promo) => Some(promo),
            Err(_) => None,
        };
        // from_x, to_x are guaranteed to be between 'a' and 'p' (inclusive)
        let from_x = from_x.to_digit(36).unwrap() as BCoord - 10;
        let to_x = to_x.to_digit(36).unwrap() as BCoord - 10;
        // Ranks are 1-indexed
        err_assert!(from_y > 0 && to_y > 0 && from_y <= 16 && to_y <= 16,
            "Invalid move format (rank must be between 1 and 16");
        Ok(MoveInfo {
            from: (from_x, from_y as BCoord - 1),
            to: (to_x, to_y as BCoord - 1),
            promotion,
        })
    }
}

impl PartialEq<Move> for MoveInfo {
    fn eq(&self, other: &Move) -> bool {
        self == &MoveInfo::from(*other)
    }
}

/// Outputs the long algebraic notation for the move (without the piece letter in front, 
/// or check/checkmate indicators).
impl std::fmt::Display for MoveInfo {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}{}", tuple_to_rank_file(self.from), tuple_to_rank_file(self.to))?;
        if let Some(prom) = self.promotion {
            write!(f, "={prom}")?;
        }
        Ok(())
    }
}
