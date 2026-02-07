import pytest

import spooky_chess


def test_standard_game_initial_position_fen() -> None:
    game = spooky_chess.Game.standard()
    expected_fen = "rnbqkbnr/pppppppp/8/8/8/8/PPPPPPPP/RNBQKBNR w KQkq - 0 1"
    assert game.to_fen() == expected_fen


def test_standard_game_fen_handling() -> None:
    game = spooky_chess.Game.standard()

    # Initial position FEN
    initial_fen = "rnbqkbnr/pppppppp/8/8/8/8/PPPPPPPP/RNBQKBNR w KQkq - 0 1"
    assert game.to_fen() == initial_fen

    # Test setting from FEN (with invalid en passant that will be corrected)
    test_fen = "rnbqkbnr/pppppppp/8/8/4P3/8/PPPP1PPP/RNBQKBNR b KQkq e3 0 1"
    game = spooky_chess.Game(width=8, height=8, fen=test_fen, castling_enabled=True)

    # En passant e3 is invalid (no enemy pawn can capture), so it should be "-"
    expected_fen = "rnbqkbnr/pppppppp/8/8/4P3/8/PPPP1PPP/RNBQKBNR b KQkq - 0 1"
    assert game.to_fen() == expected_fen
    assert game.turn() == spooky_chess.BLACK


def test_standard_game_fen_parsing_and_generation_roundtrip() -> None:
    # Test FENs that should roundtrip exactly
    test_fens = [
        "rnbqkbnr/pppppppp/8/8/8/8/PPPPPPPP/RNBQKBNR w KQkq - 0 1",
        "rnbqkbnr/pp1ppppp/8/2p5/4P3/5N2/PPPP1PPP/RNBQKB1R b KQkq - 1 2",
        "r1bqkbnr/pppp1ppp/2n5/4p3/4P3/5N2/PPPP1PPP/RNBQKB1R w KQkq - 2 3",
        # Valid en passant that should be preserved
        "rnbqkbnr/ppp1pppp/8/3pP3/8/8/PPPP1PPP/RNBQKBNR w KQkq d6 0 3",
    ]

    for fen in test_fens:
        game = spooky_chess.Game(width=8, height=8, fen=fen, castling_enabled=True)
        result_fen = game.to_fen()
        assert result_fen == fen, f"Roundtrip failed for FEN: {fen}"

    # Test FENs where invalid en passant should be corrected
    invalid_ep_fens = [
        (
            "rnbqkbnr/pppppppp/8/8/4P3/8/PPPP1PPP/RNBQKBNR b KQkq e3 0 1",
            "rnbqkbnr/pppppppp/8/8/4P3/8/PPPP1PPP/RNBQKBNR b KQkq - 0 1",
        ),
        (
            "rnbqkbnr/pp1ppppp/8/2p5/4P3/8/PPPP1PPP/RNBQKBNR w KQkq c6 0 2",
            "rnbqkbnr/pp1ppppp/8/2p5/4P3/8/PPPP1PPP/RNBQKBNR w KQkq - 0 2",
        ),
    ]

    for input_fen, expected_fen in invalid_ep_fens:
        game = spooky_chess.Game(width=8, height=8, fen=input_fen, castling_enabled=True)
        result_fen = game.to_fen()
        assert result_fen == expected_fen, f"Invalid en passant not corrected for FEN: {input_fen}"


def test_invalid_fen_handling() -> None:
    invalid_fens = [
        "",
        "invalid",
        "rnbqkbnr/pppppppp/8/8/8/8/PPPPPPPP",  # Missing parts
        "rnbqkbnr/pppppppp/8/8/8/8/PPPPPPPP/RNBQKBNR w KQkq - 0",  # Missing fullmove
        "rnbqkbnr/pppppppp/8/8/8/8/PPPPPPPP/RNBQKBNR x KQkq - 0 1",  # Invalid turn
    ]

    for invalid_fen in invalid_fens:
        with pytest.raises(ValueError):  # noqa: PT011
            _game = spooky_chess.Game(width=8, height=8, fen=invalid_fen, castling_enabled=True)
