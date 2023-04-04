use crate::utils::debug::split_debug_fen;
use crate::{PieceDefinition, MoveInfo, GlobalRules, Position, MoveGen};
use crate::types::{Player, BCoord, GameMode};

use super::fen::FenData;
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


/// Defines a chess variant. Includes the rules of the game and the starting position.
#[must_use]
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct InitialState {
    /// Initial position in FEN format. It includes the walls as `'*'`, which cannot 
    /// be overridden by the user when loading another FEN.
    /// Only the first part of the FEN is used, the player to move, castling rights, etc. are ignored.
    pub fen: String,
    /// The player that has the first move.
    pub player_to_move: Player,
    /// Definitions of the pieces used in this variant and their behavior.
    pub piece_types: Vec<PieceDefinition>,
    /// Width of the board.
    pub board_width: BCoord,
    /// Height of the board.
    pub board_height: BCoord,
    /// Global rules of the game.
    pub global_rules: GlobalRules,
}

impl Default for InitialState {
    fn default() -> Self {
        VariantFactory::new(GameMode::Standard).make_initial_state()
    }
}

/// Contains data that changes with each move.
#[must_use]
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct StateDiff {
    /// Current position in FEN format. The walls are represented as `'*'`.
    pub fen: String,
    pub in_check: bool,
    pub player_to_move: Player,
}

impl From<&mut Position> for StateDiff {
    fn from(pos: &mut Position) -> Self {
        let fen = FenData::from(&*pos).to_string();
        let in_check = {
            if pos.leader_is_captured() { false }
            else { MoveGen::in_check(pos) }
        };
        let player_to_move = pos.whos_turn;
        StateDiff { fen, in_check, player_to_move }
    }
}
