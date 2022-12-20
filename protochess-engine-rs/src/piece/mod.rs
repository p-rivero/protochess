use crate::move_generator::bitboard_moves::BitboardMoves;
use crate::types::*;

pub type PieceId = u32;

mod piece_definition;
mod piece_factory;
mod material_score;
mod positional_score;
mod movement;

pub use piece_factory::PieceFactory;
pub use piece_definition::PieceDefinition;

use material_score::compute_material_score;
use positional_score::compute_piece_square_table;
use movement::output_moves;

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
    // Number of 1 bits in the bitboard
    num_pieces: u32,
    // num_pieces * material_score
    total_material_score: Centipawns,
}

impl Piece {
    // Don't use new() directly, use PieceFactory instead
    fn new(mut definition: PieceDefinition, player_num: Player, dims: &BDimensions) -> Piece {
        // Make sure that all promotion squares are in bounds
        definition.promotion_squares &= &dims.bounds;
        assert!(&definition.promotion_squares & !&dims.bounds == Bitboard::zero());
        
        let material_score = compute_material_score(&definition);
        let zobrist_hashes = Piece::compute_zobrist(definition.id, player_num);
        let piece_square_table = compute_piece_square_table(&definition, dims);
        Piece {
            type_def: definition,
            player_num,
            zobrist_hashes,
            material_score,
            piece_square_table,
            bitboard: Bitboard::zero(),
            num_pieces: 0,
            total_material_score: 0,
        }
    }
    
    // Get the id of this piece (piece type only)
    pub fn get_piece_id(&self) -> PieceId {
        self.type_def.id
    }
    
    // Get a char representation of this piece
    pub fn char_rep(&self) -> char {
        self.type_def.char_rep
    }
    
    // Get the player number of this piece
    pub fn get_player(&self) -> Player {
        self.player_num
    }
    
    // Returns true if this piece is a leader (king)
    pub fn is_leader(&self) -> bool {
        self.type_def.is_leader
    }
    
    // Returns true if capturing this piece causes the capturing player to win
    pub fn potential_checkmate(&self) -> bool {
        self.type_def.is_leader && self.num_pieces == 1
    }
    
    // Get the zobrist hash for this piece at the given index
    pub fn get_zobrist(&self, index: BIndex) -> u64 {
        self.zobrist_hashes[index as usize]
    }
    
    // Get the material score for 1 unit of this piece
    pub fn get_material_score(&self) -> Centipawns {
        self.material_score
    }
    
    pub fn move_piece(&mut self, from: BIndex, to: BIndex) {
        self.bitboard.clear_bit(from);
        self.bitboard.set_bit(to);
    }
    
    pub fn add_piece(&mut self, index: BIndex) {
        self.bitboard.set_bit(index);
        self.num_pieces += 1;
        self.total_material_score += self.material_score;
    }
    
    pub fn remove_piece(&mut self, index: BIndex) {
        self.bitboard.clear_bit(index);
        self.num_pieces -= 1;
        self.total_material_score -= self.material_score;
    }
    
    // Get the material score for all current units of this piece
    pub fn get_material_score_all(&self) -> Centipawns {
        self.total_material_score
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
    
    pub fn output_moves(&self, dims: &BDimensions,
            occupied: &Bitboard, enemies: &Bitboard, occ_or_not_in_bounds: &Bitboard,
            out_bb_moves: &mut Vec<BitboardMoves>, out_moves: &mut Vec<Move>)
    {
        let mut bb_copy = self.bitboard.clone();
        while !bb_copy.is_zero() {
            let index = bb_copy.lowest_one().unwrap();
            output_moves(&self.type_def, index, dims, occupied, enemies, occ_or_not_in_bounds, out_bb_moves, out_moves);
            bb_copy.clear_bit(index);
        }
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
