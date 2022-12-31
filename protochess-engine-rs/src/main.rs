//#[macro_use] extern crate scan_rules;

use std::io::Write;

use protochess_engine_rs::{Engine, MoveInfo, MakeMoveResult};

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
    
    let mut pgn_file = std::fs::File::create("pgn.txt").expect("create failed");

    
    let args: Vec<String> = std::env::args().collect();
    let mut depth = 12;
    let mut max_ply = 500;
    if args.len() > 3 {
        max_ply = args[3].parse::<u32>().unwrap();
    }
    let mut engine = {
        if args.len() > 2 {
            let fen = args[2].trim();
            if fen.ends_with("ATOMIC") {
                pgn_file.write_all(b"[Variant \"Atomic\"]\n").unwrap();
                let len = fen.len() - " ATOMIC".len();
                pgn_file.write_all(format!("[FEN \"{}\"]\n\n", &fen[..len]).as_bytes()).unwrap();
            } else {
                pgn_file.write_all(format!("[FEN \"{}\"]\n\n", fen).as_bytes()).unwrap();
            }
            Engine::from_fen(fen)
        } else {
            Engine::default()
        }
    };
    if args.len() > 1 {
        depth = args[1].parse::<u8>().unwrap();
    }
    
    println!("Start Position:\n{}", engine);
    println!("\n----------------------------------------\n");

    let start = instant::Instant::now();
    for ply in 0..max_ply {
        let mv = engine.get_best_move(depth);
        println!("\n========================================\n");
        println!("(Time since start: {:?})", start.elapsed());
        println!("PLY: {} Engine plays: \n", ply);
        print_pgn(&mut pgn_file, ply, to_long_algebraic_notation(&mv, &engine));
        match engine.make_move(&mv) {
            MakeMoveResult::Ok => {
                println!("{}", engine);
                println!("\n----------------------------------------\n");
            },
            MakeMoveResult::IllegalMove => {
                panic!("An illegal move was made");
            },
            MakeMoveResult::Checkmate(losing_player) => {
                if losing_player == 0 {
                    println!("{}", engine);
                    println!("CHECKMATE! Black wins!");
                } else {
                    println!("{}", engine);
                    println!("CHECKMATE! White wins!");
                }
                break;
            },
            MakeMoveResult::LeaderCaptured(losing_player) => {
                if losing_player == 0 {
                    println!("{}", engine);
                    println!("KING HAS BEEN CAPTURED! Black wins!");
                } else {
                    println!("{}", engine);
                    println!("KING HAS BEEN CAPTURED! White wins!");
                }
                break;
            },
            MakeMoveResult::Stalemate => {
                println!("{}", engine);
                println!("STALEMATE!");
                break;
            },
            MakeMoveResult::Repetition => {
                println!("{}", engine);
                println!("DRAW BY REPETITION!");
                break;
            },
        }
    }
}

fn print_pgn(pgn_file: &mut std::fs::File, ply: u32, move_str: String) {
    if (ply % 2) == 0 {
        let round = format!("{}. ", ply/2 + 1);
        pgn_file.write_all(round.as_bytes()).expect("write failed");
    }
    pgn_file.write_all(move_str.as_bytes()).expect("write failed");
    pgn_file.write_all(b" ").expect("write failed");
}

pub fn to_long_algebraic_notation(mv: &MoveInfo, engine: &Engine) -> String {
    // Long algebraic notation for mv
    let move_string = format!("{}{}{}{}", (b'a' + mv.from.0) as char, mv.from.1 + 1, (b'a' + mv.to.0) as char, mv.to.1 + 1);
    
    if let Some(prom) = mv.promotion {
        let prom_char = engine.get_piece_char(prom).unwrap().to_ascii_uppercase();
        return format!("{}={}", move_string, prom_char);
    }
    
    let piece_id = engine.get_piece_at(mv.from).unwrap();
    let piece_char = engine.get_piece_char(piece_id).unwrap().to_ascii_uppercase();
    let result = {
        if piece_char == 'P' {
            move_string
        } else {
            // If the piece is not a pawn, write the piece letter
            format!("{}{}", piece_char, move_string)
        }
    };
    
    match result.as_str() {
        "Ke1g1" => "O-O".to_string(),
        "Ke1c1" => "O-O-O".to_string(),
        "Ke8g8" => "O-O".to_string(),
        "Ke8c8" => "O-O-O".to_string(),
        _ => result
    }
}
