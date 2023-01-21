use ahash::AHashSet;

use crate::piece::PieceFactory;
use crate::{PieceDefinition, PieceId};
use crate::utils::{to_index, from_index};

use super::Position;
use super::global_rules::GlobalRules;
use super::position_properties::PositionProperties;
use super::{BCoord, Player, BDimensions, GameMode};

pub struct PiecePlacement {
    pub owner: Player,
    pub piece_id: PieceId,
    pub x: BCoord,
    pub y: BCoord,
    // True if it has not moved. This is an option so that JS can leave it as undefined
    pub can_castle: Option<bool>,
}
impl PiecePlacement {
    fn new(owner: Player, piece_id: PieceId, x: BCoord, y: BCoord, can_castle: bool) -> Self {
        PiecePlacement { owner, piece_id, x, y, can_castle: Some(can_castle), }
    }
}
pub struct GameState {
    pub piece_types: Vec<PieceDefinition>,
    pub valid_squares: Vec<(BCoord, BCoord)>,
    pub pieces: Vec<PiecePlacement>,
    pub whos_turn: Player,
    pub ep_square_and_victim: Option<((BCoord, BCoord), (BCoord, BCoord))>,
    pub global_rules: GlobalRules,
}

impl GameState {
    pub fn from_fen(fen: &str) -> Self {
        // FEN constants
        const BOARD_WIDTH: BCoord = 8;
        const BOARD_HEIGHT: BCoord = 8;
        const ID_KING: PieceId = 0;
        const ID_QUEEN: PieceId = 1;
        const ID_ROOK: PieceId = 2;
        const ID_BISHOP: PieceId = 3;
        const ID_KNIGHT: PieceId = 4;
        const ID_PAWN: PieceId = 5;
        
        let fen_parts: Vec<&str> = fen.split_whitespace().collect();
        assert!(fen_parts.len() >= 6, "Invalid FEN string: {}", fen);
        let mode = fen_parts.get(6).map_or(GameMode::Standard, |s| (*s).into());
        
        let whos_turn = if fen_parts[1] == "w" {0} else {1};
        let factory = PieceFactory::new(mode);
        let piece_types = vec![
            factory.make_king(ID_KING),
            factory.make_queen(ID_QUEEN),
            factory.make_rook(ID_ROOK),
            factory.make_bishop(ID_BISHOP),
            factory.make_knight(ID_KNIGHT),
            factory.make_pawn(ID_PAWN, true, BOARD_WIDTH, BOARD_HEIGHT, vec![ID_QUEEN, ID_ROOK, ID_BISHOP, ID_KNIGHT]),
            factory.make_pawn(ID_PAWN, false, BOARD_WIDTH, BOARD_HEIGHT, vec![ID_QUEEN, ID_ROOK, ID_BISHOP, ID_KNIGHT]),
        ];
        
        let mut valid_squares = Vec::new();
        for x in 0..BOARD_WIDTH {
            for y in 0..BOARD_HEIGHT {
                valid_squares.push((x, y));
            }
        }
        
        // Piece placement
        let mut pieces = Vec::new();
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
            let player = {
                if c.is_ascii_uppercase() { 0 } // White piece
                else { 1 } // Black piece
            };
            pieces.push(PiecePlacement::new(player, Self::get_piece_id(&piece_types, c), x, y, false));
            x += 1;
        }
        
        // Enable castling rights
        let mut enable_castle = |is_white: bool, kingside: bool| {
            let len = pieces.len();
            let row_y = if is_white { 0 } else { BOARD_HEIGHT - 1 };
            let mut found_rook = false;
            for i in 0..len {
                let p = {
                    // If kingside, traverse ranks from right to left, otherwise from left to right
                    if kingside { &mut pieces[len - i - 1] }
                    else { &mut pieces[i] }
                };
                // Visit only pieces on the correct row and of the correct color
                if p.y != row_y { continue; }
                if (p.owner == 0) != is_white { continue; }
                // Find the first rook and enable it
                if !found_rook && p.piece_id == ID_ROOK {
                    found_rook = true;
                    assert!(p.can_castle == Some(false), "Rook should not have been able to castle, FEN might be invalid");
                    p.can_castle = Some(true);
                    continue;
                }
                // Next search for the king, but stop if we find another rook instead
                if found_rook && p.piece_id == ID_KING {
                    p.can_castle = Some(true);
                }
                if found_rook && p.piece_id == ID_ROOK {
                    break;
                }
            }
        };
        if fen_parts[2].contains('K') {
            enable_castle(true, true);
        }
        if fen_parts[2].contains('Q') {
            enable_castle(true, false);
        }
        if fen_parts[2].contains('k') {
            enable_castle(false, true);
        }
        if fen_parts[2].contains('q') {
            enable_castle(false, false);
        }
        
        let ep_square_and_victim = {
            if fen_parts[3] != "-" {
                let ep_x = (fen_parts[3].as_bytes()[0] - b'a') as BCoord;
                let ep_y = (fen_parts[3].as_bytes()[1] - b'1') as BCoord;
                let ep_square = (ep_x, ep_y);
                let ep_victim = {
                    if whos_turn == 0 { (ep_x, ep_y + 1) }
                    else { (ep_x, ep_y - 1) }
                };
                Some((ep_square, ep_victim))
            } else {
                None
            }
        };
        
        let global_rules = GlobalRules::for_mode(mode);
        
        GameState { piece_types, valid_squares, pieces, whos_turn, ep_square_and_victim, global_rules }
    }
    
    fn get_piece_id(piece_types: &Vec<PieceDefinition>, c: char) -> PieceId {
        for piece in piece_types {
            if c.eq_ignore_ascii_case(&piece.char_rep) {
                return piece.id;
            }
        }
        panic!("Invalid piece char: {}", c);
    }
}

impl From<&Position> for GameState {
    fn from(pos: &Position) -> Self {
        let mut piece_types_set = AHashSet::<&PieceDefinition>::new();
        let mut pieces = Vec::new();
        let mut valid_squares = Vec::new();
        for x in 0..pos.dimensions.width {
            for y in 0..pos.dimensions.height {
                let index = to_index(x, y);
                if let Some(piece) = pos.piece_at(index) {
                    piece_types_set.insert(piece.get_movement());
                    pieces.push(PiecePlacement::new(piece.get_player(), piece.get_piece_id(), x, y, piece.has_not_moved(index)));
                }
                if pos.dimensions.in_bounds(x, y) {
                    valid_squares.push((x, y));
                }
            }
        }
        // Convert set of &PieceDefinition to Vec of PieceDefinition
        let piece_types = piece_types_set.into_iter().cloned().collect::<Vec<_>>(); 
        // Extract EP square
        let ep_square_and_victim = {
            if let Some(ep_square) = pos.get_ep_square() {
                let ep_victim = pos.get_ep_victim();
                Some((from_index(ep_square), from_index(ep_victim)))
            } else {
                None
            }
        };
        GameState {
            piece_types,
            valid_squares,
            pieces,
            whos_turn: pos.whos_turn,
            ep_square_and_victim,
            global_rules: pos.global_rules.clone().into(),
        }
    }
}

impl From<GameState> for Position {
    fn from(state: GameState) -> Self {
        let dims = BDimensions::from_valid_squares(&state.valid_squares);
    
        // Assert that all pieces are placed on valid squares
        for p in &state.pieces {
            assert!(dims.in_bounds(p.x, p.y));
        }
        
        // Update props
        let mut props = PositionProperties::default();
        if let Some(((sx,sy),(vx,vy))) = state.ep_square_and_victim {
            assert!(dims.in_bounds(sx, sy), "Invalid EP square: {:?}", (sx,sy));
            assert!(dims.in_bounds(vx, vy), "Invalid EP victim: {:?}", (vx,vy));
            props.set_ep_square(to_index(sx, sy), to_index(vx, vy));
        }
        if state.whos_turn == 1 {
            // Use the top bit as player zobrist key
            props.zobrist_key ^= 0x8000000000000000;
        }

        // Instantiate position and register piecetypes
        let mut pos = Position::new(dims, state.whos_turn, props, state.global_rules);
        for definition in &state.piece_types {
            pos.register_piecetype(definition);
        }
        
        // Add pieces
        for p in state.pieces {
            // By default, assume pieces have not moved
            let can_castle = p.can_castle.unwrap_or(true);
            pos.public_add_piece(p.owner, p.piece_id, to_index(p.x, p.y), can_castle);
        }
        pos
    }
}

impl Default for GameState {
    fn default() -> Self {
        GameState::from_fen("rnbqkbnr/pppppppp/8/8/8/8/PPPPPPPP/RNBQKBNR w KQkq - 0 1")
    }
}
