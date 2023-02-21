use crate::types::{Player, BCoord};

#[derive(Debug, PartialEq, Eq, Clone)]
#[must_use]
pub enum MakeMoveResultFlag {
    Ok,
    IllegalMove,
    Checkmate,
    LeaderCaptured,
    PieceInWinSquare,
    CheckLimit,
    Stalemate,
    Repetition,
}

#[derive(Debug, PartialEq, Eq, Clone)]
pub enum MakeMoveResultWinner {
    White,
    Black,
    None,
}

#[derive(Debug, PartialEq, Eq, Clone)]
#[must_use]
pub struct MakeMoveResult {
    pub flag: MakeMoveResultFlag,
    pub winner: MakeMoveResultWinner,
    pub exploded: Vec<(BCoord, BCoord)>,
}



impl From<MakeMoveResultFlag> for String {
    fn from(f: MakeMoveResultFlag) -> Self {
        format!("{:?}", f)
    }
}
impl From<String> for MakeMoveResultFlag {
    fn from(s: String) -> Self {
        match s.as_str() {
            "Ok" => Self::Ok,
            "IllegalMove" => Self::IllegalMove,
            "Checkmate" => Self::Checkmate,
            "LeaderCaptured" => Self::LeaderCaptured,
            "PieceInWinSquare" => Self::PieceInWinSquare,
            "CheckLimit" => Self::CheckLimit,
            "Stalemate" => Self::Stalemate,
            "Repetition" => Self::Repetition,
            _ => panic!("Invalid flag"),
        }
    }
}


impl From<Option<Player>> for MakeMoveResultWinner {
    fn from(p: Option<Player>) -> Self {
        match p {
            Some(0) => Self::White,
            Some(1) => Self::Black,
            None => Self::None,
            _ => panic!("Invalid player"),
        }
    }
}
impl From<MakeMoveResultWinner> for String {
    fn from(w: MakeMoveResultWinner) -> Self {
        format!("{:?}", w)
    }
}
impl From<String> for MakeMoveResultWinner {
    fn from(s: String) -> Self {
        match s.as_str() {
            "White" => Self::White,
            "Black" => Self::Black,
            "None" => Self::None,
            _ => panic!("Invalid winner"),
        }
    }
}


impl MakeMoveResult {
    pub fn ok(exploded: Vec<(BCoord, BCoord)>) -> Self {
        Self {
            flag: MakeMoveResultFlag::Ok,
            winner: None.into(),
            exploded,
        }
    }
    pub fn illegal_move() -> Self {
        Self {
            flag: MakeMoveResultFlag::IllegalMove,
            winner: None.into(),
            exploded: Vec::new(),
        }
    }
    pub fn checkmate(winner: Player, exploded: Vec<(BCoord, BCoord)>) -> Self {
        Self {
            flag: MakeMoveResultFlag::Checkmate,
            winner: Some(winner).into(),
            exploded,
        }
    }
    pub fn leader_captured(winner: Player, exploded: Vec<(BCoord, BCoord)>) -> Self {
        Self {
            flag: MakeMoveResultFlag::LeaderCaptured,
            winner: Some(winner).into(),
            exploded,
        }
    }
    pub fn piece_in_win_square(winner: Player, exploded: Vec<(BCoord, BCoord)>) -> Self {
        Self {
            flag: MakeMoveResultFlag::PieceInWinSquare,
            winner: Some(winner).into(),
            exploded,
        }
    }
    pub fn check_limit(winner: Player, exploded: Vec<(BCoord, BCoord)>) -> Self {
        Self {
            flag: MakeMoveResultFlag::CheckLimit,
            winner: Some(winner).into(),
            exploded,
        }
    }
    pub fn stalemate(winner: Option<Player>, exploded: Vec<(BCoord, BCoord)>) -> Self {
        Self {
            flag: MakeMoveResultFlag::Stalemate,
            winner: winner.into(),
            exploded,
        }
    }
    pub fn repetition() -> Self {
        Self {
            flag: MakeMoveResultFlag::Repetition,
            winner: None.into(),
            // Since this is a repetition, this move cannot be a capture, so there is no explosion
            exploded: Vec::new(),
        }
    }
}