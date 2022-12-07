use crate::types::{Player, Bitboard, BIndex};
use crate::constants::piece_scores::*;

pub type PieceId = u32;
pub type PieceIdWithPlayer = u64;

// Represents a piece type. Specific instances of this type are represented by a 1 in the bitboard
#[derive(Clone, Debug)]
pub struct Piece {
    id: PieceId,
    char_rep: char,
    //Player num for the owner of this piece
    player_num: Player,
    zobrist_hashes: Vec<u64>,
    // TODO: Make private
    pub bitboard: Bitboard, // Occupancy bitboard
    is_leader: bool,
    can_double_move: bool,
    can_castle: bool,
}

impl Piece {
    pub fn new(id: PieceId, char_rep: char, player_num: Player, is_leader: bool, can_double_move: bool, can_castle: bool) -> Piece {
        Piece {
            id,
            char_rep,
            player_num,
            zobrist_hashes: Piece::compute_zobrist(id, player_num),
            bitboard: Bitboard::zero(),
            is_leader,
            can_double_move,
            can_castle,
        }
    }
    
    // Get the full id of this piece (piece type + player_num)
    #[inline(always)]
    pub fn get_full_id(&self) -> PieceIdWithPlayer {
        self.id as PieceIdWithPlayer | (self.player_num as PieceIdWithPlayer) << 32
    }
    
    // TODO: Remove this
    // Get the id of this piece (piece type only)
    #[inline(always)]
    pub fn get_piece_id(&self) -> PieceId {
        self.id
    }
    
    // Get a char representation of this piece
    #[inline(always)]
    pub fn char_rep(&self) -> char {
        self.char_rep
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
    
    
    
    // TODO: Remove this
    pub fn blank_custom(player_num: Player, char_rep: char) -> Piece {
        Piece::new(
            BASE_ID_CUSTOM + char_rep as PieceId,
            char_rep,
            player_num,
            false,
            false,
            false,
        )
    }
    pub fn blank_pawn(player_num: Player) -> Piece{
        let ch = if player_num == 0 { 'P' } else { 'p' };
        Piece::new(
            ID_PAWN,
            ch,
            player_num,
            false,
            true,
            false,
        )
    }
    pub fn blank_knight(player_num: Player) -> Piece{
        let ch = if player_num == 0 { 'N' } else { 'n' };
        Piece::new(
            ID_KNIGHT,
            ch,
            player_num,
            false,
            false,
            false,
        )
    }
    pub fn blank_king(player_num: Player) -> Piece{
        let ch = if player_num == 0 { 'K' } else { 'k' };
        Piece::new(
            ID_KING,
            ch,
            player_num,
            true,
            false,
            false,
        )
    }
    pub fn blank_rook(player_num: Player) -> Piece{
        let ch = if player_num == 0 { 'R' } else { 'r' };
        Piece::new(
            ID_ROOK,
            ch,
            player_num,
            false,
            false,
            true,
        )
    }
    pub fn blank_bishop(player_num: Player) -> Piece{
        let ch = if player_num == 0 { 'B' } else { 'b' };
        Piece::new(
            ID_BISHOP,
            ch,
            player_num,
            false,
            false,
            false,
        )
    }
    pub fn blank_queen(player_num: Player) -> Piece{
        let ch = if player_num == 0 { 'Q' } else { 'q' };
        Piece::new(
            ID_QUEEN,
            ch,
            player_num,
            false,
            false,
            false,
        )
    }
}

// Print as a string
impl std::fmt::Display for Piece {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        write!(f, "Piece {} (id={}, player={})", self.char_rep, self.id, self.player_num)
    }
}
