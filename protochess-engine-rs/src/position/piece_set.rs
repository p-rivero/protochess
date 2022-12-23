use std::slice::{Iter, IterMut};

use crate::PieceDefinition;
//Pieces that a player has
use crate::types::{Bitboard, BIndex, Player, BDimensions, Centipawns};
use crate::piece::Piece;

/// Represents a set of pieces for a player
/// custom is a vec of custom piece
#[derive(Clone, Debug)]
pub struct PieceSet {
    pieces: Vec<Piece>,
    leader_piece_index: isize,
    occupied: Bitboard,
    player_num: Player,
    // Inverse attack pattern of all the pieces in the set
    inverse_attack: PieceDefinition,
}

impl PieceSet {
    pub fn new(player_num: Player) -> PieceSet {
        PieceSet {
            occupied: Bitboard::zero(),
            pieces: Vec::new(),
            leader_piece_index: -1,
            player_num,
            inverse_attack: PieceDefinition::default(),
        }
    }
    
    pub fn register_piecetype(&mut self, definition: PieceDefinition, dims: &BDimensions) {
        // Update the inverse movement pattern
        self.inverse_attack.update_inverse_attack(&definition);
        
        if definition.is_leader {
            assert!(self.leader_piece_index == -1, "Only one leader piece per player");
            self.leader_piece_index = self.pieces.len() as isize;
        }
        
        let piece = Piece::new(definition, self.player_num, dims);
        self.pieces.push(piece);
    }
    
    pub fn iter(&self) -> Iter<Piece> {
        self.pieces.iter()
    }
    pub fn iter_mut(&mut self) -> IterMut<Piece> {
        self.pieces.iter_mut()
    }
    
    pub fn get_occupied(&self) -> &Bitboard {
        &self.occupied
    }
    
    pub fn get_player_num(&self) -> Player {
        self.player_num
    }

    pub fn piece_at(&self, index: BIndex) -> Option<&Piece> {
        self.pieces.iter().find(|&p| p.is_at_index(index))
    }
    pub fn piece_at_mut(&mut self, index: BIndex) -> Option<&mut Piece> {
        self.pieces.iter_mut().find(|p| p.is_at_index(index))
    }
    
    pub fn search_by_char(&mut self, c: char) -> Option<&mut Piece> {
        self.pieces.iter_mut().find(|p| p.char_rep() == c)
    }
    
    pub fn get_leader(&self) -> &Piece {
        assert!(self.leader_piece_index >= 0);
        &self.pieces[self.leader_piece_index as usize]
    }
    
    pub fn get_inverse_attack(&self) -> &PieceDefinition {
        &self.inverse_attack
    }

    //Recomputes occupied bb
    pub fn update_occupied(&mut self) {
        self.occupied = Bitboard::zero();
        for p in &self.pieces {
            self.occupied |= p.get_bitboard();
        }
    }
    
    // Returns the material score of all pieces in the set, and of only the leader pieces
    pub fn get_material_score(&self) -> (Centipawns, Centipawns) {
        let mut score = 0;
        let mut leader_score = 0;
        for piece in &self.pieces {
            let piece_total_score = piece.get_material_score_all();
            score += piece_total_score;
            if piece.is_leader() {
                leader_score += piece_total_score;
            }
        }
        (score, leader_score)
    }
    
    pub fn get_positional_score(&self, is_endgame: bool) -> Centipawns {
        let mut score = 0;
        for piece in &self.pieces {
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