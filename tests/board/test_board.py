import spooky_chess


def test_standard_board_creation() -> None:
    board = spooky_chess.Board.standard()
    assert board is not None
    assert board.width() == 8
    assert board.height() == 8


def test_standard_board_manipulation() -> None:
    board = spooky_chess.Board.standard()

    # Initially empty
    assert board.get_piece(4, 4) is None

    # Place a piece
    king = spooky_chess.Piece("king", spooky_chess.WHITE)
    board.set_piece(4, 4, king)

    # Check it's there
    piece = board.get_piece(4, 4)
    assert piece is not None
    assert piece.piece_type() == "king"
    assert piece.color() == spooky_chess.WHITE

    # Remove it
    board.set_piece(4, 4, None)
    assert board.get_piece(4, 4) is None


def test_out_of_bounds_access() -> None:
    board = spooky_chess.Board.empty(8, 8)

    # Should return None for out of bounds positions
    assert board.get_piece(8, 0) is None
    assert board.get_piece(0, 8) is None
    assert board.get_piece(10, 10) is None

    # Setting pieces out of bounds should not crash
    king = spooky_chess.Piece("king", spooky_chess.WHITE)
    board.set_piece(8, 0, king)  # Should be ignored
    board.set_piece(0, 8, king)  # Should be ignored
