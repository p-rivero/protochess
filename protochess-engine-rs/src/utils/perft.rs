use crate::State;
use crate::types::Depth;

use super::{from_index, to_rank_file};

/// Returns the number of possible moves from a board position up to a given depth
/// See https://www.chessprogramming.org/Perft
pub fn perft(state: &mut State, depth: Depth) -> u64 {
    let mut nodes = 0u64;

    let moves = state.movegen.get_pseudo_moves(&mut state.position);

    if depth == 1 {
        return state.movegen.count_legal_moves(&mut state.position);
    }
    for mv in moves{
        if !state.movegen.is_move_legal(&mv, &mut state.position) {
            continue;
        }
        state.position.make_move(mv);
        nodes += perft(state, depth - 1);
        state.position.unmake_move();
    }
    nodes
}

/// Like perft, but prints the moves at the first ply
pub fn perft_divide(state: &mut State, depth: Depth) -> u64 {
    let mut nodes = 0u64;

    let moves = state.movegen.get_pseudo_moves(&mut state.position);
    if depth == 1 {
        return state.movegen.count_legal_moves(&mut state.position);
    }
    let mut printing = Vec::new();
    for mv in moves{
        if !state.movegen.is_move_legal(&mv, &mut state.position) {
            continue;
        }

        let (x,y) = from_index(mv.get_from());
        let (x2,y2) = from_index(mv.get_to());
        state.position.make_move(mv);
        let plus = perft(state, depth - 1);
        nodes += plus;
        state.position.unmake_move();
        //Print nodes
        printing.push(format!("{}{}: {}", to_rank_file(x,y), to_rank_file(x2,y2), plus));
    }
    printing.sort();
    for s in printing {
        println!("{}",s);
    }
    nodes
}
  