mod bitboard;
mod chess_move;
mod searcher;

use std::convert::TryFrom;

pub use bitboard::*;
pub use chess_move::*;
pub use searcher::*;



#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum GameMode {
    Standard,
    Atomic,
    Horde,
    Antichess,
    KingOfTheHill,
    RacingKings,
    ThreeCheck,
}

impl TryFrom<&str> for GameMode {
    type Error = &'static str;
    fn try_from(value: &str) -> Result<Self, Self::Error> {
        match value.to_ascii_lowercase().as_str() {
            "standard" => Ok(GameMode::Standard),
            "atomic" => Ok(GameMode::Atomic),
            "horde" => Ok(GameMode::Horde),
            "antichess" => Ok(GameMode::Antichess),
            "kingofthehill" => Ok(GameMode::KingOfTheHill),
            "racingkings" => Ok(GameMode::RacingKings),
            "3check" => Ok(GameMode::ThreeCheck),
            _ => Err("Invalid game mode"),
        }
    }
}
