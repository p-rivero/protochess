use std::sync::Arc;

use arrayvec::ArrayVec;

use crate::constants::fen;
use crate::types::*;
use crate::utils::to_index;

use super::Position;
use super::piece_set::PieceSet;
use super::position_properties::PositionProperties;
use super::zobrist_table::ZobristTable;

pub fn parse_fen(fen: String) -> Position {
    let dims = BDimensions{ width: fen::BOARD_WIDTH, height: fen::BOARD_HEIGHT };

    let mut wb_pieces = ArrayVec::<[_;4]>::new();
    let mut w_pieces = PieceSet::new(0);
    let mut b_pieces = PieceSet::new(1);

    let mut x: BCoord = 0;
    let mut y: BCoord = 7;
    let mut field = 0;

    let mut whos_turn = 0;
    let mut can_w_castle_k = false;
    let mut can_b_castle_k = false;
    let mut can_w_castle_q = false;
    let mut can_b_castle_q = false;
    
    let mut ep_x: i8 = -1;
    let mut ep_y: i8 = -1;
    
    for c in fen.chars() {
        if c == ' ' {
            field += 1;
            continue;
        }
        match field{
            //position
            0 => {
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
                let bitboard: &mut Bitboard = match c.to_ascii_lowercase() {
                    'k' => { &mut pieces.king.bitboard },
                    'q' => { &mut pieces.queen.bitboard },
                    'r' => { &mut pieces.rook.bitboard },
                    'b' => { &mut pieces.bishop.bitboard },
                    'n' => { &mut pieces.knight.bitboard },
                    'p' => { &mut pieces.pawn.bitboard },
                    _ => continue,
                };

                bitboard.set_bit(index);
                if c.is_uppercase() {
                    w_pieces.occupied.set_bit(index)
                } else {
                    b_pieces.occupied.set_bit(index)
                };
                x += 1;
            }
            //next to move
            1 => {
                if c == 'w' {
                    whos_turn = 0;
                } else {
                    whos_turn = 1;
                }
            }
            //Castling rights
            2 => {
                match c {
                    'K' => {can_w_castle_k = true;}
                    'Q' => {can_w_castle_q = true;}
                    'k' => {can_b_castle_k = true;}
                    'q' => {can_b_castle_q = true;}
                    _ => {}
                }
            }
            //En Passant square
            3 => {
                // This field can be either '-' or a square in the form of a letter followed by a number
                if c == '-' {
                    continue;
                } else if c.is_numeric() {
                    ep_y = c.to_digit(10).expect("Not a digit!") as i8 - 1;
                } else {
                    ep_x = c as i8 - 'a' as i8;
                }
            }
            _ => continue,
        }
    }

    let mut occupied = Bitboard::zero();
    occupied |= &w_pieces.occupied;
    occupied |= &b_pieces.occupied;
    
    // TODO: Remove this
    let zobrist_table = ZobristTable::new();
    
    let mut zobrist_key = 0;

    let mut properties = PositionProperties::default();
    zobrist_key ^= zobrist_table.get_castling_zobrist(0, true);
    zobrist_key ^= zobrist_table.get_castling_zobrist(0, false);
    zobrist_key ^= zobrist_table.get_castling_zobrist(1, true);
    zobrist_key ^= zobrist_table.get_castling_zobrist(1, false);
    if !can_w_castle_k {
        properties.castling_rights.disable_kingside_castle(0);
        zobrist_key ^= zobrist_table.get_castling_zobrist(0, true);
    }

    if !can_b_castle_k {
        properties.castling_rights.disable_kingside_castle(1);
        zobrist_key ^= zobrist_table.get_castling_zobrist(1, true);
    }

    if !can_w_castle_q {
        properties.castling_rights.disable_queenside_castle(0);
        zobrist_key ^= zobrist_table.get_castling_zobrist(0, false);
    }

    if !can_b_castle_q {
        properties.castling_rights.disable_queenside_castle(1);
        zobrist_key ^= zobrist_table.get_castling_zobrist(1, false);
    }
    
    if ep_x != -1 {
        if ep_y == -1 || (ep_y != 2 && ep_y != 5) {
            panic!("Invalid en passant square: {}", fen);
        }
        properties.ep_square = Some(to_index(ep_x as BCoord, ep_y as BCoord));
        zobrist_key ^= zobrist_table.get_ep_zobrist_file(ep_x as BCoord);
    }


    for piece in w_pieces.get_piece_refs().into_iter().chain(b_pieces.get_piece_refs().into_iter()) {
        let mut bb_copy = (&piece.bitboard).to_owned();
        while !bb_copy.is_zero() {
            let indx = bb_copy.lowest_one().unwrap();
            zobrist_key ^= piece.get_zobrist(indx);
            bb_copy.clear_bit(indx);
        }
    }
    
    zobrist_key ^= zobrist_table.get_player_zobrist(whos_turn);

    properties.zobrist_key = zobrist_key;

    wb_pieces.push(w_pieces);
    wb_pieces.push(b_pieces);

    let mut bounds = Bitboard::zero();
    for x in 0..8 {
        for y in 0..8 {
            bounds.set_bit_at(x,y);
        }
    }

    let pos = Position{
        whos_turn,
        num_players: 2,
        dimensions: dims,
        pieces: wb_pieces,
        occupied,
        bounds,
        properties: Arc::new(properties)
    };

  pos
}
