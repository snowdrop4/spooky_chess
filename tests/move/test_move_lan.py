import spooky_chess


def test_move_to_lan() -> None:
    test_cases = [
        ((0, 0), (0, 1), "a1a2"),
        ((4, 1), (4, 3), "e2e4"),
        ((7, 7), (7, 6), "h8h7"),
        ((1, 0), (2, 2), "b1c3"),
    ]

    for (src_col, src_row), (dst_col, dst_row), lan in test_cases:
        move = spooky_chess.Move.from_rowcol(src_col, src_row, dst_col, dst_row)
        assert move.to_lan() == lan


def test_move_from_lan() -> None:
    test_cases = [
        ((0, 0), (0, 1), "a1a2"),
        ((4, 1), (4, 3), "e2e4"),
        ((7, 7), (7, 6), "h8h7"),
        ((1, 0), (2, 2), "b1c3"),
    ]

    for (src_col, src_row), (dst_col, dst_row), lan in test_cases:
        move = spooky_chess.Move.from_lan(lan, 8, 8)

        actual_src_col, actual_src_row = move.src_square()
        actual_dst_col, actual_dst_row = move.dst_square()

        assert actual_src_col == src_col
        assert actual_src_row == src_row
        assert actual_dst_col == dst_col
        assert actual_dst_row == dst_row
