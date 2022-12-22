use crate::constants::fen;
use crate::types::*;
use crate::utils::to_index;

use super::Position;
use super::piece_set::PieceSet;
use super::position_properties::PositionProperties;

pub fn parse_fen(fen: String) -> Position {
    let mut bounds = Bitboard::zero();
    for x in 0..fen::BOARD_WIDTH {
        for y in 0..fen::BOARD_HEIGHT {
            bounds.set_bit_at(x,y);
        }
    }
    let dims = BDimensions{ width: fen::BOARD_WIDTH, height: fen::BOARD_HEIGHT, bounds};
    
    let mut w_pieces = PieceSet::new(0, &dims);
    let mut b_pieces = PieceSet::new(1, &dims);

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
        
        if c.is_uppercase() {
            w_pieces.occupied.set_bit(index)
        } else {
            b_pieces.occupied.set_bit(index)
        };
        x += 1;
    }
    
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
        if ep_y == -1 || (ep_y != 2 && ep_y != 5) {
            panic!("Invalid en passant square: {}", fen);
        }
        let ep_index = to_index(ep_x as BCoord, ep_y as BCoord);
        let offset = if whos_turn == 0 {1} else {-1};
        properties.ep_square = Some(ep_index);
        properties.ep_victim = to_index(ep_x as BCoord, (ep_y + offset) as BCoord);
        // Use the en passant square as a zobrist key
        zobrist_key ^= ep_index as u64;
    }


    for piece in w_pieces.get_piece_refs().chain(b_pieces.get_piece_refs()) {
        let mut bb_copy = piece.bitboard.to_owned();
        while !bb_copy.is_zero() {
            let indx = bb_copy.lowest_one().unwrap();
            zobrist_key ^= piece.get_zobrist(indx);
            bb_copy.clear_bit(indx);
        }
    }
    
    if whos_turn == 1 {
        // Use the top bit as player zobrist key
        zobrist_key ^= 0x8000000000000000;
    }
    
    properties.zobrist_key = zobrist_key;
    

    Position::new(dims, vec![w_pieces, b_pieces], whos_turn, properties)
}
