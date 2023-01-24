extern crate protochess_engine_rs;



#[cfg(test)]
mod perft {
    use protochess_engine_rs::Engine;
    // https://www.chessprogramming.org/Perft_Results
    
    #[test]
    fn starting_pos() {
        let fen = "rnbqkbnr/pppppppp/8/8/8/8/PPPPPPPP/RNBQKBNR w KQkq - 0 1";
        test_perft(fen, vec![
            20,
            400,
            8902,
            197281,
            4865609,
            119060324,
        ]);
    }

    #[test]
    fn kiwipete() {
        let fen = "r3k2r/p1ppqpb1/bn2pnp1/3PN3/1p2P3/2N2Q1p/PPPBBPPP/R3K2R w KQkq - 0 1";
        test_perft(fen, vec![
            48,
            2039,
            97862,
            4085603,
            193690690,
        ]);
    }

    #[test]
    fn pos3() {
        let fen = "8/2p5/3p4/KP5r/1R3p1k/8/4P1P1/8 w - - 0 1";
        test_perft(fen, vec![
            14,
            191,
            2812,
            43238,
            674624,
            11030083,
        ]);
    }

    #[test]
    fn pos4() {
        let fen = "r3k2r/Pppp1ppp/1b3nbN/nP6/BBP1P3/q4N2/Pp1P2PP/R2Q1RK1 w kq - 0 1";
        test_perft(fen, vec![
            6,
            264,
            9467,
            422333,
            15833292,
        ]);
    }

    #[test]
    fn pos5() {
        let fen = "rnbq1k1r/pp1Pbppp/2p5/8/2B5/8/PPP1NnPP/RNBQK2R w KQ - 1 8  ";
        test_perft(fen, vec![
            44,
            1486,
            62379,
            2103487,
            89941194,
        ]);
    }

    #[test]
    fn pos6() {
        let fen = "r4rk1/1pp1qppp/p1np1n2/2b1p1B1/2B1P1b1/P1NP1N2/1PP1QPPP/R4RK1 w - - 0 10 ";
        test_perft(fen, vec![
            46,
            2079,
            89890,
            3894594,
            164075551,
        ]);
    }
    
    
    // https://github.com/niklasf/python-chess/blob/master/examples/perft/tricky.perft
    
    #[test]
    fn xfen960_0() {
        let fen = "r1k1r2q/p1ppp1pp/8/8/8/8/P1PPP1PP/R1K1R2Q w KQkq - 0 1";
        test_perft(fen, vec![
            23,
            522,
            12333,
            285754,
            7096972,
        ]);
    }
    
    #[test]
    fn xfen960_1() {
        let fen = "r1k2r1q/p1ppp1pp/8/8/8/8/P1PPP1PP/R1K2R1Q w KQkq - 0 1";
        test_perft(fen, vec![
            28,
            738,
            20218,
            541480,
            15194841,
        ]);
    }
    
    #[test]
    fn xfen960_2() {
        let fen = "8/8/8/4B2b/6nN/8/5P2/2R1K2k w Q - 0 1";
        test_perft(fen, vec![
            34,
            318,
            9002,
            118388,
            3223406,
        ]);
    }
    
    #[test]
    fn xfen960_3() {
        let fen = "2r5/8/8/8/8/8/6PP/k2KR3 w K - 0 1";
        test_perft(fen, vec![
            17,
            242,
            3931,
            57700,
            985298,
            14751778,
        ]);
    }
    
    #[test]
    fn xfen960_4() {
        let fen = "4r3/3k4/8/8/8/8/6PP/qR1K1R2 w KQ - 0 1";
        test_perft(fen, vec![
            19,
            628,
            12858,
            405636,
            8992652,
        ]);
    }
    
    #[test]
    fn gotta_love_perft_1() {
        let fen = "8/ppp3p1/8/8/3p4/5Q2/1ppp2K1/brk4n w - - 0 1";
        test_perft(fen, vec![
            27,
            390,
            9354,
            134167,
            2922659,
            42959630,
        ]);
    }
    
    #[test]
    fn gotta_love_perft_2() {
        let fen = "8/6kR/8/8/8/bq6/1rqqqqqq/K1nqnbrq b - - 0 1";
        test_perft(fen, vec![
            7,
            52,
            4593,
            50268,
            4634384,
        ]);
    }
    
    #[test]
    fn chess960_swap_castling() {
        let fen = "2rkr3/8/8/8/8/1PP5/P4PP1/5KR1 b Kkq - 0 2";
        test_perft(fen, vec![
            22,
            222,
            5182,
            60618,
            1499136,
            20508951,
        ]);
    }
    
    
    // https://github.com/niklasf/python-chess/blob/master/examples/perft/atomic.perft
    
    #[test]
    fn atomic_start() {
        let fen = "rnbqkbnr/pppppppp/8/8/8/8/PPPPPPPP/RNBQKBNR w KQkq - 0 1 ATOMIC";
        test_perft(fen, vec![
            20,
            400,
            8902,
            197326,
            4864979,
            118926425,
        ]);
    }
    
    #[test]
    fn atomic_programfox_1() {
        let fen = "rn2kb1r/1pp1p2p/p2q1pp1/3P4/2P3b1/4PN2/PP3PPP/R2QKB1R b KQkq - 0 1 ATOMIC";
        test_perft(fen, vec![
            40,
            1238,
            45237,
            1434825,
            50504249,
        ]);
    }
    
    #[test]
    fn atomic_programfox_2() {
        let fen = "rn1qkb1r/p5pp/2p5/3p4/N3P3/5P2/PPP4P/R1BQK3 w Qkq - 0 1 ATOMIC";
        test_perft(fen, vec![
            28,
            833,
            23353,
            714499,
            21134061,
        ]);
    }
    
    #[test]
    fn atomic_checks() {
        let fen = "7r/2N2k1p/p1n3pb/3p1p2/6Pq/1PP1P3/P4P1P/R1BK1B1R w - - 3 17 ATOMIC";
        test_perft(fen, vec![
            32,
            1275,
            39093,
            1425274,
        ]);
    }
    
    #[test]
    fn atomic_double_check() {
        let fen = "r3k1nr/ppp2ppp/8/8/P3q3/1n1b1P2/RPPb2PP/1NBQKBNR w Kkq - 1 18 ATOMIC";
        test_perft(fen, vec![
            3,
            113,
            3011,
            110029,
            2972933,
        ]);
    }
    
    #[test]
    fn atomic960_castle_1() {
        let fen = "8/8/8/8/8/8/2k5/rR4KR w KQ - 0 1 ATOMIC";
        test_perft(fen, vec![
            18,
            180,
            4364,
            61401,
            1603055,
            23969896,
        ]);
    }
    
    #[test]
    fn atomic960_castle_2() {
        let fen = "r3k1rR/5K2/8/8/8/8/8/8 b kq - 0 1 ATOMIC";
        test_perft(fen, vec![
            25,
            282,
            6753,
            98729,
            2587730,
        ]);
    }
    
    #[test]
    fn atomic960_castle_3() {
        let fen = "Rr2k1rR/3K4/3p4/8/8/8/7P/8 w kq - 0 1 ATOMIC";
        test_perft(fen, vec![
            21,
            465,
            10631,
            241478,
            5800275,
        ]);
    }
    
    #[test]
    fn shakmaty_bench() {
        let fen = "rn2kb1r/1pp1p2p/p2q1pp1/3P4/2P3b1/4PN2/PP3PPP/R2QKB1R b KQkq - 0 1 ATOMIC";
        test_perft(fen, vec![
            40,
            1238,
            45237,
            1434825,
            50504249,
        ]);
    }

    
    // https://github.com/niklasf/python-chess/blob/master/examples/perft/horde.perft
    
    #[test]
    fn horde_start() {
        let fen = "rnbqkbnr/pppppppp/8/1PP2PP1/PPPPPPPP/PPPPPPPP/PPPPPPPP/PPPPPPPP w kq - 0 1 HORDE";
        test_perft(fen, vec![
            8,
            128,
            1274,
            23310,
            265223,
            5396554,
        ]);
    }
    
    #[test]
    fn horde_open_flank() {
        let fen = "4k3/pp4q1/3P2p1/8/P3PP2/PPP2r2/PPP5/PPPP4 b - - 0 1 HORDE";
        test_perft(fen, vec![
            30,
            241,
            6633,
            56539,
            1573347,
            14177327,
        ]);
    }
    
    #[test]
    fn horde_en_passant() {
        let fen = "k7/5p2/4p2P/3p2P1/2p2P2/1p2P2P/p2P2P1/2P2P2 w - - 0 1 HORDE";
        test_perft(fen, vec![
            13,
            172,
            2205,
            33781,
            426584,
            7174007,
        ]);
    }
    
    #[test]
    fn horde_endgame() {
        let fen = "7r/1b2pppp/P5n1/P4k2/5b2/8/8/5N2 w - - 0 2 HORDE";
        test_perft(fen, vec![
            6,
            164,
            1233,
            40474,
            371518,
            11514471,
        ]);
    }
    
    
    // https://github.com/niklasf/python-chess/blob/master/examples/perft/giveaway.perft
    
    #[test]
    fn antichess_start() {
        let fen = "rnbqkbnr/pppppppp/8/8/8/8/PPPPPPPP/RNBQKBNR w - - 0 1 ANTICHESS";
        test_perft(fen, vec![
            20,
            400,
            8067,
            153299,
            2732672,
            46264162,
        ]);
    }
    
    #[test]
    fn antichess_a_pawn_vs_b_pawn() {
        let fen = "8/1p6/8/8/8/8/P7/8 w - - 0 1 ANTICHESS";
        test_perft(fen, vec![
            2,
            4,
            4,
            3,
            1,
            0,
            0,
        ]);
    }
    
    #[test]
    fn antichess_a_pawn_vs_c_pawn() {
        let fen = "8/2p5/8/8/8/8/P7/8 w - - 0 1 ANTICHESS";
        test_perft(fen, vec![
            2,
            4,
            4,
            4,
            4,
            4,
            4,
            4,
            12,
            36,
            312,
            2557,
            30873,
            343639,
        ]);
    }
    
    
    // https://github.com/niklasf/python-chess/blob/master/examples/perft/racingkings.perft
    
    #[test]
    fn racingkings_start() {
        let fen = "8/8/8/8/8/8/krbnNBRK/qrbnNBRQ w - - 0 1 RACINGKINGS";
        test_perft(fen, vec![
            21,
            421,
            11264,
            296242,
            9472927,
        ]);
    }
    // We cannot test Racing Kings endgame positions because this engine does not implement the
    // draw rule where black has the option move their king to the end immediately after white.
    // In King of the Hill, this draw rule does not exist.
    
    #[test]
    fn kingofthehill_end() {
        let fen = "r1bq1bnr/pppp1ppp/2n1pk2/8/8/2N1PK2/PPPP1PPP/R1BQ1BNR w - - 6 5 KINGOFTHEHILL";
        test_perft(fen, vec![
            32,
            921,
            26371,
            749025,
            21562005,
        ]);
    }
    
    
    // https://github.com/niklasf/python-chess/blob/master/examples/perft/3check.perft
    
    #[test]
    fn three_check_kiwipete() {
        let fen = "r3k2r/p1ppqpb1/bn2pnp1/3PN3/1p2P3/2N2Q1p/PPPBBPPP/R3K2R w KQkq - 0 1 +2+2 3CHECK";
        test_perft(fen, vec![
            48,
            2039,
            97848,
            4081798,
        ]);
    }
    
    #[test]
    fn three_check_castling() {
        let fen = "r3k2r/8/8/8/8/8/8/R3K2R w KQkq - 0 1 +2+2 3CHECK";
        test_perft(fen, vec![
            26,
            562,
            13410,
            302770,
            7193131,
        ]);
    }
    
    fn test_perft(fen: &str, results: Vec<u64>) {
        let mut engine = Engine::from_fen(fen);
        for (i, result) in results.iter().enumerate() {
            let depth = i as u8 + 1;
            assert_eq!(engine.perft(depth), *result);
        }
    }
}
