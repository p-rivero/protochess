mod utils;
mod serialize_types;

use protochess_engine_rs::Engine;
use serde_wasm_bindgen::to_value;
use wasm_bindgen::prelude::*;

use serialize_types::*;
use utils::{set_panic_hook, SerVec};

#[cfg(feature = "parallel")]
pub use wasm_bindgen_rayon::init_thread_pool;

#[wasm_bindgen]
pub fn greet() {
    #[cfg(feature = "parallel")]
    console_log!("Hello from protochess-engine-wasm! (multithreading enabled)");
    #[cfg(not(feature = "parallel"))]
    console_log!("Hello from protochess-engine-wasm! (multithreading disabled)");
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
    #[wasm_bindgen(js_name = toString)]
    pub fn to_string(&self) -> String {
        self.engine.to_string()
    }

    #[wasm_bindgen(js_name = playBestMove)]
    pub fn play_best_move(&mut self, depth: u8) -> Result<JsValue, String> {
        let (best_move, _) = self.engine.get_best_move(depth)?;
        let move_result = self.engine.make_move(&best_move);
        Ok(MakeMoveResultSer::to_js(move_result))
    }
    #[wasm_bindgen(js_name = playBestMoveTimeout)]
    pub fn play_best_move_timeout(&mut self, time: usize) -> Result<JsValue, String> {
        let (best_move, _, search_depth) = self.engine.get_best_move_timeout(time as u64)?;
        let move_result = self.engine.make_move(&best_move);
        Ok(MakeMoveResultWithDepthSer::to_js(move_result, search_depth))
    }

    #[wasm_bindgen(js_name = makeMove)]
    pub fn make_move(&mut self, mv: JsValue) -> Result<JsValue, String> {
        let mv = MoveInfoSer::from_js(mv)?;
        let move_result = self.engine.make_move(&mv);
        Ok(MakeMoveResultSer::to_js(move_result))
    }
    #[wasm_bindgen(js_name = makeMoveStr)]
    pub fn make_move_str(&mut self, mv: &str) -> Result<JsValue, String> {
        let move_result = self.engine.make_move_str(mv)?;
        Ok(MakeMoveResultSer::to_js(move_result))
    }

    #[wasm_bindgen(js_name = getBestMove)]
    pub fn get_best_move(&mut self, depth: u8) -> Result<JsValue, String> {
        let (best_move, eval) = self.engine.get_best_move(depth)?;
        Ok(MoveInfoWithEvalSer::to_js(best_move, eval))
    }
    #[wasm_bindgen(js_name = getBestMoveTimeout)]
    pub fn get_best_move_timeout(&mut self, time: usize) -> Result<JsValue, String> {
        let (best_move, eval, depth) = self.engine.get_best_move_timeout(time as u64)?;
        Ok(MoveInfoWithEvalDepthSer::to_js(best_move, eval, depth))
    }

    #[wasm_bindgen(js_name = toMoveInCheck)]
    pub fn to_move_in_check(&mut self) -> bool {
        self.engine.to_move_in_check()
    }
    
    #[wasm_bindgen(js_name = setState)]
    pub fn set_state(&mut self, state: JsValue) -> Result<(), String> {
        let state = GameStateSer::from_js(state)?;
        self.engine.set_state(state)?;
        Ok(())
    }
    
    #[wasm_bindgen(js_name = getState)]
    pub fn get_state(&self) -> JsValue {
        let state = self.engine.get_state();
        GameStateSer::to_js(state)
    }
    
    #[wasm_bindgen(js_name = loadFen)]
    pub fn load_fen(&mut self, fen: &str) -> Result<(), String> {
        self.engine = Engine::from_fen(fen)?;
        Ok(())
    }
    
    #[wasm_bindgen(js_name = movesFrom)]
    pub fn moves_from(&mut self, x: u8, y: u8) -> Result<JsValue, String> {
        let moves: SerVec<MoveInfoSer> = self.engine.moves_from(x, y)?.into();
        Ok(to_value(&moves).unwrap())
    }
    
    #[wasm_bindgen(js_name = getMaxThreads)]
    pub fn get_max_threads(&self) -> u32 {
        Engine::get_max_threads()
    }
    #[wasm_bindgen(js_name = setNumThreads)]
    pub fn set_num_threads(&mut self, num_threads: u32) -> Result<(), String> {
        self.engine.set_num_threads(num_threads)
    }
}
