use crate::{MoveGen, Position};
use crate::types::Depth;

use super::{from_index, to_rank_file};

/// Returns the number of possible moves from a board position up to a given depth
/// See <https://www.chessprogramming.org/Perft>
pub fn perft(position: &mut Position, depth: Depth) -> u64 {
    let mut nodes = 0;

    if depth == 1 {
        return MoveGen::count_legal_moves(position);
    }
    for mv in MoveGen::get_pseudo_moves(position, true) {
        if !MoveGen::make_move_only_if_legal(mv, position) {
            continue;
        }
        nodes += perft(position, depth - 1);
        position.unmake_move();
    }
    nodes
}

/// Like perft, but prints the moves at the first ply
pub fn perft_divide(position: &mut Position, depth: Depth) -> u64 {
    let mut nodes = 0;

    let mut printing = Vec::new();
    for mv in MoveGen::get_pseudo_moves(position, true) {
        if !MoveGen::is_move_legal(mv, position) {
            continue;
        }

        let (x,y) = from_index(mv.get_from());
        let (x2,y2) = from_index(mv.get_to());
        let promo = mv.get_promotion_piece().map(|p| position.search_piece_by_id(p));
        let promo_char = {
            if let Some(Some(p)) = promo { p.char_rep().to_string() }
            else { "".to_string() }
        };
        if depth == 1 {
            nodes += 1;
            printing.push(format!("{}{}{}: 1", to_rank_file(x,y), to_rank_file(x2,y2), promo_char));
        } else {
            position.make_move(mv);
            let plus = perft(position, depth - 1);
            nodes += plus;
            position.unmake_move();
            //Print nodes
            printing.push(format!("{}{}{}: {}", to_rank_file(x,y), to_rank_file(x2,y2), promo_char, plus));
        }
    }
    printing.sort();
    for s in printing {
        println!("{}",s);
    }
    nodes
}
  