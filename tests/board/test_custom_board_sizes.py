import pytest

import rust_chess


def test_board_size_limits() -> None:
    # Valid sizes with FENs
    for i in range(4, 33):
        board = rust_chess.Board.empty(width=i, height=i)

        king = rust_chess.Piece("king", rust_chess.WHITE)
        board.set_piece(0, 0, king)
        king = rust_chess.Piece("king", rust_chess.BLACK)
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
        rust_chess.Board.empty(0, 0)

    with pytest.raises(ValueError):  # noqa: PT011
        rust_chess.Board.empty(33, 33)


def test_minimum_board_size() -> None:
    board = rust_chess.Board.empty(1, 1)
    assert board.width() == 1
    assert board.height() == 1

    # Should be able to place and remove pieces
    king = rust_chess.Piece("king", rust_chess.WHITE)
    board.set_piece(0, 0, king)
    assert board.get_piece(0, 0) is not None

    board.set_piece(0, 0, None)
    assert board.get_piece(0, 0) is None


def test_maximum_board_size() -> None:
    board = rust_chess.Board.empty(32, 32)
    assert board.width() == 32
    assert board.height() == 32

    # Should be able to place pieces at corners
    king = rust_chess.Piece("king", rust_chess.WHITE)
    board.set_piece(0, 0, king)
    king = rust_chess.Piece("king", rust_chess.BLACK)
    board.set_piece(31, 31, king)

    assert board.get_piece(0, 0) is not None
    assert board.get_piece(31, 31) is not None
