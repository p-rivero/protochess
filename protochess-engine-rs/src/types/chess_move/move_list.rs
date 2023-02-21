use crate::types::BCoord;

use super::move_info::MoveInfo;

// A list of moves that can be done from a given square
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct MoveList {
    pub x: BCoord, 
    pub y: BCoord,
    pub moves: Vec<MoveInfo>
}