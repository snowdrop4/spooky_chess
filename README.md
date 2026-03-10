# spooky_chess 🎃👻

Rust and Python library for the game of Chess.

# Features

- Supports variable board sizes.
- Relatively fast.
- Out-of-the-box support for DL/ML (action encoding and decoding methods).

# Performance

Threadripper 9980x, 6400 MT/s CL36 DDR5:

```fish
> uv run python -m pytest -k test_compare_random_game_playout -s
50000 random game playouts
  spooky_chess: 13.4548s
  python-chess: 113.2344s
  Speedup: 8.42x
```

Most of the runtime is Python overhead, and not spooky_chess itself.

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

## Develop

### Tests

- `fish run_tests.fish`
    - `fish run_python_tests.fish`
    - `fish run_rust_tests.fish`

### Lints

- `fish run_lints.fish`

### Performance

- `fish run_benchmark.fish`
- `fish run_profile.fish`
