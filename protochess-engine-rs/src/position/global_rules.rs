use crate::types::{Bitboard, BCoord, GameMode};
use crate::utils::from_index;


#[derive(Debug, Clone, PartialEq, Eq)]
pub struct GlobalRules {
    // Successfully moving a leader to a win square is an instant win
    pub win_positions_white: Vec<(BCoord, BCoord)>,
    pub win_positions_black: Vec<(BCoord, BCoord)>,
    // If true, a player must capture if they can
    pub capturing_is_forced: bool,
    // If true, a player who is stalemated loses. If false, the game is a draw
    pub stalemated_player_loses: bool,
    // If true, what would be a win for white is a win for black, and vice versa
    pub invert_win_conditions: bool,
    // Number of times that the same position is reached to draw by repetition
    // 0 means no repetition draw
    pub repetitions_draw: u8,
}


impl GlobalRules {
    pub fn for_mode(mode: GameMode) -> GlobalRules {
        match mode {
            GameMode::Standard | GameMode::Atomic | GameMode::Horde => {
                GlobalRules {
                    win_positions_white: vec![],
                    win_positions_black: vec![],
                    capturing_is_forced: false,
                    stalemated_player_loses: false,
                    invert_win_conditions: false,
                    repetitions_draw: 3,
                }
            },
            GameMode::Antichess => {
                GlobalRules {
                    win_positions_white: vec![],
                    win_positions_black: vec![],
                    capturing_is_forced: true,
                    stalemated_player_loses: true,
                    invert_win_conditions: true,
                    repetitions_draw: 3,
                }
            },
            GameMode::KingOfTheHill => {
                GlobalRules {
                    win_positions_white: vec![(3,3), (3,4), (4,3), (4,4)],
                    win_positions_black: vec![(3,3), (3,4), (4,3), (4,4)],
                    capturing_is_forced: false,
                    stalemated_player_loses: false,
                    invert_win_conditions: false,
                    repetitions_draw: 3,
                }
            },
        }
    }
}


#[derive(Debug, Clone, PartialEq, Eq)]
pub struct GlobalRulesInternal {
    pub win_positions: [Bitboard; 2],
    pub capturing_is_forced: bool,
    pub stalemated_player_loses: bool,
    pub invert_win_conditions: bool,
    pub repetition_draw: u8,
}


impl From<GlobalRulesInternal> for GlobalRules {
    fn from(mut rules: GlobalRulesInternal) -> GlobalRules {
        let mut win_positions_white = Vec::new();
        let mut win_positions_black = Vec::new();
        while let Some(sq) = rules.win_positions[0].lowest_one() {
            win_positions_white.push(from_index(sq));
            rules.win_positions[0].clear_bit(sq);
        }
        while let Some(sq) = rules.win_positions[1].lowest_one() {
            win_positions_black.push(from_index(sq));
            rules.win_positions[1].clear_bit(sq);
        }
        GlobalRules {
            win_positions_white,
            win_positions_black,
            capturing_is_forced: rules.capturing_is_forced,
            stalemated_player_loses: rules.stalemated_player_loses,
            invert_win_conditions: rules.invert_win_conditions,
            repetitions_draw: rules.repetition_draw,
        }
    }
}

impl From<GlobalRules> for GlobalRulesInternal {
    fn from(rules: GlobalRules) -> GlobalRulesInternal {
        GlobalRulesInternal {
            win_positions: [
                Bitboard::from_coord_list(&rules.win_positions_white),
                Bitboard::from_coord_list(&rules.win_positions_black),
            ],
            capturing_is_forced: rules.capturing_is_forced,
            stalemated_player_loses: rules.stalemated_player_loses,
            invert_win_conditions: rules.invert_win_conditions,
            repetition_draw: rules.repetitions_draw,
        }
    }
}


impl Default for GlobalRules {
    fn default() -> Self {
        GlobalRules::for_mode(GameMode::Standard)
    }
}

impl Default for GlobalRulesInternal {
    fn default() -> Self {
        GlobalRules::default().into()
    }
}
