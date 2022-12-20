use crate::constants::piece_scores::*;
use crate::types::*;
use crate::position::Position;
use crate::position::piece_set::PieceSet;
use crate::move_generator::attack_tables::AttackTables;
use crate::move_generator::bitboard_moves::BitboardMoves;
use crate::utils::{from_index, to_index};

pub mod attack_tables;
pub mod bitboard_moves;


lazy_static! {
    static ref ATTACK_TABLES: AttackTables = AttackTables::new();
}


#[derive(Clone, Debug)]
pub struct MoveGen { }
impl MoveGen {
    
    // TODO: Remove this function
    pub fn attack_tables() -> &'static AttackTables {
        &ATTACK_TABLES
    }
    
    pub fn get_legal_moves_as_tuples(position: &mut Position) -> Vec<((BCoord,BCoord), (BCoord,BCoord))> {
        let mut legal_tuples = Vec::new();
        for mv in MoveGen::get_pseudo_moves(position) {
            if !MoveGen::is_move_legal(&mv, position) {
                continue;
            }
            legal_tuples.push((from_index(mv.get_from()), from_index(mv.get_to())));
        }
        legal_tuples
    }

    /// Iterator that yields pseudo-legal moves from a position
    pub fn get_pseudo_moves(position:&mut Position) -> impl Iterator<Item=Move> {
        MoveGen::get_classical_pseudo_moves(position)
            .chain(MoveGen::get_custom_psuedo_moves(position))
    }

    ///Iterator that yields only capture moves
    pub fn get_capture_moves(position:&mut Position) -> impl Iterator<Item=Move> {
        MoveGen::get_classical_pseudo_moves(position).filter(|x| x.is_capture())
            .chain(MoveGen::get_custom_psuedo_moves(position).filter(|x| x.is_capture()))
    }

    /// Iterator that yields pseudo-legal moves from a position
    /// Considering only the classical piece set
    fn get_classical_pseudo_moves(position:&mut Position) -> impl Iterator<Item=Move> {
        let my_pieces: &PieceSet = &position.pieces[position.whos_turn as usize];
        let enemies = &position.occupied & !&my_pieces.occupied;

        //create a vector of iterators
        let mut iters:Vec<BitboardMoves> = Vec::with_capacity(6);
        let occ_or_not_in_bounds = &position.occupied | !&position.dimensions.bounds;

        let mut apply_to_each = |mut pieceset:Bitboard, func: fn(&AttackTables, BIndex, &Bitboard, &Bitboard)-> Bitboard| {
            while !pieceset.is_zero() {
                let index = pieceset.lowest_one().unwrap();
                let mut raw_attacks = func(&ATTACK_TABLES, index, &occ_or_not_in_bounds, &enemies);
                //Do not attack ourselves
                raw_attacks &= !&my_pieces.occupied;
                //Keep only in bounds
                raw_attacks &= &position.dimensions.bounds;
                iters.push(BitboardMoves::new(
                    enemies.to_owned(),
                    raw_attacks,
                    index,
                    Bitboard::zero(),
                    Vec::new(),
                ));
                pieceset.clear_bit(index);
            }
        };

        apply_to_each(my_pieces.king.bitboard.to_owned(), AttackTables::get_king_attack);
        apply_to_each(my_pieces.queen.bitboard.to_owned(), AttackTables::get_queen_attack);
        apply_to_each(my_pieces.rook.bitboard.to_owned(), AttackTables::get_rook_attack);
        apply_to_each(my_pieces.bishop.bitboard.to_owned(), AttackTables::get_bishop_attack);
        apply_to_each(my_pieces.knight.bitboard.to_owned(), AttackTables::get_knight_attack);

        let mut extra_moves = Vec::new();
        let mut p_copy = my_pieces.pawn.bitboard.to_owned();
        while !p_copy.is_zero() {
            let index = p_copy.lowest_one().unwrap();
            let mut raw_attacks = {
                if position.whos_turn == 0 {
                    ATTACK_TABLES.get_north_pawn_attack(index, &position.occupied, &enemies)
                } else {
                    ATTACK_TABLES.get_south_pawn_attack(index, &position.occupied, &enemies)
                }
            };
            //Do not attack ourselves
            raw_attacks &= !&my_pieces.occupied;
            //Keep only in bounds
            raw_attacks &= &position.dimensions.bounds;
            let promotion_squares = {
                if position.whos_turn == 0 {
                    ATTACK_TABLES.masks.get_rank(position.dimensions.height - 1).to_owned()
                } else {
                    ATTACK_TABLES.masks.get_rank(0).to_owned()
                }
            };
            // TODO: Store these as IDs in the Piece struct
            let promo_vals = vec![ID_ROOK, ID_BISHOP, ID_KNIGHT, ID_QUEEN];
            iters.push(BitboardMoves::new(
                enemies.to_owned(),
                raw_attacks,
                index,
                promotion_squares,
                promo_vals
            ));
            //Check EP
            if let Some(ep_sq) = position.properties.ep_square {
                let attack_only = {
                    if position.whos_turn == 0 {
                        ATTACK_TABLES.get_north_pawn_attack_raw(index) & !(&my_pieces.occupied)
                    } else {
                        ATTACK_TABLES.get_south_pawn_attack_raw(index) & !(&my_pieces.occupied)
                    }
                };
                if attack_only.get_bit(ep_sq) {
                    let (cap_x, mut cap_y) = from_index(ep_sq);

                    if position.whos_turn == 0 {
                        cap_y -= 1;
                    } else {
                        cap_y += 1;
                    }
                    let mv = Move::new(index, ep_sq,  Some(to_index(cap_x,cap_y)), MoveType::Capture, None);
                    extra_moves.push(mv);
                }
            }
            p_copy.clear_bit(index);
        }
        //Castling
        if let Some(king_index) = my_pieces.king.bitboard.lowest_one() {
            let (kx, ky) = from_index(king_index);
            let whos_turn = position.whos_turn;
            if position.properties.castling_rights.can_player_castle_kingside(position.whos_turn) {
                let rook_index = to_index(position.dimensions.width - 1, ky);
                if let Some(pt) = position.piece_at(rook_index) {
                    if pt.get_player() == whos_turn && pt.get_piece_id() == ID_ROOK {
                        //See if the space between is clear
                        let east = ATTACK_TABLES.masks.get_east(king_index);
                        let mut occ = east & &position.occupied;
                        occ.clear_bit(rook_index);
                        if occ.is_zero() {
                            //See if we can move the king one step east without stepping into check
                            let king_one_step_indx = to_index(kx + 1, ky);
                            if MoveGen::is_move_legal(&Move::null(), position)
                                && MoveGen::is_move_legal(&Move::new(king_index, king_one_step_indx, None, MoveType::Quiet, None), position)
                            {
                                let to_index = to_index(kx + 2, ky);
                                extra_moves.push(Move::new(king_index, to_index, Some(rook_index), MoveType::KingsideCastle, None));
                            }
                        }
                    }
                }
            }
            if position.properties.castling_rights.can_player_castle_queenside(position.whos_turn) {
                let rook_index = to_index(0 ,ky);
                if let Some(pt) = position.piece_at(rook_index) {
                    if pt.get_player() == whos_turn && pt.get_piece_id() == ID_ROOK {
                        let west = ATTACK_TABLES.masks.get_west(king_index);
                        let mut occ = west & &position.occupied;
                        occ.clear_bit(rook_index);

                        if occ.is_zero() {
                            //See if we can move the king one step east without stepping into check
                            let king_one_step_indx = to_index(kx - 1, ky);
                            if MoveGen::is_move_legal(&Move::null(), position)
                                && MoveGen::is_move_legal(&Move::new(king_index, king_one_step_indx, None, MoveType::Quiet, None), position)
                            {
                                let to_index = to_index(kx - 2, ky);
                                extra_moves.push(Move::new(king_index, to_index, Some(rook_index), MoveType::QueensideCastle, None));
                            }
                        }
                    }
                }
            }
        }


        //Flatten our vector of iterators and combine with ep moves
        iters.into_iter().flatten().chain(extra_moves.into_iter())
    }

    /// Iterator that yields pseudo-legal moves from a positon
    /// Considering only custom piece types
    fn get_custom_psuedo_moves(position: &Position) -> impl Iterator<Item=Move> {
        let my_pieces: &PieceSet = &position.pieces[position.whos_turn as usize];

        let mut out_bb_moves: Vec<BitboardMoves> = Vec::with_capacity(50);
        let mut out_moves = Vec::with_capacity(50);

        let enemies = &position.occupied & !&my_pieces.occupied;
        let occ_or_not_in_bounds = &position.occupied | !&position.dimensions.bounds;

        for p in &my_pieces.custom {
            p.output_moves(&position.dimensions, &position.occupied, &enemies, &occ_or_not_in_bounds, &mut out_bb_moves, &mut out_moves);
        }
        out_bb_moves.into_iter().flatten().chain(out_moves.into_iter())
    }

    // TODO: Remove this function
    /// Returns whether or not a player is in check for a given position
    fn is_in_check_from_king(position: &Position, my_player_num: Player) -> bool {
        let my_pieces: &PieceSet = &position.pieces[my_player_num as usize];
        let enemies = &position.occupied & !&my_pieces.occupied;
        let occ_or_not_in_bounds = &position.occupied | !&position.dimensions.bounds;

        //Calculate enemies piece sets
        let enemy_pieces: &PieceSet = &position.pieces[position.whos_turn as usize];
        //TODO generalize for >2 players
        let enemy_pawns = &enemy_pieces.pawn.bitboard;
        let enemy_knights = &enemy_pieces.knight.bitboard;
        let enemy_bishops = &enemy_pieces.bishop.bitboard;
        let enemy_queens = &enemy_pieces.queen.bitboard;
        let enemy_rooks = &enemy_pieces.rook.bitboard;
        let enemy_kings = &enemy_pieces.king.bitboard;

        let loc_index = my_pieces.king.bitboard.lowest_one().unwrap();

        //Pawn
        let patt = {
            if my_player_num == 0 {
                ATTACK_TABLES.get_north_pawn_attack_masked(loc_index, &occ_or_not_in_bounds, &enemies)
            } else {
                ATTACK_TABLES.get_south_pawn_attack_masked(loc_index, &occ_or_not_in_bounds, &enemies)
            }
        };

        if !(patt & enemy_pawns).is_zero() {
            return true;
        };

        //Knight
        let natt = ATTACK_TABLES.get_knight_attack(loc_index, &occ_or_not_in_bounds, &enemies);
        if !(natt & enemy_knights).is_zero() {
            return true;
        };
        //King
        let katt = ATTACK_TABLES.get_king_attack(loc_index, &occ_or_not_in_bounds, &enemies);
        if !(katt & enemy_kings).is_zero() {
            return true;
        };

        //Rook & Queen
        let ratt = ATTACK_TABLES.get_rook_attack(loc_index, &occ_or_not_in_bounds, &enemies);
        if !(&ratt & enemy_queens).is_zero() || !(&ratt & enemy_rooks).is_zero() {
            return true;
        };
        //Bishop & Queen
        let batt = ATTACK_TABLES.get_bishop_attack(loc_index, &occ_or_not_in_bounds, &enemies);
        if !(&batt & enemy_queens).is_zero() || !(&batt & enemy_bishops).is_zero() {
            return true;
        };

        false
    }

    ///Checks if the player to move is in check
    pub fn in_check(position:&mut Position) -> bool {
        let my_player_num = position.whos_turn;
        let mut in_check = false;
        // Pass turn
        position.make_move(Move::null());
        if MoveGen::is_in_check_from_king(position, my_player_num) {
            in_check = true;
        }
        // See if the opponent has a move that captures our last leader
        for mv in MoveGen::get_custom_psuedo_moves(position) {
            if mv.is_capture() && position.piece_at(mv.get_target()).unwrap().potential_checkmate() {
                in_check = true;
                break;
            }
        }
        position.unmake_move();
        in_check
    }

    ///Checks if a move is legal
    pub fn is_move_legal(mv:&Move, position:&mut Position) -> bool{
        let my_player_num = position.whos_turn;
        let mut legality = true;
        // Try the move
        position.make_move(mv.to_owned());
        if MoveGen::is_in_check_from_king(position, my_player_num) {
            legality = false;
        }
        // See if the opponent has a move that captures our last leader
        for mv in MoveGen::get_custom_psuedo_moves(position)  {
            if mv.is_capture() && position.piece_at(mv.get_target()).unwrap().potential_checkmate() {
                legality = false;
                break;
            }
        }
        position.unmake_move();
        legality
    }


    ///Returns the number of legal moves for a position
    pub fn count_legal_moves(position: &mut Position) -> u64{
        let mut nodes = 0u64;
        for mv in MoveGen::get_pseudo_moves(position) {
            if !MoveGen::is_move_legal(&mv, position) {
                continue;
            }
            nodes += 1;
        }
        nodes
    }
}
