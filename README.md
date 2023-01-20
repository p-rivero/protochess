# Protochess engine

This is a fork of [raytran/protochess](https://github.com/raytran/protochess) with some major modifications.

- 100% user-defined pieces, instead of hardcoding the standard chess pieces.

- Add complex (slow) generation of piece-square tables and material values, since it greatly improves strength and it's only done once at the start.

- Full support for chess960 and atomic chess (more variants to come).

- Fixed some bugs in the original engine and added more tests.

- Faster move generation and more aggresive game tree pruning.
