#[cfg(test)]
mod move_generator_test {
    use protochess_engine_rs::Position;
    use protochess_engine_rs::move_generator::MoveGen;
    use protochess_engine_rs::types::{Move, MoveType};

    #[test]
    fn capture_moves() {
        let mut pos = Position::from_fen("rnb1kbnr/ppppqppp/8/8/5P2/8/PPPP1PPP/RNBQKBNR w KQkq - 0 1");
        let z1 = pos.get_zobrist();
        assert!(MoveGen::in_check(&mut pos));
        let z2 = pos.get_zobrist();
        assert_eq!(z1, z2);
        for mv in MoveGen::get_capture_moves(&mut pos) {
            println!("{}", mv);
            assert!(mv.is_capture());
        }
    }
    
    #[test]
    fn test_move_type() {
        let mv = Move::new(0xAB, 0xCD, Some(0xEF), MoveType::Capture, Some(123));
        assert_eq!(mv.get_from(), 0xAB);
        assert_eq!(mv.get_to(), 0xCD);
        assert_eq!(mv.get_target(), 0xEF);
        assert_eq!(mv.get_move_type(), MoveType::Capture);
        assert_eq!(mv.is_capture(), true);
        assert_eq!(mv.is_null(), false);
        assert_eq!(mv.get_promotion_piece(), Some(123));
        
        assert!(Move::null().is_null());
        assert!(Move::null().get_promotion_piece().is_none());
        
        assert!(Move::new(0, 0, None, MoveType::Quiet, None).is_capture() == false);
        assert!(Move::new(0, 0, None, MoveType::Capture, None).is_capture() == true);
        assert!(Move::new(0, 0, None, MoveType::KingsideCastle, None).is_capture() == false);
        assert!(Move::new(0, 0, None, MoveType::QueensideCastle, None).is_capture() == false);
        assert!(Move::new(0, 0, None, MoveType::Promotion, None).is_capture() == false);
        assert!(Move::new(0, 0, None, MoveType::PromotionCapture, None).is_capture() == true);
        assert!(Move::new(0, 0, None, MoveType::Null, None).is_capture() == false);
    }
}
