use crate::types::{BCoord, Move, MoveType};
use crate::Position;

use super::from_index;

/// Converts an (x, y) location to chess rank-file notation
/// Ex: `to_rank_file(0, 1)` = a2
pub fn to_rank_file(x: BCoord, y: BCoord) -> String {
    format!("{}{}", (b'a' + x) as char, (y + 1))
}
pub fn tuple_to_rank_file((x, y): (BCoord, BCoord)) -> String {
    to_rank_file(x, y)
}


/// Converts the move to user-friendly algebraic notation
/// **IMPORTANT:** Call this **before** making the move
pub fn get_algebraic_notation(pos: &mut Position, mv: Move, all_moves: &[Move]) -> String {
    if mv.is_castling() {
        return castling_notation(mv, all_moves);
    }
    let piece = pos.piece_at(mv.get_from()).unwrap();
    let prefix = piece.get_notation_prefix();
    let disamb = disambiguate(pos, mv, all_moves);
    
    let capture = if mv.is_capture() { "x" } else { "" };
    
    let to = tuple_to_rank_file(from_index(mv.get_to()));
    
    let promo = {
        if mv.is_promotion() {
            let promo_piece = pos.lookup_piece(mv.get_promotion_piece().unwrap()).unwrap();
            let promo = promo_piece.get_notation_prefix();
            format!("={promo}")
        } else {
            "".to_string()
        }
    };
    
    let ep = if mv.is_en_passant() { " e.p." } else { "" };
    
    format!("{prefix}{disamb}{capture}{to}{promo}{ep}")
}
pub fn add_suffix(mv: String, suf: &str) -> String {
    if mv.ends_with(" e.p.") {
        mv.replace(" e.p.", format!("{suf} e.p.").as_str())
    } else {
        mv + suf
    }
}

fn castling_notation(mv: Move, all_moves: &[Move]) -> String {
    let mut kingside_castles = 0;
    let mut queenside_castles = 0;
    for m in all_moves {
        let ty = m.get_move_type();
        if ty == MoveType::KingsideCastle {
            kingside_castles += 1;
        } else if ty == MoveType::QueensideCastle {
            queenside_castles += 1;
        }
    }
    
    let mv_rank = from_index(mv.get_from()).1;
    if mv.get_move_type() == MoveType::KingsideCastle {
        if kingside_castles > 1 { format!("O-O({})", mv_rank+1) }
        else { "O-O".to_string() }
    } else if mv.get_move_type() == MoveType::QueensideCastle {
        if queenside_castles > 1 { format!("O-O-O({})", mv_rank+1) }
        else { "O-O-O".to_string() }
    } else {
        panic!("Not a castling move");
    }
}

/// Returns the necessary disambiguation for the move
fn disambiguate(pos: &mut Position, mv: Move, all_moves: &[Move]) -> String {
    let from = from_index(mv.get_from());
    let mv_piece = pos.piece_at(mv.get_from()).unwrap().get_piece_id();
    let mut print_rank = false;
    let mut print_file = false;
    
    for m in all_moves {
        if m.get_to() == mv.get_to() && m.get_from() != mv.get_from() 
        && pos.piece_at(m.get_from()).unwrap().get_piece_id() == mv_piece {
            // Got a match, determine if we need to disambiguate rank or file
            let m_from = from_index(m.get_from());
            if m_from.0 == from.0 {
                print_rank = true;
            } else {
                print_file = true;
            }
        }
    }
    
    let mut result = String::new();
    if print_file {
        result.push((b'a' + from.0) as char);
    }
    if print_rank {
        result.push((b'1' + from.1) as char);
    }
    result
}
