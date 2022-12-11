use crate::constants::piece_scores::*;
use crate::types::{Player, Bitboard, BIndex, Centipawns, BDimensions};

pub type PieceId = u32;
pub type PieceIdWithPlayer = u64;

mod piece_definition;
mod piece_factory;
mod material_score;
mod positional_score;

pub use piece_factory::PieceFactory;
pub use piece_definition::PieceDefinition;

use material_score::compute_material_score;
use positional_score::compute_piece_square_table;

// Represents a piece type. Specific instances of this type are represented by a 1 in the bitboard
#[derive(Clone, Debug)]
pub struct Piece {
    // Info about this piece type
    type_def: PieceDefinition,
    //Player num for the owner of this piece
    player_num: Player,
    // Zobrist hashes for this piece at each board index
    zobrist_hashes: Vec<u64>,
    // Material score for this piece
    material_score: Centipawns,
    // Table of positional scores for this piece
    piece_square_table: Vec<Centipawns>,
    // TODO: Make private
    pub bitboard: Bitboard, // Occupancy bitboard
}

impl Piece {
    // Don't use new() directly, use PieceFactory instead
    fn new(definition: PieceDefinition, player_num: Player, dims: &BDimensions) -> Piece {
        let material_score = compute_material_score(&definition);
        let zobrist_hashes = Piece::compute_zobrist(definition.id, player_num);
        // TODO: Once the hardcoded pieces are removed, the piece_square_table can be computed directly
        let mut new_piece = Piece {
            type_def: definition,
            player_num,
            zobrist_hashes,
            material_score,
            piece_square_table: Vec::new(),
            bitboard: Bitboard::zero(),
        };
        new_piece.piece_square_table = compute_piece_square_table(&new_piece.type_def, dims, &new_piece);
        new_piece
    }
    
    // Get the full id of this piece (piece type + player_num)
    #[inline(always)]
    pub fn get_full_id(&self) -> PieceIdWithPlayer {
        self.type_def.id as PieceIdWithPlayer | (self.player_num as PieceIdWithPlayer) << 32
    }
    
    // TODO: Remove this??
    // Get the id of this piece (piece type only)
    #[inline(always)]
    pub fn get_piece_id(&self) -> PieceId {
        self.type_def.id
    }
    
    // Get a char representation of this piece
    #[inline(always)]
    pub fn char_rep(&self) -> char {
        self.type_def.char_rep
    }
    
    // Get the player number of this piece
    #[inline(always)]
    pub fn player_num(&self) -> Player {
        self.player_num
    }
    
    // Returns true if this piece is a leader (king)
    #[inline(always)]
    pub fn is_leader(&self) -> bool {
        self.type_def.is_leader
    }
    
    // Get the zobrist hash for this piece at the given index
    #[inline(always)]
    pub fn get_zobrist(&self, index: BIndex) -> u64 {
        self.zobrist_hashes[index as usize]
    }
    
    // Get the movement pattern for this piece
    // TODO: Remove this
    #[inline(always)]
    pub fn get_movement(&self) -> &PieceDefinition {
        &self.type_def
    }
    
    // Get the material score for 1 unit of this piece
    #[inline(always)]
    pub fn get_material_score(&self) -> Centipawns {
        // TODO: Remove this. First add an assert that the material score is correct
        match self.type_def.id {
            ID_PAWN => { PAWN_SCORE }
            ID_KNIGHT => { KNIGHT_SCORE }
            ID_BISHOP => { BISHOP_SCORE }
            ID_ROOK => { ROOK_SCORE }
            ID_QUEEN => { QUEEN_SCORE }
            ID_KING => { KING_SCORE }
            _ => { self.material_score }
        }
    }
    
    // Get the material score for all current units of this piece
    pub fn get_material_score_all(&self) -> Centipawns {
        self.material_score * self.bitboard.count_ones() as Centipawns
    }
    
    // Get the positional score for 1 unit of this piece at the given index
    pub fn get_positional_score(&self, index: BIndex) -> Centipawns {
        self.piece_square_table[index as usize]
    }
    
    // Get the positional score for all current units of this piece
    pub fn get_positional_score_all(&self) -> Centipawns {
        let mut bb_copy = self.bitboard.clone();
        let mut score = 0;
        while !bb_copy.is_zero() {
            let index = bb_copy.lowest_one().unwrap();
            score += self.get_positional_score(index);
            bb_copy.clear_bit(index);
        }
        score
    }
    
    // Helpers for getting the original id and the player_num from the id
    #[inline(always)]
    pub fn get_player_from_id(id: PieceIdWithPlayer) -> Player {
        (id >> 32) as Player
    }
    #[inline(always)]
    pub fn get_piecetype_from_id(id: PieceIdWithPlayer) -> PieceId {
        (id & 0xFFFFFFFF) as PieceId
    }
    
    fn compute_zobrist(id: PieceId, player_num: Player) -> Vec<u64> {
        let mut zobrist = Vec::with_capacity(256);
        for i in 0..=255 {
            zobrist.push(Piece::compute_zobrist_at(id, player_num, i));
        }
        zobrist
    }
    // TODO: Make private, use RNG
    pub fn compute_zobrist_at(id: PieceId, player_num: Player, index: BIndex) -> u64 {
        let seed = (id as u64) << 16 | (player_num as u64) << 8 | index as u64;
        
        // Use Donald Knuth's multiplicative hash
        const KNUTH_MUL: u64 = 6364136223846793005;
        // const KNUTH_ADD: u64 = 1442695040888963407; // Unused, since we only do 1 iteration
        seed.wrapping_mul(KNUTH_MUL)
    }
}

// Print as a string
impl std::fmt::Display for Piece {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        write!(f, "Piece {} (id={}, player={})", self.type_def.char_rep, self.type_def.id, self.player_num)
    }
}
