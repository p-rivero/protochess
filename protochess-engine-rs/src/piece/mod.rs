use rand::rngs::StdRng;
use rand::{SeedableRng, Rng};

use crate::move_generator::bitboard_moves::BitboardMoves;
use crate::{types::*, Position};

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
use movement::{output_translations, output_captures};

// Represents a piece type. Specific instances of this type are represented by a 1 in the bitboard
#[derive(Clone, Debug)]
pub struct Piece {
    // Info about this piece type
    type_def: PieceDefinition,
    // Occupancy bitboard
    bitboard: Bitboard,
    //Player num for the owner of this piece
    player_num: Player,
    // Zobrist hashes for this piece at each board index
    zobrist_hashes: Vec<u64>,
    
    // Material score for this piece
    material_score: Centipawns,
    // Table of positional scores for this piece
    piece_square_table: Vec<Centipawns>,
    
    // Number of 1 bits in the bitboard
    num_pieces: u32,
    // num_pieces * material_score
    total_material_score: Centipawns,
    
    // Positions at which this piece can castle. Used if can_castle or is_castle_rook are true
    castle_squares: Bitboard,
}

impl Piece {
    pub fn new(mut definition: PieceDefinition, player_num: Player, dims: &BDimensions) -> Piece {
        // Make sure that all promotion squares are in bounds
        definition.promotion_squares &= &dims.bounds;
        assert!(&definition.promotion_squares & !&dims.bounds == Bitboard::zero());
        // Cannot be castle rook and can castle at the same time
        assert!(!(definition.can_castle && definition.is_castle_rook));
        
        let material_score = compute_material_score(&definition);
        let zobrist_hashes = Piece::random_zobrist(definition.id, player_num);
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
            castle_squares: Bitboard::zero(),
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
    
    // Access to the bitboard
    pub fn get_bitboard(&self) -> &Bitboard {
        &self.bitboard
    }
    pub fn is_at_index(&self, index: BIndex) -> bool {
        self.bitboard.get_bit(index)
    }
    // Get the indexes of all pieces of this type
    pub fn get_indexes(&self) -> Vec<BIndex> {
        let mut bb_copy = self.bitboard.clone();
        let mut indexes = Vec::new();
        while !bb_copy.is_zero() {
            let index = bb_copy.lowest_one().unwrap();
            bb_copy.clear_bit(index);
            indexes.push(index);
        }
        indexes
    }
    pub fn get_first_index(&self) -> Option<BIndex> {
        self.bitboard.lowest_one()
    }
    
    // Returns true if this piece is a leader (king)
    pub fn is_leader(&self) -> bool {
        self.type_def.is_leader
    }
    
    pub fn get_num_pieces(&self) -> u32 {
        self.num_pieces
    }
    
    pub fn is_castle_rook(&self, index: BIndex) -> bool {
        self.type_def.is_castle_rook && self.castle_squares.get_bit(index)
    }
    
    // Get the zobrist hash for this piece at the given index
    pub fn get_zobrist(&self, index: BIndex) -> u64 {
        self.zobrist_hashes[index as usize]
    }
    
    // Get the zobrist hash for the castling right of this piece at the given index
    pub fn get_castle_zobrist(&self, index: BIndex) -> u64 {
        // This could be implemented with a separate random array, but this is simpler
        self.zobrist_hashes[index as usize] >> 1
    }
    
    // Get the material score for 1 unit of this piece
    pub fn get_material_score(&self) -> Centipawns {
        self.material_score
    }
    
    // Move a piece from one index to another
    // If set_can_castle is true, set the new index as a castle square.
    // Returns true if the piece could castle before this move
    pub fn move_piece(&mut self, from: BIndex, to: BIndex, set_can_castle: bool) -> bool {
        let could_castle = self.castle_squares.get_bit(from);
        self.bitboard.clear_bit(from);
        self.bitboard.set_bit(to);
        
        if self.type_def.can_castle || self.type_def.is_castle_rook {
            self.castle_squares.clear_bit(from);
            if set_can_castle {
                self.castle_squares.set_bit(to);
            } else {
                self.castle_squares.clear_bit(to);
            }
        }
        could_castle
    }
    
    // Add a piece to this piece type.
    pub fn add_piece(&mut self, index: BIndex, set_can_castle: bool) {
        self.bitboard.set_bit(index);
        self.num_pieces += 1;
        self.total_material_score += self.material_score;
        
        if set_can_castle && (self.type_def.can_castle || self.type_def.is_castle_rook) {
            self.castle_squares.set_bit(index);
        }
    }
    
    // Remove a piece from this piece type
    // Returns true if the piece could castle before this move
    pub fn remove_piece(&mut self, index: BIndex) -> bool {
        let could_castle = self.castle_squares.get_bit(index);
        self.bitboard.clear_bit(index);
        
        self.num_pieces -= 1;
        self.total_material_score -= self.material_score;
        could_castle
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
    
    pub fn output_moves(&self, position: &Position, enemies: &Bitboard, occ_or_not_in_bounds: &Bitboard,
            out_bb_moves: &mut Vec<BitboardMoves>, out_moves: &mut Vec<Move>)
    {
        let mut bb_copy = self.bitboard.clone();
        while !bb_copy.is_zero() {
            let index = bb_copy.lowest_one().unwrap();
            let can_castle = self.type_def.can_castle && self.castle_squares.get_bit(index);
            let can_double_jump = self.type_def.double_move_squares.get_bit(index);
            output_translations(&self.type_def, index, position, enemies,
                occ_or_not_in_bounds, can_castle, can_double_jump, out_bb_moves, out_moves);
            output_captures(&self.type_def, index, position, enemies, 
                occ_or_not_in_bounds, out_bb_moves, out_moves);
            bb_copy.clear_bit(index);
        }
    }
    
    pub fn output_captures(&self, position: &Position, enemies: &Bitboard, occ_or_not_in_bounds: &Bitboard,
            out_bb_moves: &mut Vec<BitboardMoves>, out_moves: &mut Vec<Move>)
    {
        let mut bb_copy = self.bitboard.clone();
        while !bb_copy.is_zero() {
            let index = bb_copy.lowest_one().unwrap();
            output_captures(&self.type_def, index, position, enemies, 
                occ_or_not_in_bounds, out_bb_moves, out_moves);
            bb_copy.clear_bit(index);
        }
    }
    
    pub fn get_movement(&self) -> &PieceDefinition {
        &self.type_def
    }
    
    fn random_zobrist(piece_id: u32, player: Player) -> Vec<u64> {
        // Generate a predictable seed for the rng
        let seed = (player as u64) << 32 | (piece_id as u64);
        let mut rng = StdRng::seed_from_u64(seed);
        
        let mut zobrist = Vec::with_capacity(256);
        for _ in 0..=255 {
            zobrist.push(rng.gen::<u64>());
        }
        zobrist
    }
}

// Print as a string
impl std::fmt::Display for Piece {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        write!(f, "Piece {} (id={}, player={})", self.type_def.char_rep, self.type_def.id, self.player_num)
    }
}
