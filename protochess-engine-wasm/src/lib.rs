mod utils;

use protochess_engine_rs::{Engine, MoveInfo, MakeMoveResult};
use serde_wasm_bindgen::to_value;
use wasm_bindgen::prelude::*;

// When the `wee_alloc` feature is enabled, use `wee_alloc` as the global
// allocator.
#[cfg(feature = "wee_alloc")]
#[global_allocator]
static ALLOC: wee_alloc::WeeAlloc = wee_alloc::WeeAlloc::INIT;

#[wasm_bindgen]
extern {
    fn alert(s: &str);
}

#[wasm_bindgen]
pub fn greet() {
    alert("Hello, protochess-engine-wasm!");
}

#[wasm_bindgen]
pub struct Protochess {
    engine: Engine
}

#[wasm_bindgen]
impl Protochess {
    #[wasm_bindgen(constructor)]
    pub fn new() -> Protochess {
        utils::set_panic_hook();
        Protochess{
            engine: Engine::default()
        }
    }

    pub fn to_string(&mut self) -> String {
        self.engine.to_string()
    }

    pub fn play_best_move(&mut self, depth: u8) -> bool {
        let best_move = self.engine.get_best_move(depth);
        let result = self.engine.make_move(&best_move);
        result == MakeMoveResult::Ok
    }
    
    pub fn play_best_move_timeout(&mut self, time: usize) -> String {
        let (best_move, search_depth) = self.engine.get_best_move_timeout(time as u64);
        let move_result = self.engine.make_move(&best_move);
        
        match move_result {
            MakeMoveResult::Ok => format!("OK. Depth: {}", search_depth),
            MakeMoveResult::IllegalMove => "ILLEGAL_MOVE".to_string(),
            MakeMoveResult::Checkmate(player) => format!("CHECKMATE. Player: {}", player),
            MakeMoveResult::Stalemate => "STALEMATE".to_string(),
            MakeMoveResult::Repetition => "REPETITION".to_string(),
        }
    }

    pub fn make_move(&mut self, x1: u8, y1: u8, x2: u8, y2: u8, promotion_id: u32) -> String {
        let prom = {
            if promotion_id == 0 {
                None
            } else {
                Some(promotion_id)
            }
        };
        let mv = MoveInfo {
            from: (x1, y1),
            to: (x2, y2),
            promotion: prom
        };
        let move_result = self.engine.make_move(&mv);
        
        match move_result {
            MakeMoveResult::Ok => "OK".to_string(),
            MakeMoveResult::IllegalMove => "ILLEGAL_MOVE".to_string(),
            MakeMoveResult::Checkmate(player) => format!("CHECKMATE. Player: {}", player),
            MakeMoveResult::Stalemate => "STALEMATE".to_string(),
            MakeMoveResult::Repetition => "REPETITION".to_string(),
        }
    }


    pub fn get_best_move_timeout(&mut self, time: usize) -> String {
        let (best_move, depth) = self.engine.get_best_move_timeout(time as u64);
        best_move.to_string() + " " + &depth.to_string()
    }

    pub fn to_move_in_check(&mut self) -> bool {
        self.engine.to_move_in_check()
    }

    ///True on succcess
    pub fn set_state(&mut self, _val: &JsValue) -> bool {
        // TODO
        // let request_game_state: GameState = from_value(val.to_owned()).unwrap();
        // if let Some((movements, valid_squares, valid_pieces)) =
        // validate_gamestate_request(request_game_state.tiles,
        //                            request_game_state.pieces,
        //                            request_game_state.movement_patterns){
        //     self.engine.set_state(movements,
        //                         &valid_squares,
        //                         valid_pieces);
        //     return true;
        // }
        false
    }

    pub fn moves_from(&mut self, x:u8, y:u8) -> JsValue{
        let moves = self.engine.moves_from(x, y);
        to_value(&moves).unwrap()
    }
}