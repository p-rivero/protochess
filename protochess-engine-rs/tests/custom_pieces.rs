extern crate protochess_engine_rs;

#[cfg(test)]
mod custom_pieces {
    use protochess_engine_rs::PieceDefinition;
    use protochess_engine_rs::piece::{PieceId, Piece, PieceFactory};
    use protochess_engine_rs::types::{Bitboard, BDimensions};
    use protochess_engine_rs::utils::to_index;

    #[test]
    fn custom_pieces() {
        let mut engine = protochess_engine_rs::Engine::default();

        // Queen
        engine.register_piecetype(&PieceDefinition {
            id: 123,
            char_rep: 'Q',
            is_leader: false,
            can_castle: false,
            is_castle_rook: false,
            promotion_squares: Bitboard::zero(),
            double_move_squares: Bitboard::zero(),
            promo_vals: vec![],
            attack_sliding_deltas: vec![],
            attack_jump_deltas: vec![],
            attack_north: true,
            attack_south: true,
            attack_east: true,
            attack_west: true,
            attack_northeast: true,
            attack_northwest: true,
            attack_southeast: true,
            attack_southwest: true,
            translate_jump_deltas: vec![],
            translate_sliding_deltas: vec![],
            translate_north: true,
            translate_south: true,
            translate_east: true,
            translate_west: true,
            translate_northeast: true,
            translate_northwest: true,
            translate_southeast: true,
            translate_southwest: true
        });
        
        // Initial score should be 0
        assert_eq!(engine.get_score(), 0);
        // Add a queen to the board
        engine.add_piece(0, 123 as PieceId, 0, 3);
        let queen_material = 1040;
        let queen_position = 10;
        assert_eq!(engine.get_score(), queen_material + queen_position);
    }
    
    #[test]
    fn zobrist_hashing() {
        let h1 = Piece::compute_zobrist_at(1, 2, 3);
        let h1_again = Piece::compute_zobrist_at(1, 2, 3);
        let h2 = Piece::compute_zobrist_at(1, 2, 4);
        let h3 = Piece::compute_zobrist_at(1, 3, 3);
        let h4 = Piece::compute_zobrist_at(2, 2, 3);
        
        assert_eq!(h1, h1_again);
        assert_ne!(h1, h2);
        assert_ne!(h1, h3);
        assert_ne!(h1, h4);
        assert_ne!(h2, h3);
        assert_ne!(h2, h4);
        assert_ne!(h3, h4);
    }
    
    #[test]
    fn piece_factory_pawn() {
        let dims = BDimensions::new_without_walls(8, 8);
        let white_pawn = PieceFactory::make_pawn(123, 0, &dims, vec![1, 2, 3]);
        
        assert_eq!(white_pawn.get_piece_id(), 123);
        assert_eq!(white_pawn.get_player(), 0);
        assert_eq!(white_pawn.get_material_score(), 100);
        assert_eq!(white_pawn.get_material_score_all(), 0); // No pieces on the board
        assert_eq!(white_pawn.get_positional_score_all(), 0); // No pieces on the board
        
        println!("White pawn positional scores:");
        for y in (0..dims.height).rev() {
            for x in 0..dims.width {
                print!("{} ", white_pawn.get_positional_score(to_index(x, y)));
            }
            println!();
        }
        println!();
        
        let black_pawn = PieceFactory::make_pawn(123, 1, &dims, vec![1, 2, 3]);
        
        assert_eq!(black_pawn.get_piece_id(), 123);
        assert_eq!(black_pawn.get_player(), 1);
        assert_eq!(black_pawn.get_material_score(), 100);
        assert_eq!(black_pawn.get_material_score_all(), 0); // No pieces on the board
        assert_eq!(black_pawn.get_positional_score_all(), 0); // No pieces on the board
        
        println!("Black pawn positional scores:");
        for y in (0..dims.height).rev() {
            for x in 0..dims.width {
                print!("{} ", black_pawn.get_positional_score(to_index(x, y)));
            }
            println!();
        }
    }
    
    #[test]
    fn piece_factory_knight() {
        let dims = BDimensions::new_without_walls(8, 8);
        let knight = PieceFactory::make_knight(123, 0, &dims);
        
        assert_eq!(knight.get_piece_id(), 123);
        assert_eq!(knight.get_player(), 0);
        assert_eq!(knight.get_material_score(), 320);
        assert_eq!(knight.get_material_score_all(), 0); // No pieces on the board
        assert_eq!(knight.get_positional_score_all(), 0); // No pieces on the board
        
        println!("Knight positional scores:");
        for y in (0..dims.height).rev() {
            for x in 0..dims.width {
                print!("{} ", knight.get_positional_score(to_index(x, y)));
            }
            println!();
        }
    }
    
    #[test]
    fn piece_factory_bishop() {
        let dims = BDimensions::new_without_walls(8, 8);
        let bishop = PieceFactory::make_bishop(123, 0, &dims);
        
        assert_eq!(bishop.get_piece_id(), 123);
        assert_eq!(bishop.get_player(), 0);
        assert_eq!(bishop.get_material_score(), 370);
        assert_eq!(bishop.get_material_score_all(), 0); // No pieces on the board
        assert_eq!(bishop.get_positional_score_all(), 0); // No pieces on the board
        
        println!("Bishop positional scores:");
        for y in (0..dims.height).rev() {
            for x in 0..dims.width {
                print!("{} ", bishop.get_positional_score(to_index(x, y)));
            }
            println!();
        }
    }
    
    #[test]
    fn piece_factory_rook() {
        let dims = BDimensions::new_without_walls(8, 8);
        let rook = PieceFactory::make_rook(123, 0, &dims);
        
        assert_eq!(rook.get_piece_id(), 123);
        assert_eq!(rook.get_player(), 0);
        assert_eq!(rook.get_material_score(), 520);
        assert_eq!(rook.get_material_score_all(), 0); // No pieces on the board
        assert_eq!(rook.get_positional_score_all(), 0); // No pieces on the board
        
        println!("Rook positional scores:");
        for y in (0..dims.height).rev() {
            for x in 0..dims.width {
                print!("{} ", rook.get_positional_score(to_index(x, y)));
            }
            println!();
        }
    }
    
    #[test]
    fn piece_factory_queen() {
        let dims = BDimensions::new_without_walls(8, 8);
        let queen = PieceFactory::make_queen(123, 0, &dims);
        
        assert_eq!(queen.get_piece_id(), 123);
        assert_eq!(queen.get_player(), 0);
        assert_eq!(queen.get_material_score(), 1040);
        assert_eq!(queen.get_material_score_all(), 0); // No pieces on the board
        assert_eq!(queen.get_positional_score_all(), 0); // No pieces on the board
        
        println!("Queen positional scores:");
        for y in (0..dims.height).rev() {
            for x in 0..dims.width {
                print!("{} ", queen.get_positional_score(to_index(x, y)));
            }
            println!();
        }
    }
    
    #[test]
    fn piece_factory_king() {
        let dims = BDimensions::new_without_walls(8, 8);
        let king = PieceFactory::make_king(123, 0, &dims);
        
        assert_eq!(king.get_piece_id(), 123);
        assert_eq!(king.get_player(), 0);
        assert_eq!(king.get_material_score(), 320 * 2);
        assert_eq!(king.get_material_score_all(), 0); // No pieces on the board
        assert_eq!(king.get_positional_score_all(), 0); // No pieces on the board
        
        println!("King positional scores:");
        for y in (0..dims.height).rev() {
            for x in 0..dims.width {
                print!("{} ", king.get_positional_score(to_index(x, y)));
            }
            println!();
        }
    }

}
