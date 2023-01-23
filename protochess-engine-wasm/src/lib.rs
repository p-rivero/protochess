mod utils;
mod serialize_types;

use protochess_engine_rs::Engine;
use serde_wasm_bindgen::to_value;
use wasm_bindgen::prelude::*;

use serialize_types::*;
use utils::{set_panic_hook, SerVec};

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
        set_panic_hook();
        Protochess {
            engine: Engine::default()
        }
    }

    #[allow(clippy::inherent_to_string)]
    pub fn to_string(&self) -> String {
        self.engine.to_string()
    }

    pub fn play_best_move(&mut self, depth: u8) -> JsValue {
        let (best_move, _) = self.engine.get_best_move(depth);
        let move_result = self.engine.make_move(&best_move);
        MakeMoveResultSer::to_js(move_result)
    }
    pub fn play_best_move_timeout(&mut self, time: usize) -> JsValue {
        let (best_move, _, search_depth) = self.engine.get_best_move_timeout(time as u64);
        let move_result = self.engine.make_move(&best_move);
        MakeMoveResultWithDepthSer::to_js(move_result, search_depth)
    }

    pub fn make_move(&mut self, mv: JsValue) -> JsValue {
        let mv = MoveInfoSer::from_js(mv);
        let move_result = self.engine.make_move(&mv);
        MakeMoveResultSer::to_js(move_result)
    }

    pub fn get_best_move(&mut self, depth: u8) -> JsValue {
        let (best_move, eval) = self.engine.get_best_move(depth);
        MoveInfoWithEvalSer::to_js(best_move, eval)
    }
    pub fn get_best_move_timeout(&mut self, time: usize) -> JsValue {
        let (best_move, eval, depth) = self.engine.get_best_move_timeout(time as u64);
        MoveInfoWithEvalDepthSer::to_js(best_move, eval, depth)
    }

    pub fn to_move_in_check(&mut self) -> bool {
        self.engine.to_move_in_check()
    }
    
    pub fn set_state(&mut self, state: JsValue) {
        let state = GameStateSer::from_js(state);
        self.engine.set_state(state);
    }
    
    pub fn get_state(&self) -> JsValue {
        let state = self.engine.get_state();
        GameStateSer::to_js(state)
    }
    
    pub fn load_fen(&mut self, fen: &str) {
        self.engine = Engine::from_fen(fen);
    }
    
    pub fn moves_from(&mut self, x: u8, y: u8) -> JsValue {
        let moves: SerVec<MoveInfoSer> = self.engine.moves_from(x, y).into();
        to_value(&moves).unwrap()
    }
}
