use crate::types::*;
use crate::position::Position;
use crate::move_generator::attack_tables::AttackTables;
use crate::move_generator::bitboard_moves::BitboardMoves;
use crate::utils::from_index;

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


    ///Iterator that yields only capture moves
    pub fn get_capture_moves(position: &mut Position) -> impl Iterator<Item=Move> {
        MoveGen::get_pseudo_moves(position).filter(|x| x.is_capture())
    }

    /// Iterator that yields pseudo-legal moves from a positon
    /// Considering only custom piece types
    pub fn get_pseudo_moves(position: &mut Position) -> impl Iterator<Item=Move> {
        let my_pieces = &position.pieces[position.whos_turn as usize];

        let mut out_bb_moves: Vec<BitboardMoves> = Vec::with_capacity(50);
        let mut out_moves = Vec::with_capacity(50);

        let enemies = &position.occupied & !&my_pieces.occupied;
        let occ_or_not_in_bounds = &position.occupied | !&position.dimensions.bounds;

        for p in &my_pieces.custom {
            p.output_moves(position, &enemies, &occ_or_not_in_bounds, &mut out_bb_moves, &mut out_moves);
        }
        out_bb_moves.into_iter().flatten().chain(out_moves.into_iter())
    }

    /// Checks if the player to move is in check
    pub fn in_check(position:&mut Position) -> bool {
        // The position is in check if skipping the turn causes the opponent to have a move that captures the last leader
        !MoveGen::is_move_legal(&Move::null(), position)
    }

    /// Checks if a move is legal
    pub fn is_move_legal(mv: &Move, position: &mut Position) -> bool {
        
        // If the move is castling, check extra conditions
        if mv.get_move_type() == MoveType::KingsideCastle || mv.get_move_type() == MoveType::QueensideCastle {
            // Cannot castle while in check
            if MoveGen::in_check(position) {
                return false;
            }
            let step_index = { if mv.get_move_type() == MoveType::KingsideCastle { mv.get_from()+1 } else { mv.get_from()-1 } };
            let step_mv = Move::new(mv.get_from(), step_index, None, MoveType::Quiet, None);
            // Cannot step through check
            if !MoveGen::is_move_legal(&step_mv, position) {
                return false;
            }
        }
        
        let mut legality = true;
        // Try the move
        position.make_move(mv.to_owned());
        // See if the opponent has a move that captures our last leader
        for mv in MoveGen::get_pseudo_moves(position)  {
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
