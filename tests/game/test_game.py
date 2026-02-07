import rust_chess


def test_standard_game_game_creation() -> None:
    game = rust_chess.Game.standard()
    assert game is not None


def test_custom_game_creation() -> None:
    # Test custom FEN (6x6 board)
    custom_fen = "rnbkqr/pppppp/6/6/PPPPPP/RNBKQR w - - 0 1"
    game = rust_chess.Game(width=6, height=6, fen=custom_fen, castling_enabled=True)
    assert game is not None
    assert game.width() == 6
    assert game.height() == 6
    assert game.to_fen() == custom_fen


def test_standard_game_initial_state() -> None:
    game = rust_chess.Game.standard()

    # Check initial state
    assert game.turn() == rust_chess.WHITE
    assert game.fullmove_number() == 1
    assert game.halfmove_clock() == 0
    assert not game.is_check()
    assert not game.is_checkmate()
    assert not game.is_stalemate()
    assert not game.is_over()

    # Should have legal moves
    legal_moves = game.legal_moves()
    assert len(legal_moves) > 0
