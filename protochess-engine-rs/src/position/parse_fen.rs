use std::convert::TryInto;

use regex::Regex;
use scan_fmt::scan_fmt;

use crate::{PieceId, GameState, GlobalRules, PiecePlacement, wrap_res, err_assert, err};
use crate::piece::PieceFactory;
use crate::types::{GameMode, BCoord};


const ID_KING: PieceId = 0;
const ID_QUEEN: PieceId = 1;
const ID_ROOK: PieceId = 2;
const ID_BISHOP: PieceId = 3;
const ID_KNIGHT: PieceId = 4;
const ID_PAWN: PieceId = 5;

impl GameState {
    pub fn from_fen(fen: &str) -> wrap_res!(Self) {
        // Split FEN string into parts, there must be at least 6 parts
        let fen_parts: Vec<&str> = fen.split_whitespace().collect();
        err_assert!(fen_parts.len() >= 6, "Incorrect number of parts in FEN string");
        // Last part is the game mode
        let mode = (*fen_parts.last().unwrap()).try_into().unwrap_or(GameMode::Standard);
        
        // Count the number of ranks
        let board_height = fen_parts[0].chars().filter(|c| *c == '/').count() + 1;
        err_assert!(board_height <= 16, "The FEN string has too many ranks ({board_height} > 16)");
        
        // Piece placement
        let mut piece_tuples = Vec::new();
        let mut ranks_with_kings = [vec![], vec![]];
        let mut x = 0;
        let mut y = board_height as BCoord - 1;
        let mut skip_x = 0;
        let mut board_width = 0;
        for c in fen_parts[0].chars() {
            if c == '/' {
                board_width = std::cmp::max(board_width, x + skip_x);
                x = 0;
                y -= 1;
                skip_x = 0;
                continue;
            } else if c.is_ascii_digit() {
                skip_x = 10 * skip_x + c.to_digit(10).unwrap();
                continue;
            }
            x += skip_x;
            skip_x = 0;
            let player = {
                if c.is_ascii_uppercase() { 0 } // White piece
                else { 1 } // Black piece
            };
            if c.to_ascii_uppercase() == 'K' && !ranks_with_kings[player as usize].contains(&y) {
                ranks_with_kings[player as usize].push(y);
            }
            piece_tuples.push((player, c, x as BCoord, y));
            x += 1;
        }
        board_width = std::cmp::max(board_width, x + skip_x);
        err_assert!(board_width <= 16, "The FEN string has too many files ({board_width} > 16)");
        
        // Generate piece types
        let factory = PieceFactory::new(mode);
        let mut pawn_promotions = vec![ID_QUEEN, ID_ROOK, ID_BISHOP, ID_KNIGHT];
        if mode == GameMode::Antichess {
            pawn_promotions.push(ID_KING);
        }
        let piece_types = vec![
            factory.make_king(ID_KING),
            factory.make_queen(ID_QUEEN),
            factory.make_rook(ID_ROOK),
            factory.make_bishop(ID_BISHOP),
            factory.make_knight(ID_KNIGHT),
            factory.make_pawn(ID_PAWN, true, pawn_promotions.clone()),
            factory.make_pawn(ID_PAWN, false, pawn_promotions),
        ];
        
        // Times in check
        let mut times_in_check = [0, 0];
        if mode == GameMode::ThreeCheck || mode == GameMode::FiveCheck {
            const EXPECTED_REGEX: &str = r"^\+([0-9]+)\+([0-9]+)$";
            err_assert!(fen_parts.len() >= 7, "Missing number of checks in FEN string");
            err_assert!(Regex::new(EXPECTED_REGEX).unwrap().is_match(fen_parts[6]),
                "Invalid check count format, use +W+B, where W is the number of times White put Black in check.
                In 3-Check, '3+1' is equivalent to '+0+2'");
            let (white_checks, black_checks) = match scan_fmt!(fen_parts[6], "+{d}+{d}", u8, u8) {
                Ok(parts) => parts,
                Err(_) => err!("Invalid check format, make sure it's between +0+0 and +255+255"),
            };
            times_in_check = [black_checks, white_checks];
        }
        
        // Player to move
        let whos_turn = {
            if fen_parts[1].to_ascii_lowercase() == "w" { 0 }
            else if fen_parts[1].to_ascii_lowercase() == "b" { 1 } 
            else { err!("The player to move must be 'w' or 'b'") }
        };
        
        let pieces = tuples_to_pieces(piece_tuples, fen_parts[2], &ranks_with_kings)?;
        let mut valid_squares = Vec::new();
        for x in 0..board_width {
            for y in 0..board_height {
                valid_squares.push((x as BCoord, y as BCoord));
            }
        }
        
        let ep_square_and_victim = {
            if fen_parts[3] == "-" {
                None
            } else {
                const EXPECTED_REGEX: &str = r"^[a-p][0-9]+$";
                err_assert!(Regex::new(EXPECTED_REGEX).unwrap().is_match(fen_parts[3]), "Invalid en passant square in FEN string");
                let (ep_x, ep_y) = match scan_fmt!(fen_parts[3], "{[a-p]}{d}", char, BCoord) {
                    Ok(parts) => parts,
                    Err(_) => err!("Invalid en passant square in FEN string")
                };
                err_assert!(ep_y != 0 && ep_y <= board_height as BCoord, "Invalid en passant square in FEN string");
                // ep_x is guaranteed to be a valid character between 'a' and 'p'
                let ep_x = ep_x.to_digit(36).unwrap() as BCoord - 10;
                let ep_y = ep_y - 1;
                
                let ep_square = (ep_x, ep_y);
                let mut ep_victim = ep_square;
                if whos_turn == 0 { ep_victim.1 += 1 }
                else { ep_victim.1 -= 1 }
                Some((ep_square, ep_victim))
            }
        };
        
        let global_rules = GlobalRules::for_mode(mode);
        let times_in_check = Some(times_in_check);
        
        Ok(GameState { piece_types, valid_squares, pieces, whos_turn, ep_square_and_victim, times_in_check, global_rules })
    }
}

// Converts a Vec of tuples of (player, char, x, y) to a Vec of PiecePlacements
fn tuples_to_pieces(piece_tuples: Vec<(u8, char, u8, u8)>, castling: &str, ranks_with_kings: &[Vec<u8>; 2]) -> wrap_res!(Vec<PiecePlacement>) {
    // Convert the tuples to PiecePlacements, set can_castle to false
    let mut pieces = Vec::with_capacity(piece_tuples.len());
    for t in piece_tuples {
        let piece_id = match t.1.to_ascii_uppercase() {
            'K' => ID_KING,
            'Q' => ID_QUEEN,
            'R' => ID_ROOK,
            'B' => ID_BISHOP,
            'N' => ID_KNIGHT,
            'P' => ID_PAWN,
            _ => err!("Invalid piece character '{}' in FEN string", t.1),
        };
        pieces.push(PiecePlacement::new(t.0, piece_id, t.2, t.3, false));
    }
    let mut enable_castle = |is_white: bool, kingside: bool, row_y: BCoord| -> wrap_res!() {
        let num_pieces = pieces.len();
        let mut found_rook = false;
        for i in 0..num_pieces {
            let p = {
                // If kingside, traverse ranks from right to left, otherwise from left to right
                if kingside { &mut pieces[num_pieces - i - 1] }
                else { &mut pieces[i] }
            };
            // Visit only pieces on the correct row and of the correct color
            if p.y != row_y as BCoord { continue; }
            if (p.owner == 0) != is_white { continue; }
            // Find the first rook and enable it
            if !found_rook && p.piece_id == ID_ROOK {
                found_rook = true;
                err_assert!(p.can_castle == Some(false), "Should not have been able to castle, FEN might be invalid");
                p.can_castle = Some(true);
                continue;
            }
            if !found_rook && p.piece_id == ID_KING {
                // Found the king before the rook, so the rook is missing
                // Don't throw an error here to support multiple kings
                break;
            }
            // Next search for the king, but stop if we find another rook instead
            if found_rook && p.piece_id == ID_KING {
                p.can_castle = Some(true);
            }
            if found_rook && p.piece_id == ID_ROOK {
                break;
            }
        }
        Ok(())
    };
    err_assert!(castling == "-" || castling.chars().all(|c| c=='K'||c=='Q'||c=='k'||c=='q'),
        "Invalid castling rights in FEN string: '{castling}'");
    if castling.contains('K') {
        for row_y in &ranks_with_kings[0] {
            enable_castle(true, true, *row_y)?;
        }
    }
    if castling.contains('Q') {
        for row_y in &ranks_with_kings[0] {
            enable_castle(true, false, *row_y)?;
        }
    }
    if castling.contains('k') {
        for row_y in &ranks_with_kings[1] {
            enable_castle(false, true, *row_y)?;
        }
    }
    if castling.contains('q') {
        for row_y in &ranks_with_kings[1] {
            enable_castle(false, false, *row_y)?;
        }
    }
    Ok(pieces)
}
