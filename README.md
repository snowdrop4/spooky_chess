# spooky_chess 🎃👻

Rust and Python library for the game of Chess.

# Features

- Drive external engines with [Universal Chess Interface](https://en.wikipedia.org/wiki/Universal_Chess_Interface).
- Variable board sizes.
- Relatively fast.
- Out-of-the-box support for DL/ML (action encoding and decoding methods).
- Consistent interface with [spooky-connect4](https://github.com/snowdrop4/spooky-connect4) and [spooky-go](https://github.com/snowdrop4/spooky-go).

# Performance

Measured with a Threadripper 9980x, and 6400 MT/s CL36 DDR5.

```fish
> cargo run
50000 random game playouts
  spooky_chess (Rust Bindings):
    moves:   4937416
    time:    8.64s
    moves/s: 571164.98
```

```fish
> uv run python -m pytest -k test_compare_random_game_playout -s --run-slow
50000 random game playouts
  spooky_chess (Python Bindings):
    moves:   4935959
    time:    13.20s
    moves/s: 373852.68
  python-chess:
    moves:   4936438
    time:    112.38s
    moves/s: 43927.96
  Speedup: 8.51x
```

# Validity

Fuzz-tested against python-chess, with 5 million random playouts.

# Install

## Rust

```fish
cargo add spooky_chess
```

## Python

```fish
uv add spooky-chess
```

Includes type hints.

# Develop

### Tests

- `fish run_tests.fish`
    - `fish run_python_tests.fish`
    - `fish run_rust_tests.fish`

### Lints

- `fish run_lints.fish`

### Performance

- `fish run_benchmark.fish`
- `fish run_profile.fish`

# See Also

* spooky-chess
* [spooky-connect4](https://github.com/snowdrop4/spooky-connect4)
* [spooky-go](https://github.com/snowdrop4/spooky-go)
