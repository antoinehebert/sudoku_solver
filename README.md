# Description
Learning Rust by reimplementing Peter Norvig's sudoku solver (https://norvig.com/sudoku.html).

# How to run

Just use `cargo run` and pass one of the puzzles in from the `puzzles/` folder as an argument.

```bash
cargo run --release puzzles/grid1.txt
```

# Parallelization

The `search` function has been parallelized using the `rayon` crate to improve performance. This allows the solver to iterate over possible values for a cell in parallel, speeding up the solving process.
