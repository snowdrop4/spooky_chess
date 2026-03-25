import spooky_chess


def test_10x10_move_strings_support_multi_digit_ranks() -> None:
    game = spooky_chess.Game(
        width=10,
        height=10,
        fen="10/R9/9k/10/10/10/10/10/10/4K5 w - - 0 1",
        castling_enabled=False,
    )

    move = game.move_from_lan("a9a10")
    assert move.to_lan() == "a9a10"
    assert game.move_to_san(move) == "Ra10"
    assert game.move_from_san("Ra10") == move
