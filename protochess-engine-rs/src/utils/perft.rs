use crate::{MoveGen, Position};
use crate::types::Depth;

use super::{from_index, to_rank_file};

/// Returns the number of possible moves from a board position up to a given depth
/// See https://www.chessprogramming.org/Perft
pub fn perft(position: &mut Position, depth: Depth) -> u64 {
    let mut nodes = 0u64;

    if depth == 1 {
        return MoveGen::count_legal_moves(position);
    }
    for mv in MoveGen::get_pseudo_moves(position) {
        if !MoveGen::is_move_legal(&mv, position) {
            continue;
        }
        position.make_move(mv);
        nodes += perft(position, depth - 1);
        position.unmake_move();
    }
    nodes
}

/// Like perft, but prints the moves at the first ply
pub fn perft_divide(position: &mut Position, depth: Depth) -> u64 {
    let mut nodes = 0u64;

    if depth == 1 {
        return MoveGen::count_legal_moves(position);
    }
    let mut printing = Vec::new();
    for mv in MoveGen::get_pseudo_moves(position) {
        if !MoveGen::is_move_legal(&mv, position) {
            continue;
        }

        let (x,y) = from_index(mv.get_from());
        let (x2,y2) = from_index(mv.get_to());
        position.make_move(mv);
        let plus = perft(position, depth - 1);
        nodes += plus;
        position.unmake_move();
        //Print nodes
        printing.push(format!("{}{}: {}", to_rank_file(x,y), to_rank_file(x2,y2), plus));
    }
    printing.sort();
    for s in printing {
        println!("{}",s);
    }
    nodes
}
  