use crate::piece::Piece;
use crate::position::piece_set::PieceSet;
use crate::types::{BCoord, BIndex, Bitboard, Move, MoveType};
use crate::position::Position;
use crate::move_generator::attack_tables::AttackTables;
use crate::utils::{from_index, to_index};

pub mod attack_tables;


lazy_static! {
    static ref ATTACK_TABLES: AttackTables = AttackTables::new();
}


#[derive(Clone, Debug)]
pub struct MoveGen;

impl MoveGen {
    pub fn attack_tables() -> &'static AttackTables {
        &ATTACK_TABLES
    }
    
    pub fn get_legal_moves(position: &mut Position) -> Vec<Move> {
        let mut legal_moves = Vec::new();
        for mv in MoveGen::get_pseudo_moves(position, true) {
            if !MoveGen::is_move_legal(mv, position) {
                continue;
            }
            legal_moves.push(mv);
        }
        legal_moves
    }

    /// Iterator that yields pseudo-legal moves from a positon
    pub fn get_pseudo_moves(position: &mut Position, output_translations: bool) -> Vec<Move> {
        let my_pieces = &position.pieces[position.whos_turn as usize];

        let mut out_moves = Vec::with_capacity(50);

        let enemies_or_out_bounds = &position.occ_or_out_bounds & !my_pieces.get_occupied();
        let occ_or_not_in_bounds = &position.occ_or_out_bounds;
        
        for p in my_pieces.iter() {
            p.output_captures(position, &enemies_or_out_bounds, occ_or_not_in_bounds, &mut out_moves);
        }
        let skip_translations = position.global_rules.capturing_is_forced && !out_moves.is_empty();
        if output_translations && !skip_translations {
            for p in my_pieces.iter() {
                p.output_translations(position, &enemies_or_out_bounds, occ_or_not_in_bounds, &mut out_moves);
            }
        }
        out_moves
    }

    /// Checks if the player to move is in check
    pub fn in_check(position: &mut Position) -> bool {
        let my_pieces = &position.pieces[position.whos_turn as usize];
        if let Some(my_leader) = my_pieces.get_leader() {
            if my_leader.get_num_pieces() > 1 {
                // There are multiple leaders, so the position cannot be in check
                return false;
            }
            // There is only one bit set to 1 in the bitboard
            let index = my_leader.get_bitboard().lowest_one().unwrap();
            MoveGen::index_in_check(index, position)
        } else {
            // If I have no leader, I cannot be in check (only lose when all pieces are captured)
            false
        }
    }

    /// Attempts to make a pseudo-legal move, succeeding and returning true only if the move was legal
    pub fn make_move_if_legal(mv: Move, position: &mut Position) -> bool {
        // Cannot castle while in check or step through check
        if mv.is_castling() {
            let kingside = mv.get_move_type() == MoveType::KingsideCastle;
            let from = mv.get_from();
            let to = mv.get_to();
            // Edge case in chess960 where castling does not move the king,
            // in which case the loop below will not check the king's position
            if from == to && MoveGen::index_in_check(from, position) {
                return false;
            }
            let start_index = { if kingside { from } else { to + 1 } };
            let end_index = { if kingside { to - 1 } else { from } };
            // Hide the castling piece from the occupied bitboard so that it doesn't get in the way of check detection
            position.occ_or_out_bounds.clear_bit(from);
            for step_index in start_index..=end_index {
                if MoveGen::index_in_check(step_index, position) {
                    position.occ_or_out_bounds.set_bit(from);
                    return false;
                }
            }
            position.occ_or_out_bounds.set_bit(from);
        }
        
        // Try the move and skip a turn, then see if we are in check
        // Also, if after making the move the enemy is in check, the move is illegal if check_is_forbidden
        position.make_move(mv);
        if position.global_rules.check_is_forbidden && MoveGen::in_check(position) {
            position.unmake_move();
            return false;
        }
        position.make_move(Move::null());
        // See if we are in check or an explosion has killed the last leader
        // However, if the move causes us to capture the last enemy leader, the move is legal (even if it leaves us in check)
        let legal = !position.leader_is_captured() && (position.enemy_leader_is_captured() || !MoveGen::in_check(position));
        position.unmake_move();
        if !legal {
            // If the move is illegal, clean up the position
            position.unmake_move();
        }
        legal
    }
    
    /// Checks if a move is legal
    pub fn is_move_legal(mv: Move, position: &mut Position) -> bool {
        let legal = Self::make_move_if_legal(mv, position);
        // Restore previous state of the position
        if legal {
            position.unmake_move();
        }
        legal
    }


    ///Returns the number of legal moves for a position
    pub fn count_legal_moves(position: &mut Position) -> u64{
        let mut nodes = 0u64;
        for mv in MoveGen::get_pseudo_moves(position, true) {
            if !MoveGen::is_move_legal(mv, position) {
                continue;
            }
            nodes += 1;
        }
        nodes
    }
    
    
    /// Checks if a given square is attacked by the enemy
    fn index_in_check(index: BIndex, position: &mut Position) -> bool {
        let (x, y) = from_index(index);
        let enemy = 1 - position.whos_turn;
        let enemy_pieces = &position.pieces[enemy as usize];
        let enemy_occupied = enemy_pieces.get_occupied();
        let (inverse_attack, jumps) = enemy_pieces.get_inverse_attack(index);
        // Use inverse attack pattern to get the squares that can potentially attack the square
        let attack_tables = MoveGen::attack_tables();
        let occ_or_not_in_bounds = &position.occ_or_out_bounds;
        
        let mut slides = attack_tables.get_sliding_moves_bb(
            index,
            occ_or_not_in_bounds,
            inverse_attack.attack_north,
            inverse_attack.attack_east,
            inverse_attack.attack_south,
            inverse_attack.attack_west,
            inverse_attack.attack_northeast,
            inverse_attack.attack_northwest,
            inverse_attack.attack_southeast,
            inverse_attack.attack_southwest
        );
        slides &= enemy_occupied;
        while let Some(enemy_piece_index) = slides.lowest_one() {
            // Found an enemy piece that might attack the last leader
            let enemy_piece = enemy_pieces.piece_at(enemy_piece_index).unwrap();
            // If this attack will kill the remaining enemy leaders, the move is illegal so it is not a check
            let kills_remaining_leaders = enemy_piece.explodes() && explosion_kills_enemy(index, enemy_pieces, enemy_piece, enemy_piece_index);
            if !kills_remaining_leaders && MoveGen::slide_targets_coords(x, y, enemy_piece, enemy_piece_index) {
                return true;
            }
            slides.clear_bit(enemy_piece_index);
        }
        
        // Check jump attacks
        let mut jump_attacks = jumps & enemy_occupied;
        while let Some(enemy_piece_index) = jump_attacks.lowest_one() {
            // Found an enemy piece that might attack the last leader
            let enemy_piece = enemy_pieces.piece_at(enemy_piece_index).unwrap();
            // If this attack will kill the remaining enemy leaders, the move is illegal so it is not a check
            let kills_remaining_leaders = enemy_piece.explodes() && explosion_kills_enemy(index, enemy_pieces, enemy_piece, enemy_piece_index);
            if !kills_remaining_leaders && enemy_piece.get_capture_jumps(enemy_piece_index).get_bit(index) {
                return true;
            }
            jump_attacks.clear_bit(enemy_piece_index);
        }
        
        // Check sliding deltas
        for run in &inverse_attack.attack_sliding_deltas {
            for (dx, dy) in run {
    
                let (x2, y2) = (x as i8 + *dx, y as i8 + *dy);
                if x2 < 0 || y2 < 0 {
                    break;
                }
                //Out of bounds, next sliding moves can be ignored
                if !position.in_bounds(x2 as BCoord, y2 as BCoord) {
                    break;
                }
                let to = to_index(x2 as BCoord, y2 as BCoord);
                if enemy_occupied.get_bit(to) {
                    // Found an enemy piece that might attack the last leader
                    let enemy_piece = enemy_pieces.piece_at(to).unwrap();
                    // If this attack will kill the remaining enemy leaders, the move is illegal so it is not a check
                    let kills_remaining_leaders = enemy_piece.explodes() && explosion_kills_enemy(index, enemy_pieces, enemy_piece, to);
                    if !kills_remaining_leaders && MoveGen::sliding_delta_targets_index(enemy_piece, to, index, occ_or_not_in_bounds) {
                        return true;
                    }
                    break;
                }
                //Occupied by own team
                if position.occ_or_out_bounds.get_bit(to) {
                    break;
                }
            }
        }
        false
    }
    fn slide_targets_coords(x: BCoord, y: BCoord, piece: &Piece, piece_index: BIndex) -> bool {
        // We already know that this piece is on the same rank, file, diagonal or antidiagonal as the target (x, y)
        let piece_movement = piece.get_movement();
        let (px, py) = from_index(piece_index);
        if px == x {
            // Same file
            if py < y { piece_movement.attack_north }
            else { piece_movement.attack_south }
        } else if py == y {
            // Same rank
            if px < x { piece_movement.attack_east }
            else { piece_movement.attack_west }
        } else if px < x {
            // Same diagonal
            if py < y { piece_movement.attack_northeast}
            // Same antidiagonal
            else { piece_movement.attack_southeast }
        } else {
            // Same antidiagonal
            if py < y { piece_movement.attack_northwest }
            // Same diagonal
            else { piece_movement.attack_southwest }
        }
    }
    fn sliding_delta_targets_index(piece: &Piece, piece_index: BIndex, target_index: BIndex, occ_or_not_in_bounds: &Bitboard) -> bool {
        let piece_movement = piece.get_movement();
        let (x, y) = from_index(piece_index);
        for run in &piece_movement.attack_sliding_deltas {
            for (dx, dy) in run {
                let (x2, y2) = (x as i8 + *dx, y as i8 + *dy);
                if x2 < 0 || y2 < 0 || x2 > 15 || y2 > 15 {
                    break;
                }
                let to = to_index(x2 as BCoord, y2 as BCoord);
                if to == target_index {
                    return true;
                }
                // Occupied or out of bounds
                if occ_or_not_in_bounds.get_bit(to) {
                    break;
                }
            }
        }
        false
    }
}


/// Returns true if an explosion in this coordinates would kill all the remaining enemy leaders
fn explosion_kills_enemy(index: BIndex, enemy_pieces: &PieceSet, enemy_piece: &Piece, enemy_piece_index: u8) -> bool {
    if let Some(enemy_leader) = enemy_pieces.get_leader() {
        let mut killed_enemy_leaders = 0;
        let affected_squares = enemy_leader.get_bitboard() & enemy_piece.get_explosion(index);
        // Take into account that the attacking piece might be a leader from far away
        if enemy_piece.is_leader() && !affected_squares.get_bit(enemy_piece_index) {
            killed_enemy_leaders += 1;
        }
        killed_enemy_leaders += affected_squares.count_ones();
        killed_enemy_leaders == enemy_leader.get_num_pieces()
    } else {
        // If the enemy has no leaders, then they only lose when all pieces are captured
        let mut killed_enemies = 0;
        let affected_squares = enemy_pieces.get_occupied() & enemy_piece.get_explosion(index);
        // Take into account that the attacking piece might be far away
        if !affected_squares.get_bit(enemy_piece_index) {
            killed_enemies += 1;
        }
        killed_enemies += affected_squares.count_ones();
        killed_enemies == enemy_pieces.get_occupied().count_ones()
    }
}
