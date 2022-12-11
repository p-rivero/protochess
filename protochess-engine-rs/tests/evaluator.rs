#[cfg(test)]
mod eval_test {
    use protochess_engine_rs::piece::evaluator::Evaluator;
    use protochess_engine_rs::position::parse_fen;

    #[test]
    fn test() {
        let mut pos = parse_fen("rnbqkbnr/pppppppp/8/8/8/3PP3/PPP2PPP/RNBQKBNR w KQkq - 0 1".parse().unwrap());
        // TODO: Make more tests
        assert_eq!(Evaluator::evaluate(&mut pos), 20);
    }
}
