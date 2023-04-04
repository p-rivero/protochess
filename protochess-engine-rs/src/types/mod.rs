mod bitboard;
mod chess_move;
mod searcher;

use std::convert::TryFrom;

pub use bitboard::*;
pub use chess_move::*;
pub use searcher::*;

use crate::{wrap_res, err};



#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[must_use]
pub enum GameMode {
    Standard,
    Atomic,
    Horde,
    Antichess,
    KingOfTheHill,
    RacingKings,
    ThreeCheck,
    FiveCheck,
}

impl TryFrom<&str> for GameMode {
    type Error = String;
    fn try_from(value: &str) -> wrap_res!(Self) {
        match value.to_lowercase().as_str() {
            "standard" => Ok(GameMode::Standard),
            "atomic" => Ok(GameMode::Atomic),
            "horde" => Ok(GameMode::Horde),
            "antichess" => Ok(GameMode::Antichess),
            "kingofthehill" => Ok(GameMode::KingOfTheHill),
            "racingkings" => Ok(GameMode::RacingKings),
            "3check" => Ok(GameMode::ThreeCheck),
            "5check" => Ok(GameMode::FiveCheck),
            _ => err!("Invalid game mode '{value}'"),
        }
    }
}

impl std::fmt::Display for GameMode {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            GameMode::Standard => write!(f, "Standard"),
            GameMode::Atomic => write!(f, "Atomic"),
            GameMode::Horde => write!(f, "Horde"),
            GameMode::Antichess => write!(f, "Antichess"),
            GameMode::KingOfTheHill => write!(f, "King of the Hill"),
            GameMode::RacingKings => write!(f, "Racing Kings"),
            GameMode::ThreeCheck => write!(f, "Three-check"),
            GameMode::FiveCheck => write!(f, "Five-check"),
        }
    }
}