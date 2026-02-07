import spooky_chess


def test_move_creation() -> None:
    move = spooky_chess.Move.from_rowcol(4, 1, 4, 3)  # e2-e4
    src_square = move.src_square()
    dst_square = move.dst_square()

    assert src_square == (4, 1)
    assert dst_square == (4, 3)
    assert move.to_lan() == "e2e4"


def test_move_creation_to_same_square() -> None:
    game = spooky_chess.Game.standard()
    same_square_move = spooky_chess.Move.from_rowcol(4, 1, 4, 1)  # e2 to e2

    # Should be rejected as invalid
    assert game.make_move(same_square_move) is False, f"Move {same_square_move} should be invalid"
