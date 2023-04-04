use crate::{PieceDefinition, InitialState, GlobalRules};
use crate::types::GameMode;
use crate::piece::PieceFactory;


/// This is a factory for creating InitialState objects for different hardcoded variants.
/// It's mainly used for testing, but could be used for other purposes as well.
pub struct VariantFactory {
    mode: GameMode,
}

impl VariantFactory {
    pub fn new(mode: GameMode) -> Self {
        Self { mode }
    }
    
    pub fn get_piece_set(&self) -> Vec<PieceDefinition> {
        PieceFactory::new(self.mode).make_piece_set(8, 8)
    }
    
    pub fn make_initial_state(&self) -> InitialState {
        // For now, these are hardcoded to 8x8
        let fen = match self.mode {
            GameMode::Horde => "rnbqkbnr/pppppppp/8/1PP2PP1/PPPPPPPP/PPPPPPPP/PPPPPPPP/PPPPPPPP",
            GameMode::RacingKings => "8/8/8/8/8/8/krbnNBRK/qrbnNBRQ",
            _ => "rnbqkbnr/pppppppp/8/8/8/8/PPPPPPPP/RNBQKBNR",
        }.to_string();
        
        let global_rules = GlobalRules::for_mode(self.mode);
        
        // Same for all variants
        let player_to_move = 0;
        let piece_types = self.get_piece_set();
        let board_width = 8;
        let board_height = 8;
        
        InitialState {
            fen,
            player_to_move,
            piece_types,
            board_width,
            board_height,
            global_rules,
        }
    }
}
