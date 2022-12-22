// Convert between x-y coordinates and bitboard indices

use std::collections::VecDeque;

use crate::types::{BCoord, BIndex, Bitboard};

pub fn to_index(x: BCoord, y: BCoord) -> BIndex{
    (16 * y + x) as BIndex
}

pub fn from_index(index: BIndex) -> (BCoord, BCoord) {
    ((index % 16) as BCoord , (index / 16) as BCoord)
}

// DFS to find distance to nearest 1, using a callback function to get neighbors
pub fn distance_to_one<F>(x_start: BCoord, y_start: BCoord, board: &Bitboard, get_neighbors: F) -> isize 
    where F: Fn(BCoord, BCoord) -> Vec<(BCoord, BCoord)>
{
    let mut visited = Bitboard::zero();
    let mut queue = VecDeque::new();
    queue.push_back((x_start, y_start, 0));
    while !queue.is_empty() {
        let (x, y, dist) = queue.pop_front().unwrap();
        if x >= 16 || y >= 16 {
            continue;
        }
        if board.get_bit_at(x, y) {
            return dist as isize;
        }
        visited.set_bit_at(x, y);
        for (x2, y2) in get_neighbors(x, y) {
            if !visited.get_bit_at(x2, y2) {
                queue.push_back((x2, y2, dist + 1));
            }
        }
    }
    isize::MAX
}



/// Converts an (x, y) location to chess rank-file notation
/// Ex: to_rank_file(0, 1) = a2
pub fn to_rank_file(x: BCoord, y: BCoord) -> String {
    let mut return_string = String::new();
    let ascii_a = b'a';
    return_string.push((ascii_a + x) as char);
    return_string.push_str(format!("{}", (y + 1)).as_ref());
    return_string
}

pub fn to_long_algebraic_notation(from: (BCoord,BCoord), to: (BCoord,BCoord), mut piece: char, promotion: Option<char>) -> String {
    if let Some(prom) = promotion {
        return format!("{}{}={} ", to_rank_file(from.0, from.1), to_rank_file(to.0, to.1), prom.to_ascii_uppercase());
    }
    
    let mut result = format!("{}{} ", to_rank_file(from.0, from.1), to_rank_file(to.0, to.1));
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
