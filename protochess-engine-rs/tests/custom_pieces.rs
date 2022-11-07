extern crate protochess_engine_rs;

#[cfg(test)]
mod custom_pieces {
    use protochess_engine_rs::{MovementPatternExternal, PieceType};

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

        println!("{}", engine.get_zobrist());
        println!("BASE SCORE: {}", engine.get_score());
        engine.add_piece(0, PieceType::Custom('a'), 0, 3);
        println!("NEW SCORE: {}", engine.get_score());
        println!("{}", engine.to_string());


        let mut ply = 0;
        engine.play_best_move(1);
        ply += 1;
        println!("PLY: {} Engine plays: \n", ply);
        println!("{}", engine.to_string());
        println!("========================================");
    }

}
