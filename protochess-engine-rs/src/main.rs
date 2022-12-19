//#[macro_use] extern crate scan_rules;

use std::io::Write;

use protochess_engine_rs::{MoveInfo, MakeMoveResult};
use protochess_engine_rs::piece::PieceId;
use protochess_engine_rs::utils::to_long_algebraic_notation;

pub fn main() {
    
    // Some interesting FENs:
    // "R3b3/4k3/2n5/p4p1p/4p3/2B5/1PP2PPP/5K2 w - - 10 36"
    // "rnbqkbnr/nnnnnnnn/rrrrrrrr/8/8/8/QQQQQQQQ/RNBQKBNR w KQkq - 0 1"
    // "rnbqkbnr/pp4pp/8/8/8/8/PPPPPPPP/RNBQKBNR w KQkq - 0 1"
    // "r1b3nr/ppqk1Bbp/2pp4/4P1B1/3n4/3P4/PPP2QPP/R4RK1 w - - 1 0"
    // "1Q6/5pk1/2p3p1/1pbbN2p/4n2P/8/r5P1/5K2 b - - 0 1"
    // "rnbqkbnr/pppppppp/8/8/8/8/8/RNBQKBNR w KQkq - 0 1"
    
    
    // Usage: cargo run -- <depth> <fen> <num_ply>
    // By default, <depth> is 12, <fen> is the starting position, and <num_ply> is 500
    // Example: cargo run -- 4 "1Q6/5pk1/2p3p1/1pbbN2p/4n2P/8/r5P1/5K2 b - - 0 1"
    
    let mut engine = protochess_engine_rs::Engine::default();
    
    let mut pgn_file = std::fs::File::create("pgn.txt").expect("create failed");

    
    let args: Vec<String> = std::env::args().collect();
    let mut depth = 12;
    let mut max_ply = 500;
    if args.len() > 3 {
        max_ply = args[3].parse::<u32>().unwrap();
    }
    if args.len() > 2 {
        let fen = args[2].to_owned();
        engine = protochess_engine_rs::Engine::from_fen(fen);
    }
    if args.len() > 1 {
        depth = args[1].parse::<u8>().unwrap();
    }
    
    println!("{}", engine);

    let start = instant::Instant::now();
    let mut ply = 0;
    for _ in 0..max_ply {
        let mv = engine.get_best_move(depth);
        print_pgn(&mut pgn_file, ply, &mv, engine.get_piece_at(mv.from).unwrap());
        match engine.make_move(mv) {
            MakeMoveResult::Ok => {},
            MakeMoveResult::IllegalMove => {
                assert!(false, "An illegal move was made");
            },
            MakeMoveResult::Checkmate(losing_player) => {
                if losing_player == 0 {
                    println!("CHECKMATE! Black wins!");
                } else {
                    println!("CHECKMATE! White wins!");
                }
                break;
            },
            MakeMoveResult::Stalemate => {
                println!("STALEMATE!");
                break;
            },
            MakeMoveResult::Repetition => {
                println!("DRAW BY REPETITION!");
                break;
            },
        }
        ply += 1;
        println!("(Time since start: {:?})", start.elapsed());
        println!("PLY: {} Engine plays: \n", ply);
        println!("{}", engine);
        println!("\n========================================\n");

    }
}

fn print_pgn(pgn_file: &mut std::fs::File, ply: u32, mv: &MoveInfo, piece: PieceId) {
    if (ply % 2) == 0 {
        let round = format!("{}. ", ply/2 + 1);
        pgn_file.write_all(round.as_bytes()).expect("write failed");
    }
    let prom = mv.promotion.map(pieceid_to_char);
    let move_str = to_long_algebraic_notation(mv.from, mv.to, pieceid_to_char(piece), prom);
    pgn_file.write_all(move_str.as_bytes()).expect("write failed");
}

// TODO: The user should keep track of the piece IDs.
const ID_KING: PieceId = 0;
const ID_QUEEN: PieceId = 1;
const ID_ROOK: PieceId = 2;
const ID_BISHOP: PieceId = 3;
const ID_KNIGHT: PieceId = 4;
const ID_PAWN: PieceId = 5;
const BASE_ID_CUSTOM: PieceId = 100;
fn pieceid_to_char(piece_id: PieceId) -> char {
    match piece_id {
        ID_KING => {'K'}
        ID_QUEEN => {'Q'}
        ID_ROOK => {'R'}
        ID_BISHOP => {'B'}
        ID_KNIGHT => {'N'}
        ID_PAWN => {'P'}
        _ => {(piece_id - BASE_ID_CUSTOM) as u8 as char}
    }
}
