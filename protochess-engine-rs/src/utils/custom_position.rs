use std::collections::HashMap;

use crate::{types::{Player, BCoord, Bitboard, BDimensions}, PieceType};
use super::to_index;
use crate::{Position, MovementPatternExternal};


pub fn make_custom_position(movement_patterns: HashMap<char, MovementPatternExternal>,
    valid_squares: &Vec<(BCoord, BCoord)>, pieces: &Vec<(Player, BCoord, BCoord, char)>) -> Position
{
    assert!(each_owner_contains_k(&pieces));
    let mut width = 0;
    let mut height = 0;
    let mut bounds = Bitboard::zero();
    for sq in valid_squares {
        if sq.0 >= width { width = sq.0 + 1; }
        if sq.1 >= height { height = sq.1 + 1; }
        bounds.set_bit_at(sq.0, sq.1);
    }

    let pieces = pieces.into_iter()
        .map(|(owner, x, y, pce_chr)| (*owner, to_index(*x, *y), PieceType::from_char(*pce_chr)))
        .collect();
    Position::custom(BDimensions{width, height}, bounds, movement_patterns, pieces)
}

fn each_owner_contains_k(vec: &Vec<(Player, BCoord, BCoord, char)>) -> bool {
    let mut num_players = 0;
    for (owner, _, _, _) in vec {
        if *owner >= num_players {
            num_players = owner + 1;
        }
    }
    let mut has_k = vec![false; num_players as usize];
    for (owner, _, _, pce_char) in vec {
        if pce_char.to_ascii_lowercase() == 'k' {
            has_k[*owner as usize] = true;
        }
    }
    for k in has_k {
        if !k { return false; }
    }
    true
}