import pytest

import spooky_chess


def test_board_size_limits() -> None:
    # Valid sizes with FENs
    for i in range(5, 16):
        board = spooky_chess.Board.empty(width=i, height=i)

        king = spooky_chess.Piece("k", spooky_chess.WHITE)
        board.set_piece(0, 0, king)
        king = spooky_chess.Piece("k", spooky_chess.BLACK)
        board.set_piece(i - 1, i - 1, king)

        assert board.width() == i
        assert board.height() == i

        # Should be able to get FEN
        fen = board.to_fen()
        assert isinstance(fen, str)
        assert len(fen) > 0
        assert "k" in fen
        assert "K" in fen

    # Invalid sizes should raise ValueError
    with pytest.raises(ValueError):  # noqa: PT011
        spooky_chess.Board.empty(0, 0)

    with pytest.raises(ValueError):  # noqa: PT011
        spooky_chess.Board.empty(33, 33)


def test_minimum_board_size() -> None:
    board = spooky_chess.Board.empty(5, 5)

    # Should be able to place pieces at corners
    king = spooky_chess.Piece("k", spooky_chess.WHITE)
    board.set_piece(0, 0, king)
    king = spooky_chess.Piece("k", spooky_chess.BLACK)
    board.set_piece(4, 4, king)

    assert board.get_piece(0, 0) is not None
    assert board.get_piece(4, 4) is not None


def test_maximum_board_size() -> None:
    board = spooky_chess.Board.empty(16, 16)

    # Should be able to place pieces at corners
    king = spooky_chess.Piece("k", spooky_chess.WHITE)
    board.set_piece(0, 0, king)
    king = spooky_chess.Piece("k", spooky_chess.BLACK)
    board.set_piece(15, 15, king)

    assert board.get_piece(0, 0) is not None
    assert board.get_piece(15, 15) is not None
