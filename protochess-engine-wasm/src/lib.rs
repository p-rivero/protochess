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
    engine: Engine,
    stop_flag: bool,
}

#[wasm_bindgen]
impl Protochess {
    #[wasm_bindgen(constructor)]
    #[allow(clippy::new_without_default)]
    pub fn new() -> Protochess {
        set_panic_hook();
        Protochess {
            engine: Engine::default(),
            stop_flag: false,
        }
    }

    #[allow(clippy::inherent_to_string)]
    #[wasm_bindgen(js_name = toString)]
    pub fn to_string(&self) -> String {
        self.engine.to_string()
    }
    
    #[wasm_bindgen(js_name = playerToMove)]
    pub fn player_to_move(&self) -> u8 {
        self.engine.player_to_move()
    }

    #[wasm_bindgen(js_name = validatePosition)]
    pub fn validate_position(&mut self) -> Result<(), String> {
        self.engine.validate_position()
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
    pub fn get_best_move_timeout(&mut self) -> Result<JsValue, String> {
        self.stop_flag = false;
        let (best_move, eval, depth) = self.engine.get_best_move_timeout(&self.stop_flag)?;
        Ok(MoveInfoWithEvalDepthSer::to_js(best_move, eval, depth))
    }
    
    #[wasm_bindgen(js_name = setState)]
    pub fn set_state(&mut self, state: JsValue) -> Result<JsValue, String> {
        let state = GameStateSer::from_js(state)?;
        let result = self.engine.set_state(state)?;
        Ok(MakeMoveResultSer::to_js(result))
    }
    
    #[wasm_bindgen(js_name = loadFen)]
    pub fn load_fen(&mut self, fen: &str) -> Result<(), String> {
        self.engine.load_fen(fen)?;
        Ok(())
    }
    
    #[wasm_bindgen(js_name = getState)]
    pub fn get_state(&mut self) -> JsValue {
        let state = self.engine.get_state();
        GameStateSer::to_js(state.clone())
    }
    
    #[wasm_bindgen(js_name = getStateDiff)]
    pub fn get_state_diff(&mut self) -> JsValue {
        let state = self.engine.get_state_diff();
        StateDiffSer::to_js(state)
    }
    
    #[wasm_bindgen(js_name = legalMoves)]
    pub fn legal_moves(&mut self) -> Result<JsValue, String> {
        let moves: SerVec<MoveListSer> = self.engine.legal_moves().into();
        Ok(to_value(&moves).unwrap())
    }
    
    #[wasm_bindgen(js_name = possiblePromotions)]
    pub fn possible_promotions(&mut self, from_x: u8, from_y: u8, to_x: u8, to_y: u8) -> Result<JsValue, String> {
        let from = (from_x, from_y);
        let to = (to_x, to_y);
        let promotions: SerVec<char> = self.engine.possible_promotions(from, to).into();
        Ok(to_value(&promotions).unwrap())
    }
    
    #[wasm_bindgen(js_name = getMaxThreads)]
    pub fn get_max_threads(&self) -> u32 {
        Engine::get_max_threads()
    }
    #[wasm_bindgen(js_name = setNumThreads)]
    pub fn set_num_threads(&mut self, num_threads: u32) -> Result<(), String> {
        self.engine.set_num_threads(num_threads)
    }
    
    
    
    #[wasm_bindgen(js_name = getStopFlagPtr)]
    pub fn get_stop_flag_ptr(&mut self) -> *mut bool {
        &mut self.stop_flag
    }
}
