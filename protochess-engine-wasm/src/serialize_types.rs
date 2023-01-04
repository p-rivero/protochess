use protochess_engine_rs::{MoveInfo, MakeMoveResult, PieceDefinition};
use serde_wasm_bindgen::to_value;
use wasm_bindgen::prelude::*;


macro_rules! generate_wrapper {
    ($wrapper_name:ident, $wrapped_type:ident, [$($field:ident, $type:ty),*]) => {
        #[derive(serde::Serialize, serde::Deserialize)]
        pub struct $wrapper_name {
            $( $field: $type ),*
        }
        impl $wrapper_name {
            #[allow(dead_code)]
            pub fn unwrap(self) -> $wrapped_type {
                $wrapped_type {
                    $( $field: self.$field ),*
                }
            }
            #[allow(dead_code)]
            pub fn wrap(val: $wrapped_type) -> $wrapper_name {
                $wrapper_name {
                    $( $field: (val.$field) ),*
                }
            }
            #[allow(dead_code)]
            pub fn to_js(val: $wrapped_type) -> JsValue {
                to_value(&Self::wrap(val)).unwrap()
            }
        }
    }
}

generate_wrapper!(MoveInfoSer, MoveInfo, [
    from, (u8, u8),
    to, (u8, u8),
    promotion, Option<u32>
]);


#[derive(serde::Serialize, serde::Deserialize)]
pub struct MakeMoveResultSer {
    result: String,
    checkmated_player_num: Option<u8>,
}
impl MakeMoveResultSer {
    pub fn wrap(mmr: &MakeMoveResult) -> MakeMoveResultSer {
        if let MakeMoveResult::Checkmate(player_num) = mmr {
            MakeMoveResultSer {
                result: "Checkmate".to_string(),
                checkmated_player_num: Some(*player_num)
            }
        } else {
            MakeMoveResultSer {
                result: format!("{:?}", mmr),
                checkmated_player_num: None
            }
        }
    }
    pub fn to_js(mmr: &MakeMoveResult) -> JsValue {
        to_value(&MakeMoveResultSer::wrap(mmr)).unwrap()
    }
}

#[derive(serde::Serialize, serde::Deserialize)]
pub struct MoveInfoWithDepthSer {
    move_info: MoveInfoSer,
    depth: u8
}
impl MoveInfoWithDepthSer {
    pub fn wrap(mv: MoveInfo, depth: u8) -> MoveInfoWithDepthSer {
        MoveInfoWithDepthSer {
            move_info: MoveInfoSer::wrap(mv),
            depth
        }
    }
    pub fn to_js(mv: MoveInfo, depth: u8) -> JsValue {
        to_value(&MoveInfoWithDepthSer::wrap(mv, depth)).unwrap()
    }
}


#[derive(serde::Serialize, serde::Deserialize)]
pub struct MakeMoveResultWithDepthSer {
    make_move_result: MakeMoveResultSer,
    depth: u8
}
impl MakeMoveResultWithDepthSer {
    pub fn to_js(mmr: &MakeMoveResult, depth: u8) -> JsValue {
        let val = MakeMoveResultWithDepthSer {
            make_move_result: MakeMoveResultSer::wrap(mmr),
            depth
        };
        to_value(&val).unwrap()
    }
}

generate_wrapper!(PieceDefinitionSer, PieceDefinition, [
    id, u32,
    char_rep, char,
    available_for, Vec<u8>,
    is_leader, bool,
    can_castle, bool,
    is_castle_rook, bool,
    explodes, bool,
    immune_to_explosion, bool,
    promotion_squares, Vec<(u8, u8)>,
    promo_vals, Vec<u32>,
    double_jump_squares, Vec<(u8, u8)>,
    attack_sliding_deltas, Vec<Vec<(i8, i8)>>,
    attack_jump_deltas, Vec<(i8, i8)>,
    attack_north, bool,
    attack_south, bool,
    attack_east, bool,
    attack_west, bool,
    attack_northeast, bool,
    attack_northwest, bool,
    attack_southeast, bool,
    attack_southwest, bool,
    translate_jump_deltas, Vec<(i8, i8)>,
    translate_sliding_deltas, Vec<Vec<(i8, i8)>>,
    translate_north, bool,
    translate_south, bool,
    translate_east, bool,
    translate_west, bool,
    translate_northeast, bool,
    translate_northwest, bool,
    translate_southeast, bool,
    translate_southwest, bool
]);



#[derive(serde::Serialize, serde::Deserialize)]
pub struct GameState {
    pub piece_types: Vec<PieceDefinitionSer>,
    // BCoord, BCoord
    pub valid_squares: Vec<(u8, u8)>,
    // Player, BCoord, BCoord, PieceId
    pub pieces: Vec<(u8, u8, u8, u32)>,
    // Player
    pub whos_turn: u8,
}
