//#[macro_use] extern crate scan_rules;

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

        if !engine.play_best_move_timeout(10).0 {
            break;
        }
        ply += 1;
        println!("(Time since start: {:?})", start.elapsed());
        println!("PLY: {} Engine plays: \n", ply);
        println!("{}", engine.to_string());
        println!("\n========================================\n");



        /*
        readln! {
            // Space-separated ints
            (let x1: u8, let y1: u8, let x2: u8, let y2: u8) => {
                println!("x1 y1 x2 y2: {} {} {} {}", x1, y1, x2, y2);
                engine.make_move(x1, y1, x2, y2);
                println!("{}", engine.to_string());
            }
        }

         */

    }
}