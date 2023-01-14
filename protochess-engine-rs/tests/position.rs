#[cfg(test)]
mod position_test {
    use protochess_engine_rs::{Position, GameState};
    use protochess_engine_rs::types::Move;
    
    
    #[test]
    fn pieces_tiles_as_tuples() {
        let pos = Position::from(GameState::default());
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
                assert!(false);
            }
        }
        assert!(white_tiles == 32);
        assert!(black_tiles == 32);

    }

    #[test]
    fn null_move_eq() {
        let mut pos = Position::from(GameState::default());
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
    
}

