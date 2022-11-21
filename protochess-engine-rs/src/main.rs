//#[macro_use] extern crate scan_rules;

use std::io::Write;

use protochess_engine_rs::utils::to_long_algebraic_notation;

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
            engine.make_move(mv.0, mv.1, mv.2, mv.3);
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

fn print_pgn(pgn_file: &mut std::fs::File, ply: u32, mv: (u8, u8, u8, u8), piece: char) {
    if (ply % 2) == 0 {
        let round = format!("{}. ", ply/2 + 1);
        pgn_file.write_all(round.as_bytes()).expect("write failed");
    }
    let move_str = to_long_algebraic_notation(mv.0, mv.1, mv.2, mv.3, piece);
    pgn_file.write_all(move_str.as_bytes()).expect("write failed");
}
