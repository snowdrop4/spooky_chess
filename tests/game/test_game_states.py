import rust_chess


def test_standard_game_initial_position_state() -> None:
    game = rust_chess.Game.standard()

    assert not game.is_check()
    assert not game.is_checkmate()
    assert not game.is_stalemate()
    assert not game.is_over()
    assert game.turn() == rust_chess.WHITE
    assert game.fullmove_number() == 1
    assert game.halfmove_clock() == 0


def test_standard_game_turn_alternation() -> None:
    game = rust_chess.Game.standard()

    # Initially white's turn
    assert game.turn() == rust_chess.WHITE

    # Make a move
    legal_moves = game.legal_moves()
    assert game.make_move(legal_moves[0]) is True, f"Move {legal_moves[0]} should be legal"

    # Now it's black's turn
    assert game.turn() == rust_chess.BLACK

    # Make another move
    legal_moves = game.legal_moves()
    assert game.make_move(legal_moves[0]) is True, f"Move {legal_moves[0]} should be legal"

    # Back to white's turn
    assert game.turn() == rust_chess.WHITE


def test_standard_game_fullmove_counter() -> None:
    game = rust_chess.Game.standard()

    assert game.fullmove_number() == 1

    # Make a white move
    legal_moves = game.legal_moves()
    assert game.make_move(legal_moves[0]) is True, f"Move {legal_moves[0]} should be legal"

    # Still move 1 (black hasn't moved yet)
    assert game.fullmove_number() == 1

    # Make a black move
    legal_moves = game.legal_moves()
    assert game.make_move(legal_moves[0]) is True, f"Move {legal_moves[0]} should be legal"

    # Now it's move 2
    assert game.fullmove_number() == 2


def test_standard_game_halfmove_clock_pawn_move() -> None:
    game = rust_chess.Game.standard()

    # Halfmove clock should start at 0
    assert game.halfmove_clock() == 0, "Halfmove clock should start at 0"

    # Make a white knight move
    knight_move = rust_chess.Move.from_lan("b1c3", 8, 8)
    assert game.make_move(knight_move) is True

    # Halfmove clock should increment
    assert game.halfmove_clock() == 1, "Halfmove clock should increment after non-pawn move"

    # Make a black pawn move
    pawn_move = rust_chess.Move.from_lan("e7e5", 8, 8)
    assert game.make_move(pawn_move) is True

    # Halfmove clock should reset to 0 after pawn move
    assert game.halfmove_clock() == 0, "Halfmove clock should reset after pawn move"


def test_standard_game_move_making_and_unmaking() -> None:
    game = rust_chess.Game.standard()

    initial_fen = game.to_fen()
    initial_turn = game.turn()
    initial_fullmove = game.fullmove_number()

    # Make a move
    legal_moves = game.legal_moves()
    move = legal_moves[0]
    assert game.make_move(move) is True, f"Move {move} should be legal"

    assert game.turn() != initial_turn

    # Unmake the move
    assert game.unmake_move() is True

    assert game.to_fen() == initial_fen
    assert game.turn() == initial_turn
    assert game.fullmove_number() == initial_fullmove


def test_standard_game_invalid_move_rejection() -> None:
    game = rust_chess.Game.standard()

    # Try to move from an empty square
    invalid_move = rust_chess.Move.from_rowcol(4, 3, 4, 4)  # e4-e5 (no piece on e4)
    assert game.make_move(invalid_move) is False, f"Move {invalid_move} should be invalid"

    assert game.turn() == rust_chess.WHITE  # Turn shouldn't change


def test_standard_game_game_reset() -> None:
    game = rust_chess.Game.standard()

    initial_fen = game.to_fen()

    # Make some moves
    for _ in range(4):
        legal_moves = game.legal_moves()
        if legal_moves:
            assert game.make_move(legal_moves[0]) is True, f"Move {legal_moves[0]} should be legal"

    # Game state should be different
    assert game.to_fen() != initial_fen

    # Create a new game with initial position
    game = rust_chess.Game(width=8, height=8, fen=initial_fen, castling_enabled=True)

    # Should be back to initial state
    assert game.to_fen() == initial_fen
    assert game.turn() == rust_chess.WHITE
    assert game.fullmove_number() == 1
    assert game.halfmove_clock() == 0


def test_standard_game_legal_moves_consistency() -> None:
    game = rust_chess.Game.standard()

    legal_moves = game.legal_moves()

    # All moves should be valid
    for move in legal_moves:
        # Create a copy of the game to test the move
        test_game = rust_chess.Game(width=8, height=8, fen=game.to_fen(), castling_enabled=True)

        # The move should be legal
        assert test_game.make_move(move) is True, f"Move {move.to_lan()} should be legal"


def test_standard_game_setup_initial_position() -> None:
    game = rust_chess.Game.standard()

    # Make a move first
    legal_moves = game.legal_moves()
    assert game.make_move(legal_moves[0]) is True, f"Move {legal_moves[0]} should be legal"

    # Game state should be different
    assert game.fullmove_number() == 1
    assert game.turn() == rust_chess.BLACK
    assert game.to_fen() != "rnbqkbnr/pppppppp/8/8/8/8/PPPPPPPP/RNBQKBNR w KQkq - 0 1"

    # Create new game with initial position
    game = rust_chess.Game.standard()

    # Should be back to starting position
    assert game.to_fen() == "rnbqkbnr/pppppppp/8/8/8/8/PPPPPPPP/RNBQKBNR w KQkq - 0 1"


def test_standard_game_multiple_move_unmake_sequence() -> None:
    game = rust_chess.Game.standard()

    initial_fen = game.to_fen()
    moves_made = []

    # Make several moves
    for _ in range(6):
        legal_moves = game.legal_moves()
        if legal_moves:
            move = legal_moves[0]
            if game.make_move(move) is True:
                moves_made.append(move)

    # Game should be in a different state
    assert game.to_fen() != initial_fen

    # Unmake all moves
    for _ in range(len(moves_made)):
        assert game.unmake_move() is True, "Unmaking move should succeed"

    # Should be back to initial position
    assert game.to_fen() == initial_fen
