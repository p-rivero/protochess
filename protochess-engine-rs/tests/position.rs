#[cfg(test)]
mod position_test {

    use std::convert::TryFrom;

    use protochess_engine_rs::position::create::position_factory::PositionFactory;
    use protochess_engine_rs::{GameState, MoveInfo};
    use protochess_engine_rs::types::Move;
    
    #[test]
    fn null_move_eq() {
        let mut pos = PositionFactory::default().set_state(GameState::default(), None).unwrap().unwrap();
        //let movegen = MoveGenerator::new();
        let zob_0 = pos.get_zobrist();
        pos.make_move(Move::null());
        pos.make_move(Move::null());
        pos.make_move(Move::null());
        pos.make_move(Move::null());
        pos.unmake_move();
        pos.unmake_move();
        pos.unmake_move();
        pos.unmake_move();
        assert_eq!(zob_0, pos.get_zobrist());
    }
    
    #[test]
    fn game_state_eq_position() {
        let mut factory = PositionFactory::default();
        let state = GameState::default();
        let pos1 = factory.set_state(state.clone(), None)
            .expect("Cannot load default GameState")
            .expect("set_state() returned None");
        let state2 = factory.get_state().clone();
        assert_eq!(state, state2);
        let pos2 = factory.set_state(state2, None)
            .expect("Cannot load default GameState")
            .expect("set_state() returned None");
        assert_eq!(pos1, pos2);
    }
    
    #[test]
    fn fen_vs_move_history_1() {
        let mut factory = PositionFactory::default();
        
        let mut state1 = GameState::default();
        state1.initial_fen = Some("rnbqkbnr/pppp1ppp/8/4p3/4P3/8/PPPP1PPP/RNBQKBNR w KQkq e6 0 2".to_string());
        let mut state2 = GameState::default();
        state2.move_history = build_move_history(vec!["e2e4", "e7e5"]);
        
        let pos1 = factory.set_state(state1, None)
            .expect("Cannot load GameState")
            .expect("set_state() returned None");
        let pos2 = factory.set_state(state2, None)
            .expect("Cannot load GameState")
            .expect("set_state() returned None");
        
        assert_eq!(pos1, pos2);
    }
    
    #[test]
    fn fen_vs_move_history_2() {
        let mut factory = PositionFactory::default();
        
        let mut state1 = GameState::default();
        state1.initial_fen = Some("r1bqkb1r/pppp1ppp/2n2n2/1B2p3/4P3/5N2/PPPP1PPP/RNBQ1RK1 b Qkq - 5 4".to_string());
        let mut state2 = GameState::default();
        state2.move_history = build_move_history(vec!["e2e4", "e7e5", "g1f3", "b8c6", "f1b5", "g8f6", "e1h1"]);
        
        
        let pos1 = factory.set_state(state1, None)
            .expect("Cannot load GameState")
            .expect("set_state() returned None");
        let pos2 = factory.set_state(state2, None)
            .expect("Cannot load GameState")
            .expect("set_state() returned None");
        
        println!("{:?}", pos1.get_properties());
        println!("{:?}", pos2.get_properties());
        
        assert_eq!(pos1.get_zobrist(), pos2.get_zobrist());
    }
    
    #[test]
    fn fen_vs_move_history_3() {
        let mut factory = PositionFactory::default();
        
        let mut state1 = GameState::default();
        state1.initial_fen = Some("r1bqr1k1/pppp1ppp/8/2bP4/8/8/PPP2PPP/RNBQ1BK1 w Qq - 0 12".to_string());
        let mut state2 = GameState::default();
        state2.move_history = build_move_history(vec!["e2e4", "e7e5", "g1f3", "b8c6", "f1b5", "g8f6", "e1h1", "f6e4", "f1e1", "e4d6", "f3e5", "f8e7", "b5f1", "c6e5", "e1e5", "e8h8", "d2d4", "d6e8", "d4d5", "e7c5", "e5e8", "f8e8"]);
        
        
        let pos1 = factory.set_state(state1, None)
            .expect("Cannot load GameState")
            .expect("set_state() returned None");
        let pos2 = factory.set_state(state2, None)
            .expect("Cannot load GameState")
            .expect("set_state() returned None");
        
        println!("{}", pos1);
        println!("{}", pos2);
        
        // assert_eq!(pos1, pos2);
    }
    
    
    fn build_move_history(moves: Vec<&str>) -> Vec<MoveInfo> {
        moves.iter().map(|mv| MoveInfo::try_from(*mv).unwrap()).collect()
    }
}

