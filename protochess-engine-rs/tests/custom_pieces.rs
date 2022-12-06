extern crate protochess_engine_rs;

#[cfg(test)]
mod custom_pieces {
    use protochess_engine_rs::{MovementPatternExternal, position::piece::PieceId};

    #[test]
    fn custom_pieces() {
        let mut engine = protochess_engine_rs::Engine::default();

        // Queen
        engine.register_piecetype('a', MovementPatternExternal {
            promotion_squares: None,
            promo_vals: None,
            attack_sliding_deltas: vec![],
            attack_jump_deltas: vec![],
            attack_north: true,
            attack_south: true,
            attack_east: true,
            attack_west: true,
            attack_northeast: true,
            attack_northwest: true,
            attack_southeast: true,
            attack_southwest: true,
            translate_jump_deltas: vec![],
            translate_sliding_deltas: vec![],
            translate_north: true,
            translate_south: true,
            translate_east: true,
            translate_west: true,
            translate_northeast: true,
            translate_northwest: true,
            translate_southeast: true,
            translate_southwest: true
        });
        
        // Initial score should be 0
        assert_eq!(engine.get_score(), 0);
        // Add a queen to the board
        engine.add_piece(0, 100 + 'a' as PieceId, 0, 3);
        let queen_material = 1040;
        let queen_position = 10;
        assert_eq!(engine.get_score(), queen_material + queen_position);
    }

}
