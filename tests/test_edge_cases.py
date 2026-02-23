import spooky_chess


def test_very_long_game() -> None:
    game = spooky_chess.Game.standard()

    moves_made = 0
    max_moves = 200  # Prevent infinite loops

    while moves_made < max_moves and not game.is_over():
        legal_moves = game.legal_moves()
        if not legal_moves:
            break

        # Make the first legal move
        if game.make_move(legal_moves[0]):
            moves_made += 1

    # Game should still be in valid state
    assert game.fullmove_number() > 1
    assert isinstance(game.to_fen(), str)


def test_unmake_move_on_initial_position() -> None:
    game = spooky_chess.Game.standard()

    # Should return False (no move to unmake)
    assert game.unmake_move() is False

    # Game state should be unchanged
    expected_fen = "rnbqkbnr/pppppppp/8/8/8/8/PPPPPPPP/RNBQKBNR w KQkq - 0 1"
    assert game.to_fen() == expected_fen


def test_multiple_unmake_moves() -> None:
    game = spooky_chess.Game.standard()

    # Make one move
    legal_moves = game.legal_moves()
    game.make_move(legal_moves[0])

    # Unmake it
    assert game.unmake_move() is True

    # Try to unmake another (should fail)
    assert game.unmake_move() is False


def test_extreme_fen_values() -> None:
    # Test with large move numbers
    extreme_fen = "rnbqkbnr/pppppppp/8/8/8/8/PPPPPPPP/RNBQKBNR w KQkq - 99 999"

    game = spooky_chess.Game.standard()
    game = spooky_chess.Game(width=8, height=8, fen=extreme_fen, castling_enabled=True)

    assert game.halfmove_clock() == 99
    assert game.fullmove_number() == 999


def test_fen_with_no_castling_rights() -> None:
    no_castling_fen = "rnbqkbnr/pppppppp/8/8/8/8/PPPPPPPP/RNBQKBNR w - - 0 1"

    game = spooky_chess.Game.standard()
    game = spooky_chess.Game(width=8, height=8, fen=no_castling_fen, castling_enabled=True)

    # Should parse successfully
    result_fen = game.to_fen()
    assert "w -" in result_fen


def test_board_with_only_kings() -> None:
    # Create a game with only kings using FEN
    kings_only_fen = "8/8/8/8/8/8/8/4K2k w - - 0 1"
    game = spooky_chess.Game(width=8, height=8, fen=kings_only_fen, castling_enabled=False)

    # Should have some legal moves for the white king
    legal_moves = game.legal_moves()
    assert len(legal_moves) > 0

    # All moves should be king moves (from e1)
    for move in legal_moves:
        src_col, src_row = move.src_square()
        assert src_col == 4
        assert src_row == 0

    # This is insufficient material
    assert game.is_insufficient_material()
