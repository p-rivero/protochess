use crate::types::Player;

#[derive(Clone, Debug)]
pub struct CastleRights(u8, u8, u8);

/// Castling rights for up to 8 players
/// CastleRights.0 -- kingside rights
/// CastleRights.1 -- Queenside rights
/// CastleRights.2 -- 1 if the player actually castled
/// Where each bit in the u8 represents the castling right for the player at that index
/// Ex if CastleRights.0 == 1 then the 0th player can castle kingside
impl CastleRights {
    pub fn new() -> CastleRights {
        CastleRights(u8::MAX, u8::MAX, 0)
    }

    pub fn can_player_castle_kingside(&self, playernum: Player) -> bool {
        (self.0 >> playernum) & 1 != 0
    }

    pub fn can_player_castle_queenside(&self, playernum: Player) -> bool {
        (self.1 >> playernum) & 1 != 0
    }

    pub fn can_player_castle(&self, playernum: Player) -> bool {
        self.can_player_castle_kingside(playernum) ||
            self.can_player_castle_queenside(playernum)
    }

    pub fn did_player_castle(&self, playernum: Player) -> bool {
        (self.2 >> playernum) & 1 != 0
    }

    pub fn set_player_castled(&mut self, playernum: Player) {
        self.2 |= 1 << playernum
    }

    pub fn disable_kingside_castle(&mut self, playernum:  Player) {
        self.0 &= !(1 << playernum)
    }

    pub fn disable_queenside_castle(&mut self, playernum: Player) {
        self.1 &= !(1 << playernum)
    }
}
