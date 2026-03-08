import random
import time

import chess

import spooky_chess


def test_compare_random_game_playout() -> None:
    def simulate_game_rust(moves_count: int) -> int:
        rust_game = spooky_chess.Game.standard()
        moves_made = 0

        while moves_made < moves_count and not rust_game.is_over():
            legal_moves = rust_game.legal_moves()
            if legal_moves:
                rust_game.make_move(random.choice(legal_moves))
                moves_made += 1
            else:
                break

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

    game_count = 50000
    move_count = 100

    # Time spooky_chess
    rust_start = time.time()
    for _ in range(game_count):
        simulate_game_rust(moves_count=move_count)
    rust_time = time.time() - rust_start

    # Time python-chess
    python_start = time.time()
    for _ in range(game_count):
        simulate_game_python(moves_count=move_count)
    python_time = time.time() - python_start

    print(f"\n{game_count} random game playouts")
    print(f"  spooky_chess: {rust_time:.4f}s")
    print(f"  python-chess: {python_time:.4f}s")
    print(f"  Speedup: {python_time / rust_time:.2f}x")
