use crate::types::{BIndex, BCoord, Player};
use std::collections::HashMap;
use rand::rngs::StdRng;
use rand::{SeedableRng, Rng};
use crate::position::piece::Piece;

use super::piece::{PieceIdWithPlayer, PieceId};

//Holds the random numbers used in generating zobrist keys
pub struct ZobristTable {
    //Map of piecetype id (piece + playernum) -> Vec of length 256, one random # for each index
    custom_zobrist: HashMap<PieceIdWithPlayer, Vec<u64>>,
    ep_zobrist: Vec<u64>,
    white_to_move: u64,
    w_q_castle: u64,
    b_q_castle: u64,
    w_k_castle: u64,
    b_k_castle: u64,
    rng: StdRng
}

impl ZobristTable {
    pub fn new() -> ZobristTable {
        let mut rng = StdRng::seed_from_u64(5435651169991665628);
        let mut ep_zobrist = Vec::with_capacity(16);
        // TODO: Remove =
        for _ in 0..=16 {
            ep_zobrist.push(rng.gen::<u64>());
        }

        let table = ZobristTable{
            custom_zobrist: Default::default(),
            ep_zobrist,
            white_to_move: rng.gen::<u64>(),
            w_q_castle: rng.gen::<u64>(),
            b_q_castle: rng.gen::<u64>(),
            w_k_castle: rng.gen::<u64>(),
            b_k_castle: rng.gen::<u64>(),
            rng
        };
        //Initialize the table with the default piece set in a repeatable way
        //Mostly for easier testing
        table
    }

    /// Zobrist for the player to move
    pub fn get_player_zobrist(&self, player_num: Player) -> u64 {
        if player_num == 0 {
            self.white_to_move
        } else {
            0
        }
    }

    pub fn get_castling_zobrist(&self, player_num: Player, kingside: bool) -> u64 {
        match (player_num, kingside) {
            (0, true) => {self.w_k_castle}
            (0, false) => {self.w_q_castle}
            (1, true) => {self.b_k_castle}
            (1, false) => {self.b_q_castle}
            _ => {0}
        }
    }

    pub fn get_zobrist_sq(&self, piece: &Piece, index: BIndex) -> u64 {
        let piece_id = &piece.get_full_id();
        if !self.custom_zobrist.contains_key(&piece_id) {
            // Register the piece in the table
            panic!("Piece not registered in zobrist table: {}", piece)
        }
        self.custom_zobrist.get(&piece_id).unwrap()[index as usize]
    }
    
    // TODO: Remove
    pub fn get_zobrist_sq_from(&self, piece_type: PieceId, owner: Player, index: BIndex) -> u64 {
        // new(piece_id: u64, char_rep: char, player_num: Player, is_leader: bool, can_double_move: bool, can_castle: bool)
        let tmp = Piece::new(piece_type, '_', owner, false, false, false);
        self.get_zobrist_sq(&tmp, index)
    }

    pub fn get_ep_zobrist_file(&self, rank: BCoord) -> u64 {
        self.ep_zobrist[rank as usize]
    }

    //Registers a custom piece type
    pub fn register_piecetype(&mut self, piece: Piece) {
        let randoms = self.make_randoms();
        self.custom_zobrist.insert(piece.get_full_id(), randoms);
    }

    fn make_randoms(&mut self) -> Vec<u64> {
        let mut randoms = Vec::with_capacity(256);
        for _i in 0..=255 {
            randoms.push(self.rng.gen::<u64>());
        }
        randoms
    }
}

