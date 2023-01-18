// Convert between x-y coordinates and bitboard indices

use std::collections::VecDeque;

use crate::types::{BCoord, BIndex, Bitboard};

pub fn to_index(x: BCoord, y: BCoord) -> BIndex{
    (16 * y + x) as BIndex
}

pub fn from_index(index: BIndex) -> (BCoord, BCoord) {
    ((index % 16) as BCoord , (index / 16) as BCoord)
}

// BFS to find distance to nearest 1, using a callback function to get neighbors
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
    format!("{}{}", (b'a' + x) as char, (y + 1))
}

