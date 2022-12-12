// Convert between x-y coordinates and bitboard indices

use std::collections::VecDeque;

use crate::types::{BCoord, BIndex, Bitboard};

pub fn to_index(x: BCoord, y: BCoord) -> BIndex{
    (16 * y + x) as BIndex
}

pub fn from_index(index: BIndex) -> (BCoord, BCoord) {
    ((index % 16) as BCoord , (index / 16) as BCoord)
}

// DFS to find distance to nearest 1
pub fn distance_to_one(x_start: BCoord, y_start: BCoord, board: &Bitboard) -> u8 {
    let mut visited = Bitboard::zero();
    let mut queue = VecDeque::new();
    queue.push_back((x_start, y_start, 0));
    while let Some((x, y, dist)) = queue.pop_front() {
        if visited.get_bit_at(x, y) {
            continue;
        }
        visited.set_bit_at(x, y);
        if board.get_bit_at(x, y) {
            return dist;
        }
        if x > 0 {
            queue.push_back((x - 1, y, dist + 1));
        }
        if x < 15 {
            queue.push_back((x + 1, y, dist + 1));
        }
        if y > 0 {
            queue.push_back((x, y - 1, dist + 1));
        }
        if y < 15 {
            queue.push_back((x, y + 1, dist + 1));
        }
    }
    u8::MAX
}



//TODO
/*
fn to_xy(rank_file:String) -> (u8, u8) {
    let file = rank_file.chars()[0];
    let rank = rank_file.chars().skip(0).take(rank_file.len()).collect();
    ((file.to_digit(10) - 65).unwrap(), rank.parse::<u8>().unwrap() - 1)
}
*/

/// Converts an (x, y) location to chess rank-file notation
/// Ex: to_rank_file(0, 1) = a2
pub fn to_rank_file(x: BCoord, y: BCoord) -> String {
    let mut return_string = String::new();
    let ascii_a = b'a';
    return_string.push((ascii_a + x) as char);
    return_string.push_str(format!("{}", (y + 1)).as_ref());
    return_string
}

pub fn to_long_algebraic_notation(x1: BCoord, y1: BCoord, x2: BCoord, y2: BCoord, mut piece: char, promotion: Option<char>) -> String {
    if let Some(prom) = promotion {
        return format!("{}{}={} ", to_rank_file(x1, y1), to_rank_file(x2, y2), prom.to_ascii_uppercase());
    }
    
    let mut result = format!("{}{} ", to_rank_file(x1, y1), to_rank_file(x2, y2));
    // If the piece is not a pawn, we write the piece letter
    piece = piece.to_ascii_uppercase();
    if piece != 'P' {
        result = format!("{}{}", piece, result);
    }
    if result == "Ke1g1 " {
        result = "O-O ".to_string();
    } else if result == "Ke1c1 " {
        result = "O-O-O ".to_string();
    } else if result == "Ke8g8 " {
        result = "O-O ".to_string();
    } else if result == "Ke8c8 " {
        result = "O-O-O ".to_string();
    }
    result
}
