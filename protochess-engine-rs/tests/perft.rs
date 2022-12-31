extern crate protochess_engine_rs;



#[cfg(test)]
mod perft {
    
    // https://www.chessprogramming.org/Perft_Results
    
    #[test]
    fn starting_pos() {
        use protochess_engine_rs::Engine;
        let mut engine = Engine::default();
        assert_eq!(engine.perft(1), 20);
        assert_eq!(engine.perft(2), 400);
        assert_eq!(engine.perft(3), 8902);
        assert_eq!(engine.perft(4), 197281);
        assert_eq!(engine.perft(5), 4865609);
        assert_eq!(engine.perft(6), 119060324);
    }

    #[test]
    fn kiwipete() {
        use protochess_engine_rs::Engine;
        let mut engine = Engine::from_fen("r3k2r/p1ppqpb1/bn2pnp1/3PN3/1p2P3/2N2Q1p/PPPBBPPP/R3K2R w KQkq - 0 1");
        assert_eq!(engine.perft(1), 48);
        assert_eq!(engine.perft(2), 2039);
        assert_eq!(engine.perft(3), 97862);
        assert_eq!(engine.perft(4), 4085603);
        assert_eq!(engine.perft(5), 193690690);
    }

    #[test]
    fn pos3() {
        use protochess_engine_rs::Engine;
        let mut engine = Engine::from_fen("8/2p5/3p4/KP5r/1R3p1k/8/4P1P1/8 w - - 0 1");
        assert_eq!(engine.perft(1), 14);
        assert_eq!(engine.perft(2), 191);
        assert_eq!(engine.perft(3), 2812);
        assert_eq!(engine.perft(4), 43238);
        assert_eq!(engine.perft(5), 674624);
        assert_eq!(engine.perft(6), 11030083);
    }

    #[test]
    fn pos4() {
        use protochess_engine_rs::Engine;
        let mut engine = Engine::from_fen("r3k2r/Pppp1ppp/1b3nbN/nP6/BBP1P3/q4N2/Pp1P2PP/R2Q1RK1 w kq - 0 1");
        assert_eq!(engine.perft(1), 6);
        assert_eq!(engine.perft(2), 264);
        assert_eq!(engine.perft(3), 9467);
        assert_eq!(engine.perft(4), 422333);
        assert_eq!(engine.perft(5), 15833292);
    }

    #[test]
    fn pos5() {
        use protochess_engine_rs::Engine;
        let mut engine = Engine::from_fen("rnbq1k1r/pp1Pbppp/2p5/8/2B5/8/PPP1NnPP/RNBQK2R w KQ - 1 8  ");
        assert_eq!(engine.perft(1), 44);
        assert_eq!(engine.perft(2), 1486);
        assert_eq!(engine.perft(3), 62379);
        assert_eq!(engine.perft(4), 2103487);
        assert_eq!(engine.perft(5), 89941194);
    }

    #[test]
    fn pos6() {
        use protochess_engine_rs::Engine;
        let mut engine = Engine::from_fen("r4rk1/1pp1qppp/p1np1n2/2b1p1B1/2B1P1b1/P1NP1N2/1PP1QPPP/R4RK1 w - - 0 10 ");
        assert_eq!(engine.perft(1), 46);
        assert_eq!(engine.perft(2), 2079);
        assert_eq!(engine.perft(3), 89890);
        assert_eq!(engine.perft(4), 3894594);
        assert_eq!(engine.perft(5), 164075551);
    }
    
    
    // https://github.com/niklasf/python-chess/blob/master/examples/perft/tricky.perft
    
    #[test]
    fn gotta_love_perft_1() {
        use protochess_engine_rs::Engine;
        let mut engine = Engine::from_fen("8/ppp3p1/8/8/3p4/5Q2/1ppp2K1/brk4n w - - 0 1");
        assert_eq!(engine.perft(1), 27);
        assert_eq!(engine.perft(2), 390);
        assert_eq!(engine.perft(3), 9354);
        assert_eq!(engine.perft(4), 134167);
    }
    
    #[test]
    fn gotta_love_perft_2() {
        use protochess_engine_rs::Engine;
        let mut engine = Engine::from_fen("8/6kR/8/8/8/bq6/1rqqqqqq/K1nqnbrq b - - 0 1");
        assert_eq!(engine.perft(1), 7);
        assert_eq!(engine.perft(2), 52);
        assert_eq!(engine.perft(3), 4593);
        assert_eq!(engine.perft(4), 50268);
    }
    
    
    // https://github.com/niklasf/python-chess/blob/master/examples/perft/atomic.perft
    
    #[test]
    fn atomic_start() {
        use protochess_engine_rs::Engine;
        let mut engine = Engine::from_fen("rnbqkbnr/pppppppp/8/8/8/8/PPPPPPPP/RNBQKBNR w KQkq - 0 1 ATOMIC");
        assert_eq!(engine.perft(1), 20);
        assert_eq!(engine.perft(2), 400);
        assert_eq!(engine.perft(3), 8902);
        assert_eq!(engine.perft(4), 197326);
        assert_eq!(engine.perft(5), 4864979);
        assert_eq!(engine.perft(6), 118926425);
    }
    
    #[test]
    fn atomic_programfox_1() {
        use protochess_engine_rs::Engine;
        let mut engine = Engine::from_fen("rn2kb1r/1pp1p2p/p2q1pp1/3P4/2P3b1/4PN2/PP3PPP/R2QKB1R b KQkq - 0 1 ATOMIC");
        assert_eq!(engine.perft(1), 40);
        assert_eq!(engine.perft(2), 1238);
        assert_eq!(engine.perft(3), 45237);
        assert_eq!(engine.perft(4), 1434825);
    }
    
    #[test]
    fn atomic_programfox_2() {
        use protochess_engine_rs::Engine;
        let mut engine = Engine::from_fen("rn1qkb1r/p5pp/2p5/3p4/N3P3/5P2/PPP4P/R1BQK3 w Qkq - 0 1 ATOMIC");
        assert_eq!(engine.perft(1), 28);
        assert_eq!(engine.perft(2), 833);
        assert_eq!(engine.perft(3), 23353);
        assert_eq!(engine.perft(4), 714499);
    }
    
    #[test]
    fn shakmaty_bench() {
        use protochess_engine_rs::Engine;
        let mut engine = Engine::from_fen("rn2kb1r/1pp1p2p/p2q1pp1/3P4/2P3b1/4PN2/PP3PPP/R2QKB1R b KQkq - 0 1 ATOMIC");
        assert_eq!(engine.perft(1), 40);
        assert_eq!(engine.perft(2), 1238);
        assert_eq!(engine.perft(3), 45237);
        assert_eq!(engine.perft(4), 1434825);
    }

}
