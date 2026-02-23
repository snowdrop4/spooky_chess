import spooky_chess


def test_standard_game_initial_position_legal_moves() -> None:
    game = spooky_chess.Game.standard()
    legal_moves = game.legal_moves()

    # Standard chess starting position has 20 legal moves:
    # 16 pawn moves (2 per pawn) + 4 knight moves (2 per knight)
    assert len(legal_moves) == 20

    # Check that all moves are valid Move objects
    for move in legal_moves:
        assert isinstance(move, spooky_chess.Move)
        assert isinstance(move.to_lan(), str)
        assert len(move.to_lan()) >= 4


def test_standard_game_pawn_moves() -> None:
    game = spooky_chess.Game.standard()

    # Test moves from e2
    e2_moves = game.legal_moves_for_position(4, 1)  # e2
    assert len(e2_moves) == 2  # e3 and e4

    uci_moves = [move.to_lan() for move in e2_moves]
    assert "e2e3" in uci_moves
    assert "e2e4" in uci_moves


def test_standard_game_knight_moves() -> None:
    game = spooky_chess.Game.standard()

    # Test moves from b1
    b1_moves = game.legal_moves_for_position(1, 0)  # b1
    assert len(b1_moves) == 2  # Na3 and Nc3

    uci_moves = [move.to_lan() for move in b1_moves]
    assert "b1a3" in uci_moves
    assert "b1c3" in uci_moves


def test_standard_game_no_moves_from_empty_square() -> None:
    game = spooky_chess.Game.standard()

    # Test moves from e4 (empty square)
    e4_moves = game.legal_moves_for_position(4, 3)  # e4
    assert len(e4_moves) == 0


def test_standard_game_no_moves_from_opponent_piece() -> None:
    game = spooky_chess.Game.standard()

    # Test moves from e7 (black pawn) when it's white's turn
    e7_moves = game.legal_moves_for_position(4, 6)  # e7
    assert len(e7_moves) == 0


def test_standard_game_moves_after_game_progression() -> None:
    game = spooky_chess.Game.standard()

    # Make e2-e4
    move = spooky_chess.Move.from_rowcol(4, 1, 4, 3)
    assert game.make_move(move) is True, f"Move {move} should be legal"

    # Now it's black's turn
    assert game.turn() == spooky_chess.BLACK

    # Legal moves should include black pieces
    legal_moves = game.legal_moves()
    assert len(legal_moves) == 20  # Same number as white's initial moves

    # Test that we can get moves from black pieces
    e7_moves = game.legal_moves_for_position(4, 6)  # e7 (black pawn)
    assert len(e7_moves) == 2  # e6 and e5


def test_standard_game_move_objects() -> None:
    move = spooky_chess.Move.from_rowcol(4, 1, 4, 3)  # e2-e4

    # Test basic properties
    src_square = move.src_square()
    dst_square = move.dst_square()

    assert src_square == (4, 1)
    assert dst_square == (4, 3)
    assert move.to_lan() == "e2e4"

    # Test string representations
    assert str(move) == "e2e4"
    assert "Move" in repr(move)


def test_standard_game_move_equality() -> None:
    move1 = spooky_chess.Move.from_rowcol(4, 1, 4, 3)
    move2 = spooky_chess.Move.from_rowcol(4, 1, 4, 3)
    move3 = spooky_chess.Move.from_rowcol(4, 1, 4, 2)

    assert move1 == move2
    assert move1 != move3


def test_standard_game_move_hashing() -> None:
    move1 = spooky_chess.Move.from_rowcol(4, 1, 4, 3)
    move2 = spooky_chess.Move.from_rowcol(4, 1, 4, 3)
    move3 = spooky_chess.Move.from_rowcol(4, 1, 4, 2)

    # Equal moves should have equal hashes
    assert hash(move1) == hash(move2)

    # Can put moves in sets
    move_set = {move1, move2, move3}
    assert len(move_set) == 2  # move1 and move2 are equal


def test_standard_game_promotion_moves() -> None:
    # Create a promotion move using LAN notation
    move = spooky_chess.Move.from_lan(lan="a7a8q", board_width=8, board_height=8)  # a7-a8 promoting to queen

    # Should be able to query promotion
    promotion = move.promotion()

    assert promotion is not None
    assert promotion == "q"
