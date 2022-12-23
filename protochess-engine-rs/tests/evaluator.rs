#[cfg(test)]
mod eval_test {
    use protochess_engine_rs::Position;
    use protochess_engine_rs::searcher::eval;

    #[test]
    fn test() {
        let mut pos = Position::from_fen("rnbqkbnr/pppppppp/8/8/8/3PP3/PPP2PPP/RNBQKBNR w KQkq - 0 1");
        // TODO: Make more tests
        assert_eq!(eval::evaluate(&mut pos), 30);
    }
}
