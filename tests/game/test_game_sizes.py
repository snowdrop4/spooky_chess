import pytest

import spooky_chess


def test_game_size_limits() -> None:
    # Valid FEN
    spooky_chess.Game(
        width=8,
        height=8,
        fen="rnbqkbnr/pppppppp/8/8/8/8/PPPPPPPP/RNBQKBNR w KQkq - 0 1",
        castling_enabled=True,
    )

    # Invalid FEN should raise ValueError
    with pytest.raises(ValueError):  # noqa: PT011
        spooky_chess.Game(width=8, height=8, fen="invalid", castling_enabled=True)

    with pytest.raises(ValueError):  # noqa: PT011
        spooky_chess.Game(
            width=8,
            height=8,
            fen="rnbqkbnr/pppppppp/8/8/8/8/PPPPPPPP/RNBQKBNR",
            castling_enabled=True,
        )  # Incomplete FEN
