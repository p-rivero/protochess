use crate::types::{BCoord, Player};
use rand::rngs::StdRng;
use rand::{SeedableRng, Rng};


//Holds the random numbers used in generating zobrist keys
pub struct ZobristTable {
    ep_zobrist: Vec<u64>,
    white_to_move: u64,
    w_q_castle: u64,
    b_q_castle: u64,
    w_k_castle: u64,
    b_k_castle: u64,
}

impl ZobristTable {
    pub fn new() -> ZobristTable {
        let mut rng = StdRng::seed_from_u64(5435651169991665628);
        let mut ep_zobrist = Vec::with_capacity(16);
        for _ in 0..16 {
            ep_zobrist.push(rng.gen::<u64>());
        }

        //Initialize the table with the default piece set in a repeatable way
        //Mostly for easier testing
        ZobristTable{
            ep_zobrist,
            white_to_move: rng.gen::<u64>(),
            w_q_castle: rng.gen::<u64>(),
            b_q_castle: rng.gen::<u64>(),
            w_k_castle: rng.gen::<u64>(),
            b_k_castle: rng.gen::<u64>(),
        }
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

    pub fn get_ep_zobrist_file(&self, rank: BCoord) -> u64 {
        self.ep_zobrist[rank as usize]
    }
}

