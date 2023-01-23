use crate::types::GameMode;


#[derive(Debug, Clone, PartialEq, Eq)]
pub struct GlobalRules {
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
            GameMode::Antichess => {
                GlobalRules {
                    capturing_is_forced: true,
                    stalemated_player_loses: true,
                    invert_win_conditions: true,
                    repetitions_draw: 3,
                }
            },
            _ => {
                GlobalRules {
                    capturing_is_forced: false,
                    stalemated_player_loses: false,
                    invert_win_conditions: false,
                    repetitions_draw: 3,
                }
            },
        }
    }
}

impl Default for GlobalRules {
    fn default() -> Self {
        GlobalRules::for_mode(GameMode::Standard)
    }
}
