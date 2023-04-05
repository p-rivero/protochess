# Custom FEN format

Some variants can have complex rules regarding castling and en passant, which means that the standard FEN format is not enough.
For example, a player can have many kings (possibly in many different ranks of the board) and we want to specify which ones can castle. Also, since the piece that can double jump can move in arbitrary ways, the traditional en passant square is not enough.

The FEN format used by the engine has the following space-separated fields:

  1. **Piece placement:** Same as standard FEN.
  
      Walls are represented using `*`, like in XBoard.
  
      Required.

  2. **Player to move:** Same as standard FEN (`w` or `b`).
  
      Optional, defaults to `w`.
  
  3. **Castling rights:** Custom format.
  
      This field has the form `(a1,b2,c3)`, where each square contains a *piece that has **not** moved*. The squares are enclosed in parentheses and separated by commas (without spaces). To keep the format short, only the relevant squares are included (pieces that participate in castling as king or as rook).
      
      The special value `(ALL)` can be used to indicate that all pieces can castle because they have not moved.
      Like in standard FEN, the special value `-` can be used to indicate that no piece can castle.
  
      For example, the starting position would be either `(a1,h1,e1,a8,h8,e8)` or `(ALL)`.
    
      Optional, defaults to `(ALL)`.
      
      > To mantain compatibility with traditional FEN, `QKqk` and `AHah` formats are also supported, but with some limitations:
      > - Use `QKqk` only when the piece positions are the same as in standard chess (rooks at the edges, king in the middle). Chess960 is not supported with this format.
      > - When using `AHah`, also include the file(s) of the king(s). The starting position would be `AEHaeh`. This format is compatible with Chess960 as long as the king file is included.

  4. **En passant:** Custom format.
  
      This field has the form `a1(b2)`, where the first square is the *en passant square* and the second square is the *en passant victim square*.
      
      - En passant square: The square traditionally used in standard FEN. If you want to perform an en passant capture, you must move your "pawn" (or custom piece) to this square.
      - En passant victim square: The square that contains the piece that will be captured by the en passant capture. In standard chess, this is the square in front of the en passant square. In custom variants, this can be any square.
      
      For example, in standard chess, making the move `e2e4` (pawn to e4) would result in the en passant square `e3(e4)`, which in standard FEN would be just `e3`.
      
      > To mantain compatibility with traditional FEN, the victim square can be omitted. In this case, it is assumed that the piece moves like a pawn and the victim square is in front of the en passant square.
      
      Optional, defaults to `-` (en passant not available).
  
  5. **Halfmove and fullmove clocks:** Ignored. Can be omitted.
  
  6. **Check count:** `+W+B`, where `W` is the number of times White put Black in check. 
 

**IMPORTANT:** Some fields are optional, but if you want to include a field that comes after them, you must include all the previous fields, even if they are optional. The only exception is the *Check count* field, which can be specified without including the halfmove and fullmove clocks.

For example, since move clocks are ignored, the following strings are all legal and equivalent:
```
rnbqkbnr/8/8/8/8/8/8/RNBQKBNR w (all) - 1 2 +1+2
rnbqkbnr/8/8/8/8/8/8/RNBQKBNR w (all) - +1+2 3 4
rnbqkbnr/8/8/8/8/8/8/RNBQKBNR w (all) - +1+2
```
And due to the default values, the following strings are also legal and equivalent:
```
rnbqkbnr/8/8/8/8/8/8/RNBQKBNR w (all) - 12 34 +0+0
rnbqkbnr/8/8/8/8/8/8/RNBQKBNR
```
