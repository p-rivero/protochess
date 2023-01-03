#[cfg(test)]
mod zobrist_test {
    use protochess_engine_rs::{Engine, MoveInfo};
    
    #[test]
    fn zobrist_pawn_push() {
        let mv = vec!["e2e3"];
        let expected_fen = "rnbqkbnr/pppppppp/8/8/8/4P3/PPPP1PPP/RNBQKBNR b KQkq - 0 1";
        test_zobrist_sequence(&mv, expected_fen);
    }
    
    #[test]
    fn zobrist_pawn_double_push() {
        let mv = vec!["e2e4"];
        let expected_fen = "rnbqkbnr/pppppppp/8/8/4P3/8/PPPP1PPP/RNBQKBNR b KQkq e3 0 1";
        test_zobrist_sequence(&mv, expected_fen);
    }
    
    #[test]
    fn zobrist_pawn_capture() {
        let mv = vec!["e2e4", "e7e5", "d2d4", "e5d4"];
        let expected_fen = "rnbqkbnr/pppp1ppp/8/8/3pP3/8/PPP2PPP/RNBQKBNR w KQkq - 0 3";
        test_zobrist_sequence(&mv, expected_fen);
    }
    
    #[test]
    fn zobrist_pawn_en_passant() {
        let mv = vec!["e2e4", "h7h6", "e4e5", "d7d5", "e5d6"];
        let expected_fen = "rnbqkbnr/ppp1ppp1/3P3p/8/8/8/PPPP1PPP/RNBQKBNR b KQkq - 0 3";
        test_zobrist_sequence(&mv, expected_fen);
    }
    
    #[test]
    fn zobrist_castle() {
        let mv = vec!["e2e4", "e7e5", "g1f3", "b8c6", "f1c4", "g8f6", "e1h1"];
        let expected_fen = "r1bqkb1r/pppp1ppp/2n2n2/4p3/2B1P3/5N2/PPPP1PPP/RNBQ1RK1 b Qkq - 5 4";
        test_zobrist_sequence(&mv, expected_fen);
    }
    
    #[test]
    fn zobrist_capture() {
        let mv = vec!["e2e4", "f7f6", "f1c4", "f6f5", "c4g8"];
        let expected_fen = "rnbqkbBr/ppppp1pp/8/5p2/4P3/8/PPPP1PPP/RNBQK1NR b KQkq - 0 3";
        test_zobrist_sequence(&mv, expected_fen);
    }
    
    #[test]
    fn zobrist_capture_2() {
        let mv = vec!["e2e4", "f7f6", "f1c4", "f6f5", "c4g8", "h8g8", "e4f5"];
        let expected_fen = "rnbqkbr1/ppppp1pp/8/5P2/8/8/PPPP1PPP/RNBQK1NR b KQq - 0 4";
        test_zobrist_sequence(&mv, expected_fen);
    }
    
    #[test]
    fn zobrist_capture_3() {
        let mv = vec!["e2e4", "f7f6", "f1c4", "f6f5", "c4g8", "h8g8"];
        let expected_fen = "rnbqkbr1/ppppp1pp/8/5p2/4P3/8/PPPP1PPP/RNBQK1NR w KQq - 0 4";
        test_zobrist_sequence(&mv, expected_fen);
    }
    
    #[test]
    fn zobrist_capture_rook() {
        // Make sure castling gets disabled for captured rook
        let mv = vec!["b2b3", "g7g6", "c1b2", "g6g5", "b2h8"];
        let expected_fen = "rnbqkbnB/pppppp1p/8/6p1/8/1P6/P1PPPPPP/RN1QKBNR b KQq - 0 3";
        test_zobrist_sequence(&mv, expected_fen);
    }
    
    #[test]
    fn zobrist_rook_captures_rook() {
        // Make sure castling gets disabled for both rooks
        let mv = vec!["h2h4", "h7h5", "g2g4", "g7g5", "h4g5", "h5g4", "h1h8"];
        let expected_fen = "rnbqkbnR/pppppp2/8/6P1/6p1/8/PPPPPP2/RNBQKBN1 b Qq - 0 4";
        test_zobrist_sequence(&mv, expected_fen);
    }
    
    #[test]
    fn zobrist_promotion() {
        // ID_KNIGHT = 4
        let mv = vec!["h2h4", "g7g5", "h4g5", "h7h6", "g5g6", "h6h5", "g6g7", "g8f6", "g7g8=4"];
        let expected_fen = "rnbqkbNr/pppppp2/5n2/7p/8/8/PPPPPPP1/RNBQKBNR b KQkq - 0 5";
        test_zobrist_sequence(&mv, expected_fen);
    }
    
    #[test]
    fn zobrist_promotion_capture() {
        // ID_BISHOP = 3
        let mv = vec!["h2h4", "g7g5", "h4g5", "h7h6", "g5g6", "h6h5", "g6g7", "g8f6", "g7f8=3"];
        let expected_fen = "rnbqkB1r/pppppp2/5n2/7p/8/8/PPPPPPP1/RNBQKBNR b KQkq - 0 5";
        test_zobrist_sequence(&mv, expected_fen);
    }
    
    #[test]
    fn zobrist_promotion_capture_rook() {
        // ID_QUEEN = 1
        let mv = vec!["h2h4", "g7g5", "h4g5", "h7h6", "g5g6", "h6h5", "g6g7", "g8f6", "g7h8=1"];
        let expected_fen = "rnbqkb1Q/pppppp2/5n2/7p/8/8/PPPPPPP1/RNBQKBNR b KQq - 0 5";
        test_zobrist_sequence(&mv, expected_fen);
    }
    
    #[test]
    fn zobrist_transposition() {
        let mv1 = vec!["e2e4", "e7e5", "g1f3", "b8c6", "f1c4", "g8f6", "e1h1"];
        let mv2 = vec!["g1f3", "e7e5", "e2e4", "g8f6", "f1c4", "b8c6", "e1h1"];
        let expected_fen = "r1bqkb1r/pppp1ppp/2n2n2/4p3/2B1P3/5N2/PPPP1PPP/RNBQ1RK1 b Qkq - 5 4";
        test_zobrist_sequence(&mv1, expected_fen);
        test_zobrist_sequence(&mv2, expected_fen);
    }
    
    #[test]
    fn zobrist_opposite_player_1() {
        // zobrist_opposite_player_2() results in the same position, but with black to move
        let mv = vec!["e2e3", "d7d6", "f1e2", "c8e6"];
        let expected_fen = "rn1qkbnr/ppp1pppp/3pb3/8/8/4P3/PPPPBPPP/RNBQK1NR w KQkq - 2 3";
        test_zobrist_sequence(&mv, expected_fen);
    }
    
    #[test]
    fn zobrist_opposite_player_2() {
        // zobrist_opposite_player_1() results in the same position, but with white to move
        // player_to_move_affects_zobrist() tests that the zobrist hash is different for the two
        let mv = vec!["e2e3", "d7d6", "f1d3", "c8e6", "d3e2"];
        let expected_fen = "rn1qkbnr/ppp1pppp/3pb3/8/8/4P3/PPPPBPPP/RNBQK1NR b KQkq - 2 3";
        test_zobrist_sequence(&mv, expected_fen);
    }
    
    #[test]
    fn castling_rights_affect_zobrist() {
        let mut zobrist = vec![];
        for i in 0..16 {
            let mut castling = String::from("");
            if i & 1 != 0 {
                castling.push('K');
            }
            if i & 2 != 0 {
                castling.push('Q');
            }
            if i & 4 != 0 {
                castling.push('k');
            }
            if i & 8 != 0 {
                castling.push('q');
            }
            if castling.is_empty() {
                castling.push('-');
            }
            let fen = format!("rnbqkbnr/pppppppp/8/8/8/8/PPPPPPPP/RNBQKBNR w {} - 0 1", castling);
            let engine = Engine::from_fen(&fen);
            zobrist.push(engine.get_zobrist());
        }
        for i in 0..16 {
            for j in 0..16 {
                if i != j {
                    assert_ne!(zobrist[i], zobrist[j]);
                }
            }
        }
    }
    
    #[test]
    fn ep_square_affects_zobrist() {
        let engine1 = Engine::from_fen("rnbqkbnr/pppppppp/8/8/8/8/PPPPPPPP/RNBQKBNR w KQkq - 0 1");
        let engine2 = Engine::from_fen("rnbqkbnr/pppppppp/8/8/8/8/PPPPPPPP/RNBQKBNR w KQkq e3 0 1");
        assert_ne!(engine1.get_zobrist(), engine2.get_zobrist());
    }
    
    #[test]
    fn player_to_move_affects_zobrist() {
        let engine1 = Engine::from_fen("rnbqkbnr/pppppppp/8/8/8/8/PPPPPPPP/RNBQKBNR w KQkq - 0 1");
        let engine2 = Engine::from_fen("rnbqkbnr/pppppppp/8/8/8/8/PPPPPPPP/RNBQKBNR b KQkq - 0 1");
        assert_ne!(engine1.get_zobrist(), engine2.get_zobrist());
    }
    
    #[test]
    fn turn_does_not_affect_zobrist() {
        let engine1 = Engine::from_fen("rnbqkbnr/pppppppp/8/8/8/8/PPPPPPPP/RNBQKBNR w KQkq - 0 1");
        let engine2 = Engine::from_fen("rnbqkbnr/pppppppp/8/8/8/8/PPPPPPPP/RNBQKBNR w KQkq - 0 2");
        assert_eq!(engine1.get_zobrist(), engine2.get_zobrist());
    }
    
    #[test]
    fn halfmove_clock_does_not_affect_zobrist() {
        // Halfmove clock is not implemented
        let engine1 = Engine::from_fen("rnbqkbnr/pppppppp/8/8/8/8/PPPPPPPP/RNBQKBNR w KQkq - 0 1");
        let engine2 = Engine::from_fen("rnbqkbnr/pppppppp/8/8/8/8/PPPPPPPP/RNBQKBNR w KQkq - 1 1");
        assert_eq!(engine1.get_zobrist(), engine2.get_zobrist());
    }

    fn test_zobrist_sequence(moves: &[&str], expected_fen: &str) {
        let mut engine1 = Engine::default();
        let mut engine2 = Engine::default();
        let zob_start_1 = engine1.get_zobrist();
        let zob_start_2 = engine2.get_zobrist();
        assert_eq!(zob_start_1, zob_start_2);
        
        for m in moves {
            engine1.make_move(&MoveInfo::from_string(m));
            engine2.make_move(&MoveInfo::from_string(m));
            let zob_1 = engine1.get_zobrist();
            let zob_2 = engine2.get_zobrist();
            assert_eq!(zob_1, zob_2);
        }
        
        let engine3 = Engine::from_fen(expected_fen);
        assert_eq!(engine1.get_zobrist(), engine3.get_zobrist());
    }
}