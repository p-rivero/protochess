use crate::piece::{PieceFactory, PieceId};
use crate::types::*;
use crate::utils::to_index;

use super::Position;
use super::piece_set::PieceSet;
use super::position_properties::PositionProperties;

const BOARD_WIDTH: BCoord = 8;
const BOARD_HEIGHT: BCoord = 8;

#[allow(clippy::iter_nth_zero)]
pub fn parse_fen(fen: &str) -> Position {
    let mut w_pieces = PieceSet::new(0);
    let mut b_pieces = PieceSet::new(1);
    register_pieces(&mut w_pieces, &mut b_pieces);

    let fen_parts: Vec<&str> = fen.split_whitespace().collect();
    
    // Next to move
    let whos_turn = if fen_parts[1] == "w" {0} else {1};
    
    // Castling rights
    let can_w_castle_k = fen_parts[2].contains('K');
    let can_b_castle_k = fen_parts[2].contains('k');
    let can_w_castle_q = fen_parts[2].contains('Q');
    let can_b_castle_q = fen_parts[2].contains('q');
    
    // En passant square
    let mut ep_x: i8 = -1;
    let mut ep_y: i8 = -1;
    if fen_parts[3] != "-" {
        ep_x = fen_parts[3].chars().nth(0).unwrap() as i8 - 'a' as i8;
        ep_y = fen_parts[3].chars().nth(1).unwrap() as i8 - '1' as i8;
    }
    
    // Piece placement
    let mut x: BCoord = 0;
    let mut y: BCoord = BOARD_HEIGHT - 1;
    for c in fen_parts[0].chars() {
        if c == '/' {
            x = 0;
            y -= 1;
            continue;
        } else if c.is_numeric() {
            x += c.to_digit(10).expect("Not a digit!") as BCoord;
            continue;
        }

        let index = to_index(x, y);
        let pieces = if c.is_ascii_uppercase() {
            &mut w_pieces
        } else {
            &mut b_pieces
        };
        let p = pieces.search_by_char(c).expect("Piece not found!");
        p.add_piece(index, false);
        
        x += 1;
    }
    w_pieces.update_occupied();
    b_pieces.update_occupied();
    
    let mut zobrist_key = 0;
    
    let enable_castle = |pieces: &mut PieceSet, x: BCoord, y: BCoord, zob: &mut u64| {
        let index = to_index(x, y);
        if let Some(piece) = pieces.piece_at_mut(index) {
            let could_castle = piece.move_piece(index, index, true);
            if !could_castle {
                // Enable the castle square in zobrist key
                *zob ^= piece.get_castle_zobrist(index);
            }
        }
    };
    if can_w_castle_k {
        enable_castle(&mut w_pieces, 7, 0, &mut zobrist_key);
        enable_castle(&mut w_pieces, 4, 0, &mut zobrist_key);
    }
    if can_b_castle_k {
        enable_castle(&mut b_pieces, 7, 7, &mut zobrist_key);
        enable_castle(&mut b_pieces, 4, 7, &mut zobrist_key);
    }
    if can_w_castle_q {
        enable_castle(&mut w_pieces, 0, 0, &mut zobrist_key);
        enable_castle(&mut w_pieces, 4, 0, &mut zobrist_key);
    }
    if can_b_castle_q {
        enable_castle(&mut b_pieces, 0, 7, &mut zobrist_key);
        enable_castle(&mut b_pieces, 4, 7, &mut zobrist_key);
    }
    
    let mut properties = PositionProperties::default();
    if ep_x != -1 {
        assert!(ep_y != -1, "Invalid en passant square: {}", fen);
        assert!(ep_y == 2 || ep_y == 5, "Invalid en passant square: {}", fen);
        let ep_index = to_index(ep_x as BCoord, ep_y as BCoord);
        let offset = if whos_turn == 0 {1} else {-1};
        properties.ep_square = Some(ep_index);
        properties.ep_victim = to_index(ep_x as BCoord, (ep_y + offset) as BCoord);
        // Use the en passant square as a zobrist key
        zobrist_key ^= ep_index as u64;
    }


    for piece in w_pieces.iter().chain(b_pieces.iter()) {
        for indx in piece.get_indexes() {
            zobrist_key ^= piece.get_zobrist(indx);
        }
    }
    
    if whos_turn == 1 {
        // Use the top bit as player zobrist key
        zobrist_key ^= 0x8000000000000000;
    }
    
    properties.zobrist_key = zobrist_key;
    
    let dims = BDimensions::new_without_walls(BOARD_WIDTH, BOARD_HEIGHT);
    Position::new(dims, vec![w_pieces, b_pieces], whos_turn, properties)
}

fn register_pieces(w_pieces: &mut PieceSet, b_pieces: &mut PieceSet) {
    const ID_KING: PieceId = 0;
    const ID_QUEEN: PieceId = 1;
    const ID_ROOK: PieceId = 2;
    const ID_BISHOP: PieceId = 3;
    const ID_KNIGHT: PieceId = 4;
    const ID_PAWN: PieceId = 5;
    let dims = BDimensions::new_without_walls(BOARD_WIDTH, BOARD_HEIGHT);
    
    let mut register_piece = |def: crate::PieceDefinition| {
        w_pieces.register_piecetype(def.clone(), &dims);
        b_pieces.register_piecetype(def, &dims);
    };
    register_piece(PieceFactory::make_king(ID_KING));
    register_piece(PieceFactory::make_queen(ID_QUEEN));
    register_piece(PieceFactory::make_rook(ID_ROOK));
    register_piece(PieceFactory::make_bishop(ID_BISHOP));
    register_piece(PieceFactory::make_knight(ID_KNIGHT));
    w_pieces.register_piecetype(PieceFactory::make_pawn(ID_PAWN, true, &dims, vec![ID_QUEEN, ID_ROOK, ID_BISHOP, ID_KNIGHT]), &dims);
    b_pieces.register_piecetype(PieceFactory::make_pawn(ID_PAWN, false, &dims, vec![ID_QUEEN, ID_ROOK, ID_BISHOP, ID_KNIGHT]), &dims);
}
