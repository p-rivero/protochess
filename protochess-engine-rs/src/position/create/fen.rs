use regex::Regex;
use scan_fmt::scan_fmt;

use crate::utils::{from_index, to_index, tuple_to_rank_file};
use crate::{wrap_res, err_assert, err, PieceId, Position};
use crate::types::{BCoord, Player};


#[derive(Debug, Clone)]
pub struct PiecePlacement {
    pub x: BCoord,
    pub y: BCoord,
    pub piece_id: PieceId,
}

/// Summary of the data encoded in a FEN string. Used as an intermediate step when converting between FEN and Position.
/// 
/// See [this document](https://github.com/p-rivero/protochess-engine/tree/master/docs/FEN.md) for the custom FEN format.
/// 
/// **FEN -> Position**: Don't use this class directly, use `PositionFactory::set_state()` or `PositionFactory::load_fen()`,
/// both of which call `FenData::parse_fen()` internally.
/// 
/// **Position -> FEN**: Use `fen = FenData::from(position)` to extract the FEN data from the position, then use
/// `fen.to_string()` to convert it to a FEN string.
#[must_use]
#[derive(Debug, Clone)]
pub struct FenData {
    pub width: BCoord,
    pub height: BCoord,
    pub piece_placements: Vec<PiecePlacement>,
    pub walls: Vec<(BCoord, BCoord)>,
    pub player_to_move: Player,
    /// List of squares that have not been moved. `None` means that the castling has not been specified
    /// in the string. Assume that no pieces have moved (everyone can castle).
    pub castling_availability: Option<Vec<(BCoord, BCoord)>>, 
    /// The EP square, and the square of the EP victim (piece that just did a double move and will be captured)
    /// `None` means that EP is not available in this position
    pub ep_square_and_victim: Option<((BCoord, BCoord), (BCoord, BCoord))>,
    /// Number of times each player has been in check. `None` means that this information is not available
    /// in the FEN string (not aplicable to this variant): assume that no player has been in check (`[0,0]`).
    pub times_in_check: Option<[u8; 2]>,
    
    // Fullmove and halfmove clocks are not used
}



impl FenData {
    pub fn parse_fen(fen: &str) -> wrap_res!(Self) {
        // Split FEN string into parts
        let fen_parts: Vec<&str> = fen.split_whitespace().collect();
        if fen_parts.is_empty() {
            err!("Invalid FEN string, it must have at least 1 part");
        }
        
        // Count the number of ranks
        let board_height = fen_parts[0].chars().filter(|c| *c == '/').count() + 1;
        err_assert!(board_height <= 16, "The FEN string has {board_height} ranks, but the limit is 16");
        
        // Piece placement
        let mut piece_placements = Vec::new();
        let mut walls = Vec::new();
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
            if c == '*' {
                walls.push((x as BCoord, y));
            } else {
                piece_placements.push(PiecePlacement { x: x as BCoord, y, piece_id: c });
            }
            x += 1;
        }
        board_width = std::cmp::max(board_width, x + skip_x);
        err_assert!(board_width <= 16, "The FEN string has too many files ({board_width} > 16)");
        
        let board_width = board_width as BCoord;
        let board_height = board_height as BCoord;
        
        // Player to move
        let player_to_move = {
            // By default, white moves first
            if fen_parts.len() <= 1 || fen_parts[1].to_ascii_lowercase() == "w" { 0 }
            else if fen_parts[1].to_ascii_lowercase() == "b" { 1 }
            else { err!("The player to move must be 'w' or 'b'") }
        };
        
        // If the castling availability is not specified, it is assumed that no pieces have moved yet
        // (can always castle). Also use the special value "(ALL)" to indicate that all pieces can castle.
        // As in standard FEN, "-" means that no pieces can castle.
        let castling_availability = {
            if fen_parts.len() <= 2 { None }
            else if fen_parts[2] == "-" { Some(vec![]) }
            else if fen_parts[2].to_lowercase() == "(all)" { None }
            else { Some(parse_castling(fen_parts[2], board_height, board_width)?) }
        };
        
        let ep_square_and_victim = {
            if fen_parts.len() <= 3 || fen_parts[3] == "-" {
                None
            } else {
                // Expected formats: a1, a1(b2)
                const EXPECTED_REGEX: &str = r"^[a-p][0-9]+(\([a-p][0-9]+\))?$";
                err_assert!(Regex::new(EXPECTED_REGEX).unwrap().is_match(fen_parts[3]), "Invalid en passant square in FEN string");
                let (ep_x, ep_y) = match scan_fmt!(fen_parts[3], "{[a-p]}{d}", char, isize) {
                    Ok(parts) => parts,
                    Err(_) => err!("Invalid en passant square in FEN string")
                };
                let (vic_x, vic_y) = match scan_fmt!(fen_parts[3], "{*[a-p]}{*d}({[a-p]}{d})", char, isize) {
                    Ok(parts) => parts,
                    Err(_) => {
                        if player_to_move == 0 { (ep_x, ep_y + 1) }
                        else { (ep_x, ep_y - 1) }
                    }
                };
                // ep_x and ev_x are guaranteed to be a valid character between 'a' and 'p'
                let ep_x = ep_x.to_digit(36).unwrap() as BCoord - 10;
                let ep_y = ep_y - 1;
                let vic_x = vic_x.to_digit(36).unwrap() as BCoord - 10;
                let vic_y = vic_y - 1;
                // Only need to check the y coordinates, since the x coordinates are already guaranteed to be valid
                err_assert!(ep_y >= 0 && ep_y < board_height as isize, "Invalid en passant square in FEN string");
                err_assert!(vic_y >= 0 && vic_y < board_height as isize, "Invalid en passant victim in FEN string");
                
                Some(((ep_x, ep_y as BCoord), (vic_x, vic_y as BCoord)))
            }
        };
        
        // Times in check: search all remaining parts for a +W+B format
        let mut times_in_check = None;
        for part in fen_parts.iter().skip(4) {
            const TIMES_IN_CHECK_REGEX: &str = r"^\+([0-9]+)\+([0-9]+)$";
            const WRONG_FORMAT_REGEX: &str = r"^([0-9]+)\+([0-9]+)$";
            if !Regex::new(TIMES_IN_CHECK_REGEX).unwrap().is_match(part) {
                // Check if this is an alternative check count format
                if Regex::new(WRONG_FORMAT_REGEX).unwrap().is_match(part) {
                    err!("Invalid check count format, use +W+B, where W is the number of times White put Black in check.
                    In 3-Check, '3+1' is equivalent to '+0+2'");
                }
                continue;
            }
            // Some of the parts match the check count format
            let (white_checks, black_checks) = match scan_fmt!(fen_parts[6], "+{d}+{d}", u8, u8) {
                Ok(parts) => parts,
                Err(_) => err!("Invalid check format, make sure it's between +0+0 and +255+255"),
            };
            // Important: Note that the order is reversed
            times_in_check = Some([black_checks, white_checks]);
        }
        
        Ok(Self {
            width: board_width,
            height: board_height,
            piece_placements,
            walls,
            times_in_check,
            player_to_move,
            castling_availability,
            ep_square_and_victim,
        })
    }
}


/// Returns a list of the squares that have not moved
fn parse_castling(castling: &str, board_height: BCoord, board_width: BCoord) -> wrap_res!(Vec<(BCoord, BCoord)>) {
    if castling.starts_with('(') {
        parse_custom_castling(castling)
    } else {
        parse_traditional_castling(castling, board_height, board_width)
    }
}

/// Converts a custom castling string to a list of squares that have not moved
/// The format is `(a1,b2,c3)`
fn parse_custom_castling(castling: &str) -> wrap_res!(Vec<(BCoord, BCoord)>) {
    let mut result = vec![];
    // Remove the parentheses
    let castling = &castling[1..castling.len() - 1];
    let squares = castling.split(',');
    for square in squares {
        let (x, y) = match scan_fmt!(square, "{[a-p]}{d}", char, BCoord) {
            Ok(parts) => parts,
            Err(_) => err!("Invalid castling square in FEN string")
        };
        // x is guaranteed to be a valid character between 'a' and 'p'
        let x = x.to_digit(36).unwrap() as BCoord - 10;
        let y = y - 1;
        result.push((x, y));
    }
    Ok(result)
}

/// Converts a traditional castling string to a list of squares that have not moved
/// It is assumed that the pieces are on the traditional starting squares (king in the middle, rooks on the corners)
fn parse_traditional_castling(castling: &str, height: BCoord, width: BCoord) -> wrap_res!(Vec<(BCoord, BCoord)>) {
    let mut result = vec![];
    err_assert!(castling.to_ascii_lowercase().chars().all(|c| c == 'k' || c == 'q' || ('a'..='p').contains(&c)),
        "Invalid castling rights in FEN string: '{castling}'");
    for c in castling.chars() {
        if c == 'K' {
            result.push((width - 1, 0)); // Right rook
            result.push((width / 2, 0)); // King
        } else if c == 'Q' {
            result.push((0, 0)); // Left rook
            result.push((width / 2, 0)); // King
        } else if c == 'k' {
            result.push((width - 1, height - 1)); // Right rook
            result.push((width / 2, height - 1)); // King
        } else if c == 'q' {
            result.push((0, height - 1)); // Left rook
            result.push((width / 2, height - 1)); // King
        } else {
            // When using the AHah format, the king file must also be specified
            let x = c.to_ascii_lowercase().to_digit(36).unwrap() as BCoord - 10;
            let is_white = c.is_ascii_uppercase();
            let y = if is_white { 0 } else { height - 1 };
            result.push((x, y));
        }
    }
    Ok(result)
}


/// Extracts the FEN data from a Position, so that it can be converted to a string
impl From<&Position> for FenData {
    fn from(pos: &Position) -> Self {
        let mut piece_placements = Vec::new();
        let mut castling = Vec::new();
        let mut walls = Vec::new();
        let width = pos.dimensions.width;
        let height = pos.dimensions.height;
        for x in 0..width {
            for y in 0..height {
                let index = to_index(x, y);
                if let Some(piece) = pos.piece_at(index) {
                    // Piece found, add it to the list
                    piece_placements.push(PiecePlacement { x, y, piece_id: piece.get_piece_id() });
                    // If this square can be used for castling, add it to the list
                    if piece.has_not_moved(index) && piece.used_in_castling() {
                        castling.push((x, y));
                    }
                }
                if !pos.dimensions.in_bounds(x, y) {
                    walls.push((x, y));
                }
            }
        }        
        // Extract EP square
        let ep_square_and_victim = {
            if let Some(ep_square) = pos.get_ep_square() {
                let ep_victim = pos.get_ep_victim();
                Some((from_index(ep_square), from_index(ep_victim)))
            } else {
                None
            }
        };
        
        FenData {
            width,
            height,
            piece_placements,
            walls,
            times_in_check: pos.get_times_checked().copied(),
            player_to_move: pos.whos_turn,
            castling_availability: Some(castling),
            ep_square_and_victim,
        }
    }
}



/// Outputs the FEN data as a FEN-formatted string
impl std::fmt::Display for FenData {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        // STEP 1: Piece placements
        let mut empty_count = 0;
        // For each square in the board
        for y in (0..self.height).rev() {
            for x in 0..self.width {
                // Add walls as '*' to the FEN string
                if self.walls.contains(&(x, y)) {
                    if empty_count > 0 {
                        write!(f, "{empty_count}")?;
                        empty_count = 0;
                    }
                    write!(f, "*")?;
                    continue;
                }
                let mut found = false;
                // Find the piece placed on that square
                for piece in &self.piece_placements {
                    if piece.x == x && piece.y == y {
                        if empty_count > 0 {
                            write!(f, "{empty_count}")?;
                            empty_count = 0;
                        }
                        write!(f, "{}", piece.piece_id)?;
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
                write!(f, "{empty_count}")?;
                empty_count = 0;
            }
            // Don't add a slash at the end of the last row
            if y > 0 {
                write!(f, "/")?;
            }
        }
        
        // STEP 2: Player to move
        let player_char = if self.player_to_move == 0 { 'w' } else { 'b' };
        write!(f, " {player_char}")?;
        
        // STEP 3: Output castling rights
        if let Some(castling) = &self.castling_availability {
            if castling.is_empty() {
                write!(f, " -")?;
            } else {
                // Write the list of squares in castling rights, in the format "(a1,b2,c3)"
                write!(f, " (")?;
                for (i, square) in castling.iter().enumerate() {
                    if i > 0 { write!(f, ",")?; }
                    write!(f, "{}", tuple_to_rank_file(*square))?;
                }
                write!(f, ")")?;
            }
        } else {
            write!(f, " (ALL)")?;
        }
        
        // STEP 4: EP square
        if let Some((ep_square, ep_victim)) = self.ep_square_and_victim {
            write!(f, " {}({})", tuple_to_rank_file(ep_square), tuple_to_rank_file(ep_victim))?;
        } else {
            write!(f, " -")?;
        }
        
        // STEP 5: Times in check
        // Important: Note that the order is reversed
        if let Some(times_in_check) = self.times_in_check {
            write!(f, " +{}+{}", times_in_check[1], times_in_check[0])?;
        }
        Ok(())
    }
}
