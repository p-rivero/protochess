use crate::types::Player;

#[derive(Clone, Debug, Copy)]
pub struct CastledPlayers(u8);

/// Castling rights for up to 8 players
/// CastleRights.0 -- kingside rights
/// CastleRights.1 -- Queenside rights
/// CastleRights.2 -- 1 if the player actually castled
/// Where each bit in the u8 represents the castling right for the player at that index
/// Ex if CastleRights.0 == 1 then the 0th player can castle kingside
impl CastledPlayers {
    pub fn new() -> CastledPlayers {
        CastledPlayers(0)
    }

    pub fn did_player_castle(&self, player_num: Player) -> bool {
        (self.0 >> player_num) & 1 != 0
    }
    
    pub fn set_player_castled(&mut self, player_num: Player) {
        self.0 |= 1 << player_num;
    }
}

impl Default for CastledPlayers {
    fn default() -> Self {
        Self::new()
    }
}
