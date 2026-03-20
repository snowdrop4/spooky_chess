use std::error::Error;
use std::fs;

use spooky_chess::pgn::parse_pgn;
use spooky_chess::uci::{SearchResult, UciEngine};

const PGN_PATH: &str = "pgn/example/scholars_mate.pgn";
const DEPTH: u32 = 10;
const ENGINE_PATH: &str = "stockfish";

fn print_analysis(position_index: usize, result: &SearchResult) {
    let info = result.info.last();
    let depth = info.and_then(|line| line.depth);
    let score_cp = info.and_then(|line| line.score_cp);
    let score_mate = info.and_then(|line| line.score_mate);
    let pv = info.map(|line| line.pv.join(" ")).unwrap_or_default();

    println!(
        "  position {:>3}: bestmove={} depth={depth:?} score_cp={score_cp:?} score_mate={score_mate:?} pv={pv}",
        position_index, result.best_move_lan
    );
}

fn main() -> Result<(), Box<dyn Error>> {
    let pgn = fs::read_to_string(PGN_PATH)?;
    let games = parse_pgn(&pgn)?;
    println!("Loaded {} game(s) from {PGN_PATH}", games.len());

    let mut engine = UciEngine::new(ENGINE_PATH, &[])?;
    engine.set_option("Hash", "32")?;
    engine.is_ready()?;

    for (game_index, pgn_game) in games.iter().enumerate() {
        println!();
        println!(
            "Game {}: {} vs {} ({})",
            game_index + 1,
            pgn_game.headers.white().unwrap_or("?"),
            pgn_game.headers.black().unwrap_or("?"),
            pgn_game.result
        );

        engine.set_position_pgn_start(pgn_game)?;

        for (ply_index, played_move) in pgn_game.moves.iter().enumerate() {
            // Get best move
            let search = engine.go_depth(DEPTH)?;
            print_analysis(ply_index, &search);

            // Play played move
            let engine_ok = engine.make_move(played_move)?;
            if !engine_ok {
                return Err(format!(
                    "failed to replay PGN move {} at ply {}",
                    played_move.to_lan(),
                    ply_index + 1
                )
                .into());
            }
        }

        if engine.is_over() {
            println!(
                "  position {:>3}: terminal position, no legal best move",
                pgn_game.moves.len()
            );
        } else {
            let final_search = engine.go_depth(DEPTH)?;
            print_analysis(pgn_game.moves.len(), &final_search);
        }
    }

    engine.quit()?;
    Ok(())
}
