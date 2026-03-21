import random
import time

import chess
import pytest

import spooky_chess


def simulate_game_rust(moves_count: int) -> int:
    rust_game = spooky_chess.Game.standard()
    moves_made = 0

    while moves_made < moves_count:
        turn_state = rust_game.turn_state()
        legal_moves = turn_state.legal_moves()
        if not legal_moves:
            break

        rust_game.make_move_unchecked(random.choice(legal_moves))
        moves_made += 1

    return moves_made


def simulate_game_python(moves_count: int) -> int:
    python_board = chess.Board()
    moves_made = 0

    while moves_made < moves_count and not python_board.is_game_over():
        legal_moves = list(python_board.legal_moves)
        if legal_moves:
            python_board.push(random.choice(legal_moves))
            moves_made += 1
        else:
            break

    return moves_made


@pytest.mark.slow
def test_compare_random_game_playout() -> None:
    game_count = 50000
    move_count = 100

    rust_moves_made = 0
    python_moves_made = 0

    # Time spooky_chess
    rust_start = time.time()
    for _ in range(game_count):
        rust_moves_made += simulate_game_rust(moves_count=move_count)
    rust_time = time.time() - rust_start

    # Time python-chess
    python_start = time.time()
    for _ in range(game_count):
        python_moves_made += simulate_game_python(moves_count=move_count)
    python_time = time.time() - python_start

    print(f"\n{game_count} random game playouts")
    print("  spooky_chess (Python Bindings):")
    print(f"    moves:   {rust_moves_made}")
    print(f"    time:    {rust_time:.2f}s")
    print(f"    moves/s: {rust_moves_made / rust_time:.2f}")
    print("  python-chess:")
    print(f"    moves:   {python_moves_made}")
    print(f"    time:    {python_time:.2f}s")
    print(f"    moves/s: {python_moves_made / python_time:.2f}")
    print(f"  Speedup: {python_time / rust_time:.2f}x")
