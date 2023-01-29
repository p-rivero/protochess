# Protochess engine

This is a fork of [raytran/protochess](https://github.com/raytran/protochess) with some major modifications.

- 100% user-defined pieces, instead of hardcoding the standard chess pieces.

- Add complex (slow) generation of piece-square tables and material values, since it greatly improves strength and it's only done once at the start.

- Full support for:
  - Standard chess
  - Chess960, Transcendental chess
  - Antichess / Giveaway / Losing chess
  - Atomic chess
  - Horde
  - Racing kings
  - Three-check, Five-check
  - King of the hill
  - Any custom variant that can be defined using the currently supported rules.

- Fixed some bugs in the original engine and added more tests.

- Faster move generation and more aggresive game tree pruning.

- Multithreading support, using the Lazy SMP algorithm with a lockless transposition table.
  (Use the `parallel` feature when compiling. For example: `cargo build --release --features parallel`)

- Better WASM support, using web workers to run the engine (with or without multithreading) without blocking the UI.
