import time

import chess

import spooky_chess


def test_move_generation_speed() -> None:
    iterations = 1000

    # Time spooky_chess
    rust_start = time.time()
    for _ in range(iterations):
        game = spooky_chess.Game.standard()
        moves = game.legal_moves()
        assert len(moves) == 20
    rust_time = time.time() - rust_start

    # Time python-chess
    python_start = time.time()
    for _ in range(iterations):
        board = chess.Board()
        moves = list(board.legal_moves)
        assert len(moves) == 20
    python_time = time.time() - python_start

    print(f"\nMove generation ({iterations} iterations):")
    print(f"  spooky_chess: {rust_time:.4f}s")
    print(f"  python-chess: {python_time:.4f}s")
    print(f"  Speedup: {python_time / rust_time:.2f}x")


def test_fen_parsing_speed() -> None:
    iterations = 1000
    test_fen = "rnbqkbnr/pppppppp/8/8/4P3/8/PPPP1PPP/RNBQKBNR b KQkq e3 0 1"

    # Time spooky_chess
    rust_start = time.time()
    for _ in range(iterations):
        spooky_chess.Game(width=8, height=8, fen=test_fen, castling_enabled=True)
    rust_time = time.time() - rust_start

    # Time python-chess
    python_start = time.time()
    for _ in range(iterations):
        board = chess.Board()
        board.set_fen(test_fen)
    python_time = time.time() - python_start

    print(f"\nFEN parsing ({iterations} iterations):")
    print(f"  spooky_chess: {rust_time:.4f}s")
    print(f"  python-chess: {python_time:.4f}s")
    print(f"  Speedup: {python_time / rust_time:.2f}x")


def test_move_making_speed() -> None:
    iterations = 1000

    # Time spooky_chess
    rust_start = time.time()
    for _ in range(iterations):
        game = spooky_chess.Game.standard()
        move = spooky_chess.Move.from_rowcol(4, 1, 4, 3)  # e2-e4
        game.make_move(move)
        game.unmake_move()
    rust_time = time.time() - rust_start

    # Time python-chess
    python_start = time.time()
    for _ in range(iterations):
        board = chess.Board()
        move = chess.Move.from_uci("e2e4")
        board.push(move)
        board.pop()
    python_time = time.time() - python_start

    print(f"\nMove making/unmaking ({iterations} iterations):")
    print(f"  spooky_chess: {rust_time:.4f}s")
    print(f"  python-chess: {python_time:.4f}s")
    print(f"  Speedup: {python_time / rust_time:.2f}x")


def test_game_simulation_speed() -> None:
    def simulate_game_rust(moves_count: int) -> int:
        rust_game = spooky_chess.Game.standard()
        moves_made = 0

        while moves_made < moves_count and not rust_game.is_over():
            legal_moves = rust_game.legal_moves()
            if legal_moves:
                rust_game.make_move(legal_moves[0])
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
                python_board.push(legal_moves[0])
                moves_made += 1
            else:
                break

        return moves_made

    iterations = 100
    moves_count = 20

    # Time spooky_chess
    rust_start = time.time()
    for _ in range(iterations):
        simulate_game_rust(moves_count=moves_count)
    rust_time = time.time() - rust_start

    # Time python-chess
    python_start = time.time()
    for _ in range(iterations):
        simulate_game_python(moves_count=moves_count)
    python_time = time.time() - python_start

    print(f"\nGame simulation ({iterations} games, 20 moves each):")
    print(f"  spooky_chess: {rust_time:.4f}s")
    print(f"  python-chess: {python_time:.4f}s")
    print(f"  Speedup: {python_time / rust_time:.2f}x")
