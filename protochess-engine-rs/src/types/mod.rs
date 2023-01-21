mod bitboard;
mod chess_move;
mod searcher;

pub use bitboard::*;
pub use chess_move::*;
pub use searcher::*;



#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum GameMode {
    Standard,
    Atomic,
    Horde,
}

impl From<&str> for GameMode {
    fn from(s: &str) -> GameMode {
        match s.to_ascii_lowercase().as_str() {
            "standard" => GameMode::Standard,
            "atomic" => GameMode::Atomic,
            "horde" => GameMode::Horde,
            _ => panic!("Invalid game mode: {}", s)
        }
    }
}
