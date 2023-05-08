use crate::{Position, MoveInfo, MoveGen, MakeMoveResult};
use crate::types::{Move, MoveType};
use crate::utils::notation::{get_algebraic_notation, add_suffix};

use super::position_properties::PositionProperties;

impl Position {
    
    /// Public interface for making a move. Checks if the move is legal, and if so, makes it.
    pub fn pub_make_move(&mut self, target_move: &MoveInfo) -> MakeMoveResult {
        let moves = MoveGen::get_pseudo_moves(self, true);
        for mv in &moves {
            if target_move != mv {
                continue;
            }
            // Found the move, try to play it
            let exploded = mv.get_potential_explosion(self);
            let mut move_notation = get_algebraic_notation(self, *mv, &moves);
            if !MoveGen::make_move_if_legal(*mv, self) {
                continue;
            }
            
            let winner = {
                if self.global_rules.invert_win_conditions {
                    self.whos_turn
                } else {
                    1 - self.whos_turn
                }
            };
            
            // Leader captured (atomic chess, or playing without a king)
            if self.leader_is_captured() {
                move_notation = add_suffix(move_notation, "#");
                if self.pieces[self.whos_turn as usize].get_leader().is_none() {
                    return MakeMoveResult::all_pieces_captured(winner, exploded, move_notation);
                }
                return MakeMoveResult::leader_captured(winner, exploded, move_notation);
            }
            // Piece moved to winning square (king of the hill, racing kings)
            if self.piece_is_on_winning_square() {
                move_notation = add_suffix(move_notation, "#");
                return MakeMoveResult::piece_in_win_square(winner, exploded, move_notation);
            }
            let in_check = MoveGen::in_check(self);
            // No legal moves, check if it's checkmate or stalemate
            if MoveGen::get_legal_moves(self).is_empty() {
                if in_check {
                    move_notation = add_suffix(move_notation, "#");
                    return MakeMoveResult::checkmate(winner, exploded, move_notation);
                }
                if self.global_rules.stalemated_player_loses {
                    move_notation = add_suffix(move_notation, "#");
                    return MakeMoveResult::stalemate(Some(winner), exploded, move_notation);
                }
                // Don't add "#" since it's a draw
                return MakeMoveResult::stalemate(None, exploded, move_notation);
            }
            // Checked N times (N=3 in 3-check)
            if in_check && self.increment_num_checks() {
                move_notation = add_suffix(move_notation, "#");
                return MakeMoveResult::check_limit(winner, exploded, move_notation);
            }
            // Threefold Repetition
            if self.draw_by_repetition() {
                return MakeMoveResult::repetition(move_notation);
            }
            
            if in_check {
                move_notation = add_suffix(move_notation, "+");
            }
            return MakeMoveResult::ok(exploded, move_notation);
        }
        MakeMoveResult::illegal_move()
    }
    
    
    /// Internal function for making a move that is assumed to be legal.
    pub fn make_move(&mut self, mv: Move) {
        let my_player_num = self.whos_turn;
        let mut new_props = *self.get_properties(); // Copy the current properties
        new_props.num_captures = 0;
        let move_type = mv.get_move_type();
        
        // Update the player
        self.whos_turn = 1 - self.whos_turn;
        // Update the player zobrist key
        // For simplicity, use the lowest bit to represent the player
        new_props.zobrist_key ^= 1;
        
        // In the special case of the null move, don't do anything except update whos_turn
        // And update props
        if move_type == MoveType::Null {
            // Update props
            // Since we're passing, there cannot be an ep square
            new_props.clear_ep_square();
            new_props.move_played = mv;
            self.properties_stack.push(new_props);
            return;
        }

        // If this move is a capture, remove the captured piece before moving
        if move_type == MoveType::Capture || move_type == MoveType::PromotionCapture {
            let capt_index = mv.get_target();
            let captured_piece = self.player_piece_at(self.whos_turn, capt_index).unwrap();
            let piece_id = captured_piece.get_piece_id();
            let capt_player = captured_piece.get_player();
            let castling_zob = captured_piece.get_castle_zobrist(capt_index);
            new_props.zobrist_key ^= captured_piece.get_zobrist(capt_index);
    
            let could_castle = self.pieces[capt_player as usize].remove_piece(capt_index);
            if could_castle {
                new_props.zobrist_key ^= castling_zob;
            }
            self.captures_stack.push((piece_id, capt_player, could_castle, capt_index));
            new_props.num_captures += 1;
    
            // Check if the capturing piece explodes
            self.explode_piece(mv, my_player_num, &mut new_props);
        }
        
        // If this move is a castle, first remove the rook (in chess960 the king could move to the rook's square
        // and the rook would be overwritten)
        let mut rook_id = None;
        if move_type == MoveType::KingsideCastle || move_type == MoveType::QueensideCastle {
            let rook_from = mv.get_target();
            let rook_piece = self.player_piece_at(my_player_num, rook_from).unwrap();
            new_props.zobrist_key ^= rook_piece.get_zobrist(rook_from);
            new_props.zobrist_key ^= rook_piece.get_castle_zobrist(rook_from);
            rook_id = Some(rook_piece.get_piece_id());
            self.pieces[my_player_num as usize].remove_piece(rook_from);
        }

        let from = mv.get_from();
        let to = mv.get_to();
        // Move the piece (only if it hasn't exploded)
        if self.pieces[my_player_num as usize].index_has_piece(from) {
            // Move piece to location
            new_props.moved_piece_castle = self.pieces[my_player_num as usize].move_piece(from, to, false);
            let moved_piece = self.player_piece_at(my_player_num, to).unwrap();
            new_props.zobrist_key ^= moved_piece.get_zobrist(from);
            new_props.zobrist_key ^= moved_piece.get_zobrist(to);
            if new_props.moved_piece_castle {
                // A castling piece was moved, so it cannot castle anymore
                // Remove the castling ability from the zobrist key
                new_props.zobrist_key ^= moved_piece.get_castle_zobrist(from);
            }
            
            // Promotion
            if let Some(promo) = mv.get_promotion_piece() {
                // Remove zobrist hash of the old piece
                new_props.zobrist_key ^= moved_piece.get_zobrist(to);
                new_props.promote_from = moved_piece.get_piece_id();
                // Remove old piece
                self.pieces[my_player_num as usize].remove_piece(to);
                // Add new piece
                self.pieces[my_player_num as usize].add_piece(promo, to, false);
                let piece = self.player_piece_at(my_player_num, to).unwrap();
                new_props.zobrist_key ^= piece.get_zobrist(to);
            }
        }
        
        // If this move is a castle, add the rook back
        if move_type == MoveType::KingsideCastle || move_type == MoveType::QueensideCastle {
            let rook_to = {
                if move_type == MoveType::KingsideCastle { to - 1 }
                else { to + 1 }
            };
            self.pieces[my_player_num as usize].add_piece(rook_id.unwrap(), rook_to, false);
            let rook_piece = self.player_piece_at(my_player_num, rook_to).unwrap();
            new_props.zobrist_key ^= rook_piece.get_zobrist(rook_to);
        }

        // Pawn en-passant
        // Check for a pawn double push to set ep square
        if move_type == MoveType::DoubleJump {
            new_props.set_ep_square(mv.get_target(), mv.get_to());
        } else {
            new_props.clear_ep_square();
        }
        
        // Update props
        new_props.move_played = mv;
        self.properties_stack.push(new_props);
        
        // Update occupied bbs for future calculations
        self.update_occupied();
    }

    #[inline]
    fn explode_piece(&mut self, mv: Move, my_player_num: u8, new_props: &mut PositionProperties) {
        let from = mv.get_from();
        let moved_piece = self.pieces[my_player_num as usize].piece_at_mut(from).unwrap();
        if !moved_piece.explodes_on_capture() {
            return;
        }
        // Clone explosion radius bitboard
        let mut explosion = moved_piece.get_explosion(mv.get_to()).clone();
        // Update zobrist key
        new_props.zobrist_key ^= moved_piece.get_zobrist(from);
        let moved_piece_castle_zob = moved_piece.get_castle_zobrist(from);
        let moved_piece_id = moved_piece.get_piece_id();
        // Remove the capturing piece
        let capturing_could_castle = self.pieces[my_player_num as usize].remove_piece(from);
        if capturing_could_castle {
            new_props.zobrist_key ^= moved_piece_castle_zob;
        }
        self.captures_stack.push((moved_piece_id, my_player_num, capturing_could_castle, from));
        new_props.num_captures += 1;
        // Remove all pieces in the explosion radius
        while let Some(nindex) = explosion.lowest_one() {
            explosion.clear_bit(nindex);
            if let Some(exploded_piece) = self.piece_at_mut(nindex) {
                if exploded_piece.immune_to_explosion() {
                    continue;
                }
                new_props.zobrist_key ^= exploded_piece.get_zobrist(nindex);
                let exploded_id = exploded_piece.get_piece_id();
                let exploded_player = exploded_piece.get_player();
                let exploded_castle_zob = exploded_piece.get_castle_zobrist(nindex);
                let could_castle = self.pieces[exploded_player as usize].remove_piece(nindex);
                if could_castle {
                    new_props.zobrist_key ^= exploded_castle_zob;
                }
                self.captures_stack.push((exploded_id, exploded_player, could_castle, nindex));
                new_props.num_captures += 1;
            }
        }
    }



    /// Returns true if there is a move to undo
    pub fn can_unmake_move(&self) -> bool {
        // We always have at least one move in the stack
        self.properties_stack.len() > 1
    }

    /// Undo the most recent move
    pub fn unmake_move(&mut self) {
        // Update props
        // Consume prev props; never to return again
        let props = self.properties_stack.pop().expect("No move to undo");
        
        // Update player turn
        self.whos_turn = 1 - self.whos_turn;

        let my_player_num = self.whos_turn;
        let mv = props.move_played;
        let move_type = mv.get_move_type();
        
        // Undo null moves
        if move_type == MoveType::Null {
            return;
        }
        let from = mv.get_from();
        let to = mv.get_to();
        
        // If this move is a castle, remove the rook
        let mut rook_id = None;
        if move_type == MoveType::KingsideCastle || move_type == MoveType::QueensideCastle {
            let rook_to = {
                if move_type == MoveType::KingsideCastle { to - 1 }
                else { to + 1 }
            };
            let rook_piece = self.pieces[my_player_num as usize].piece_at_mut(rook_to).unwrap();
            rook_id = Some(rook_piece.get_piece_id());
            self.pieces[my_player_num as usize].remove_piece(rook_to);
        }

        // Undo move piece to location
        if self.pieces[my_player_num as usize].index_has_piece(to) {
            self.pieces[my_player_num as usize].move_piece(to, from, props.moved_piece_castle);
            
            // Undo Promotion
            if move_type == MoveType::Promotion || move_type == MoveType::PromotionCapture {
                // Remove old piece
                self.pieces[my_player_num as usize].remove_piece(from);
                let promoted_from = props.promote_from;
                // Assume that the piece that promoted must have moved, so it can't castle
                self.pieces[my_player_num as usize].add_piece(promoted_from, from, false);
            }
        }

        // Undo special moves
        // Special moves
        match move_type {
            MoveType::Capture | MoveType::PromotionCapture => {
                for _ in 0..props.num_captures {
                    let (piece_id, owner, captured_can_castle, capt_index) = self.captures_stack.pop().unwrap();
                    self.pieces[owner as usize].add_piece(piece_id, capt_index, captured_can_castle);
                }
            },
            MoveType::KingsideCastle | MoveType::QueensideCastle => {
                // Add back the rook
                let rook_from = mv.get_target();
                self.pieces[my_player_num as usize].add_piece(rook_id.unwrap(), rook_from, true);
            }
            _ => {}
        }
        
        // Update occupied bbs for future calculations
        self.update_occupied();
    }
}
