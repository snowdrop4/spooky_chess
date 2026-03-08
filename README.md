# spooky_chess

Rust and Python library for the game of Chess.

# Performance

```
50000 random game playouts
  spooky_chess: 13.7757s
  python-chess: 116.3016s
  Speedup: 8.44x
```

# Validity

Identical to python-chess across 5m random game playouts.

# Install

## Rust

```fish
cargo add spooky_chess
```

## Python

```fish
uv add spooky-chess
```

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
