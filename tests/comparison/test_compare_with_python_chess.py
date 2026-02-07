import chess

import spooky_chess


def test_initial_position_fen() -> None:
    rust_game = spooky_chess.Game.standard()
    python_board = chess.Board()

    rust_fen = rust_game.to_fen()
    python_fen = python_board.fen()

    assert rust_fen == python_fen


def test_fen_parsing_comparison() -> None:
    test_fens = [
        "rnbqkbnr/pppppppp/8/8/8/8/PPPPPPPP/RNBQKBNR w KQkq - 0 1",
        "rnbqkbnr/pppppppp/8/8/4P3/8/PPPP1PPP/RNBQKBNR b KQkq e3 0 1",
        "rnbqkbnr/pp1ppppp/8/2p5/4P3/8/PPPP1PPP/RNBQKBNR w KQkq c6 0 2",
        "rnbqkbnr/pp1ppppp/8/2p5/4P3/5N2/PPPP1PPP/RNBQKB1R b KQkq - 1 2",
    ]

    for fen in test_fens:
        rust_game = spooky_chess.Game.standard()
        python_board = chess.Board()

        rust_game = spooky_chess.Game(width=8, height=8, fen=fen, castling_enabled=True)
        python_board.set_fen(fen)

        # Compare resulting FEN
        rust_result_fen = rust_game.to_fen()
        python_result_fen = python_board.fen()

        assert rust_result_fen == python_result_fen, f"FEN parsing mismatch for {fen}"

        # Compare legal moves count
        rust_moves = rust_game.legal_moves()
        python_moves = list(python_board.legal_moves)

        assert len(rust_moves) == len(python_moves), f"Legal moves count mismatch for {fen}"


def test_move_making_and_unmaking() -> None:
    rust_game = spooky_chess.Game.standard()
    python_board = chess.Board()

    initial_fen = rust_game.to_fen()

    # Make a move in both
    rust_move = spooky_chess.Move.from_rowcol(4, 1, 4, 3)  # e2e4
    python_move = chess.Move.from_uci("e2e4")

    assert rust_game.make_move(rust_move), f"Move {rust_move} should be legal"
    python_board.push(python_move)

    # Check FEN matches after move
    assert rust_game.to_fen() == python_board.fen()

    # Unmake move
    assert rust_game.unmake_move() is True
    python_board.pop()

    # Check we're back to initial position
    assert rust_game.to_fen() == python_board.fen()
    assert rust_game.to_fen() == initial_fen


def test_move_sequence_compatibility() -> None:
    rust_game = spooky_chess.Game.standard()
    python_board = chess.Board()

    # Italian Game opening
    move_sequence = ["e2e4", "e7e5", "g1f3", "b8c6", "f1c4", "f8c5", "d2d3", "g8f6", "c1g5", "h7h6", "g5h4", "d7d6"]

    for i, move_uci in enumerate(move_sequence):
        assert rust_game.is_check() == python_board.is_check()
        assert rust_game.is_checkmate() == python_board.is_checkmate()
        assert rust_game.is_stalemate() == python_board.is_stalemate()
        assert rust_game.is_over() == python_board.is_game_over()

        # Compare turn
        rust_turn = rust_game.turn()
        python_turn = python_board.turn
        expected_rust_turn = spooky_chess.WHITE if python_turn else spooky_chess.BLACK
        assert rust_turn == expected_rust_turn

        # Compare move counts
        assert rust_game.fullmove_number() == python_board.fullmove_number
        assert rust_game.halfmove_clock() == python_board.halfmove_clock

        # Check legal moves count before move
        rust_legal = rust_game.legal_moves()
        python_legal = list(python_board.legal_moves)

        assert len(rust_legal) == len(python_legal), f"Legal moves count mismatch before move {i}: {move_uci}"

        # Convert UCI to our move format
        src_col = ord(move_uci[0]) - ord("a")
        src_row = int(move_uci[1]) - 1
        dst_col = ord(move_uci[2]) - ord("a")
        dst_row = int(move_uci[3]) - 1

        rust_move = spooky_chess.Move.from_rowcol(src_col, src_row, dst_col, dst_row)
        python_move = chess.Move.from_uci(move_uci)

        # Make moves
        assert rust_game.make_move(rust_move) is True, f"Rust move {move_uci} should be legal"
        python_board.push(python_move)

        # Verify exact match after each move
        assert rust_game.to_fen() == python_board.fen(), f"FEN mismatch after {move_uci}"


def test_castling_compatibility() -> None:
    # Set up position where castling is possible
    castling_fen = "r3k2r/pppppppp/8/8/8/8/PPPPPPPP/R3K2R w KQkq - 0 1"

    rust_game = spooky_chess.Game.standard()
    python_board = chess.Board()

    rust_game = spooky_chess.Game(width=8, height=8, fen=castling_fen, castling_enabled=True)
    python_board.set_fen(castling_fen)

    # Both should have castling moves available
    rust_legal = rust_game.legal_moves()
    python_legal = list(python_board.legal_moves)

    # Should have same number of legal moves
    assert len(rust_legal) == len(python_legal)

    # Both should detect castling is possible
    # (This tests the castling rights parsing from FEN)
    rust_fen = rust_game.to_fen()
    python_fen = python_board.fen()
    assert rust_fen == python_fen


def test_en_passant_compatibility() -> None:
    # Set up position with en passant possible
    en_passant_fen = "rnbqkbnr/ppp1pppp/8/3pP3/8/8/PPPP1PPP/RNBQKBNR w KQkq d6 0 3"

    rust_game = spooky_chess.Game.standard()
    python_board = chess.Board()

    rust_game = spooky_chess.Game(width=8, height=8, fen=en_passant_fen, castling_enabled=True)
    python_board.set_fen(en_passant_fen)

    # FEN should match exactly (including en passant square)
    assert rust_game.to_fen() == python_board.fen()

    # Legal moves should match
    rust_legal = rust_game.legal_moves()
    python_legal = list(python_board.legal_moves)
    assert len(rust_legal) == len(python_legal)


def test_check_detection_compatibility() -> None:
    # Position with white in check
    check_positions = [
        "rnb1kbnr/pppp1ppp/8/4p3/6Pq/5P2/PPPPP2P/RNBQKBNR w KQkq - 1 3",
        "rnbqkbnr/ppp1pppp/8/3p4/4P3/8/PPPP1PPP/RNBQKBNR w KQkq - 0 2",  # No check
        "r1bqkb1r/pppp1ppp/2n2n2/4p2Q/2B1P3/8/PPPP1PPP/RNB1K1NR w KQkq - 4 4",  # Check
    ]

    for fen in check_positions:
        rust_game = spooky_chess.Game.standard()
        python_board = chess.Board()

        rust_game = spooky_chess.Game(width=8, height=8, fen=fen, castling_enabled=True)
        python_board.set_fen(fen)

        # Check detection should match exactly
        assert rust_game.is_check() == python_board.is_check(), f"Check detection mismatch for {fen}"
        assert rust_game.is_checkmate() == python_board.is_checkmate(), f"Checkmate detection mismatch for {fen}"
        assert rust_game.is_stalemate() == python_board.is_stalemate(), f"Stalemate detection mismatch for {fen}"


def test_move_make_unmake_compatibility() -> None:
    rust_game = spooky_chess.Game.standard()
    python_board = chess.Board()

    original_fen = rust_game.to_fen()

    # Make same move in both
    rust_move = spooky_chess.Move.from_rowcol(4, 1, 4, 3)  # e2-e4
    python_move = chess.Move.from_uci("e2e4")

    assert rust_game.make_move(rust_move) is True, f"Move {rust_move} should be legal"
    python_board.push(python_move)

    # States should match after move
    assert rust_game.to_fen() == python_board.fen()

    # Unmake move
    assert rust_game.unmake_move() is True, "Unmaking move should succeed"
    python_board.pop()

    # Should be back to original state
    assert rust_game.to_fen() == python_board.fen()
    assert rust_game.to_fen() == original_fen


def test_50_move_rule_compatibility() -> None:
    # Position close to 50-move rule
    fifty_move_fen = "8/8/8/8/8/8/k1K5/8 w - - 98 150"

    rust_game = spooky_chess.Game.standard()
    python_board = chess.Board()

    rust_game = spooky_chess.Game(width=8, height=8, fen=fifty_move_fen, castling_enabled=True)
    python_board.set_fen(fifty_move_fen)

    # Both should parse the halfmove clock correctly
    assert rust_game.halfmove_clock() == python_board.halfmove_clock
    assert rust_game.halfmove_clock() == 98

    # Both should handle 50-move rule the same way
    # (Note: actual implementation may differ in when exactly it's enforced)
    rust_fen = rust_game.to_fen()
    python_fen = python_board.fen()
    assert rust_fen == python_fen
