//#[macro_use] extern crate scan_rules;

use std::io::Write;

use protochess_engine_rs::{utils::to_long_algebraic_notation, piece::PieceId};

pub fn main() {
    
    // Some interesting FENs:
    // "R3b3/4k3/2n5/p4p1p/4p3/2B5/1PP2PPP/5K2 w - - 10 36"
    // "rnbqkbnr/nnnnnnnn/rrrrrrrr/8/8/8/QQQQQQQQ/RNBQKBNR w KQkq - 0 1"
    // "rnbqkbnr/pp4pp/8/8/8/8/PPPPPPPP/RNBQKBNR w KQkq - 0 1"
    // "r1b3nr/ppqk1Bbp/2pp4/4P1B1/3n4/3P4/PPP2QPP/R4RK1 w - - 1 0"
    // "1Q6/5pk1/2p3p1/1pbbN2p/4n2P/8/r5P1/5K2 b - - 0 1"
    // "rnbqkbnr/pppppppp/8/8/8/8/8/RNBQKBNR w KQkq - 0 1"
    
    
    // Usage: cargo run -- <threads> <fen>
    // By default, <threads> is 1 and <fen> is the starting position.
    // Example: cargo run -- 4 "1Q6/5pk1/2p3p1/1pbbN2p/4n2P/8/r5P1/5K2 b - - 0 1"
    
    let mut engine = protochess_engine_rs::Engine::default();
    
    let mut pgn_file = std::fs::File::create("pgn.txt").expect("create failed");

    
    let args: Vec<String> = std::env::args().collect();
    if args.len() > 2 {
        let fen = args[2].to_owned();
        engine = protochess_engine_rs::Engine::from_fen(fen);
    }
    if args.len() > 1 {
        let num_threads = args[1].parse::<u32>().unwrap();
        engine.set_num_threads(num_threads);
    }
    
    println!("{}", engine.to_string());

    let start = instant::Instant::now();
    let mut ply = 0;
    loop {

        if let Some(mv) = engine.get_best_move(9) {
            engine.make_move(mv.0, mv.1, mv.2, mv.3, mv.4);
            print_pgn(&mut pgn_file, ply, mv, engine.get_piece_at(mv.2, mv.3).unwrap());
        } else {
            break;
        }
        ply += 1;
        println!("(Time since start: {:?})", start.elapsed());
        println!("PLY: {} Engine plays: \n", ply);
        println!("{}", engine.to_string());
        println!("\n========================================\n");

    }
}

fn print_pgn(pgn_file: &mut std::fs::File, ply: u32, mv: (u8, u8, u8, u8, Option<PieceId>), piece: PieceId) {
    if (ply % 2) == 0 {
        let round = format!("{}. ", ply/2 + 1);
        pgn_file.write_all(round.as_bytes()).expect("write failed");
    }
    let prom = mv.4.map(|x| pieceid_to_char(x));
    let move_str = to_long_algebraic_notation(mv.0, mv.1, mv.2, mv.3, pieceid_to_char(piece), prom);
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
    println!("piece_id: {}", piece_id);
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
