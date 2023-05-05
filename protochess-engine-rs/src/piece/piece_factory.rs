use super::PieceDefinition;
use crate::types::{GameMode, BCoord};

pub struct PieceFactory {
    mode: GameMode
}

impl PieceFactory {
    pub fn new(mode: GameMode) -> PieceFactory {
        PieceFactory { mode }
    }
    
    pub fn make_piece_set(&self,  width: BCoord, height: BCoord) -> Vec<PieceDefinition> {
        vec![
            self.make_king(width, height),
            self.make_queen(),
            self.make_rook(),
            self.make_bishop(),
            self.make_knight(),
            self.make_pawn(true, width, height),
            self.make_pawn(false, width, height),
        ]
    } 
    
    pub fn make_pawn(&self, is_white: bool, width: BCoord, height: BCoord) -> PieceDefinition {
        let promotion_rank = { if is_white { height-1 } else { 0 } };
        let double_move_rank1 = { if is_white { 1 } else { height-2 } };
        let double_move_rank2 = { if is_white { 0 } else { height-1 } }; // Needed for horde
        let mut promotion_squares = vec![];
        let mut double_jump_squares = vec![];
        for i in 0..width {
            promotion_squares.push((i, promotion_rank));
            double_jump_squares.push((i, double_move_rank1));
            double_jump_squares.push((i, double_move_rank2));
        }
        let move_dir = { if is_white { 1 } else { -1 } };
        let ids = {
            if self.mode == GameMode::RacingKings {
                [None, None]
            } else if is_white {
                [Some('P'), None]
            } else {
                [None, Some('p')]
            }
        };
        let mut promo_vals = [vec!['Q', 'R', 'B', 'N'], vec!['q', 'r', 'b', 'n']];
        if self.mode == GameMode::Antichess {
            promo_vals[0].push('K');
            promo_vals[1].push('k');
        }
        
        PieceDefinition {
            ids,
            is_leader: false,
            castle_files: None,
            is_castle_rook: false,
            explode_on_capture: self.mode == GameMode::Atomic,
            explosion_deltas: vec![(-1, -1), (-1, 0), (-1, 1), (0, -1), (0, 1), (1, -1), (1, 0), (1, 1)],
            immune_to_explosion: true,
            promotion_squares,
            double_jump_squares,
            promo_vals,
            attack_sliding_deltas: vec![],
            attack_jump_deltas: vec![(-1, move_dir), (1, move_dir)],
            attack_north: false,
            attack_south: false,
            attack_east: false,
            attack_west: false,
            attack_northeast: false,
            attack_northwest: false,
            attack_southeast: false,
            attack_southwest: false,
            translate_jump_deltas: vec![(0, move_dir)],
            translate_sliding_deltas: vec![],
            translate_north: false,
            translate_south: false,
            translate_east: false,
            translate_west: false,
            translate_northeast: false,
            translate_northwest: false,
            translate_southeast: false,
            translate_southwest: false,
            win_squares: vec![],
        }
    }
    
    pub fn make_knight(&self) -> PieceDefinition {
        PieceDefinition {
            ids: [Some('N'), Some('n')],
            is_leader: false,
            castle_files: None,
            is_castle_rook: false,
            explode_on_capture: self.mode == GameMode::Atomic,
            explosion_deltas: vec![(-1, -1), (-1, 0), (-1, 1), (0, -1), (0, 1), (1, -1), (1, 0), (1, 1)],
            immune_to_explosion: false,
            promotion_squares: vec![],
            double_jump_squares: vec![],
            promo_vals: [vec![], vec![]],
            attack_sliding_deltas: vec![],
            attack_jump_deltas: vec![(1, 2), (1, -2), (-1, 2), (-1, -2), (2, 1), (2, -1), (-2, 1), (-2, -1)],
            attack_north: false,
            attack_south: false,
            attack_east: false,
            attack_west: false,
            attack_northeast: false,
            attack_northwest: false,
            attack_southeast: false,
            attack_southwest: false,
            translate_jump_deltas: vec![(1, 2), (1, -2), (-1, 2), (-1, -2), (2, 1), (2, -1), (-2, 1), (-2, -1)],
            translate_sliding_deltas: vec![],
            translate_north: false,
            translate_south: false,
            translate_east: false,
            translate_west: false,
            translate_northeast: false,
            translate_northwest: false,
            translate_southeast: false,
            translate_southwest: false,
            win_squares: vec![],
        }
    }
    
    pub fn make_bishop(&self) -> PieceDefinition {
        PieceDefinition {
            ids: [Some('B'), Some('b')],
            is_leader: false,
            castle_files: None,
            is_castle_rook: false,
            explode_on_capture: self.mode == GameMode::Atomic,
            explosion_deltas: vec![(-1, -1), (-1, 0), (-1, 1), (0, -1), (0, 1), (1, -1), (1, 0), (1, 1)],
            immune_to_explosion: false,
            promotion_squares: vec![],
            double_jump_squares: vec![],
            promo_vals: [vec![], vec![]],
            attack_sliding_deltas: vec![],
            attack_jump_deltas: vec![],
            attack_north: false,
            attack_south: false,
            attack_east: false,
            attack_west: false,
            attack_northeast: true,
            attack_northwest: true,
            attack_southeast: true,
            attack_southwest: true,
            translate_jump_deltas: vec![],
            translate_sliding_deltas: vec![],
            translate_north: false,
            translate_south: false,
            translate_east: false,
            translate_west: false,
            translate_northeast: true,
            translate_northwest: true,
            translate_southeast: true,
            translate_southwest: true,
            win_squares: vec![],
        }
    }
    
    pub fn make_rook(&self) -> PieceDefinition {
        PieceDefinition {
            ids: [Some('R'), Some('r')],
            is_leader: false,
            castle_files: None,
            is_castle_rook: true,
            explode_on_capture: self.mode == GameMode::Atomic,
            explosion_deltas: vec![(-1, -1), (-1, 0), (-1, 1), (0, -1), (0, 1), (1, -1), (1, 0), (1, 1)],
            immune_to_explosion: false,
            promotion_squares: vec![],
            double_jump_squares: vec![],
            promo_vals: [vec![], vec![]],
            attack_sliding_deltas: vec![],
            attack_jump_deltas: vec![],
            attack_north: true,
            attack_south: true,
            attack_east: true,
            attack_west: true,
            attack_northeast: false,
            attack_northwest: false,
            attack_southeast: false,
            attack_southwest: false,
            translate_jump_deltas: vec![],
            translate_sliding_deltas: vec![],
            translate_north: true,
            translate_south: true,
            translate_east: true,
            translate_west: true,
            translate_northeast: false,
            translate_northwest: false,
            translate_southeast: false,
            translate_southwest: false,
            win_squares: vec![],
        }
    }
    
    pub fn make_king(&self, width: BCoord, height: BCoord) -> PieceDefinition {
        let win_squares = {
            if self.mode == GameMode::KingOfTheHill {
                vec![(3,3), (3,4), (4,3), (4,4)]
            } else if self.mode == GameMode::RacingKings {
                let mut squares = Vec::new();
                for i in 0..width {
                    squares.push((i, height-1));
                }
                squares
            } else {
                vec![]
            }
        };
        let can_castle = self.mode != GameMode::Antichess && self.mode != GameMode::RacingKings;
        let ids = {
            if self.mode == GameMode::Horde { [None, Some('k')] }
            else { [Some('K'), Some('k')] }
        };
        PieceDefinition {
            ids,
            is_leader: self.mode != GameMode::Antichess,
            castle_files: if can_castle { Some((2, 6)) } else { None },
            is_castle_rook: false,
            explode_on_capture: self.mode == GameMode::Atomic,
            explosion_deltas: vec![(-1, -1), (-1, 0), (-1, 1), (0, -1), (0, 1), (1, -1), (1, 0), (1, 1)],
            immune_to_explosion: false,
            promotion_squares: vec![],
            double_jump_squares: vec![],
            promo_vals: [vec![], vec![]],
            attack_sliding_deltas: vec![],
            attack_jump_deltas: vec![(-1, -1), (-1, 0), (-1, 1), (0, -1), (0, 1), (1, -1), (1, 0), (1, 1)],
            attack_north: false,
            attack_south: false,
            attack_east: false,
            attack_west: false,
            attack_northeast: false,
            attack_northwest: false,
            attack_southeast: false,
            attack_southwest: false,
            translate_jump_deltas: vec![(-1, -1), (-1, 0), (-1, 1), (0, -1), (0, 1), (1, -1), (1, 0), (1, 1)],
            translate_sliding_deltas: vec![],
            translate_north: false,
            translate_south: false,
            translate_east: false,
            translate_west: false,
            translate_northeast: false,
            translate_northwest: false,
            translate_southeast: false,
            translate_southwest: false,
            win_squares,
        }
    }
    
    pub fn make_queen(&self) -> PieceDefinition {
        PieceDefinition {
            ids: [Some('Q'), Some('q')],
            is_leader: false,
            castle_files: None,
            is_castle_rook: false,
            explode_on_capture: self.mode == GameMode::Atomic,
            explosion_deltas: vec![(-1, -1), (-1, 0), (-1, 1), (0, -1), (0, 1), (1, -1), (1, 0), (1, 1)],
            immune_to_explosion: false,
            promotion_squares: vec![],
            double_jump_squares: vec![],
            promo_vals: [vec![], vec![]],
            attack_sliding_deltas: vec![],
            attack_jump_deltas: vec![],
            attack_north: true,
            attack_south: true,
            attack_east: true,
            attack_west: true,
            attack_northeast: true,
            attack_northwest: true,
            attack_southeast: true,
            attack_southwest: true,
            translate_jump_deltas: vec![],
            translate_sliding_deltas: vec![],
            translate_north: true,
            translate_south: true,
            translate_east: true,
            translate_west: true,
            translate_northeast: true,
            translate_northwest: true,
            translate_southeast: true,
            translate_southwest: true,
            win_squares: vec![],
        }
    }
}

impl Default for PieceFactory {
    fn default() -> Self {
        PieceFactory { mode: GameMode::Standard }
    }
}
