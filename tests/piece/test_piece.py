import pytest

import spooky_chess


def test_piece_colors() -> None:
    assert hasattr(spooky_chess, "WHITE")
    assert hasattr(spooky_chess, "BLACK")
    assert spooky_chess.WHITE == 1
    assert spooky_chess.BLACK == -1


def test_piece_creation() -> None:
    white_king = spooky_chess.Piece("king", spooky_chess.WHITE)
    assert white_king.piece_type() == "king"
    assert white_king.color() == spooky_chess.WHITE
    assert white_king.symbol() == "K"

    black_pawn = spooky_chess.Piece("pawn", spooky_chess.BLACK)
    assert black_pawn.piece_type() == "pawn"
    assert black_pawn.color() == spooky_chess.BLACK
    assert black_pawn.symbol() == "p"


def test_invalid_piece_types() -> None:
    with pytest.raises(ValueError):  # noqa: PT011
        spooky_chess.Piece("invalid", spooky_chess.WHITE)

    with pytest.raises(ValueError):  # noqa: PT011
        spooky_chess.Piece("", spooky_chess.WHITE)


def test_invalid_colors() -> None:
    with pytest.raises(ValueError):  # noqa: PT011
        spooky_chess.Piece("king", 0)

    with pytest.raises(ValueError):  # noqa: PT011
        spooky_chess.Piece("king", 2)


def test_piece_symbols() -> None:
    pieces = [
        ("king", spooky_chess.WHITE, "K"),
        ("king", spooky_chess.BLACK, "k"),
        ("queen", spooky_chess.WHITE, "Q"),
        ("queen", spooky_chess.BLACK, "q"),
        ("rook", spooky_chess.WHITE, "R"),
        ("rook", spooky_chess.BLACK, "r"),
        ("bishop", spooky_chess.WHITE, "B"),
        ("bishop", spooky_chess.BLACK, "b"),
        ("knight", spooky_chess.WHITE, "N"),
        ("knight", spooky_chess.BLACK, "n"),
        ("pawn", spooky_chess.WHITE, "P"),
        ("pawn", spooky_chess.BLACK, "p"),
    ]

    for piece_type, color, expected_symbol in pieces:
        piece = spooky_chess.Piece(piece_type, color)
        assert piece.symbol() == expected_symbol
        assert piece.piece_type() == piece_type
        assert piece.color() == color
