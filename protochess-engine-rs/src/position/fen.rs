use std::convert::TryInto;

use regex::Regex;
use scan_fmt::scan_fmt;

use crate::{GameState, GlobalRules, PiecePlacement, wrap_res, err_assert, err, PieceId};
use crate::piece::PieceFactory;
use crate::types::{GameMode, BCoord};


impl GameState {
    
    /// Creates a FEN string from the piece placement of the game state
    pub fn create_fen(&self) -> String {
        let mut fen = String::new();
        let mut empty_count = 0;
        // For each square in the board
        for y in (0..self.board_height).rev() {
            for x in 0..self.board_width {
                let mut found = false;
                // Find the piece placed on that square
                for piece in &self.pieces {
                    if piece.x == x && piece.y == y {
                        if empty_count > 0 {
                            fen.push_str(&empty_count.to_string());
                            empty_count = 0;
                        }
                        fen.push(piece.piece_id);
                        found = true;
                        break;
                    }
                }
                // No piece in this square
                if !found {
                    empty_count += 1;
                }
            }
            if empty_count > 0 {
                fen.push_str(&empty_count.to_string());
                empty_count = 0;
            }
            // Don't add a slash at the end of the last row
            if y > 0 {
                fen.push('/');
            }
        }
        fen
    }
    
    
    /// Parses a FEN string into a game state
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
            piece_tuples.push((c, x as BCoord, y));
            x += 1;
        }
        board_width = std::cmp::max(board_width, x + skip_x);
        err_assert!(board_width <= 16, "The FEN string has too many files ({board_width} > 16)");
        
        // Generate piece types
        let factory = PieceFactory::new(mode);
        let piece_types = vec![
            factory.make_king(),
            factory.make_queen(),
            factory.make_rook(),
            factory.make_bishop(),
            factory.make_knight(),
            factory.make_pawn(true),
            factory.make_pawn(false),
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
        let player_to_move = {
            if fen_parts[1].to_ascii_lowercase() == "w" { 0 }
            else if fen_parts[1].to_ascii_lowercase() == "b" { 1 }
            else { err!("The player to move must be 'w' or 'b'") }
        };
        
        let pieces = tuples_to_pieces(piece_tuples, fen_parts[2])?;
        
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
                if player_to_move == 0 { ep_victim.1 += 1 }
                else { ep_victim.1 -= 1 }
                Some((ep_square, ep_victim))
            }
        };
        
        let global_rules = GlobalRules::for_mode(mode);
        let times_in_check = Some(times_in_check);
        
        // In standard FEN strings, there are no walls
        let board_width = board_width as BCoord;
        let board_height = board_height as BCoord;
        let invalid_squares = Vec::new();
        
        Ok(GameState {
            piece_types, board_width, board_height, invalid_squares, pieces, player_to_move,
            ep_square_and_victim, times_in_check, global_rules
        })
    }
}


// Converts a Vec of tuples of (id, x, y) to a Vec of PiecePlacements
fn tuples_to_pieces(piece_tuples: Vec<(PieceId, BCoord, BCoord)>, castling: &str) -> wrap_res!(Vec<PiecePlacement>) {
    // Function for enabling castling for the rooks and king
    let enable_castle = |pieces: &mut [PiecePlacement], rook: PieceId, king: PieceId, kingside: bool, row_y: BCoord| -> wrap_res!() {
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
            // Find the first rook and enable it
            if !found_rook && p.piece_id == rook {
                found_rook = true;
                err_assert!(p.can_castle == Some(false), "Should not have been able to castle, FEN might be invalid");
                p.can_castle = Some(true);
                continue;
            }
            if !found_rook && p.piece_id == king {
                // Found the king before the rook, so the rook is missing
                // Don't throw an error here to support multiple kings
                break;
            }
            // Next search for the king, but stop if we find another rook instead
            if found_rook && p.piece_id == king {
                p.can_castle = Some(true);
            }
            if found_rook && p.piece_id == rook {
                break;
            }
        }
        Ok(())
    };
    // Function for finding the ranks of a piece
    let find_ranks_with_char = |pieces: &[PiecePlacement], c: PieceId| -> Vec<BCoord> {
        let mut ranks = Vec::new();
        for p in pieces {
            if p.piece_id == c {
                ranks.push(p.y);
            }
        }
        ranks.sort();
        ranks.dedup();
        ranks
    };
    
    // Convert the tuples to PiecePlacements, set can_castle to false
    let mut pieces = Vec::with_capacity(piece_tuples.len());
    for t in piece_tuples {
        pieces.push(PiecePlacement::new(t.0, t.1, t.2, false));
    }
    err_assert!(castling == "-" || castling.chars().all(|c| c=='K'||c=='Q'||c=='k'||c=='q'),
        "Invalid castling rights in FEN string: '{castling}'");
    if castling.contains('K') {
        for row_y in find_ranks_with_char(&pieces, 'K') {
            enable_castle(&mut pieces, 'R', 'K', true, row_y)?;
        }
    }
    if castling.contains('Q') {
        for row_y in find_ranks_with_char(&pieces, 'K') {
            enable_castle(&mut pieces, 'R', 'K', false, row_y)?;
        }
    }
    if castling.contains('k') {
        for row_y in find_ranks_with_char(&pieces, 'k') {
            enable_castle(&mut pieces, 'r', 'k', true, row_y)?;
        }
    }
    if castling.contains('q') {
        for row_y in find_ranks_with_char(&pieces, 'k') {
            enable_castle(&mut pieces, 'r', 'k', false, row_y)?;
        }
    }
    Ok(pieces)
}
