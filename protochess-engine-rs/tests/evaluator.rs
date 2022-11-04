#[cfg(test)]
mod eval_test {
    use protochess_engine_rs::evaluator::Evaluator;
    use protochess_engine_rs::position::parse_fen;
    use protochess_engine_rs::move_generator::MoveGenerator;

    #[test]
    fn test() {
        let mut eval = Evaluator::new();
        let movegen = MoveGenerator::new();
        let mut pos = parse_fen("rnbqkbnr/pppppppp/8/8/8/3PP3/PPP2PPP/RNBQKBNR w KQkq - 0 1".parse().unwrap());
        println!("{}", eval.evaluate(&mut pos, &movegen));
    }
}
