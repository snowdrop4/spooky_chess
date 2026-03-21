import chess

import spooky_chess


def _compare_game_states(rust_game: spooky_chess.Game, python_board: chess.Board, move_history: list[str]) -> None:
    # Compare FEN --------------------------------------------------------------
    rust_fen = rust_game.to_fen()
    python_fen = python_board.fen()
    assert rust_fen == python_fen, f"FEN mismatch after moves {move_history}\nRust: {rust_fen}\nPython: {python_fen}"

    # Compare check status -----------------------------------------------------
    rust_check = rust_game.is_check()
    python_check = python_board.is_check()
    assert rust_check == python_check, (
        f"Check status mismatch after moves {move_history}\nRust: {rust_check}, Python: {python_check}"
    )

    # Compare checkmate status -------------------------------------------------
    rust_checkmate = rust_game.is_checkmate()
    python_checkmate = python_board.is_checkmate()
    assert rust_checkmate == python_checkmate, (
        f"Checkmate status mismatch after moves {move_history}\nRust: {rust_checkmate}, Python: {python_checkmate}"
    )

    # Compare stalemate status -------------------------------------------------
    rust_stalemate = rust_game.is_stalemate()
    python_stalemate = python_board.is_stalemate()
    assert rust_stalemate == python_stalemate, (
        f"Stalemate status mismatch after moves {move_history}\nRust: {rust_stalemate}, Python: {python_stalemate}"
    )

    # Compare game over status -------------------------------------------------
    rust_game_over = rust_game.is_over()
    python_game_over = python_board.is_game_over()
    assert rust_game_over == python_game_over, (
        f"Game over status mismatch after moves {move_history}\nRust: {rust_game_over}, Python: {python_game_over}"
    )

    # Compare turn -------------------------------------------------------------
    rust_turn = rust_game.turn()
    python_turn = python_board.turn
    expected_rust_turn = spooky_chess.WHITE if python_turn else spooky_chess.BLACK
    assert rust_turn == expected_rust_turn, (
        f"Turn mismatch after moves {move_history}\nRust: {rust_turn}, Expected: {expected_rust_turn}"
    )

    # Compare move counts ------------------------------------------------------
    assert rust_game.fullmove_number() == python_board.fullmove_number, (
        f"Fullmove number mismatch after moves {move_history}"
    )
    assert rust_game.halfmove_clock() == python_board.halfmove_clock, (
        f"Halfmove clock mismatch after moves {move_history}"
    )

    # Compare LAN --------------------------------------------------------------
    legal_moves = rust_game.legal_moves()
    rust_moves_lan: set[str] = {move.to_lan() for move in legal_moves}
    python_moves_lan: set[str] = {move.uci() for move in python_board.legal_moves}

    # Check if the same moves are legal
    rust_only_lan = rust_moves_lan - python_moves_lan
    python_only_lan = python_moves_lan - rust_moves_lan

    assert rust_only_lan == set(), f"Rust has extra moves after {move_history}: {rust_only_lan}"
    assert python_only_lan == set(), f"Python has extra moves after {move_history}: {python_only_lan}"
    assert len(rust_moves_lan) == len(python_moves_lan), (
        f"Legal moves count mismatch after moves {move_history}\nRust: {len(rust_moves_lan)}, Python: {len(python_moves_lan)}"
    )

    # Compare SAN --------------------------------------------------------------

    rust_moves_san: set[str] = {rust_game.move_to_san(move) for move in legal_moves}
    python_moves_san: set[str] = {python_board.san(move) for move in python_board.legal_moves}

    # Check if the same moves are legal
    rust_only_san = rust_moves_san - python_moves_san
    python_only_san = python_moves_san - rust_moves_san

    assert rust_only_san == set(), f"Rust has extra moves after {move_history}: {rust_only_san}"
    assert python_only_san == set(), f"Python has extra moves after {move_history}: {python_only_san}"
    assert len(rust_moves_san) == len(python_moves_san), (
        f"Legal moves count mismatch after moves {move_history}\nRust: {len(rust_moves_san)}, Python: {len(python_moves_san)}"
    )
