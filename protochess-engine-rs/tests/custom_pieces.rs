extern crate protochess_engine_rs;

#[cfg(test)]
mod custom_pieces {
    use protochess_engine_rs::piece::{Piece, PieceFactory};
    use protochess_engine_rs::types::BDimensions;
    use protochess_engine_rs::utils::to_index;

    #[test]
    fn piece_factory_pawn() {
        let dims = BDimensions::new_without_walls(8, 8);
        let factory = PieceFactory::default();
        let white_pawn = Piece::new(factory.make_pawn(123, true, &dims, vec![1, 2, 3]), 0, &dims);
        
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
        
        let black_pawn = Piece::new(factory.make_pawn(123, false, &dims, vec![1, 2, 3]), 1, &dims);
        
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
        let factory = PieceFactory::default();
        let knight = Piece::new(factory.make_knight(123), 0, &dims);
        
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
        let factory = PieceFactory::default();
        let bishop = Piece::new(factory.make_bishop(123), 0, &dims);
        
        assert_eq!(bishop.get_piece_id(), 123);
        assert_eq!(bishop.get_player(), 0);
        assert_eq!(bishop.get_material_score(), 336);
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
        let factory = PieceFactory::default();
        let rook = Piece::new(factory.make_rook(123), 0, &dims);
        
        assert_eq!(rook.get_piece_id(), 123);
        assert_eq!(rook.get_player(), 0);
        assert_eq!(rook.get_material_score(), 528);
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
        let factory = PieceFactory::default();
        let queen = Piece::new(factory.make_queen(123), 0, &dims);
        
        assert_eq!(queen.get_piece_id(), 123);
        assert_eq!(queen.get_player(), 0);
        assert_eq!(queen.get_material_score(), 1014);
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
        let factory = PieceFactory::default();
        let king = Piece::new(factory.make_king(123), 0, &dims);
        
        assert_eq!(king.get_piece_id(), 123);
        assert_eq!(king.get_player(), 0);
        assert_eq!(king.get_material_score(), 320 * 4);
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
