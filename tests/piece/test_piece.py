import pytest

import rust_chess


def test_piece_colors() -> None:
    assert hasattr(rust_chess, "WHITE")
    assert hasattr(rust_chess, "BLACK")
    assert rust_chess.WHITE == 1
    assert rust_chess.BLACK == -1


def test_piece_creation() -> None:
    white_king = rust_chess.Piece("king", rust_chess.WHITE)
    assert white_king.piece_type() == "king"
    assert white_king.color() == rust_chess.WHITE
    assert white_king.symbol() == "K"

    black_pawn = rust_chess.Piece("pawn", rust_chess.BLACK)
    assert black_pawn.piece_type() == "pawn"
    assert black_pawn.color() == rust_chess.BLACK
    assert black_pawn.symbol() == "p"


def test_invalid_piece_types() -> None:
    with pytest.raises(ValueError):  # noqa: PT011
        rust_chess.Piece("invalid", rust_chess.WHITE)

    with pytest.raises(ValueError):  # noqa: PT011
        rust_chess.Piece("", rust_chess.WHITE)


def test_invalid_colors() -> None:
    with pytest.raises(ValueError):  # noqa: PT011
        rust_chess.Piece("king", 0)

    with pytest.raises(ValueError):  # noqa: PT011
        rust_chess.Piece("king", 2)


def test_piece_symbols() -> None:
    pieces = [
        ("king", rust_chess.WHITE, "K"),
        ("king", rust_chess.BLACK, "k"),
        ("queen", rust_chess.WHITE, "Q"),
        ("queen", rust_chess.BLACK, "q"),
        ("rook", rust_chess.WHITE, "R"),
        ("rook", rust_chess.BLACK, "r"),
        ("bishop", rust_chess.WHITE, "B"),
        ("bishop", rust_chess.BLACK, "b"),
        ("knight", rust_chess.WHITE, "N"),
        ("knight", rust_chess.BLACK, "n"),
        ("pawn", rust_chess.WHITE, "P"),
        ("pawn", rust_chess.BLACK, "p"),
    ]

    for piece_type, color, expected_symbol in pieces:
        piece = rust_chess.Piece(piece_type, color)
        assert piece.symbol() == expected_symbol
        assert piece.piece_type() == piece_type
        assert piece.color() == color
