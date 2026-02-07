import multiprocessing as mp
import random
import time

import chess

import rust_chess


def _compare_game_states(rust_game: rust_chess.Game, python_board: chess.Board, move_history: list[str]) -> None:
    # Compare FEN
    rust_fen = rust_game.to_fen()
    python_fen = python_board.fen()
    assert rust_fen == python_fen, f"FEN mismatch after moves {move_history}\nRust: {rust_fen}\nPython: {python_fen}"

    # Compare check status
    rust_check = rust_game.is_check()
    python_check = python_board.is_check()
    assert rust_check == python_check, (
        f"Check status mismatch after moves {move_history}\nRust: {rust_check}, Python: {python_check}"
    )

    # Compare checkmate status
    rust_checkmate = rust_game.is_checkmate()
    python_checkmate = python_board.is_checkmate()
    assert rust_checkmate == python_checkmate, (
        f"Checkmate status mismatch after moves {move_history}\nRust: {rust_checkmate}, Python: {python_checkmate}"
    )

    # Compare stalemate status
    rust_stalemate = rust_game.is_stalemate()
    python_stalemate = python_board.is_stalemate()
    assert rust_stalemate == python_stalemate, (
        f"Stalemate status mismatch after moves {move_history}\nRust: {rust_stalemate}, Python: {python_stalemate}"
    )

    # Compare game over status
    rust_game_over = rust_game.is_over()
    python_game_over = python_board.is_game_over()
    assert rust_game_over == python_game_over, (
        f"Game over status mismatch after moves {move_history}\nRust: {rust_game_over}, Python: {python_game_over}"
    )

    # Compare turn
    rust_turn = rust_game.turn()
    python_turn = python_board.turn
    expected_rust_turn = rust_chess.WHITE if python_turn else rust_chess.BLACK
    assert rust_turn == expected_rust_turn, (
        f"Turn mismatch after moves {move_history}\nRust: {rust_turn}, Expected: {expected_rust_turn}"
    )

    # Compare move counts
    assert rust_game.fullmove_number() == python_board.fullmove_number, (
        f"Fullmove number mismatch after moves {move_history}"
    )
    assert rust_game.halfmove_clock() == python_board.halfmove_clock, (
        f"Halfmove clock mismatch after moves {move_history}"
    )

    # Compare legal moves
    rust_moves_uci: set[str] = {move.to_lan() for move in rust_game.legal_moves()}
    python_moves_uci: set[str] = {move.uci() for move in python_board.legal_moves}

    # Check if the same moves are legal
    rust_only = rust_moves_uci - python_moves_uci
    python_only = python_moves_uci - rust_moves_uci

    assert rust_only == set(), f"Rust has extra moves after {move_history}: {rust_only}"
    assert python_only == set(), f"Python has extra moves after {move_history}: {python_only}"
    assert len(rust_moves_uci) == len(python_moves_uci), (
        f"Legal moves count mismatch after moves {move_history}\nRust: {len(rust_moves_uci)}, Python: {len(python_moves_uci)}"
    )


def _play_random_game(max_moves: int = 200, seed: int | None = None) -> tuple[int, list[str]]:
    if seed is not None:
        random.seed(seed)

    rust_game = rust_chess.Game.standard()
    python_board = chess.Board()
    move_history = []
    moves_played = 0

    for _i in range(max_moves):
        # Compare states before making a move
        _compare_game_states(rust_game, python_board, move_history)

        # Check if game is over
        if rust_game.is_over():
            break

        # Get legal moves from both implementations
        rust_moves = rust_game.legal_moves()
        python_moves = list(python_board.legal_moves)

        if not rust_moves or not python_moves:
            break

        # Choose a random move from python-chess (as reference)
        python_move = random.choice(python_moves)
        uci_move = python_move.uci()

        # Convert to rust move
        rust_move = rust_game.move_from_lan(python_move.uci())

        # Make the move in both implementations
        rust_success = rust_game.make_move(rust_move)
        assert rust_success, f"Failed to make move {uci_move} in rust after {move_history}"

        python_board.push(python_move)

        move_history.append(uci_move)
        moves_played += 1

    # Final state comparison
    _compare_game_states(rust_game, python_board, move_history)

    return moves_played, move_history


def _run_fuzz_batch(num_games: int, start_seed: int) -> dict:
    total_moves = 0
    min_moves = float("inf")
    max_moves = 0
    games_with_checkmate = 0
    games_with_stalemate = 0
    failed_games = []

    for game_num in range(num_games):
        seed = start_seed + game_num
        try:
            moves_played, move_history = _play_random_game(seed=seed)
            total_moves += moves_played
            min_moves = min(min_moves, moves_played)
            max_moves = max(max_moves, moves_played)

            # Check end condition
            rust_game = rust_chess.Game.standard()
            for move_uci in move_history:
                python_move = chess.Move.from_uci(move_uci)
                rust_move = rust_game.move_from_lan(python_move.uci())
                rust_game.make_move(rust_move)

            if rust_game.is_checkmate():
                games_with_checkmate += 1
            elif rust_game.is_stalemate():
                games_with_stalemate += 1

        except KeyboardInterrupt:
            raise
        except Exception as e:
            failed_games.append({"game_num": game_num, "seed": seed, "error": str(e)})

    return {
        "total_moves": total_moves,
        "min_moves": min_moves if min_moves != float("inf") else 0,
        "max_moves": max_moves,
        "games_with_checkmate": games_with_checkmate,
        "games_with_stalemate": games_with_stalemate,
        "failed_games": failed_games,
        "games_completed": num_games - len(failed_games),
    }


def test_fuzz() -> None:
    num_games = 5_000
    num_cores = mp.cpu_count()
    games_per_core = num_games // num_cores
    remaining_games = num_games % num_cores

    print(f"Running {num_games} games across {num_cores} cores")
    print(f"Each core will run {games_per_core} games")
    if remaining_games > 0:
        print(f"One core will run {remaining_games} additional games")

    start_time = time.time()

    # Prepare work batches
    work_batches = []
    current_seed = 0

    for i in range(num_cores):
        batch_size = games_per_core + (1 if i < remaining_games else 0)
        work_batches.append((batch_size, current_seed))
        current_seed += batch_size

    # Run batches in parallel
    with mp.Pool(processes=num_cores) as pool:
        results = pool.starmap(_run_fuzz_batch, work_batches)

    # Aggregate results
    total_moves = sum(r["total_moves"] for r in results)
    min_moves = min(r["min_moves"] for r in results if r["min_moves"] > 0)
    max_moves = max(r["max_moves"] for r in results)
    games_with_checkmate = sum(r["games_with_checkmate"] for r in results)
    games_with_stalemate = sum(r["games_with_stalemate"] for r in results)
    total_games_completed = sum(r["games_completed"] for r in results)

    # Check for any failed games
    all_failed_games = []
    for r in results:
        all_failed_games.extend(r["failed_games"])

    if all_failed_games:
        print(f"\nFailed games: {len(all_failed_games)}")
        for failed in all_failed_games[:5]:  # Show first 5 failures
            print(f"  Game {failed['game_num']} (seed {failed['seed']}): {failed['error']}")
        if len(all_failed_games) > 5:
            print(f"  ... and {len(all_failed_games) - 5} more failures")

        # Re-raise the first error to fail the test
        raise AssertionError(f"Games failed: {all_failed_games[0]['error']}")

    elapsed_time = time.time() - start_time
    avg_moves = total_moves / total_games_completed if total_games_completed > 0 else 0

    print("\nFuzz Test Results:")
    print(f"  Games played: {total_games_completed}")
    print(f"  Total moves: {total_moves}")
    print(f"  Average moves per game: {avg_moves:.1f}")
    print(f"  Min moves in a game: {min_moves}")
    print(f"  Max moves in a game: {max_moves}")
    print(f"  Games ending in checkmate: {games_with_checkmate}")
    print(f"  Games ending in stalemate: {games_with_stalemate}")
    print(f"  Time taken: {elapsed_time:.2f} seconds")
    print(f"  Average time per game: {elapsed_time / total_games_completed:.3f} seconds")
    print(f"  CPU cores used: {num_cores}")
    print(f"  Speedup factor: ~{num_cores}x")


# Sequences found during fuzzing
def test_specific_move_sequences() -> None:
    test_sequences = [
        [
            "d2d3", "c7c6", "d1d2", "f7f6", "d2g5", "f6f5", "a2a4", "f5f4", "h2h3", "g8h6", "d3d4", "b7b5", "g5g7",
            "h6f7", "g1f3", "c6c5", "b1d2", "c8b7", "d4d5", "d8a5", "g7f7", "e8f7", "f3g5", "f7g6", "h1g1", "h8g8",
            "g5e6", "g8g7", "e6d8", "a5c7", "b2b4", "c7d8", "c1b2", "b8c6", "g2g3", "b5a4", "f1g2", "d8c7", "b4c5",
            "c7d8", "g2h1", "h7h5", "b2e5", "g7g8", "h1g2", "e7e6", "g3f4", "c6e7", "g2h1", "g6h6", "e5f6", "a4a3",
            "a1d1", "h5h4", "f6g5", "h6g7", "g5e7", "g7h8", "g1g2", "d7d6", "e7d6", "d8b6", "e1f1", "b6d8", "c2c4",
            "d8e8", "d6e7", "b7a6", "e2e4", "a8c8", "g2g5", "e8c6", "e7d6", "c6a4", "f1e1", "f8d6", "g5g2", "a4d1",
            "e1d1", "d6f4", "g2g8", "h8g8", "f2f3", "f4d2", "d5d6", "e6e5", "f3f4", "d2c1", "d6d7", "a6b7", "d1c1",
            "c8c7", "c1b1", "b7c6", "d7d8q", "g8h7", "f4e5", "a3a2", "b1a2", "h7g6", "d8c7", "a7a6", "c7b6", "a6a5",
            "b6a5", "c6b5", "a5a4", "g6h7", "a4c2", "h7h8", "c2b3", "b5c4", "e5e6", "h8g7", "h1g2", "c4e6", "a2b2",
            "e6d7", "b3c3", "g7g8", "c3f6", "g8h7", "f6g7", "h7g7", "c5c6", "g7f6", "b2b3", "d7h3", "g2f3", "h3d7",
            "b3b4", "d7e8", "c6c7", "f6g7", "f3h5", "g7h7", "b4c3", "h7g7", "h5d1", "e8d7", "c7c8q", "d7c8", "c3d3",
            "g7f8", "d3c2", "c8b7", "d1f3", "f8e7", "f3g4", "e7f8", "c2b3", "b7a6", "e4e5", "h4h3", "g4h3", "a6f1",
            "e5e6", "f8g7", "h3f5", "g7g8", "f5h7", "g8h8", "h7b1", "f1b5", "b3c3", "b5f1", "b1c2", "f1b5", "e6e7",
            "b5d7", "e7e8b",
        ],  # fmt: skip
        [
            "g2g3", "b7b5", "f2f3", "g7g5", "b1c3", "a7a6", "e2e4", "g8f6", "h2h3", "c7c5", "a2a3", "d8a5", "a1a2",
            "c5c4", "a3a4", "b5b4", "b2b3", "h7h5", "c3b5", "a5b5", "c2c3", "f6g4", "d1e2", "g4h6", "a2c2", "e7e5",
            "f1g2", "b5a4", "e2c4", "a4c6", "g3g4", "h6g4", "c4b5", "c6d5", "c1a3", "f8e7", "c3b4", "f7f5", "c2c4",
            "e8f7", "h3g4", "h5g4", "b5b8", "f7f8", "h1h5", "e7b4", "c4c3", "a8a7", "h5g5", "d5d3", "g2f1", "a7c7",
            "g5g7", "b4c5", "g7g8", "h8g8", "a3b2", "f5e4", "b8c8", "f8e7", "c3c4", "d3b3", "e1e2", "e7e6", "f3f4",
            "b3b7", "d2d4", "b7b3", "c8e8", "e6f6", "c4a4", "b3e3", "e2e3", "c7c6", "a4b4", "g8h8", "e8d8", "f6g6",
            "f1a6", "h8h5", "f4e5", "c5f8", "b4b5", "c6a6", "e3f4", "a6c6", "d8d7", "h5h1", "d4d5", "h1g1", "d5d6",
            "c6c5", "d7d8", "c5c2", "b5b7", "g6h5", "f4e4", "c2c8", "b7g7", "f8d6", "g7g8", "d6c5", "g8e8", "g1d1",
            "d8g5", "h5g5", "e8c8", "d1d6", "e5e6", "c5f2", "c8f8", "g5h4", "b2a3", "d6e6", "e4f4", "e6e3", "a3c5",
            "f2g3", "f4e3", "g3e5", "f8c8", "h4h3", "c5b6", "e5b8", "c8c7", "b8a7", "c7g7", "g4g3", "b6c5", "g3g2",
            "g7f7", "g2g1b", "e3e2", "g1f2", "c5e7", "f2g1", "f7f2", "a7b6", "e7f8", "b6f2",
        ],  # fmt: skip
        [
            "b1a3", "g8h6", "a3b5", "a7a6", "b5a7", "a8a7", "b2b3", "h6g4", "d2d4", "c7c5", "h2h4", "g4e3", "a1b1",
            "b7b6", "a2a3", "b8c6", "d1d3", "c8b7", "b1a1", "c6a5", "d4c5", "b7g2", "c1d2", "e3g4", "g1f3", "g7g6",
            "d2e3", "f7f5", "d3c3", "h8g8", "c3h8", "g2f1", "f3h2", "d8c7", "h8f6", "e7f6", "h4h5", "c7c6", "c2c3",
            "a5b7", "h5g6", "c6c5", "e3c5", "e8d8", "a1c1", "f1g2", "h2f3", "f5f4", "g6h7", "f8d6", "h1h3", "g4e3",
            "h7h8b", "d6f8", "f3d2", "g8g7", "a3a4", "e3d1", "d2c4", "d1f2", "h3h4", "f2g4", "e2e3", "b6c5", "c4e5",
            "g2c6", "h4h3", "c6e4", "e1d2", "g7e7", "e5f7", "e7f7", "b3b4", "b7d6", "c1f1", "f7g7", "d2e1", "e4b1",
            "f1f3", "g7e7", "h3h4", "b1g6", "f3f4", "e7h7", "h4h3", "h7g7", "b4c5", "d8c8", "c3c4", "c8d8", "h8g7",
            "a6a5", "c5c6", "d8e7", "f4d4", "g6e4", "g7f6", "e7e8", "e1d2", "g4e3", "d4d6", "e3f1", "d2d1", "e4h7",
            "d6d5", "d7c6", "f6a1", "c6d5", "h3h1", "f8a3", "c4d5", "a3d6", "a1e5", "e8e7", "e5c3", "a7b7", "c3a5",
            "f1g3", "a5b6", "e7f8", "h1h2", "h7g8", "h2h7", "g8e6", "h7d7", "b7d7", "b6e3", "d7f7", "e3f4", "d6a3",
            "d1e1", "f7b7", "f4c7", "e6f5", "e1f2", "f5h3", "d5d6", "h3g4", "f2g3", "b7b3", "g3h4", "b3f3", "h4g4",
            "f3f6", "g4h5", "a3b2", "a4a5", "b2c3", "h5g5", "f8g8", "g5h5", "c3d4", "h5g4", "g8f7", "c7b6", "f7f8",
            "g4h3", "f6f3", "h3h4", "f3f4", "h4g3", "f4g4", "g3g4", "d4h8", "d6d7", "f8e7", "d7d8b", "e7e6", "d8h4",
            "e6d6", "a5a6", "d6e5", "b6c5", "e5e6", "h4g5", "e6d5", "c5f2", "h8a1", "f2h4", "d5e5", "h4e1", "e5d5",
            "a6a7", "a1g7", "g4f3", "g7d4", "g5c1", "d4a1", "f3f2", "a1d4", "f2f3", "d4a7",
        ],  # fmt: skip
        [
            "b1a3", "h7h6", "c2c4", "h8h7", "d1c2", "e7e6", "b2b3", "e6e5", "e1d1", "f8a3", "g1h3", "b8a6", "c2h7",
            "d7d6", "d2d3", "c8f5", "h7h8", "e5e4", "h3g1", "g7g6", "e2e3", "h6h5", "h8h6", "f5e6", "d3d4", "a6c5",
            "c1a3", "a8c8", "h6g5", "c5b3", "g1e2", "f7f5", "g5h4", "e6c4", "h4e7", "d8e7", "h2h4", "b7b6", "d4d5",
            "c7c6", "e2c1", "b3d4", "g2g4", "c4b5", "d1d2", "e7h4", "d2d1", "c8b8", "c1d3", "h4g4", "f1e2", "d4e6",
            "a3b2", "e8d8", "a1c1", "g4g3", "b2g7", "f5f4", "d3e1", "e6d4", "g7f6", "g8e7", "c1c3", "b5a6", "h1h4",
            "f4e3", "c3d3", "d4b3", "d3d4", "g3g2", "h4g4", "a6c4", "g4g2", "d8d7", "f2f3", "e7f5", "e1c2", "f5h4",
            "g2g4", "c4b5", "d5c6", "d7e6", "d4d3", "b5d3", "f6e5", "d3a6", "e5g7", "b3c5", "g4g2", "e6e7", "e2f1",
            "e7f7", "c2b4", "a6b7", "g7f8", "e4f3", "g2g6", "h4g6", "c6c7", "f7f6", "b4c6", "g6e7", "d1e1", "b7c8",
            "f8g7", "f6f7", "c6e5", "d6e5", "c7b8b", "e7d5", "g7h8", "d5b4", "b8d6", "c8g4", "d6c5", "f7g6", "f1e2",
            "g4e6", "a2a4", "e6f5", "h8f6", "b4d5", "c5b4", "b6b5", "e2d3", "d5f6", "b4a5", "f6g4", "a5b4", "g6f6",
            "d3e2", "a7a6", "b4a5", "f3f2", "e1d1", "b5b4", "e2f1", "g4h2", "d1e2", "b4b3", "a5c3", "f5c8", "e2e3",
            "c8b7", "e3d3", "f6f5", "c3e5", "h2g4", "e5a1", "b7h1", "f1e2", "g4e5", "d3e3", "h1g2", "a1e5", "f5e6",
            "e5d6", "e6d5", "e3d2", "d5d6", "e2a6", "f2f1b", "d2e1", "d6e6", "e1d2", "e6d5", "a6d3", "d5c6", "d2c1",
            "g2f3", "d3f1", "c6b7", "f1g2", "b7c8", "c1b1", "c8c7", "b1a1", "f3d5", "g2f1", "c7d7", "f1h3", "d5e6",
            "a1b2", "d7d8", "b2a3", "b3b2", "h3g4", "e6f7", "g4h5", "b2b1b", "h5e2", "f7e8", "e2g4", "b1e4", "a3b4",
            "e8a4",
        ],  # fmt: skip
        [
            "f2f4", "e7e6", "e2e3", "g8f6", "d2d3", "b8c6", "d1e2", "e8e7", "e2g4", "f6e4", "c2c4", "e7d6", "f1e2",
            "e4g5", "a2a4", "d8f6", "e1d2", "g5h3", "g4h5", "a7a6", "g2g3", "f8e7", "g1f3", "e7f8", "f3g1", "c6d4",
            "f4f5", "c7c6", "h5f7", "d4e2", "b1c3", "h8g8", "a1b1", "f6c3", "b2c3", "c6c5", "b1b4", "g7g6", "f7e7",
            "d6e5", "e7f7", "h3g5", "f7f6", "e5f6", "c1a3", "g5h3", "f5g6", "f6e5", "a3c1", "d7d6", "d2e2", "d6d5",
            "g6g7", "a8b8", "a4a5", "h7h5", "c1d2", "h5h4", "e2f1", "h3f4", "e3e4", "f8e7", "b4b7", "e5d6", "b7b5",
            "g8d8", "h2h3", "h4g3", "d3d4", "f4e2", "b5b2", "d8e8", "d4c5", "d6e5", "b2b6", "g3g2", "f1e1", "e7f6",
            "b6b5", "e2c1", "g7g8b", "f6d8", "d2h6", "d5c4", "g8h7", "d8e7", "h6c1", "b8b6", "h7f5", "a6b5", "a5b6",
            "b5b4", "c5c6", "e7f6", "c1e3", "g2h1b", "e3g5", "f6e7", "g5h6", "e8f8", "h6f4", "e5f4", "f5h7", "f8g8",
            "h7g8", "f4e3", "e1d1", "e7d8", "h3h4", "d8h4", "g1f3", "b4c3", "f3h4", "e3d3", "h4f3", "d3e4", "f3g5",
            "e4e5", "b6b7", "c3c2", "d1d2", "e5d6", "d2c1", "c8d7", "c1d2", "c2c1q", "d2e2", "d6c6", "g8h7", "c1a3",
            "h7c2", "a3a7", "e2f1", "c6c7", "g5e6", "c7c6", "c2h7", "h1d5", "b7b8q", "a7f2", "f1f2", "d5e6", "h7g6",
            "d7e8", "b8b7", "c6c5", "b7b1", "c4c3", "f2e1", "e6c8", "b1d3", "c8d7", "g6f7", "d7b5", "d3c3", "c5b6",
            "c3e3", "b6a6", "e3e2", "b5e2"
        ],  # fmt: skip
    ]  # fmt: skip

    for sequence in test_sequences:
        rust_game = rust_chess.Game.standard()
        python_board = chess.Board()

        for move_uci in sequence:
            python_move = chess.Move.from_uci(move_uci)
            rust_move = rust_game.move_from_lan(python_move.uci())

            assert rust_game.make_move(rust_move), f"Move {move_uci} should be legal"
            python_board.push(python_move)

            _compare_game_states(rust_game, python_board, sequence[: sequence.index(move_uci) + 1])


def test_edge_case_positions() -> None:
    edge_case_fens = [
        # Positions with en passant
        "rnbqkbnr/ppp1p1pp/8/3pPp2/8/8/PPPP1PPP/RNBQKBNR w KQkq f6 0 3",
        "rnbqkbnr/pppp1ppp/8/8/3pP3/8/PPP2PPP/RNBQKBNR b KQkq e3 0 2",
        # Positions with castling rights partially lost
        "r3k2r/pppppppp/8/8/8/8/PPPPPPPP/R3K2R w KQkq - 0 1",
        "r3k2r/pppppppp/8/8/8/8/PPPPPPPP/R3K2R b KQkq - 0 1",
        # Position close to promotion
        "8/P6k/8/8/8/8/6p1/6K1 w - - 0 1",
        "8/P6k/8/8/8/8/6p1/6K1 b - - 0 1",
    ]

    for fen in edge_case_fens:
        rust_game = rust_chess.Game.standard()
        python_board = chess.Board(fen)
        rust_game = rust_chess.Game(width=8, height=8, fen=fen, castling_enabled=True)

        # Play a few random moves from this position
        move_history = []
        for _ in range(min(10, 200)):  # Play up to 10 moves or until game over
            if rust_game.is_over():
                break

            _compare_game_states(rust_game, python_board, move_history)

            python_moves = list(python_board.legal_moves)
            if not python_moves:
                break

            python_move = random.choice(python_moves)
            rust_move = rust_game.move_from_lan(python_move.uci())

            assert rust_game.make_move(rust_move), f"Failed to make move {python_move.uci()} from position {fen}"
            python_board.push(python_move)
            move_history.append(python_move.uci())

        _compare_game_states(rust_game, python_board, move_history)
