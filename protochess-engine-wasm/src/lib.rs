mod utils;
mod serialize_types;

use protochess_engine_rs::{Engine, MoveInfo};
use serde_wasm_bindgen::{to_value, from_value};
use wasm_bindgen::prelude::*;

use serialize_types::*;

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
    #[allow(clippy::new_without_default)]
    pub fn new() -> Protochess {
        utils::set_panic_hook();
        Protochess{
            engine: Engine::default()
        }
    }

    #[allow(clippy::inherent_to_string)]
    pub fn to_string(&self) -> String {
        self.engine.to_string()
    }

    pub fn play_best_move(&mut self, depth: u8) -> JsValue {
        let best_move = self.engine.get_best_move(depth);
        let move_result = self.engine.make_move(&best_move);
        MakeMoveResultSer::to_js(&move_result)
    }
    pub fn play_best_move_timeout(&mut self, time: usize) -> JsValue {
        let (best_move, search_depth) = self.engine.get_best_move_timeout(time as u64);
        let move_result = self.engine.make_move(&best_move);
        MakeMoveResultWithDepthSer::to_js(&move_result, search_depth)
    }

    pub fn make_move(&mut self, x1: u8, y1: u8, x2: u8, y2: u8, promotion_id: u32) -> JsValue {
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
        MakeMoveResultSer::to_js(&move_result)
    }

    pub fn get_best_move(&mut self, depth: u8) -> JsValue {
        let best_move = self.engine.get_best_move(depth);
        MoveInfoSer::to_js(best_move)
    }
    pub fn get_best_move_timeout(&mut self, time: usize) -> JsValue {
        let (best_move, depth) = self.engine.get_best_move_timeout(time as u64);
        MoveInfoWithDepthSer::to_js(best_move, depth)
    }

    pub fn to_move_in_check(&mut self) -> bool {
        self.engine.to_move_in_check()
    }
    
    pub fn set_state(&mut self, val: &JsValue) {
        let state: GameState = from_value(val.to_owned()).unwrap();
        let pieces = state.piece_types.into_iter().map(|p| p.unwrap()).collect();
        self.engine.set_state(&pieces, &state.valid_squares, &state.pieces, state.whos_turn);
    }
    
    pub fn moves_from(&mut self, x:u8, y:u8) -> JsValue {
        let moves = self.engine.moves_from(x, y);
        let moves_ser: Vec<MoveInfoSer> = moves.into_iter().map(MoveInfoSer::wrap).collect();
        to_value(&moves_ser).unwrap()
    }
}
