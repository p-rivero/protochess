# Protochess engine

This is a fork of [raytran/protochess](https://github.com/raytran/protochess) with some major modifications.

- 100% user-defined pieces, instead of hardcoding the standard chess pieces.

- Add complex (slow) generation of piece-square tables and material values, since it greatly improves strength and it's only done once at the start.

- Full support for chess960 and atomic chess (more variants to come).

- Fixed some bugs in the original engine and added more tests.

- Much faster. In my tests, the original engine searched the starting position at depth 12 in 140 seconds, while this engine took 45 seconds (3x speedup).
