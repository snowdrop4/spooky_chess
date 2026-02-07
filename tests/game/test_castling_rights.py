import spooky_chess


def test_standard_game_initial_castling_rights() -> None:
    game = spooky_chess.Game.standard()

    # Both sides should have both castling rights initially
    assert game.has_kingside_castling_rights(spooky_chess.WHITE) is True  # White kingside
    assert game.has_queenside_castling_rights(spooky_chess.WHITE) is True  # White queenside
    assert game.has_kingside_castling_rights(spooky_chess.BLACK) is True  # Black kingside
    assert game.has_queenside_castling_rights(spooky_chess.BLACK) is True  # Black queenside


def test_standard_game_castling_rights_after_king_move() -> None:
    game = spooky_chess.Game.standard()

    # First move the pawn to make room for king
    move = spooky_chess.Move.from_rowcol(4, 1, 4, 3)  # e2 to e4
    game.make_move(move)

    # Black responds
    move = spooky_chess.Move.from_rowcol(4, 6, 4, 4)  # e7 to e5
    game.make_move(move)

    # Move white king (e1 to e2)
    move = spooky_chess.Move.from_rowcol(4, 0, 4, 1)  # e1 to e2
    game.make_move(move)

    # White should lose both castling rights
    assert game.has_kingside_castling_rights(spooky_chess.WHITE) is False
    assert game.has_queenside_castling_rights(spooky_chess.WHITE) is False

    # Black should still have both
    assert game.has_kingside_castling_rights(spooky_chess.BLACK) is True
    assert game.has_queenside_castling_rights(spooky_chess.BLACK) is True


def test_standard_game_castling_rights_after_rook_move() -> None:
    game = spooky_chess.Game.standard()

    # Move h2 pawn to make room for rook
    move = spooky_chess.Move.from_rowcol(7, 1, 7, 3)  # h2 to h4
    game.make_move(move)

    # Black responds
    move = spooky_chess.Move.from_rowcol(7, 6, 7, 4)  # h7 to h5
    game.make_move(move)

    # Move white kingside rook (h1 to h2)
    move = spooky_chess.Move.from_rowcol(7, 0, 7, 1)  # h1 to h2
    game.make_move(move)

    # White should lose only kingside castling right
    assert game.has_kingside_castling_rights(spooky_chess.WHITE) is False
    assert game.has_queenside_castling_rights(spooky_chess.WHITE) is True
    # Black should still have both
    assert game.has_kingside_castling_rights(spooky_chess.BLACK) is True
    assert game.has_queenside_castling_rights(spooky_chess.BLACK) is True

    # Move back to h1 for simplicity
    game.unmake_move()

    # Black responds
    move = spooky_chess.Move.from_rowcol(6, 6, 6, 5)  # g7 to g6
    game.make_move(move)

    # Move a2 pawn to make room for queenside rook
    move = spooky_chess.Move.from_rowcol(0, 1, 0, 3)  # a2 to a4
    game.make_move(move)

    # Black responds
    move = spooky_chess.Move.from_rowcol(0, 6, 0, 4)  # a7 to a5
    game.make_move(move)

    # Move white queenside rook (a1 to a2)
    move = spooky_chess.Move.from_rowcol(0, 0, 0, 1)  # a1 to a2
    game.make_move(move)

    # White should lose only queenside castling right
    assert game.has_kingside_castling_rights(spooky_chess.WHITE) is True
    assert game.has_queenside_castling_rights(spooky_chess.WHITE) is False


def test_standard_game_castling_rights_from_fen() -> None:
    # No castling rights
    game = spooky_chess.Game(
        width=8, height=8, fen="rnbqkbnr/pppppppp/8/8/8/8/PPPPPPPP/RNBQKBNR w - - 0 1", castling_enabled=True
    )
    assert game.has_kingside_castling_rights(spooky_chess.WHITE) is False
    assert game.has_queenside_castling_rights(spooky_chess.WHITE) is False
    assert game.has_kingside_castling_rights(spooky_chess.BLACK) is False
    assert game.has_queenside_castling_rights(spooky_chess.BLACK) is False

    # Only white kingside
    game = spooky_chess.Game(
        width=8, height=8, fen="rnbqkbnr/pppppppp/8/8/8/8/PPPPPPPP/RNBQKBNR w K - 0 1", castling_enabled=True
    )
    assert game.has_kingside_castling_rights(spooky_chess.WHITE) is True
    assert game.has_queenside_castling_rights(spooky_chess.WHITE) is False
    assert game.has_kingside_castling_rights(spooky_chess.BLACK) is False
    assert game.has_queenside_castling_rights(spooky_chess.BLACK) is False

    # Only black queenside
    game = spooky_chess.Game(
        width=8, height=8, fen="rnbqkbnr/pppppppp/8/8/8/8/PPPPPPPP/RNBQKBNR w q - 0 1", castling_enabled=True
    )
    assert game.has_kingside_castling_rights(spooky_chess.WHITE) is False
    assert game.has_queenside_castling_rights(spooky_chess.WHITE) is False
    assert game.has_kingside_castling_rights(spooky_chess.BLACK) is False
    assert game.has_queenside_castling_rights(spooky_chess.BLACK) is True

    # All castling rights
    game = spooky_chess.Game(
        width=8, height=8, fen="rnbqkbnr/pppppppp/8/8/8/8/PPPPPPPP/RNBQKBNR w KQkq - 0 1", castling_enabled=True
    )
    assert game.has_kingside_castling_rights(spooky_chess.WHITE) is True
    assert game.has_queenside_castling_rights(spooky_chess.WHITE) is True
    assert game.has_kingside_castling_rights(spooky_chess.BLACK) is True
    assert game.has_queenside_castling_rights(spooky_chess.BLACK) is True


def test_standard_game_castling_rights_disabled_game() -> None:
    game = spooky_chess.Game(
        width=8, height=8, fen="rnbqkbnr/pppppppp/8/8/8/8/PPPPPPPP/RNBQKBNR w KQkq - 0 1", castling_enabled=False
    )

    # No castling rights should be available
    assert game.has_kingside_castling_rights(spooky_chess.WHITE) is False
    assert game.has_queenside_castling_rights(spooky_chess.WHITE) is False
    assert game.has_kingside_castling_rights(spooky_chess.BLACK) is False
    assert game.has_queenside_castling_rights(spooky_chess.BLACK) is False
