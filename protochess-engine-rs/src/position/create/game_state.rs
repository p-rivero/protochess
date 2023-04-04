use crate::utils::debug::split_debug_fen;
use crate::{PieceDefinition, MoveInfo, GlobalRules};
use crate::types::{Player, BCoord, GameMode};

use super::variant_factory::VariantFactory;


/// Full state of the game, including:
/// - **Initial state:** Defines the rules of the game and starting position.
/// - **Initial fen (optional):** User-provided fen that further defines the starting position, overriding 
/// the initial state. It's applied to the initial state before playing the moves in `move_history`.
/// - **Move history:** Defines the current position and allows enforcing the repetition rules.
/// 
/// All games of a given chess variant have the same initial state, but can have different initial FENs.
#[must_use]
#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct GameState {
    pub initial_state: InitialState,
    pub initial_fen: Option<String>,
    pub move_history: Vec<MoveInfo>,
}
// The default game state is the standard chess starting position
// without any user-provided FEN and no moves played.

impl GameState {
    /// Loads a FEN string that contains a hardcoded variant name.
    /// Used for testing.
    pub fn from_debug_fen(fen: &str) -> Self {
        let (fen, variant) = split_debug_fen(fen);
        let initial_state = VariantFactory::new(variant).make_initial_state();
        let initial_fen = Some(fen);
        GameState { initial_state, initial_fen, move_history: vec![] }
    }
}


#[must_use]
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct InitialState {
    pub fen: String, // Only piece placements and walls
    pub in_check: bool, // Ignored in set_state()
    pub player_to_move: Player,
    pub piece_types: Vec<PieceDefinition>,
    pub board_width: BCoord,
    pub board_height: BCoord,
    pub global_rules: GlobalRules,
}

impl Default for InitialState {
    fn default() -> Self {
        VariantFactory::new(GameMode::Standard).make_initial_state()
    }
}
