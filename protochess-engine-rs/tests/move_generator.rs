#[cfg(test)]
mod move_generator_test {
    use std::convert::TryFrom;

    use protochess_engine_rs::{Position, GameState};
    use protochess_engine_rs::move_generator::MoveGen;
    use protochess_engine_rs::types::{Move, MoveType};

    #[test]
    fn capture_moves() {
        let gs = GameState::from_fen("rnb1kbnr/ppppqppp/8/8/5P2/8/PPPP1PPP/RNBQKBNR w KQkq - 0 1").unwrap();
        let mut pos = Position::try_from(gs).unwrap();
        let z1 = pos.get_zobrist();
        assert!(MoveGen::in_check(&mut pos));
        let z2 = pos.get_zobrist();
        assert_eq!(z1, z2);
        for mv in MoveGen::get_pseudo_moves(&mut pos, false) {
            println!("{mv}");
            assert!(mv.is_capture());
        }
    }
    
    #[test]
    fn test_move_type() {
        let mv = Move::new(0xAB, 0xCD, 0xEF, MoveType::PromotionCapture, Some(123));
        assert_eq!(mv.get_from(), 0xAB);
        assert_eq!(mv.get_to(), 0xCD);
        assert_eq!(mv.get_target(), 0xEF);
        assert_eq!(mv.get_move_type(), MoveType::PromotionCapture);
        assert!(mv.is_capture());
        assert!(!mv.is_null());
        assert_eq!(mv.get_promotion_piece(), Some(123));
        
        assert!(Move::null().is_null());
        assert!(Move::null().get_promotion_piece().is_none());
        
        assert!(!Move::new(0, 0, 0, MoveType::Quiet, None).is_capture());
        assert!(Move::new(0, 0, 0, MoveType::Capture, None).is_capture());
        assert!(!Move::new(0, 0, 0, MoveType::KingsideCastle, None).is_capture());
        assert!(!Move::new(0, 0, 0, MoveType::QueensideCastle, None).is_capture());
        assert!(!Move::new(0, 0, 0, MoveType::Promotion, None).is_capture());
        assert!(Move::new(0, 0, 0, MoveType::PromotionCapture, None).is_capture());
        assert!(!Move::new(0, 0, 0, MoveType::Null, None).is_capture());
    }
}
