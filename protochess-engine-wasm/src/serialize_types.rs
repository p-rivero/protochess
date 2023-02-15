use protochess_engine_rs::*;
use serde_wasm_bindgen::{to_value, from_value};
use wasm_bindgen::prelude::*;
use super::utils::SerVec;


macro_rules! generate_wrapper {
    ($wrapper_name:ident, $wrapped_type:ident, [$($field:ident, $type:ty),*]) => {
        #[derive(serde::Serialize, serde::Deserialize)]
        #[serde(rename_all = "camelCase")]
        #[wasm_bindgen(inspectable)]
        #[must_use]
        pub struct $wrapper_name {
            $( $field: $type ),*
        }
        impl $wrapper_name {
            pub fn to_js(val: $wrapped_type) -> JsValue {
                to_value(&Self::from(val)).unwrap()
            }
            pub fn from_js(val: JsValue) -> Result<$wrapped_type, String> {         
                const TYPE_NAME: &str = stringify!($wrapper_name);       
                let wrapper = from_value::<$wrapper_name>(val)
                    .map_err(|e| format!("Argument must be of type {TYPE_NAME}. {e}"))?;
                Ok(wrapper.into())
            }
        }
        impl From<$wrapped_type> for $wrapper_name {
            fn from(val: $wrapped_type) -> Self {
                $wrapper_name {
                    $( $field: (val.$field).into() ),*
                }
            }
        }
        impl From<$wrapper_name> for $wrapped_type {
            fn from(val: $wrapper_name) -> Self {
                $wrapped_type {
                    $( $field: (val.$field).into() ),*
                }
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
#[serde(rename_all = "camelCase")]
pub struct MakeMoveResultSer {
    result: String,
    winner_player: Option<u8>,
}
impl MakeMoveResultSer {
    pub fn to_js(mmr: MakeMoveResult) -> JsValue {
        to_value(&MakeMoveResultSer::from(mmr)).unwrap()
    }
}
impl From<MakeMoveResult> for MakeMoveResultSer {
    fn from(mmr: MakeMoveResult) -> MakeMoveResultSer {
        let result;
        let winner_player;
        match mmr {
            MakeMoveResult::Checkmate{winner} => {
                result = "Checkmate".to_string();
                winner_player = Some(winner);
            },
            MakeMoveResult::LeaderCaptured{winner} => {
                result = "LeaderCaptured".to_string();
                winner_player = Some(winner);
            },
            MakeMoveResult::PieceInWinSquare{winner} => {
                result = "PieceInWinSquare".to_string();
                winner_player = Some(winner);
            },
            MakeMoveResult::CheckLimit{winner} => {
                result = "CheckLimit".to_string();
                winner_player = Some(winner);
            },
            MakeMoveResult::Stalemate{winner} => {
                result = "Stalemate".to_string();
                winner_player = winner;
            },
            _ => {
                result = format!("{:?}", mmr);
                winner_player = None;
            }
        }
        MakeMoveResultSer { result, winner_player }
    }
}


#[derive(serde::Serialize, serde::Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct MoveInfoWithEvalSer {
    move_info: MoveInfoSer,
    evaluation: i32,
}
impl MoveInfoWithEvalSer {
    pub fn to_js(mv: MoveInfo, evaluation: i32) -> JsValue {
        let val = MoveInfoWithEvalSer {
            move_info: MoveInfoSer::from(mv),
            evaluation
        };
        to_value(&val).unwrap()
    }
}

#[derive(serde::Serialize, serde::Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct MoveInfoWithEvalDepthSer {
    move_info: MoveInfoSer,
    evaluation: i32,
    depth: u8,
}
impl MoveInfoWithEvalDepthSer {
    pub fn to_js(mv: MoveInfo, evaluation: i32, depth: u8) -> JsValue {
        let val = MoveInfoWithEvalDepthSer {
            move_info: MoveInfoSer::from(mv),
            evaluation,
            depth
        };
        to_value(&val).unwrap()
    }
}


#[derive(serde::Serialize, serde::Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct MakeMoveResultWithDepthSer {
    make_move_result: MakeMoveResultSer,
    depth: u8
}
impl MakeMoveResultWithDepthSer {
    pub fn to_js(mmr: MakeMoveResult, depth: u8) -> JsValue {
        let val = MakeMoveResultWithDepthSer {
            make_move_result: mmr.into(),
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
    castle_files, Option<(u8, u8)>,
    is_castle_rook, bool,
    explodes, bool,
    explosion_deltas, Vec<(i8, i8)>,
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
    translate_southwest, bool,
    win_squares, Vec<(u8, u8)>
]);


generate_wrapper!(PiecePlacementSer, PiecePlacement, [
    owner, u8,
    piece_id, u32,
    x, u8,
    y, u8,
    // True if it has not moved. This is an option so that JS can leave it as undefined
    can_castle, Option<bool>
]);

generate_wrapper!(GlobalRulesSer, GlobalRules, [
    capturing_is_forced, bool,
    check_is_forbidden, bool,
    stalemated_player_loses, bool,
    invert_win_conditions, bool,
    repetitions_draw, u8,
    checks_to_lose, u8
]);

generate_wrapper!(GameStateSer, GameState, [
    piece_types, SerVec<PieceDefinitionSer>,
    valid_squares, Vec<(u8, u8)>,
    pieces, SerVec<PiecePlacementSer>,
    whos_turn, u8,
    ep_square_and_victim, Option<((u8, u8), (u8, u8))>,
    times_in_check, Option<[u8; 2]>,
    global_rules, GlobalRulesSer
]);

