#[cfg(test)]
mod position_test {
    use protochess_engine_rs::Position;
    use protochess_engine_rs::MoveGen;
    use protochess_engine_rs::position::castle_rights::CastleRights;
    use protochess_engine_rs::types::Move;
    
    
    #[test]
    fn print_pieces() {
        let pos = Position::default();
        for pce in pos.pieces_as_tuples() {
            println!("{:?}", pce);
        }

        for pce in pos.tiles_as_tuples() {
            println!("{:?}", pce);
        }

    }

    #[test]
    fn null_move_eq() {
        let mut pos = Position::default();
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
    fn zobrist_equality() {
        let mut pos = Position::default();
        let zob_0 = pos.get_zobrist();
        for mv in MoveGen::get_pseudo_moves(&mut pos) {
            pos.make_move(mv);
            for mv in MoveGen::get_pseudo_moves(&mut pos) {
                pos.make_move(mv);
                for mv in MoveGen::get_pseudo_moves(&mut pos) {
                    pos.make_move(mv);
                    pos.unmake_move();
                }
                pos.unmake_move();
            }
            pos.unmake_move();
        };
        assert_eq!(zob_0, pos.get_zobrist())
    }
    
    #[test]
    fn castle_rights() {
        let mut test_rights = CastleRights::new();
        println!("{}",test_rights.can_player_castle_queenside(0));
        test_rights.disable_queenside_castle(0);
        println!("{}",test_rights.can_player_castle_queenside(0));
        println!("{}",test_rights.can_player_castle_kingside(0));
        test_rights.disable_kingside_castle(0);
        println!("{}",test_rights.can_player_castle_kingside(0));

    }
}

