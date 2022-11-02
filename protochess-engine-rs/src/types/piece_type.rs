
// Class for storing the value of a position, must allow negative values
pub type Centipawns = isize;

#[derive(Clone, PartialEq, Eq, Hash, Debug)]
pub enum PieceType {
    King,
    Queen,
    Rook,
    Bishop,
    Knight,
    Pawn,
    Custom(char),
}

impl PieceType {
    pub fn from_char(c:char) -> PieceType {
        match c.to_ascii_lowercase() {
            'k' =>{PieceType::King}
            'q' =>{PieceType::Queen}
            'r' =>{PieceType::Rook}
            'b' =>{PieceType::Bishop}
            'n' =>{PieceType::Knight}
            'p' =>{PieceType::Pawn}
            _ => {PieceType::Custom(c)}
        }
    }
}
