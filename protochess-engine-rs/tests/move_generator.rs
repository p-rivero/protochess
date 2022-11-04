#[cfg(test)]
mod move_generator_test {
    use protochess_engine_rs::move_generator::attack_tables::AttackTables;
    use protochess_engine_rs::move_generator::MoveGenerator;
    use protochess_engine_rs::position::parse_fen;
    use protochess_engine_rs::types::Bitboard;

    #[test]
    fn capture_moves() {
        let mut pos = parse_fen("rnb1kbnr/ppppqppp/8/8/5P2/8/PPPP1PPP/RNBQKBNR w KQkq - 0 1".parse().unwrap());
        let movegen = MoveGenerator::new();
        println!("{}",pos.get_zobrist());
        println!("{}", movegen.in_check(&mut pos));
        println!("{}",pos.get_zobrist());
        for mv in movegen.get_capture_moves(&mut pos) {
            println!("{}", mv);
            assert!(mv.get_is_capture());
        }
    }

    #[test]
    fn attack_tables() {
        let _attacktb = AttackTables::new();
        let mut bb = Bitboard::zero();
        bb |= 9252345218324798u64;

        // println!("occ \n{}", to_string(&bb));
        // let rankatt = _attacktb.get_rank_attack(2,&bb);
        // println!("{}", to_string(&rankatt));

    }
}
