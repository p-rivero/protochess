use std::slice::{Iter, IterMut};

use crate::PieceDefinition;
//Pieces that a player has
use crate::types::{Bitboard, BIndex, Player, BDimensions, Centipawns, BCoord};
use crate::piece::{Piece, PieceId};
use crate::utils::from_index;

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
    inverse_attack_jumps: Vec<Bitboard>,
}

impl PieceSet {
    pub fn new(player_num: Player) -> PieceSet {
        let mut inverse_attack_jumps = Vec::with_capacity(256);
        for _ in 0..256 {
            inverse_attack_jumps.push(Bitboard::zero());
        }
        PieceSet {
            occupied: Bitboard::zero(),
            pieces: Vec::new(),
            leader_piece_index: -1,
            player_num,
            inverse_attack: PieceDefinition::default(),
            inverse_attack_jumps,
        }
    }
    
    pub fn register_piecetype(&mut self, definition: PieceDefinition, dims: &BDimensions) {
        // Update the inverse movement pattern
        self.update_inverse_attack(&definition, dims);
        
        if definition.is_leader {
            assert!(self.leader_piece_index == -1, "Only one leader piece per player");
            self.leader_piece_index = self.pieces.len() as isize;
        }
        for p in &self.pieces {
            assert!(p.get_piece_id() != definition.id, "There is already a piece with this id");
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
    pub fn rook_at_mut(&mut self, index: BIndex) -> Option<&mut Piece> {
        self.pieces.iter_mut().find(|p| p.is_rook() && p.is_at_index(index))
    }
    
    pub fn search_by_char(&mut self, c: char) -> Option<&mut Piece> {
        self.pieces.iter_mut().find(|p| p.char_rep() == c)
    }
    
    pub fn search_by_id(&self, id: PieceId) -> Option<&Piece> {
        self.pieces.iter().find(|p| p.get_piece_id() == id)
    }
    
    pub fn get_leader(&self) -> Option<&Piece> {
        if self.leader_piece_index == -1 {
            return None;
        }
        self.pieces.get(self.leader_piece_index as usize)
    }
    
    pub fn get_inverse_attack(&self, index: BIndex) -> (&PieceDefinition, &Bitboard) {
        (&self.inverse_attack, &self.inverse_attack_jumps[index as usize])
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
    
    
    fn update_inverse_attack(&mut self, other: &PieceDefinition, dims: &BDimensions) {
        self.inverse_attack.attack_north |= other.attack_south;
        self.inverse_attack.attack_south |= other.attack_north;
        self.inverse_attack.attack_east |= other.attack_west;
        self.inverse_attack.attack_west |= other.attack_east;
        self.inverse_attack.attack_northeast |= other.attack_southwest;
        self.inverse_attack.attack_northwest |= other.attack_southeast;
        self.inverse_attack.attack_southeast |= other.attack_northwest;
        self.inverse_attack.attack_southwest |= other.attack_northeast;
        
        for delta in &other.attack_jump_deltas {
            self.inverse_attack.attack_jump_deltas.push((-delta.0, -delta.1));
            
            for i in 0..=255 {
                let (x, y) = from_index(i);
                let nx = x as i8 - delta.0;
                let ny = y as i8 - delta.1;
                if nx < 0 || ny < 0 || !dims.in_bounds(nx as BCoord, ny as BCoord) {
                    continue;
                }
                self.inverse_attack_jumps[i as usize].set_bit_at(nx as BCoord, ny as BCoord);
            }
        }
        
        for delta in &other.attack_sliding_deltas {
            let mut new_delta = Vec::new();
            for (x, y) in delta {
                new_delta.push((-x, -y));
            }
            self.inverse_attack.attack_sliding_deltas.push(new_delta);
        }
    }
}