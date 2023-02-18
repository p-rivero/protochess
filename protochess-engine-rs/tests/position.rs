#[cfg(test)]
mod position_test {
    use std::convert::TryFrom;

    use protochess_engine_rs::{Position, GameState};
    use protochess_engine_rs::types::Move;
    
    
    #[test]
    fn pieces_tiles_as_tuples() {
        let pos = Position::try_from(GameState::default()).unwrap();
        let pieces = pos.pieces_as_tuples();
        assert!(pieces.len() == 32);
        let mut white_pieces = 0;
        let mut black_pieces = 0;
        for pce in pos.pieces_as_tuples() {
            if pce.0 == 0 {
                white_pieces += 1;
            } else {
                black_pieces += 1;
            }
        }
        assert!(white_pieces == 16);
        assert!(black_pieces == 16);

        let tiles = pos.tiles_as_tuples();
        assert!(tiles.len() == 64);
        let mut white_tiles = 0;
        let mut black_tiles = 0;
        for pce in pos.tiles_as_tuples() {
            if pce.2 == 'w' {
                white_tiles += 1;
            } else if pce.2 == 'b' {
                black_tiles += 1;
            } else {
                panic!("Invalid tile color: {}", pce.2);
            }
        }
        assert!(white_tiles == 32);
        assert!(black_tiles == 32);

    }

    #[test]
    fn null_move_eq() {
        let mut pos = Position::try_from(GameState::default()).unwrap();
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
        let state = GameState::default();
        let pos = Position::try_from(state.clone()).unwrap();
        let state2 = GameState::from(&pos);
        assert_eq!(state, state2);
        let pos2 = Position::try_from(state2.clone()).unwrap();
        assert_eq!(pos, pos2);
    }
    
    #[test]
    fn game_state_eq_position2() {
        let state = GameState::from_fen("r3k2r/p1ppqpb1/bn2pnp1/3PN3/1p2P3/2N2Q1p/PPPBBPPP/R3K2R w KQkq - 0 1 ATOMIC").unwrap();
        let pos = Position::try_from(state.clone()).unwrap();
        let state2 = GameState::from(&pos);
        assert_eq!(state, state2);
        let pos2 = Position::try_from(state2.clone()).unwrap();
        assert_eq!(pos, pos2);
    }
    
    #[test]
    fn game_state_eq_position3() {
        let state = GameState::from_fen("rnbqkbnr/pppppppp/8/1PP2PP1/PPPPPPPP/PPPPPPPP/PPPPPPPP/PPPPPPPP w kq - 0 1 HORDE").unwrap();
        let pos = Position::try_from(state.clone()).unwrap();
        let state2 = GameState::from(&pos);
        assert_eq!(state, state2);
        let pos2 = Position::try_from(state2.clone()).unwrap();
        assert_eq!(pos, pos2);
    }
}

