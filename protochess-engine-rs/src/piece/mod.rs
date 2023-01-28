use rand::rngs::StdRng;
use rand::{SeedableRng, Rng};

use crate::{types::*, Position};

pub type PieceId = u32;

mod piece_definition;
mod piece_factory;
mod material_score;
mod positional_score;
mod movement;
mod precomputed_piece_def;

pub use piece_factory::PieceFactory;
pub use piece_definition::PieceDefinition;

use material_score::compute_material_score;
use positional_score::compute_piece_square_table;
use precomputed_piece_def::PrecomputedPieceDef;
use movement::{output_translations, output_captures};

// Represents a piece type. Specific instances of this type are represented by a 1 in the bitboard
#[derive(Clone, Debug)]
pub struct Piece {
    // Info about this piece type
    type_def: PieceDefinition,
    // Derived from type_def
    precomp: PrecomputedPieceDef,
    // Occupancy bitboard
    bitboard: Bitboard,
    //Player num for the owner of this piece
    player_num: Player,
    // Zobrist hashes for this piece at each board index
    zobrist_hashes: Vec<ZobKey>,
    
    // Material score for this piece
    material_score: Centipawns,
    // Table of positional scores for this piece
    piece_square_table: Vec<Centipawns>,
    piece_square_table_endgame: Vec<Centipawns>,
    
    // Number of 1 bits in the bitboard
    num_pieces: u32,
    // num_pieces * material_score
    total_material_score: Centipawns,
    
    // Positions at which this piece can castle. Used if can_castle or is_castle_rook are true
    castle_squares: Bitboard,
}

impl Piece {
    pub fn new(mut definition: PieceDefinition, player_num: Player, dims: &BDimensions) -> Piece {
        // Use uppercase for white pieces and lowercase for black pieces
        if player_num == 0 {
            definition.char_rep = definition.char_rep.to_ascii_uppercase();
        } else {
            definition.char_rep = definition.char_rep.to_ascii_lowercase();
        }
        
        let material_score = compute_material_score(&definition, dims);
        let zobrist_hashes = Piece::random_zobrist(definition.id, player_num);
        let piece_square_table = compute_piece_square_table(&definition, dims, false);
        let piece_square_table_endgame = compute_piece_square_table(&definition, dims, true);
        Piece {
            precomp: PrecomputedPieceDef::from((&definition, dims)),
            type_def: definition,
            player_num,
            zobrist_hashes,
            material_score,
            piece_square_table,
            piece_square_table_endgame,
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
        while let Some(index) = bb_copy.lowest_one() {
            bb_copy.clear_bit(index);
            indexes.push(index);
        }
        indexes
    }
    
    // Returns true if this piece is a leader (king)
    pub fn is_leader(&self) -> bool {
        self.type_def.is_leader
    }
    
    pub fn get_num_pieces(&self) -> u32 {
        self.num_pieces
    }
    
    pub fn is_rook(&self) -> bool {
        self.type_def.is_castle_rook
    }
    pub fn has_not_moved(&self, index: BIndex) -> bool {
        self.castle_squares.get_bit(index)
    }
    
    pub fn explodes(&self) -> bool {
        self.type_def.explodes
    }
    
    pub fn is_in_win_square(&self) -> bool {
        !((&self.precomp.instant_win_squares & &self.bitboard).is_zero())
    }
    pub fn wins_at(&self, index: BIndex) -> bool {
        self.precomp.instant_win_squares.get_bit(index)
    }
    
    pub fn immune_to_explosion(&self) -> bool {
        self.type_def.immune_to_explosion
    }
    
    // Get the zobrist hash for this piece at the given index
    pub fn get_zobrist(&self, index: BIndex) -> ZobKey {
        self.zobrist_hashes[index as usize]
    }
    
    // Get the zobrist hash for the castling right of this piece at the given index
    pub fn get_castle_zobrist(&self, index: BIndex) -> ZobKey {
        // This could be implemented with a separate random array, but this is simpler
        self.zobrist_hashes[index as usize] >> 1
    }
    
    // Get the material score for 1 unit of this piece
    pub fn get_material_score(&self) -> Centipawns {
        self.material_score
    }
    
    // Returns true if this piece is involved in castling (either can castle or is a castle rook)
    pub fn used_in_castling(&self) -> bool {
        self.type_def.can_castle() || self.type_def.is_castle_rook
    }
    
    // Move a piece from one index to another
    // If set_can_castle is true, set the new index as a castle square.
    // Returns true if the piece could castle before this move
    // Don't call this directly, use PieceSet::move_piece() instead.
    #[inline]
    pub fn move_piece_(&mut self, from: BIndex, to: BIndex, set_can_castle: bool) -> bool {
        let could_castle = self.castle_squares.get_bit(from);
        self.bitboard.clear_bit(from);
        self.bitboard.set_bit(to);
        
        if self.used_in_castling() {
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
    // Don't call this directly, use PieceSet::add_piece() instead.
    #[inline]
    pub fn add_piece_(&mut self, index: BIndex, set_can_castle: bool) {
        self.bitboard.set_bit(index);
        self.num_pieces += 1;
        self.total_material_score += self.material_score;
        
        if set_can_castle && self.used_in_castling() {
            self.castle_squares.set_bit(index);
        }
    }
    
    // Remove a piece from this piece type
    // Returns true if the piece could castle before this move
    // Don't call this directly, use PieceSet::remove_piece() instead.
    #[inline]
    pub fn remove_piece_(&mut self, index: BIndex) -> bool {
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
    #[inline]
    pub fn get_positional_score<const ENDGAME: bool>(&self, index: BIndex) -> Centipawns {
        if ENDGAME {
            self.piece_square_table_endgame[index as usize]
        } else {
            self.piece_square_table[index as usize]
        }
    }
    
    // Get the positional score for all current units of this piece
    #[inline]
    pub fn get_positional_score_all<const ENDGAME: bool>(&self) -> Centipawns {
        let mut bb_copy = self.bitboard.clone();
        let mut score = 0;
        while let Some(index) = bb_copy.lowest_one() {
            score += self.get_positional_score::<ENDGAME>(index);
            bb_copy.clear_bit(index);
        }
        score
    }
    
    pub fn output_translations(&self, position: &Position, enemies: &Bitboard,
        occ_or_not_in_bounds: &Bitboard, out_moves: &mut Vec<Move>)
    {
        let mut bb_copy = self.bitboard.clone();
        while let Some(index) = bb_copy.lowest_one() {
            let can_castle = self.type_def.can_castle() && self.castle_squares.get_bit(index);
            output_translations(
                &self.type_def,
                index,
                position,
                enemies,
                &self.precomp.promotion_squares,
                occ_or_not_in_bounds,
                can_castle,
                &self.precomp.double_jump_squares,
                &self.precomp.jump_bitboards_translate,
                out_moves
            );
            bb_copy.clear_bit(index);
        }
    }
    
    pub fn output_captures(&self, position: &Position, enemies: &Bitboard,
        occ_or_not_in_bounds: &Bitboard, out_moves: &mut Vec<Move>)
    {
        let mut bb_copy = self.bitboard.clone();
        while let Some(index) = bb_copy.lowest_one() {
            output_captures(
                &self.type_def,
                index,
                position,
                enemies,
                &self.precomp.promotion_squares,
                occ_or_not_in_bounds,
                &self.precomp.jump_bitboards_capture[index as usize],
                out_moves
            );
            bb_copy.clear_bit(index);
        }
    }
    
    pub fn get_movement(&self) -> &PieceDefinition {
        &self.type_def
    }
    pub fn get_capture_jumps(&self, index: BIndex) -> &Bitboard {
        &self.precomp.jump_bitboards_capture[index as usize]
    }
    pub fn get_explosion(&self, index: BIndex) -> &Bitboard {
        &self.precomp.explosion_bitboards[index as usize]
    }
    
    fn random_zobrist(piece_id: u32, player: Player) -> Vec<ZobKey> {
        // Generate a predictable seed for the rng
        let seed = (player as u64) << 32 | (piece_id as u64);
        let mut rng = StdRng::seed_from_u64(seed);
        
        let mut zobrist = Vec::with_capacity(256);
        for _ in 0..=255 {
            zobrist.push(rng.gen::<ZobKey>());
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
