import shutil

import pytest

import spooky_chess

needs_stockfish = pytest.mark.skipif(
    shutil.which("stockfish") is None,
    reason="stockfish not found in PATH",
)


@needs_stockfish
def test_engine_creation() -> None:
    engine = spooky_chess.UciEngine("stockfish")
    assert engine.engine_name() is not None


@needs_stockfish
def test_go_depth() -> None:
    engine = spooky_chess.UciEngine("stockfish")
    result = engine.go_depth(5)
    assert isinstance(result, spooky_chess.SearchResult)
    assert result.best_move is not None
    assert len(result.best_move_lan) >= 4


@needs_stockfish
def test_play_sequence() -> None:
    engine = spooky_chess.UciEngine("stockfish")
    assert engine.make_move_lan("e2e4")
    assert engine.make_move_lan("e7e5")
    result = engine.go_depth(5)
    assert len(result.best_move_lan) >= 4
    # Apply bestmove
    assert engine.make_move(result.best_move)


@needs_stockfish
def test_fen_position() -> None:
    engine = spooky_chess.UciEngine("stockfish")
    engine.set_position_fen("rnbqkbnr/pp1ppppp/8/2p5/4P3/8/PPPP1PPP/RNBQKBNR w KQkq c6 0 2")
    result = engine.go_depth(5)
    assert len(result.best_move_lan) >= 4


@needs_stockfish
def test_search_result_fields() -> None:
    engine = spooky_chess.UciEngine("stockfish")
    result = engine.go_depth(10)
    assert result.best_move is not None
    assert isinstance(result.best_move_lan, str)
    # At depth 10, we should have score info
    assert result.depth is not None
    assert result.score_cp is not None or result.score_mate is not None


@needs_stockfish
def test_go_bestmove_depth() -> None:
    engine = spooky_chess.UciEngine("stockfish")
    mv = engine.go_bestmove_depth(5)
    assert isinstance(mv, spooky_chess.Move)
    assert len(mv.to_lan()) >= 4


@needs_stockfish
def test_set_option() -> None:
    engine = spooky_chess.UciEngine("stockfish")
    engine.set_option("Hash", "32")
    engine.is_ready()


@needs_stockfish
def test_set_position_startpos() -> None:
    engine = spooky_chess.UciEngine("stockfish")
    engine.make_move_lan("e2e4")
    engine.set_position_startpos()
    # After reset, e2e4 should be valid again (it's a fresh game)
    assert engine.make_move_lan("e2e4")


@needs_stockfish
def test_engine_author() -> None:
    engine = spooky_chess.UciEngine("stockfish")
    assert engine.engine_author() is not None
    assert len(engine.engine_author()) > 0


@needs_stockfish
def test_go_movetime() -> None:
    engine = spooky_chess.UciEngine("stockfish")
    result = engine.go_movetime(100)
    assert isinstance(result, spooky_chess.SearchResult)
    assert len(result.best_move_lan) >= 4


@needs_stockfish
def test_go_clock() -> None:
    engine = spooky_chess.UciEngine("stockfish")
    result = engine.go_clock(60000, 60000, 1000, 1000)
    assert isinstance(result, spooky_chess.SearchResult)
    assert len(result.best_move_lan) >= 4


@needs_stockfish
def test_go_bestmove_movetime() -> None:
    engine = spooky_chess.UciEngine("stockfish")
    mv = engine.go_bestmove_movetime(100)
    assert isinstance(mv, spooky_chess.Move)
    assert len(mv.to_lan()) >= 4


@needs_stockfish
def test_quit() -> None:
    engine = spooky_chess.UciEngine("stockfish")
    engine.quit()


@needs_stockfish
def test_is_ready() -> None:
    engine = spooky_chess.UciEngine("stockfish")
    # Should not raise
    engine.is_ready()


@needs_stockfish
def test_make_move_with_object() -> None:
    engine = spooky_chess.UciEngine("stockfish")
    result = engine.go_depth(5)
    # Apply the best move as a Move object
    assert engine.make_move(result.best_move)


@needs_stockfish
def test_search_result_pv() -> None:
    engine = spooky_chess.UciEngine("stockfish")
    result = engine.go_depth(10)
    assert isinstance(result.pv, list)
    # At depth 10, there should be PV moves
    assert len(result.pv) > 0


@needs_stockfish
def test_search_result_nodes() -> None:
    engine = spooky_chess.UciEngine("stockfish")
    result = engine.go_depth(10)
    assert result.nodes is not None
    assert result.nodes > 0


@needs_stockfish
def test_search_result_ponder() -> None:
    engine = spooky_chess.UciEngine("stockfish")
    result = engine.go_depth(10)
    # Stockfish typically provides a ponder move from startpos at depth 10
    if result.ponder_move_lan is not None:
        assert len(result.ponder_move_lan) >= 4
        assert result.ponder_move is not None


@needs_stockfish
def test_search_result_repr() -> None:
    engine = spooky_chess.UciEngine("stockfish")
    result = engine.go_depth(5)
    r = repr(result)
    assert "SearchResult" in r
    assert result.best_move_lan in r


@needs_stockfish
def test_engine_repr() -> None:
    engine = spooky_chess.UciEngine("stockfish")
    r = repr(engine)
    assert "UciEngine" in r


@needs_stockfish
def test_invalid_fen_raises() -> None:
    engine = spooky_chess.UciEngine("stockfish")
    with pytest.raises(RuntimeError):
        engine.set_position_fen("not a valid fen")


@needs_stockfish
def test_invalid_move_lan_raises() -> None:
    engine = spooky_chess.UciEngine("stockfish")
    with pytest.raises(ValueError, match="z9z9"):
        engine.make_move_lan("z9z9")


def test_invalid_engine_path_raises() -> None:
    with pytest.raises(OSError, match="nonexistent"):
        spooky_chess.UciEngine("/nonexistent/engine/path")
