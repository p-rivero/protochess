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
        assert_eq!(factory.get_notation(), &vec!["e4", "e5"]);
    }
    
    #[test]
    fn fen_vs_move_history_2() {
        let mut factory = PositionFactory::default();
        
        let mut state1 = GameState::default();
        state1.initial_fen = Some("r1bqkb1r/pppp1ppp/2n2n2/1B2p3/4P3/5N2/PPPP1PPP/RNBQ1RK1 b Akq - 5 4".to_string());
        let mut state2 = GameState::default();
        state2.move_history = build_move_history(vec!["e2e4", "e7e5", "g1f3", "b8c6", "f1b5", "g8f6", "e1h1"]);
        
        
        let pos1 = factory.set_state(state1, None)
            .expect("Cannot load GameState")
            .expect("set_state() returned None");
        let pos2 = factory.set_state(state2, None)
            .expect("Cannot load GameState")
            .expect("set_state() returned None");
        
        assert_eq!(pos1, pos2);
        assert_eq!(factory.get_notation(), &vec!["e4", "e5", "Nf3", "Nc6", "Bb5", "Nf6", "O-O"]);
    }
    
    #[test]
    fn fen_vs_move_history_3() {
        let mut factory = PositionFactory::default();
        
        let mut state1 = GameState::default();
        state1.initial_fen = Some("r1bqr1k1/pppp1ppp/8/2bP4/8/8/PPP2PPP/RNBQ1BK1 w Aa - 0 12".to_string());
        let mut state2 = GameState::default();
        state2.move_history = build_move_history(vec!["e2e4", "e7e5", "g1f3", "b8c6", "f1b5", "g8f6", "e1h1", "f6e4", "f1e1", "e4d6", "f3e5", "f8e7", "b5f1", "c6e5", "e1e5", "e8h8", "d2d4", "d6e8", "d4d5", "e7c5", "e5e8", "f8e8"]);
        
        
        let pos1 = factory.set_state(state1, None)
            .expect("Cannot load GameState")
            .expect("set_state() returned None");
        let pos2 = factory.set_state(state2, None)
            .expect("Cannot load GameState")
            .expect("set_state() returned None");
        
        assert_eq!(pos1, pos2);
        // assert_eq!(factory.get_notation(), &vec![
        //     "e4", "e5",
        //     "Nf3", "Nc6",
        //     "Bb5", "Nf6",
        //     "O-O", "Nxe4",
        //     "Re1", "Nd6",
        //     "Nxe5", "Be7",
        //     "Bf1", "Nxe5",
        //     "Re5", "O-O",
        //     "d4", "Ne8",
        //     "d5", "Nc5",
        //     "Rxe8", "Rxe8"
        // ]);
    }
    
    #[test]
    fn non_incremental_updates() {
        // Sanity check for incremental_updates test below.
        // Test that the fen and move history versions of the same position are equal.
        let mut factory = PositionFactory::default();
        
        let mut state1_fen = GameState::default();
        state1_fen.initial_fen = Some("r1bqr1k1/pppp1ppp/8/2bP4/8/8/PPP2PPP/RNBQ1BK1 w Aa - 0 12".to_string());
        let mut state1_moves = GameState::default();
        state1_moves.move_history = build_move_history(vec!["e2e4", "e7e5", "g1f3", "b8c6", "f1b5", "g8f6", "e1h1", "f6e4", "f1e1", "e4d6", "f3e5", "f8e7", "b5f1", "c6e5", "e1e5", "e8h8", "d2d4", "d6e8", "d4d5", "e7c5", "e5e8", "f8e8"]);
        
        let mut state2_fen = GameState::default();
        state2_fen.initial_fen = Some("r1bqnrk1/pppp1ppp/8/3P4/8/4P3/PPP3PP/RNBQ1BK1 b Aa - 0 12".to_string());
        let mut state2_moves = GameState::default();
        state2_moves.move_history = build_move_history(vec!["e2e4", "e7e5", "g1f3", "b8c6", "f1b5", "g8f6", "e1h1", "f6e4", "f1e1", "e4d6", "f3e5", "f8e7", "b5f1", "c6e5", "e1e5", "e8h8", "d2d4", "d6e8", "d4d5", "e7c5", "e5e3", "c5e3", "f2e3"]);
        
        let pos1_fen = factory.set_state(state1_fen, None)
            .expect("Cannot load GameState")
            .expect("set_state() returned None");
        let pos1_moves = factory.set_state(state1_moves, None)
            .expect("Cannot load GameState")
            .expect("set_state() returned None");
        
        let pos2_fen = factory.set_state(state2_fen, None)
            .expect("Cannot load GameState")
            .expect("set_state() returned None");
        let pos2_moves = factory.set_state(state2_moves, None)
            .expect("Cannot load GameState")
            .expect("set_state() returned None");
        
        assert_eq!(pos1_fen, pos1_moves);
        assert_eq!(pos2_fen, pos2_moves);
    }
    
    #[test]
    fn incremental_updates() {
        let mut factory = PositionFactory::default();
        
        let mut state1_moves = GameState::default();
        state1_moves.move_history = build_move_history(vec!["e2e4", "e7e5", "g1f3", "b8c6", "f1b5", "g8f6", "e1h1", "f6e4", "f1e1", "e4d6", "f3e5", "f8e7", "b5f1", "c6e5", "e1e5", "e8h8", "d2d4", "d6e8", "d4d5", "e7c5", "e5e8", "f8e8"]);
        
        let mut state2_fen = GameState::default();
        state2_fen.initial_fen = Some("r1bqnrk1/pppp1ppp/8/3P4/8/4P3/PPP3PP/RNBQ1BK1 b Aa - 0 12".to_string());
        let mut state2_moves = GameState::default();
        // Mostly the same, but the last moves are different
        state2_moves.move_history = build_move_history(vec!["e2e4", "e7e5", "g1f3", "b8c6", "f1b5", "g8f6", "e1h1", "f6e4", "f1e1", "e4d6", "f3e5", "f8e7", "b5f1", "c6e5", "e1e5", "e8h8", "d2d4", "d6e8", "d4d5", "e7c5", "e5e3", "c5e3", "f2e3"]);
        
        let mut pos = factory.set_state(state1_moves, None)
            .expect("Cannot load GameState")
            .expect("set_state() returned None");
        
        // Reuse pos1_moves as position and only update the last moves
        let ret = factory.set_state(state2_moves, Some(&mut pos));
        assert_eq!(ret, Ok(None));
        
        let pos_target = factory.set_state(state2_fen, None)
            .expect("Cannot load GameState")
            .expect("set_state() returned None");
        
        assert_eq!(pos, pos_target);
    }
    
    #[test]
    fn incremental_updates_undo() {
        let mut factory = PositionFactory::default();
        
        let mut state1_moves = GameState::default();
        state1_moves.move_history = build_move_history(vec!["e2e4", "e7e5", "g1f3", "b8c6", "f1b5", "g8f6", "e1h1", "f6e4", "f1e1", "e4d6", "f3e5", "f8e7", "b5f1", "c6e5", "e1e5", "e8h8", "d2d4", "d6e8", "d4d5", "e7c5", "e5e8", "f8e8"]);
        
        // Return to the starting position
        let state2_fen = GameState::default();
        let mut state2_moves = GameState::default();
        state2_moves.move_history = build_move_history(vec![]);
        
        let mut pos = factory.set_state(state1_moves, None)
            .expect("Cannot load GameState")
            .expect("set_state() returned None");
        
        let ret = factory.set_state(state2_moves, Some(&mut pos));
        assert_eq!(ret, Ok(None));
        
        let pos_target = factory.set_state(state2_fen, None)
            .expect("Cannot load GameState")
            .expect("set_state() returned None");
        
        assert_eq!(pos, pos_target);
    }
    
    #[test]
    fn incremental_updates_fail() {
        let mut factory = PositionFactory::default();
        
        let mut state1_moves = GameState::default();
        state1_moves.move_history = build_move_history(vec!["e2e4", "e7e5", "g1f3", "b8c6", "f1b5", "g8f6", "e1h1", "f6e4", "f1e1", "e4d6", "f3e5", "f8e7", "b5f1", "c6e5", "e1e5", "e8h8", "d2d4", "d6e8", "d4d5", "e7c5", "e5e8", "f8e8"]);
        
        let mut state2_moves = GameState::default();
        state2_moves.move_history = build_move_history(vec!["e2e4", "e7e5", "g1f3", "b8c6", "f1b5", "g8f6", "e1h1", "f6e4", "f1e1", "e4d6", "f3e5", "f8e7", "b5f1", "c6e5", "e1e5", "e8h8", "d2d4", "d6e8", "d4d5", "e7c5", "e5e3", "c5e3", "f2e3", "a8a7", "d1e1"]);
        
        let mut pos = factory.set_state(state1_moves, None)
            .expect("Cannot load GameState")
            .expect("set_state() returned None");
        
        let pos_before = pos.clone();
        assert_eq!(pos, pos_before);
        
        // If set_state() fails, the position should not be modified
        let ret = factory.set_state(state2_moves, Some(&mut pos));
        assert_eq!(ret, Err("Invalid move: a8a7".to_string()));
        assert_eq!(pos, pos_before);
    }
    
    
    fn build_move_history(moves: Vec<&str>) -> Vec<MoveInfo> {
        moves.iter().map(|mv| MoveInfo::try_from(*mv).unwrap()).collect()
    }
}

