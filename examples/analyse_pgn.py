from __future__ import annotations

from pathlib import Path

import spooky_chess

PGN_PATH = Path("pgn/example/scholars_mate.pgn")
DEPTH = 10
ENGINE_PATH = "stockfish"


def print_analysis(position_index: int, result: spooky_chess.SearchResult) -> None:
    print(
        f"  position {position_index:>3}: "
        f"bestmove={result.best_move_lan} "
        f"depth={result.depth} "
        f"score_cp={result.score_cp} "
        f"score_mate={result.score_mate} "
        f"pv={' '.join(result.pv)}"
    )


def main() -> None:
    games = spooky_chess.parse_pgn(PGN_PATH.read_text())
    print(f"Loaded {len(games)} game(s) from {PGN_PATH}")

    engine = spooky_chess.UciEngine(ENGINE_PATH)
    engine.set_option("Hash", "32")
    engine.is_ready()

    try:
        for game_index, pgn_game in enumerate(games, start=1):
            print()
            print(f"Game {game_index}: {pgn_game.white() or '?'} vs {pgn_game.black() or '?'} ({pgn_game.result()})")

            engine.set_position_pgn_start(pgn_game)

            for ply_index, played_move in enumerate(pgn_game.moves()):
                # Get best move
                result = engine.go_depth(DEPTH)
                print_analysis(ply_index, result)

                # Play played move
                engine_ok = engine.make_move(played_move)
                if not engine_ok:
                    raise RuntimeError(f"failed to replay PGN move {played_move.to_lan()} at ply {ply_index + 1}")

            if engine.is_over():
                print(f"  position {len(pgn_game.moves()):>3}: terminal position, no legal best move")
            else:
                final_result = engine.go_depth(DEPTH)
                print_analysis(len(pgn_game.moves()), final_result)
    finally:
        engine.quit()


if __name__ == "__main__":
    main()
