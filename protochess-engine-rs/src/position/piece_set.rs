use std::slice::{Iter, IterMut};

use crate::utils::debug::eq_anyorder;
use crate::{PieceDefinition, wrap_res, err_assert};
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
    
    piece_at_index: [isize; 256],
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
            piece_at_index: [-1; 256],
        }
    }
    
    /// Add a new piece definition to the set. The following conditions must have been checked before calling this function:
    /// - Both ids (white and black) are unique among all pieces in all sets (so that it is possible to uniquely identify a piece)
    /// - This piece is available for the player (i.e. `ids[player_num]` is not `None`)
    pub fn register_piecetype(&mut self, definition: PieceDefinition, dims: &BDimensions) -> wrap_res!() {
        // Update the inverse movement pattern
        self.update_inverse_attack(&definition, dims);
        if definition.is_leader {
            err_assert!(self.leader_piece_index == -1, "Cannot have more than one leader piece per player");
            self.leader_piece_index = self.pieces.len() as isize;
        }
        let piece = Piece::new(definition, self.player_num, dims);
        self.pieces.push(piece);
        Ok(())
    }
    pub fn assert_promotion_consistency(&self) -> wrap_res!() {
        for piece in &self.pieces {
            for promotion in &piece.get_movement().promo_vals[self.player_num as usize] {
                err_assert!(self.contains_piece(*promotion), "Piece {piece} promotes to {promotion}, which does not exist");
            }
        }
        Ok(())
    }
    
    pub fn contains_piece(&self, piece_id: PieceId) -> bool {
        self.pieces.iter().any(|p| p.get_piece_id() == piece_id)
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

    pub fn index_has_piece(&self, index: BIndex) -> bool {
        self.piece_at_index[index as usize] != -1
    }
    
    pub fn piece_at(&self, index: BIndex) -> Option<&Piece> {
        let piece_index = self.piece_at_index[index as usize];
        if piece_index == -1 {
            None
        } else {
            Some(&self.pieces[piece_index as usize])
        }
    }
    pub fn piece_at_mut(&mut self, index: BIndex) -> Option<&mut Piece> {
        let piece_index = self.piece_at_index[index as usize];
        if piece_index == -1 {
            None
        } else {
            Some(&mut self.pieces[piece_index as usize])
        }
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

    /// Recomputes occupied bb
    pub fn update_occupied(&mut self) {
        self.occupied = Bitboard::zero();
        for p in &self.pieces {
            self.occupied |= p.get_bitboard();
        }
    }
    
    /// Moves a piece from one index to another.
    /// If `set_can_castle` is true, set the new index as a castle square.
    /// Returns true if the piece could castle before this move
    /// Always use this function to move pieces, never call `piece.move_piece()` directly.
    pub fn move_piece(&mut self, from: BIndex, to: BIndex, set_can_castle: bool) -> bool {
        let piece_index = self.piece_at_index[from as usize];
        let piece = &mut self.pieces[piece_index as usize];
        let could_castle = piece.move_piece_(from, to, set_can_castle);
        self.piece_at_index[from as usize] = -1;
        self.piece_at_index[to as usize] = piece_index;
        could_castle
    }
    
    /// Add a piece to a given piece type at a given index (assuming the piece type exists).
    /// Always use this function to add pieces, never call `piece.add_piece()` directly.
    pub fn add_piece(&mut self, piece_id: PieceId, index: BIndex, set_can_castle: bool) {
        let piece_index = self.pieces.iter().position(|p| p.get_piece_id() == piece_id).unwrap();
        let piece = &mut self.pieces[piece_index];
        piece.add_piece_(index, set_can_castle);
        self.piece_at_index[index as usize] = piece_index as isize;
    }
    
    /// Remove a piece from a given index (assuming there is a piece there).
    /// Returns true if the piece could castle before this move
    /// Always use this function to remove pieces, never call `piece.remove_piece()` directly.
    pub fn remove_piece(&mut self, index: BIndex) -> bool {
        let piece_index = self.piece_at_index[index as usize];
        let piece = &mut self.pieces[piece_index as usize];
        let could_castle = piece.remove_piece_(index);
        self.piece_at_index[index as usize] = -1;
        could_castle
    }
    
    /// Returns the material score of all pieces in the set, and of only the leader pieces
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
    
    #[inline]
    pub fn get_positional_score<const ENDGAME: bool>(&self) -> Centipawns {
        let mut score = 0;
        for piece in &self.pieces {
            score += piece.get_positional_score_all::<ENDGAME>();
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

impl PartialEq for PieceSet {
    fn eq(&self, other: &Self) -> bool {
        eq_anyorder(&self.pieces, &other.pieces) &&
        self.occupied == other.occupied &&
        self.player_num == other.player_num &&
        self.inverse_attack.eq_ignore_order(&other.inverse_attack) &&
        self.inverse_attack_jumps == other.inverse_attack_jumps && {
            for pos in 0..256 {
                let i1 = self.piece_at_index[pos];
                let i2 = other.piece_at_index[pos];
                if i1 == -1 && i2 == -1 { continue; }
                if i1 == -1 || i2 == -1 { return false; }
                if self.pieces[i1 as usize] != other.pieces[i2 as usize] {
                    return false;
                }
            }
            true
        } && {
            let i1 = self.leader_piece_index;
            let i2 = other.leader_piece_index;
            if i1 == -1 && i2 == -1 { true }
            else if i1 == -1 || i2 == -1 { false }
            else { self.pieces[i1 as usize] == other.pieces[i2 as usize] }
        }
    }
}
