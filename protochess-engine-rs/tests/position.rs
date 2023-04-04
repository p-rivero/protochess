#[cfg(test)]
mod position_test {

    use protochess_engine_rs::position::create::position_factory::PositionFactory;
    use protochess_engine_rs::GameState;
    use protochess_engine_rs::types::Move;
    
    #[test]
    fn null_move_eq() {
        let mut pos = PositionFactory::default().set_state(GameState::default()).unwrap();
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
        let _pos = factory.set_state(state.clone()).expect("Cannot load default GameState");
        let state2 = factory.get_state().clone();
        assert_eq!(state, state2);
    }
    
    // TODO: Test that PositionFactory performs incremental updates correctly
}

