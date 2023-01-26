#[cfg(test)]
mod principal_variation {
    use std::convert::TryFrom;

    use protochess_engine_rs::{Position, GameState, MoveGen};
    use protochess_engine_rs::searcher::Searcher;
    #[test]
    fn starting_position_1() {
        test_pv("rnbqkbnr/pppppppp/8/8/8/8/PPPPPPPP/RNBQKBNR w KQkq - 0 1", 1);
    }
    #[test]
    fn starting_position_2() {
        test_pv("rnbqkbnr/pppppppp/8/8/8/8/PPPPPPPP/RNBQKBNR w KQkq - 0 1", 2);
    }
    #[test]
    fn starting_position_3() {
        test_pv("rnbqkbnr/pppppppp/8/8/8/8/PPPPPPPP/RNBQKBNR w KQkq - 0 1", 3);
    }
    #[test]
    fn starting_position_4() {
        test_pv("rnbqkbnr/pppppppp/8/8/8/8/PPPPPPPP/RNBQKBNR w KQkq - 0 1", 4);
    }
    #[test]
    fn starting_position_5() {
        test_pv("rnbqkbnr/pppppppp/8/8/8/8/PPPPPPPP/RNBQKBNR w KQkq - 0 1", 5);
    }
    #[test]
    fn starting_position_6() {
        test_pv("rnbqkbnr/pppppppp/8/8/8/8/PPPPPPPP/RNBQKBNR w KQkq - 0 1", 6);
    }
    #[test]
    fn starting_position_7() {
        test_pv("rnbqkbnr/pppppppp/8/8/8/8/PPPPPPPP/RNBQKBNR w KQkq - 0 1", 7);
    }
    #[test]
    fn starting_position_8() {
        test_pv("rnbqkbnr/pppppppp/8/8/8/8/PPPPPPPP/RNBQKBNR w KQkq - 0 1", 8);
    }
    #[test]
    fn starting_position_9() {
        test_pv("rnbqkbnr/pppppppp/8/8/8/8/PPPPPPPP/RNBQKBNR w KQkq - 0 1", 9);
    }
    #[test]
    fn starting_position_10() {
        test_pv("rnbqkbnr/pppppppp/8/8/8/8/PPPPPPPP/RNBQKBNR w KQkq - 0 1", 10);
    }
    #[test]
    fn starting_position_11() {
        test_pv("rnbqkbnr/pppppppp/8/8/8/8/PPPPPPPP/RNBQKBNR w KQkq - 0 1", 11);
    }
    #[test]
    fn starting_position_12() {
        test_pv("rnbqkbnr/pppppppp/8/8/8/8/PPPPPPPP/RNBQKBNR w KQkq - 0 1", 11);
    }
    
    #[test]
    fn kiwipete_1() {
        test_pv("r3k2r/p1ppqpb1/bn2pnp1/3PN3/1p2P3/2N2Q1p/PPPBBPPP/R3K2R w KQkq - 0 1", 1);
    }
    #[test]
    fn kiwipete_2() {
        test_pv("r3k2r/p1ppqpb1/bn2pnp1/3PN3/1p2P3/2N2Q1p/PPPBBPPP/R3K2R w KQkq - 0 1", 2);
    }
    #[test]
    fn kiwipete_3() {
        test_pv("r3k2r/p1ppqpb1/bn2pnp1/3PN3/1p2P3/2N2Q1p/PPPBBPPP/R3K2R w KQkq - 0 1", 3);
    }
    #[test]
    fn kiwipete_4() {
        test_pv("r3k2r/p1ppqpb1/bn2pnp1/3PN3/1p2P3/2N2Q1p/PPPBBPPP/R3K2R w KQkq - 0 1", 4);
    }
    #[test]
    fn kiwipete_5() {
        test_pv("r3k2r/p1ppqpb1/bn2pnp1/3PN3/1p2P3/2N2Q1p/PPPBBPPP/R3K2R w KQkq - 0 1", 5);
    }
    #[test]
    fn kiwipete_6() {
        test_pv("r3k2r/p1ppqpb1/bn2pnp1/3PN3/1p2P3/2N2Q1p/PPPBBPPP/R3K2R w KQkq - 0 1", 6);
    }
    #[test]
    fn kiwipete_7() {
        test_pv("r3k2r/p1ppqpb1/bn2pnp1/3PN3/1p2P3/2N2Q1p/PPPBBPPP/R3K2R w KQkq - 0 1", 7);
    }
    #[test]
    fn kiwipete_8() {
        test_pv("r3k2r/p1ppqpb1/bn2pnp1/3PN3/1p2P3/2N2Q1p/PPPBBPPP/R3K2R w KQkq - 0 1", 8);
    }
    #[test]
    fn kiwipete_9() {
        test_pv("r3k2r/p1ppqpb1/bn2pnp1/3PN3/1p2P3/2N2Q1p/PPPBBPPP/R3K2R w KQkq - 0 1", 9);
    }
    #[test]
    fn kiwipete_10() {
        test_pv("r3k2r/p1ppqpb1/bn2pnp1/3PN3/1p2P3/2N2Q1p/PPPBBPPP/R3K2R w KQkq - 0 1", 10);
    }
    
    
    fn test_pv(fen: &str, depth: u8) {
        let gs = GameState::from_fen(fen).unwrap();
        let mut pos = Position::try_from(gs).unwrap();
        let (pv, _score, search_depth) = Searcher::get_best_move(&pos, depth).unwrap();
        assert!(search_depth == depth);
        // Make sure that the moves in the PV legal
        for m in pv {
            assert!(MoveGen::get_legal_moves(&mut pos).contains(&m), "Move {} is not legal", m);
            pos.make_move(m);
        }
    }
}
