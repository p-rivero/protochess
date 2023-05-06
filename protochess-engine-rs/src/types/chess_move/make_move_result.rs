use crate::types::{Player, BCoord};

#[derive(Debug, PartialEq, Eq, Clone)]
#[must_use]
pub enum MakeMoveResultFlag {
    Ok,
    IllegalMove,
    Checkmate,
    LeaderCaptured,
    AllPiecesCaptured,
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
    /// The result of attempting the move
    pub flag: MakeMoveResultFlag,
    /// Contains the winner of the game. If `None`, the game is still ongoing or ended in a draw
    pub winner: MakeMoveResultWinner,
    /// List of board coordinates that exploded
    pub exploded: Vec<(BCoord, BCoord)>,
    /// If `flag != IllegalMove`, contains the move in algebraic notation
    pub move_notation: Option<String>,
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
            "AllPiecesCaptured" => Self::AllPiecesCaptured,
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
        match w {
            MakeMoveResultWinner::White => "white",
            MakeMoveResultWinner::Black => "black",
            MakeMoveResultWinner::None => "none",
        }.to_string()
    }
}
impl From<String> for MakeMoveResultWinner {
    fn from(s: String) -> Self {
        match s.as_str() {
            "white" => Self::White,
            "black" => Self::Black,
            "none" => Self::None,
            _ => panic!("Invalid winner"),
        }
    }
}


impl MakeMoveResult {
    pub fn ok(exploded: Vec<(BCoord, BCoord)>, move_notation: String) -> Self {
        Self {
            flag: MakeMoveResultFlag::Ok,
            winner: None.into(),
            exploded,
            move_notation: Some(move_notation),
        }
    }
    pub fn illegal_move() -> Self {
        Self {
            flag: MakeMoveResultFlag::IllegalMove,
            winner: None.into(),
            exploded: Vec::new(),
            move_notation: None,
        }
    }
    pub fn checkmate(winner: Player, exploded: Vec<(BCoord, BCoord)>, move_notation: String) -> Self {
        Self {
            flag: MakeMoveResultFlag::Checkmate,
            winner: Some(winner).into(),
            exploded,
            move_notation: Some(move_notation),
        }
    }
    pub fn leader_captured(winner: Player, exploded: Vec<(BCoord, BCoord)>, move_notation: String) -> Self {
        Self {
            flag: MakeMoveResultFlag::LeaderCaptured,
            winner: Some(winner).into(),
            exploded,
            move_notation: Some(move_notation),
        }
    }
    pub fn all_pieces_captured(winner: Player, exploded: Vec<(BCoord, BCoord)>, move_notation: String) -> Self {
        Self {
            flag: MakeMoveResultFlag::AllPiecesCaptured,
            winner: Some(winner).into(),
            exploded,
            move_notation: Some(move_notation),
        }
    }
    pub fn piece_in_win_square(winner: Player, exploded: Vec<(BCoord, BCoord)>, move_notation: String) -> Self {
        Self {
            flag: MakeMoveResultFlag::PieceInWinSquare,
            winner: Some(winner).into(),
            exploded,
            move_notation: Some(move_notation),
        }
    }
    pub fn check_limit(winner: Player, exploded: Vec<(BCoord, BCoord)>, move_notation: String) -> Self {
        Self {
            flag: MakeMoveResultFlag::CheckLimit,
            winner: Some(winner).into(),
            exploded,
            move_notation: Some(move_notation),
        }
    }
    pub fn stalemate(winner: Option<Player>, exploded: Vec<(BCoord, BCoord)>, move_notation: String) -> Self {
        Self {
            flag: MakeMoveResultFlag::Stalemate,
            winner: winner.into(),
            exploded,
            move_notation: Some(move_notation),
        }
    }
    pub fn repetition(move_notation: String) -> Self {
        Self {
            flag: MakeMoveResultFlag::Repetition,
            winner: None.into(),
            // Since this is a repetition, this move cannot be a capture, so there is no explosion
            exploded: Vec::new(),
            move_notation: Some(move_notation),
        }
    }
}