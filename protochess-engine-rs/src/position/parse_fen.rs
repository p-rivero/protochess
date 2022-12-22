use crate::constants::fen;
use crate::constants::piece_scores::*;
use crate::piece::PieceFactory;
use crate::types::*;
use crate::utils::to_index;

use super::Position;
use super::piece_set::PieceSet;
use super::position_properties::PositionProperties;

#[allow(clippy::iter_nth_zero)]
pub fn parse_fen(fen: &str) -> Position {
    let mut bounds = Bitboard::zero();
    for x in 0..fen::BOARD_WIDTH {
        for y in 0..fen::BOARD_HEIGHT {
            bounds.set_bit_at(x,y);
        }
    }
    let dims = BDimensions{ width: fen::BOARD_WIDTH, height: fen::BOARD_HEIGHT, bounds};
    
    let mut w_pieces = PieceSet::new(0);
    let mut b_pieces = PieceSet::new(1);
    register_pieces(&mut w_pieces, &mut b_pieces, &dims);

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
    let mut y: BCoord = fen::BOARD_HEIGHT - 1;
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
    
    let enable_castle = |pieces: &mut PieceSet, x: BCoord, y: BCoord| {
        let index = to_index(x, y);
        if let Some(piece) = pieces.piece_at_mut(index) {
            piece.move_piece(index, index, true);
        }
    };
    if can_w_castle_k {
        enable_castle(&mut w_pieces, 7, 0);
        enable_castle(&mut w_pieces, 4, 0);
    }
    if can_b_castle_k {
        enable_castle(&mut b_pieces, 7, 7);
        enable_castle(&mut b_pieces, 4, 7);
    }
    if can_w_castle_q {
        enable_castle(&mut w_pieces, 0, 0);
        enable_castle(&mut w_pieces, 4, 0);
    }
    if can_b_castle_q {
        enable_castle(&mut b_pieces, 0, 7);
        enable_castle(&mut b_pieces, 4, 7);
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
    

    Position::new(dims, vec![w_pieces, b_pieces], whos_turn, properties)
}

fn register_pieces(w_pieces: &mut PieceSet, b_pieces: &mut PieceSet, dims: &BDimensions) {
    w_pieces.register_piecetype(PieceFactory::make_king(ID_KING, 0), dims);
    b_pieces.register_piecetype(PieceFactory::make_king(ID_KING, 1), dims);
    w_pieces.register_piecetype(PieceFactory::make_queen(ID_QUEEN, 0), dims);
    b_pieces.register_piecetype(PieceFactory::make_queen(ID_QUEEN, 1), dims);
    w_pieces.register_piecetype(PieceFactory::make_rook(ID_ROOK, 0), dims);
    b_pieces.register_piecetype(PieceFactory::make_rook(ID_ROOK, 1), dims);
    w_pieces.register_piecetype(PieceFactory::make_bishop(ID_BISHOP, 0), dims);
    b_pieces.register_piecetype(PieceFactory::make_bishop(ID_BISHOP, 1), dims);
    w_pieces.register_piecetype(PieceFactory::make_knight(ID_KNIGHT, 0), dims);
    b_pieces.register_piecetype(PieceFactory::make_knight(ID_KNIGHT, 1), dims);
    w_pieces.register_piecetype(PieceFactory::make_pawn(ID_PAWN, 0, dims, vec![ID_QUEEN, ID_ROOK, ID_BISHOP, ID_KNIGHT]), dims);
    b_pieces.register_piecetype(PieceFactory::make_pawn(ID_PAWN, 1, dims, vec![ID_QUEEN, ID_ROOK, ID_BISHOP, ID_KNIGHT]), dims);
}
