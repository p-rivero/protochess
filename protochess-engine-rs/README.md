# Protochess Engine
## This crate contains all the actual chess/chess engine logic.

The engine features:
* Bitboard representation using 256 bit integers for up to 16x16 sized boards.
* Kindergarden-based move generation using Rust iterators
* Zobrist Hashing 
* Transposition Tables
* Principal Variation Search in an Iterative-Deepening framework
* Quiescence search
* History Heuristic
* Killer Heuristic
* Null-move pruning
* Late move reductions
 

## Differences from a standard engine
Instead of standard piece-square-tables that are compile time constants, the engine dynamically generates piece square tables as well as material value for custom pieces. Custom pieces are assigned material values as a function of how many move directions they have. All pieces have their piece square tables generated dynamically by taking into account visibility of the center, distance to the center and distance to a promotion. 

This evaluation is not at all optimized for standard chess (factors like king safety, pawn structure are mostly ignored), but it still plays a standard game well enough to beat me every time (as a casual chess player).

## Future improvements
* Better time management
* UCI compliance for standard games

Multithreading with Lazy SMP was implemented, but it was later removed since wasm does not support true threads. There is a port of Stockfish to wasm, so it must be possible somehow, but I was not able to make it work.
