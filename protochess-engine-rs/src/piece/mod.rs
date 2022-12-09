use crate::types::{Player, Bitboard, BIndex};

pub type PieceId = u32;
pub type PieceIdWithPlayer = u64;

mod piece_definition;
mod piece_factory;
pub mod evaluator;

pub use piece_factory::PieceFactory;
pub use piece_definition::PieceDefinition;

// Represents a piece type. Specific instances of this type are represented by a 1 in the bitboard
#[derive(Clone, Debug)]
pub struct Piece {
    // Info about this piece type
    type_def: PieceDefinition,
    //Player num for the owner of this piece
    player_num: Player,
    // Zobrist hashes for this piece at each board index
    zobrist_hashes: Vec<u64>,
    // TODO: Make private
    pub bitboard: Bitboard, // Occupancy bitboard
}

impl Piece {
    // Don't use new() directly, use PieceFactory instead
    fn new(definition: PieceDefinition, player_num: Player) -> Piece {
        let id = definition.id;
        Piece {
            type_def: definition,
            player_num,
            zobrist_hashes: Piece::compute_zobrist(id, player_num),
            bitboard: Bitboard::zero(),
        }
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
