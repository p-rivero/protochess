extern crate protochess_engine_rs;



#[cfg(test)]
mod perft {
    use protochess_engine_rs::Engine;
    // https://www.chessprogramming.org/Perft_Results
    
    #[test]
    fn starting_pos() {
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
        let mut engine = Engine::from_fen("r3k2r/p1ppqpb1/bn2pnp1/3PN3/1p2P3/2N2Q1p/PPPBBPPP/R3K2R w KQkq - 0 1");
        assert_eq!(engine.perft(1), 48);
        assert_eq!(engine.perft(2), 2039);
        assert_eq!(engine.perft(3), 97862);
        assert_eq!(engine.perft(4), 4085603);
        assert_eq!(engine.perft(5), 193690690);
    }

    #[test]
    fn pos3() {
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
        let mut engine = Engine::from_fen("r3k2r/Pppp1ppp/1b3nbN/nP6/BBP1P3/q4N2/Pp1P2PP/R2Q1RK1 w kq - 0 1");
        assert_eq!(engine.perft(1), 6);
        assert_eq!(engine.perft(2), 264);
        assert_eq!(engine.perft(3), 9467);
        assert_eq!(engine.perft(4), 422333);
        assert_eq!(engine.perft(5), 15833292);
    }

    #[test]
    fn pos5() {
        let mut engine = Engine::from_fen("rnbq1k1r/pp1Pbppp/2p5/8/2B5/8/PPP1NnPP/RNBQK2R w KQ - 1 8  ");
        assert_eq!(engine.perft(1), 44);
        assert_eq!(engine.perft(2), 1486);
        assert_eq!(engine.perft(3), 62379);
        assert_eq!(engine.perft(4), 2103487);
        assert_eq!(engine.perft(5), 89941194);
    }

    #[test]
    fn pos6() {
        let mut engine = Engine::from_fen("r4rk1/1pp1qppp/p1np1n2/2b1p1B1/2B1P1b1/P1NP1N2/1PP1QPPP/R4RK1 w - - 0 10 ");
        assert_eq!(engine.perft(1), 46);
        assert_eq!(engine.perft(2), 2079);
        assert_eq!(engine.perft(3), 89890);
        assert_eq!(engine.perft(4), 3894594);
        assert_eq!(engine.perft(5), 164075551);
    }
    
    
    // https://github.com/niklasf/python-chess/blob/master/examples/perft/tricky.perft
    
    #[test]
    fn xfen960_0() {
        let mut engine = Engine::from_fen("r1k1r2q/p1ppp1pp/8/8/8/8/P1PPP1PP/R1K1R2Q w KQkq - 0 1");
        assert_eq!(engine.perft(1), 23);
        assert_eq!(engine.perft(2), 522);
        assert_eq!(engine.perft(3), 12333);
        assert_eq!(engine.perft(4), 285754);
        assert_eq!(engine.perft(5), 7096972);
    }
    
    #[test]
    fn xfen960_1() {
        let mut engine = Engine::from_fen("r1k2r1q/p1ppp1pp/8/8/8/8/P1PPP1PP/R1K2R1Q w KQkq - 0 1");
        assert_eq!(engine.perft(1), 28);
        assert_eq!(engine.perft(2), 738);
        assert_eq!(engine.perft(3), 20218);
        assert_eq!(engine.perft(4), 541480);
        assert_eq!(engine.perft(5), 15194841);
    }
    
    #[test]
    fn xfen960_2() {
        let mut engine = Engine::from_fen("8/8/8/4B2b/6nN/8/5P2/2R1K2k w Q - 0 1");
        assert_eq!(engine.perft(1), 34);
        assert_eq!(engine.perft(2), 318);
        assert_eq!(engine.perft(3), 9002);
        assert_eq!(engine.perft(4), 118388);
        assert_eq!(engine.perft(5), 3223406);
    }
    
    #[test]
    fn xfen960_3() {
        let mut engine = Engine::from_fen("2r5/8/8/8/8/8/6PP/k2KR3 w K - 0 1");
        assert_eq!(engine.perft(1), 17);
        assert_eq!(engine.perft(2), 242);
        assert_eq!(engine.perft(3), 3931);
        assert_eq!(engine.perft(4), 57700);
        assert_eq!(engine.perft(5), 985298);
        assert_eq!(engine.perft(6), 14751778);
    }
    
    #[test]
    fn xfen960_4() {
        let mut engine = Engine::from_fen("4r3/3k4/8/8/8/8/6PP/qR1K1R2 w KQ - 0 1");
        assert_eq!(engine.perft(1), 19);
        assert_eq!(engine.perft(2), 628);
        assert_eq!(engine.perft(3), 12858);
        assert_eq!(engine.perft(4), 405636);
        assert_eq!(engine.perft(5), 8992652);
    }
    
    #[test]
    fn gotta_love_perft_1() {
        let mut engine = Engine::from_fen("8/ppp3p1/8/8/3p4/5Q2/1ppp2K1/brk4n w - - 0 1");
        assert_eq!(engine.perft(1), 27);
        assert_eq!(engine.perft(2), 390);
        assert_eq!(engine.perft(3), 9354);
        assert_eq!(engine.perft(4), 134167);
        assert_eq!(engine.perft(5), 2922659);
        assert_eq!(engine.perft(6), 42959630);
    }
    
    #[test]
    fn gotta_love_perft_2() {
        let mut engine = Engine::from_fen("8/6kR/8/8/8/bq6/1rqqqqqq/K1nqnbrq b - - 0 1");
        assert_eq!(engine.perft(1), 7);
        assert_eq!(engine.perft(2), 52);
        assert_eq!(engine.perft(3), 4593);
        assert_eq!(engine.perft(4), 50268);
        assert_eq!(engine.perft(5), 4634384);
    }
    
    #[test]
    fn chess960_swap_castling() {
        let mut engine = Engine::from_fen("2rkr3/8/8/8/8/1PP5/P4PP1/5KR1 b Kkq - 0 2");
        assert_eq!(engine.perft(1), 22);
        assert_eq!(engine.perft(2), 222);
        assert_eq!(engine.perft(3), 5182);
        assert_eq!(engine.perft(4), 60618);
        assert_eq!(engine.perft(5), 1499136);
        assert_eq!(engine.perft(6), 20508951);
    }
    
    
    // https://github.com/niklasf/python-chess/blob/master/examples/perft/atomic.perft
    
    #[test]
    fn atomic_start() {
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
        let mut engine = Engine::from_fen("rn2kb1r/1pp1p2p/p2q1pp1/3P4/2P3b1/4PN2/PP3PPP/R2QKB1R b KQkq - 0 1 ATOMIC");
        assert_eq!(engine.perft(1), 40);
        assert_eq!(engine.perft(2), 1238);
        assert_eq!(engine.perft(3), 45237);
        assert_eq!(engine.perft(4), 1434825);
        assert_eq!(engine.perft(5), 50504249);
    }
    
    #[test]
    fn atomic_programfox_2() {
        let mut engine = Engine::from_fen("rn1qkb1r/p5pp/2p5/3p4/N3P3/5P2/PPP4P/R1BQK3 w Qkq - 0 1 ATOMIC");
        assert_eq!(engine.perft(1), 28);
        assert_eq!(engine.perft(2), 833);
        assert_eq!(engine.perft(3), 23353);
        assert_eq!(engine.perft(4), 714499);
        assert_eq!(engine.perft(5), 21134061);
    }
    
    #[test]
    fn atomic_checks() {
        let mut engine = Engine::from_fen("7r/2N2k1p/p1n3pb/3p1p2/6Pq/1PP1P3/P4P1P/R1BK1B1R w - - 3 17 ATOMIC");
        assert_eq!(engine.perft(1), 32);
        assert_eq!(engine.perft(2), 1275);
        assert_eq!(engine.perft(3), 39093);
        assert_eq!(engine.perft(4), 1425274);
    }
    
    #[test]
    fn atomic_double_check() {
        let mut engine = Engine::from_fen("r3k1nr/ppp2ppp/8/8/P3q3/1n1b1P2/RPPb2PP/1NBQKBNR w Kkq - 1 18 ATOMIC");
        assert_eq!(engine.perft(1), 3);
        assert_eq!(engine.perft(2), 113);
        assert_eq!(engine.perft(3), 3011);
        assert_eq!(engine.perft(4), 110029);
        assert_eq!(engine.perft(5), 2972933);
    }
    
    #[test]
    fn atomic960_castle_1() {
        let mut engine = Engine::from_fen("8/8/8/8/8/8/2k5/rR4KR w KQ - 0 1 ATOMIC");
        assert_eq!(engine.perft(1), 18);
        assert_eq!(engine.perft(2), 180);
        assert_eq!(engine.perft(3), 4364);
        assert_eq!(engine.perft(4), 61401);
        assert_eq!(engine.perft(5), 1603055);
        assert_eq!(engine.perft(6), 23969896);
    }
    
    #[test]
    fn atomic960_castle_2() {
        let mut engine = Engine::from_fen("r3k1rR/5K2/8/8/8/8/8/8 b kq - 0 1 ATOMIC");
        assert_eq!(engine.perft(1), 25);
        assert_eq!(engine.perft(2), 282);
        assert_eq!(engine.perft(3), 6753);
        assert_eq!(engine.perft(4), 98729);
        assert_eq!(engine.perft(5), 2587730);
    }
    
    #[test]
    fn atomic960_castle_3() {
        let mut engine = Engine::from_fen("Rr2k1rR/3K4/3p4/8/8/8/7P/8 w kq - 0 1 ATOMIC");
        assert_eq!(engine.perft(1), 21);
        assert_eq!(engine.perft(2), 465);
        assert_eq!(engine.perft(3), 10631);
        assert_eq!(engine.perft(4), 241478);
        assert_eq!(engine.perft(5), 5800275);
    }
    
    #[test]
    fn shakmaty_bench() {
        let mut engine = Engine::from_fen("rn2kb1r/1pp1p2p/p2q1pp1/3P4/2P3b1/4PN2/PP3PPP/R2QKB1R b KQkq - 0 1 ATOMIC");
        assert_eq!(engine.perft(1), 40);
        assert_eq!(engine.perft(2), 1238);
        assert_eq!(engine.perft(3), 45237);
        assert_eq!(engine.perft(4), 1434825);
        assert_eq!(engine.perft(5), 50504249);
    }

    
    // https://github.com/niklasf/python-chess/blob/master/examples/perft/horde.perft
    
    #[test]
    fn horde_start() {
        let mut engine = Engine::from_fen("rnbqkbnr/pppppppp/8/1PP2PP1/PPPPPPPP/PPPPPPPP/PPPPPPPP/PPPPPPPP w kq - 0 1 HORDE");
        assert_eq!(engine.perft(1), 8);
        assert_eq!(engine.perft(2), 128);
        assert_eq!(engine.perft(3), 1274);
        assert_eq!(engine.perft(4), 23310);
        assert_eq!(engine.perft(5), 265223);
        assert_eq!(engine.perft(6), 5396554);
    }
    
    #[test]
    fn horde_open_flank() {
        let mut engine = Engine::from_fen("4k3/pp4q1/3P2p1/8/P3PP2/PPP2r2/PPP5/PPPP4 b - - 0 1 HORDE");
        assert_eq!(engine.perft(1), 30);
        assert_eq!(engine.perft(2), 241);
        assert_eq!(engine.perft(3), 6633);
        assert_eq!(engine.perft(4), 56539);
        assert_eq!(engine.perft(5), 1573347);
        assert_eq!(engine.perft(6), 14177327);
    }
    
    #[test]
    fn horde_en_passant() {
        let mut engine = Engine::from_fen("k7/5p2/4p2P/3p2P1/2p2P2/1p2P2P/p2P2P1/2P2P2 w - - 0 1 HORDE");
        assert_eq!(engine.perft(1), 13);
        assert_eq!(engine.perft(2), 172);
        assert_eq!(engine.perft(3), 2205);
        assert_eq!(engine.perft(4), 33781);
        assert_eq!(engine.perft(5), 426584);
        assert_eq!(engine.perft(6), 7174007);
    }
    
    #[test]
    fn horde_endgame() {
        let mut engine = Engine::from_fen("7r/1b2pppp/P5n1/P4k2/5b2/8/8/5N2 w - - 0 2 HORDE");
        assert_eq!(engine.perft(1), 6);
        assert_eq!(engine.perft(2), 164);
        assert_eq!(engine.perft(3), 1233);
        assert_eq!(engine.perft(4), 40474);
        assert_eq!(engine.perft(5), 371518);
        assert_eq!(engine.perft(6), 11514471);
    }
    
    
    // https://github.com/niklasf/python-chess/blob/master/examples/perft/giveaway.perft
    
    #[test]
    fn antichess_start() {
        let mut engine = Engine::from_fen("rnbqkbnr/pppppppp/8/8/8/8/PPPPPPPP/RNBQKBNR w - - 0 1 ANTICHESS");
        assert_eq!(engine.perft(1), 20);
        assert_eq!(engine.perft(2), 400);
        assert_eq!(engine.perft(3), 8067);
        assert_eq!(engine.perft(4), 153299);
        assert_eq!(engine.perft(5), 2732672);
        assert_eq!(engine.perft(6), 46264162);
    }
    
    #[test]
    fn antichess_a_pawn_vs_b_pawn() {
        let mut engine = Engine::from_fen("8/1p6/8/8/8/8/P7/8 w - - 0 1 ANTICHESS");
        assert_eq!(engine.perft(1), 2);
        assert_eq!(engine.perft(2), 4);
        assert_eq!(engine.perft(3), 4);
        assert_eq!(engine.perft(4), 3);
        assert_eq!(engine.perft(5), 1);
        assert_eq!(engine.perft(6), 0);
        assert_eq!(engine.perft(7), 0);
    }
    
    #[test]
    fn antichess_a_pawn_vs_c_pawn() {
        let mut engine = Engine::from_fen("8/2p5/8/8/8/8/P7/8 w - - 0 1 ANTICHESS");
        assert_eq!(engine.perft(1), 2);
        assert_eq!(engine.perft(2), 4);
        assert_eq!(engine.perft(3), 4);
        assert_eq!(engine.perft(4), 4);
        assert_eq!(engine.perft(5), 4);
        assert_eq!(engine.perft(6), 4);
        assert_eq!(engine.perft(7), 4);
        assert_eq!(engine.perft(8), 4);
        assert_eq!(engine.perft(9), 12);
        assert_eq!(engine.perft(10), 36);
        assert_eq!(engine.perft(11), 312);
        assert_eq!(engine.perft(12), 2557);
        assert_eq!(engine.perft(13), 30873);
        assert_eq!(engine.perft(14), 343639);
    }
    
    
    // https://github.com/niklasf/python-chess/blob/master/examples/perft/racingkings.perft
    
    #[test]
    fn racingkings_start() {
        let mut engine = Engine::from_fen("8/8/8/8/8/8/krbnNBRK/qrbnNBRQ w - - 0 1 RACINGKINGS");
        assert_eq!(engine.perft(1), 21);
        assert_eq!(engine.perft(2), 421);
        assert_eq!(engine.perft(3), 11264);
        assert_eq!(engine.perft(4), 296242);
        assert_eq!(engine.perft(5), 9472927);
    }
    // We cannot test Racing Kings endgame positions because this engine does not implement the
    // draw rule where black has the option move their king to the end immediately after white.
    // In King of the Hill, this draw rule does not exist.
    
    #[test]
    fn kingofthehill_end() {
        let mut engine = Engine::from_fen("r1bq1bnr/pppp1ppp/2n1pk2/8/8/2N1PK2/PPPP1PPP/R1BQ1BNR w - - 6 5 KINGOFTHEHILL");
        assert_eq!(engine.perft(1), 32);
        assert_eq!(engine.perft(2), 921);
        assert_eq!(engine.perft(3), 26371);
        assert_eq!(engine.perft(4), 749025);
        assert_eq!(engine.perft(5), 21562005);
    }
    
    
    // https://github.com/niklasf/python-chess/blob/master/examples/perft/3check.perft
    
    #[test]
    fn three_check_kiwipete() {
        let mut engine = Engine::from_fen("r3k2r/p1ppqpb1/bn2pnp1/3PN3/1p2P3/2N2Q1p/PPPBBPPP/R3K2R w KQkq - 0 1 +2+2 3CHECK");
        assert_eq!(engine.perft(1), 48);
        assert_eq!(engine.perft(2), 2039);
        assert_eq!(engine.perft(3), 97848);
        assert_eq!(engine.perft(4), 4081798);
    }
    
    #[test]
    fn three_check_castling() {
        let mut engine = Engine::from_fen("r3k2r/8/8/8/8/8/8/R3K2R w KQkq - 0 1 +2+2 3CHECK");
        assert_eq!(engine.perft(1), 26);
        assert_eq!(engine.perft(2), 562);
        assert_eq!(engine.perft(3), 13410);
        assert_eq!(engine.perft(4), 302770);
        assert_eq!(engine.perft(5), 7193131);
    }
}
