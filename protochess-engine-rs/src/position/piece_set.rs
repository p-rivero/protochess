use std::slice::Iter;

//Pieces that a player has
use crate::types::{Bitboard, BIndex, Player, BDimensions, Centipawns};
use crate::piece::{Piece, PieceFactory};

use crate::constants::piece_scores::*;

/// Represents a set of pieces for a player
/// custom is a vec of custom piece
#[derive(Clone, Debug)]
pub struct PieceSet {
    pub occupied: Bitboard,
    // TODO: Treemap
    pub custom: Vec<Piece>,
    pub player_num: Player,
}

impl PieceSet {
    // TODO: Once the hardcoded pieces are removed, remove the BDimensions parameter
    pub fn new(player_num: Player, dims: &BDimensions) -> PieceSet {
        PieceSet {
            occupied: Bitboard::zero(),
            // TODO: Remove hardcoded pieces
            custom: vec![
                PieceFactory::make_king(ID_KING, player_num, dims),
                PieceFactory::make_queen(ID_QUEEN, player_num, dims),
                PieceFactory::make_bishop(ID_BISHOP, player_num, dims),
                PieceFactory::make_knight(ID_KNIGHT, player_num, dims),
                PieceFactory::make_rook(ID_ROOK, player_num, dims),
                PieceFactory::make_pawn(ID_PAWN, player_num, dims, vec![ID_QUEEN, ID_ROOK, ID_BISHOP, ID_KNIGHT]),
            ],
            player_num,
        }
    }

    pub fn piece_at(&self, index: BIndex) -> Option<&Piece> {
        self.custom.iter().find(|&p| p.bitboard.get_bit(index))
    }
    pub fn piece_at_mut(&mut self, index: BIndex) -> Option<&mut Piece> {
        self.custom.iter_mut().find(|p| p.bitboard.get_bit(index))
    }
    
    pub fn search_by_char(&mut self, c: char) -> Option<&mut Piece> {
        self.custom.iter_mut().find(|p| p.char_rep() == c)
    }

    // Returns an iterator over all pieces in the set
    pub fn get_piece_refs(&self) -> Iter<Piece> {
        self.custom.iter()
    }

    //Recomputes occupied bb
    pub fn update_occupied(&mut self) {
        self.occupied = Bitboard::zero();
        for p in &self.custom {
            self.occupied |= &p.bitboard;
        }
    }
    
    // Returns the material score of all pieces in the set, and of only the leader pieces
    pub fn get_material_score(&self) -> (Centipawns, Centipawns) {
        let mut score = 0;
        let mut leader_score = 0;
        for piece in &self.custom {
            let piece_total_score = piece.get_material_score_all();
            score += piece_total_score;
            if piece.is_leader() {
                leader_score += piece_total_score;
            }
        }
        (score, leader_score)
    }
    
    pub fn get_positional_score(&self, is_endgame: bool, ) -> Centipawns {
        let mut score = 0;
        for piece in &self.custom {
            // TODO: Inverting the leader score may not always be the best option
            // For each piece, get the positional score.
            // Invert the leader so that it stays away from the center, except in the endgame
            if piece.is_leader() && !is_endgame {
                score -= piece.get_positional_score_all();
            } else {
                score += piece.get_positional_score_all();
            }
        }
        score
    }
}