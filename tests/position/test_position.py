import rust_chess


def test_position_creation() -> None:
    pos = rust_chess.Position(4, 3)  # e4
    assert pos.col() == 4
    assert pos.row() == 3


def test_position_edge_cases() -> None:
    # Corner positions
    corners = [(0, 0), (0, 7), (7, 0), (7, 7)]

    for col, row in corners:
        pos = rust_chess.Position(col, row)
        assert pos.col() == col
        assert pos.row() == row

    # String representation should work
    a1 = rust_chess.Position(0, 0)
    assert str(a1) == "a1"

    h8 = rust_chess.Position(7, 7)
    assert str(h8) == "h8"
