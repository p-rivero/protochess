#[cfg(test)]
mod principal_variation {
    use protochess_engine_rs::{Position, GameState, MoveGen};
    use protochess_engine_rs::searcher::Searcher;
    #[test]
    fn starting_position() {
        // TODO: This test fails at depth 8
        test_pv("rnbqkbnr/pppppppp/8/8/8/8/PPPPPPPP/RNBQKBNR w KQkq - 0 1", 7);
    }
    
    fn test_pv(fen: &str, depth: u8) {
        let mut pos = Position::from(GameState::from_fen(fen));
        let (pv, _score, search_depth) = Searcher::get_best_move(&pos, depth);
        assert!(search_depth == depth);
        assert!(pv.len() == depth as usize);
        // Make sure that the moves in the PV are the best moves (this also checks that they are legal)
        assert!(MoveGen::make_move_only_if_legal(pv[0], &mut pos), "Move {} not legal in position:\n{}", pv[0], pos);
        for i in 1..depth {
            let (new_pv, _, _) = Searcher::get_best_move(&pos, depth - i);
            let best_move = new_pv[0];
            assert!(best_move == pv[i as usize]);
            assert!(MoveGen::make_move_only_if_legal(best_move, &mut pos), "Move {} not legal in position:\n{}", best_move, pos);
        }
    }
}
