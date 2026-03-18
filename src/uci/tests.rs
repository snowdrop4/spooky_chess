use super::*;
use crate::color::Color;
use crate::pgn::{PgnGame, parse_pgn, parse_pgn_single_game};
use protocol::*;
use std::process::{Command, Stdio};
use std::sync::atomic::{AtomicUsize, Ordering};
use std::sync::{Arc, Mutex};

// -----------------------------------------------------------------------------
// Protocol parser tests
// -----------------------------------------------------------------------------

#[test]
fn test_parse_bestmove_with_ponder() {
    assert_eq!(
        parse_bestmove_line("bestmove e2e4 ponder e7e5"),
        Some(("e2e4".to_string(), Some("e7e5".to_string())))
    );
}

#[test]
fn test_parse_bestmove_without_ponder() {
    assert_eq!(
        parse_bestmove_line("bestmove e2e4"),
        Some(("e2e4".to_string(), None))
    );
}

#[test]
fn test_parse_bestmove_only_keyword() {
    assert_eq!(parse_bestmove_line("bestmove"), None);
}

#[test]
fn test_parse_bestmove_rejects_non_bestmove() {
    assert_eq!(parse_bestmove_line("info depth 10"), None);
}

#[test]
fn test_parse_info_all_fields() {
    let line = "info depth 20 score cp 35 nodes 1234567 nps 500000 time 2469 pv e2e4 e7e5 g1f3";
    let info = parse_info_line(line).expect("should parse");
    assert_eq!(info.depth, Some(20));
    assert_eq!(info.score_cp, Some(35));
    assert_eq!(info.score_mate, None);
    assert_eq!(info.nodes, Some(1234567));
    assert_eq!(info.nps, Some(500000));
    assert_eq!(info.time_ms, Some(2469));
    assert_eq!(info.pv, vec!["e2e4", "e7e5", "g1f3"]);
}

#[test]
fn test_parse_info_mate_score() {
    let info = parse_info_line("info depth 15 score mate 3 pv h5f7").expect("should parse");
    assert_eq!(info.score_cp, None);
    assert_eq!(info.score_mate, Some(3));
}

#[test]
fn test_parse_info_negative_mate() {
    let info = parse_info_line("info depth 10 score mate -2").expect("should parse");
    assert_eq!(info.score_mate, Some(-2));
    assert_eq!(info.score_cp, None);
}

#[test]
fn test_parse_info_missing_optional_fields() {
    let info = parse_info_line("info depth 1").expect("should parse");
    assert_eq!(info.depth, Some(1));
    assert_eq!(info.score_cp, None);
    assert_eq!(info.nodes, None);
    assert!(info.pv.is_empty());
}

#[test]
fn test_parse_info_rejects_non_info() {
    assert!(parse_info_line("bestmove e2e4").is_none());
}

// -----------------------------------------------------------------------------
// Integration tests (require stockfish)
// -----------------------------------------------------------------------------

fn stockfish_available() -> bool {
    Command::new("stockfish")
        .stdin(Stdio::null())
        .stdout(Stdio::null())
        .stderr(Stdio::null())
        .spawn()
        .map(|mut c| {
            let _ = c.kill();
            let _ = c.wait();
            true
        })
        .unwrap_or(false)
}

macro_rules! skip_if_no_stockfish {
    () => {
        if !stockfish_available() {
            eprintln!("Skipping test: stockfish not found in PATH");
            return;
        }
    };
}

#[test]
fn test_uci_handshake() {
    skip_if_no_stockfish!();
    let engine = UciEngine::new("stockfish", &[]).expect("Failed to create UCI engine");
    assert!(engine.engine_name().is_some());
}

#[test]
fn test_engine_author_populated() {
    skip_if_no_stockfish!();
    let engine = UciEngine::new("stockfish", &[]).expect("Failed to create UCI engine");
    assert!(engine.engine_author().is_some());
}

#[test]
fn test_game_accessor() {
    skip_if_no_stockfish!();
    let engine = UciEngine::new("stockfish", &[]).expect("Failed to create UCI engine");
    // Initial game should be standard position
    assert_eq!(
        engine.game().clone().to_fen(),
        "rnbqkbnr/pppppppp/8/8/8/8/PPPPPPPP/RNBQKBNR w KQkq - 0 1"
    );
}

#[test]
fn test_set_option_and_is_ready() {
    skip_if_no_stockfish!();
    let mut engine = UciEngine::new("stockfish", &[]).expect("Failed to create UCI engine");
    engine.set_option("Hash", "32").expect("set_option failed");
    engine.is_ready().expect("is_ready failed after set_option");
}

#[test]
fn test_set_position_startpos_resets() {
    skip_if_no_stockfish!();
    let mut engine = UciEngine::new("stockfish", &[]).expect("Failed to create UCI engine");
    assert!(engine.make_move_lan("e2e4").expect("make_move_lan failed"));
    engine.set_position_startpos();
    // After reset, the game should be back to standard position
    assert_eq!(
        engine.game().clone().to_fen(),
        "rnbqkbnr/pppppppp/8/8/8/8/PPPPPPPP/RNBQKBNR w KQkq - 0 1"
    );
    // e2e4 should be valid again
    assert!(engine.make_move_lan("e2e4").expect("make_move_lan failed"));
}

#[test]
fn test_set_position_fen() {
    skip_if_no_stockfish!();
    let mut engine = UciEngine::new("stockfish", &[]).expect("Failed to create UCI engine");
    let fen = "rnbqkbnr/pp1ppppp/8/2p5/4P3/8/PPPP1PPP/RNBQKBNR w KQkq c6 0 2";
    engine
        .set_position_fen(fen)
        .expect("set_position_fen failed");
    let result = engine.go_depth(5).expect("go depth failed");
    assert!(!result.best_move_lan.is_empty());
}

#[test]
fn test_set_position_fen_invalid() {
    skip_if_no_stockfish!();
    let mut engine = UciEngine::new("stockfish", &[]).expect("Failed to create UCI engine");
    let result = engine.set_position_fen("not a valid fen");
    assert!(result.is_err());
}

#[test]
fn test_set_position_pgn_start_standard() {
    skip_if_no_stockfish!();
    let mut engine = UciEngine::new("stockfish", &[]).expect("Failed to create UCI engine");
    let pgn = include_str!("../../pgn/example/scholars_mate.pgn");
    let game = parse_pgn_single_game(pgn).expect("failed to parse PGN");
    engine
        .set_position_pgn_start(&game)
        .expect("set_position_pgn_start failed");
    assert_eq!(
        engine.game().clone().to_fen(),
        "rnbqkbnr/pppppppp/8/8/8/8/PPPPPPPP/RNBQKBNR w KQkq - 0 1"
    );
}

#[test]
fn test_set_position_pgn_start_fen() {
    skip_if_no_stockfish!();
    let mut engine = UciEngine::new("stockfish", &[]).expect("Failed to create UCI engine");
    let pgn = include_str!("../../pgn/example/fen_start.pgn");
    let game = parse_pgn_single_game(pgn).expect("failed to parse PGN");
    engine
        .set_position_pgn_start(&game)
        .expect("set_position_pgn_start failed");
    assert_eq!(
        engine.game().clone().to_fen(),
        "rnbqkbnr/pppppppp/8/8/4P3/8/PPPP1PPP/RNBQKBNR b KQkq - 0 1"
    );
}

#[test]
fn test_make_move_with_move_object() {
    skip_if_no_stockfish!();
    let mut engine = UciEngine::new("stockfish", &[]).expect("Failed to create UCI engine");
    let legal = engine.game().clone().legal_moves();
    let mv = legal.first().expect("should have legal moves");
    let ok = engine.make_move(mv).expect("make_move failed");
    assert!(ok);
    assert_eq!(engine.game().turn(), Color::Black);
}

#[test]
fn test_make_move_lan() {
    skip_if_no_stockfish!();
    let mut engine = UciEngine::new("stockfish", &[]).expect("Failed to create UCI engine");
    assert!(engine.make_move_lan("e2e4").expect("make_move_lan failed"));
    assert_eq!(engine.game().turn(), Color::Black);
}

#[test]
fn test_make_move_lan_invalid() {
    skip_if_no_stockfish!();
    let mut engine = UciEngine::new("stockfish", &[]).expect("Failed to create UCI engine");
    let result = engine.make_move_lan("z9z9");
    assert!(result.is_err());
}

#[test]
fn test_go_depth() {
    skip_if_no_stockfish!();
    let mut engine = UciEngine::new("stockfish", &[]).expect("Failed to create UCI engine");
    let result = engine.go_depth(5).expect("go depth failed");
    let legal = engine.game().clone().legal_moves();
    assert!(
        legal.iter().any(|m| m.to_lan() == result.best_move_lan),
        "bestmove {} not in legal moves",
        result.best_move_lan
    );
    assert!(!result.info.is_empty(), "should have info lines");
}

#[test]
fn test_go_movetime() {
    skip_if_no_stockfish!();
    let mut engine = UciEngine::new("stockfish", &[]).expect("Failed to create UCI engine");
    let result = engine.go_movetime(100).expect("go movetime failed");
    assert!(!result.best_move_lan.is_empty());
}

#[test]
fn test_go_clock() {
    skip_if_no_stockfish!();
    let mut engine = UciEngine::new("stockfish", &[]).expect("Failed to create UCI engine");
    let result = engine
        .go_clock(60000, 60000, 1000, 1000)
        .expect("go clock failed");
    assert!(!result.best_move_lan.is_empty());
    let legal = engine.game().clone().legal_moves();
    assert!(legal.iter().any(|m| m.to_lan() == result.best_move_lan));
}

#[test]
fn test_go_bestmove_depth_applies_move() {
    skip_if_no_stockfish!();
    let mut engine = UciEngine::new("stockfish", &[]).expect("Failed to create UCI engine");
    let mv = engine
        .go_bestmove_depth(5)
        .expect("go_bestmove_depth failed");
    assert_eq!(engine.game().turn(), Color::Black);
    assert_eq!(engine.move_history_lan.len(), 1);
    assert_eq!(engine.move_history_lan[0], mv.to_lan());
}

#[test]
fn test_go_bestmove_movetime_applies_move() {
    skip_if_no_stockfish!();
    let mut engine = UciEngine::new("stockfish", &[]).expect("Failed to create UCI engine");
    let mv = engine
        .go_bestmove_movetime(100)
        .expect("go_bestmove_movetime failed");
    assert_eq!(engine.game().turn(), Color::Black);
    assert_eq!(engine.move_history_lan.len(), 1);
    assert_eq!(engine.move_history_lan[0], mv.to_lan());
}

#[test]
fn test_bestmove_is_legal() {
    skip_if_no_stockfish!();
    let mut engine = UciEngine::new("stockfish", &[]).expect("Failed to create UCI engine");
    let result = engine.go_depth(8).expect("go depth failed");
    let legal = engine.game().clone().legal_moves();
    assert!(
        legal.contains(&result.best_move),
        "bestmove {} not found in legal_moves()",
        result.best_move_lan,
    );
}

#[test]
fn test_search_result_has_info() {
    skip_if_no_stockfish!();
    let mut engine = UciEngine::new("stockfish", &[]).expect("Failed to create UCI engine");
    let result = engine.go_depth(10).expect("go depth failed");
    assert!(!result.info.is_empty());
    let last = result.info.last().unwrap();
    assert!(last.depth.is_some());
    assert!(last.score_cp.is_some() || last.score_mate.is_some());
}

#[test]
fn test_search_result_ponder_move() {
    skip_if_no_stockfish!();
    let mut engine = UciEngine::new("stockfish", &[]).expect("Failed to create UCI engine");
    let result = engine.go_depth(10).expect("go depth failed");
    // At depth 10 from startpos, stockfish usually provides a ponder move
    if let Some(ref ponder_lan) = result.ponder_move_lan {
        assert!(!ponder_lan.is_empty());
        assert!(result.ponder_move.is_some());
    }
}

#[test]
fn test_mate_in_one() {
    skip_if_no_stockfish!();
    let mut engine = UciEngine::new("stockfish", &[]).expect("Failed to create UCI engine");
    engine
        .set_position_fen("r1bqkb1r/pppp1ppp/2n2n2/4p2Q/2B1P3/8/PPPP1PPP/RNB1K1NR w KQkq - 4 4")
        .expect("set_position_fen failed");
    let result = engine.go_depth(10).expect("go depth failed");
    assert_eq!(result.best_move_lan, "h5f7", "Expected Qxf7#");
}

#[test]
fn test_make_moves_then_search() {
    skip_if_no_stockfish!();
    let mut engine = UciEngine::new("stockfish", &[]).expect("Failed to create UCI engine");
    assert!(engine.make_move_lan("e2e4").expect("move failed"));
    assert!(engine.make_move_lan("e7e5").expect("move failed"));
    let result = engine.go_depth(5).expect("go depth failed");
    let legal = engine.game().clone().legal_moves();
    assert!(
        legal.iter().any(|m| m.to_lan() == result.best_move_lan),
        "bestmove {} not in legal moves",
        result.best_move_lan
    );
}

#[test]
fn test_quit_succeeds() {
    skip_if_no_stockfish!();
    let mut engine = UciEngine::new("stockfish", &[]).expect("Failed to create UCI engine");
    engine.quit().expect("quit failed");
}

// -----------------------------------------------------------------------------
// PGN replay tests – run stockfish at depth 4 on every position in each game
// -----------------------------------------------------------------------------

/// Collect all .pgn files from a directory (non-recursive).
fn collect_pgn_files(dir: &std::path::Path) -> Vec<std::path::PathBuf> {
    let mut files: Vec<std::path::PathBuf> = std::fs::read_dir(dir)
        .unwrap_or_else(|e| panic!("failed to read {}: {}", dir.display(), e))
        .filter_map(|e| e.ok())
        .map(|e| e.path())
        .filter(|p| p.extension().and_then(|s| s.to_str()) == Some("pgn"))
        .collect();
    files.sort();
    files
}

/// Replay every game in the given PGN files, running stockfish at depth 4 on
/// every position. Panics on any mismatch between stockfish and our move
/// generation.
fn replay_pgns_with_stockfish(files: &[std::path::PathBuf]) {
    let mut total_positions = 0usize;
    let mut total_games = 0usize;

    for path in files {
        let filename = path.file_name().unwrap().to_string_lossy();
        let content = std::fs::read_to_string(path)
            .unwrap_or_else(|e| panic!("failed to read {}: {}", filename, e));

        let games = match parse_pgn(&content) {
            Ok(g) => g,
            Err(e) => {
                eprintln!("Skipping {} (parse error: {})", filename, e);
                continue;
            }
        };

        for (gi, pgn_game) in games.iter().enumerate() {
            let mut engine = UciEngine::new("stockfish", &[]).expect("Failed to create UCI engine");

            // If the PGN has a FEN header, start from that position.
            if let Some(fen) = pgn_game.headers.get("FEN") {
                engine.set_position_fen(&fen).unwrap_or_else(|e| {
                    panic!("{}[game {}]: set_position_fen failed: {}", filename, gi, e)
                });
            }

            for (mi, mv) in pgn_game.moves.iter().enumerate() {
                // Ask stockfish to search the current position at depth 4.
                let result = engine.go_depth(4).unwrap_or_else(|e| {
                    panic!(
                        "{}[game {}] move {}: go_depth(4) failed: {}",
                        filename, gi, mi, e
                    )
                });

                // Verify stockfish's bestmove is among our legal moves.
                let legal = engine.game().clone().legal_moves();
                assert!(
                    legal.iter().any(|m| m.to_lan() == result.best_move_lan),
                    "{}[game {}] move {}: stockfish bestmove '{}' not in legal moves",
                    filename,
                    gi,
                    mi,
                    result.best_move_lan,
                );

                total_positions += 1;

                // Now play the actual game move.
                let lan = mv.to_lan();
                let ok = engine.make_move_lan(&lan).unwrap_or_else(|e| {
                    panic!(
                        "{}[game {}] move {}: make_move_lan('{}') failed: {}",
                        filename, gi, mi, lan, e
                    )
                });
                assert!(
                    ok,
                    "{}[game {}] move {}: '{}' was not a legal move",
                    filename, gi, mi, lan,
                );
            }

            total_games += 1;
            engine.quit().expect("quit failed");
        }

        eprintln!(
            "  {} done ({} games so far, {} positions)",
            filename, total_games, total_positions,
        );
    }

    eprintln!(
        "PGN replay complete: {} positions searched across {} games from {} files",
        total_positions,
        total_games,
        files.len()
    );
}

#[test]
fn test_stockfish_on_example_pgns() {
    skip_if_no_stockfish!();
    let pgn_dir = std::path::Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("pgn")
        .join("example");
    let files = collect_pgn_files(&pgn_dir);
    assert!(!files.is_empty(), "no .pgn files found in pgn/example/");
    replay_pgns_with_stockfish(&files);
}

/// Verify stockfish's bestmove is legal at every position in a single game.
/// Returns the number of positions checked.
fn replay_one_game(label: &str, pgn_game: &PgnGame) -> usize {
    let mut engine = UciEngine::new("stockfish", &[]).expect("Failed to create UCI engine");
    engine.set_option("Threads", "1").expect("set Threads");
    engine.set_option("Hash", "16").expect("set Hash");
    engine.is_ready().expect("is_ready after setoption");

    if let Some(fen) = pgn_game.headers.get("FEN") {
        engine
            .set_position_fen(fen)
            .unwrap_or_else(|e| panic!("{}: set_position_fen failed: {}", label, e));
    }

    for (mi, mv) in pgn_game.moves.iter().enumerate() {
        let result = engine
            .go_depth(4)
            .unwrap_or_else(|e| panic!("{} move {}: go_depth(4) failed: {}", label, mi, e));

        let legal = engine.game().clone().legal_moves();
        assert!(
            legal.iter().any(|m| m.to_lan() == result.best_move_lan),
            "{} move {}: stockfish bestmove '{}' not in legal moves",
            label,
            mi,
            result.best_move_lan,
        );

        let lan = mv.to_lan();
        let ok = engine.make_move_lan(&lan).unwrap_or_else(|e| {
            panic!(
                "{} move {}: make_move_lan('{}') failed: {}",
                label, mi, lan, e
            )
        });
        assert!(ok, "{} move {}: '{}' was not a legal move", label, mi, lan);
    }

    engine.quit().expect("quit failed");
    pgn_game.moves.len()
}

/// Replay every game from the lichess PGN corpus with stockfish at depth 4,
/// using all available cores via work-stealing.
///
/// This test is ignored by default because the lichess directory contains
/// millions of games. Run it explicitly with:
///
///     cargo test stockfish_on_lichess_pgns -- --ignored --nocapture
#[test]
#[ignore]
fn test_stockfish_on_lichess_pgns() {
    skip_if_no_stockfish!();

    let pgn_dir = std::path::Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("pgn")
        .join("lichess");
    if !pgn_dir.exists() {
        eprintln!("Skipping: pgn/lichess/ directory not found");
        return;
    }

    let files = collect_pgn_files(&pgn_dir);
    if files.is_empty() {
        eprintln!("Skipping: no .pgn files in pgn/lichess/");
        return;
    }

    let n_threads = std::thread::available_parallelism()
        .map(|n| n.get())
        .unwrap_or(4);

    // Queue holds (label, raw_pgn_text) — workers parse + replay.
    let queue: Arc<Mutex<Vec<(String, String)>>> = Arc::new(Mutex::new(Vec::new()));
    let done_loading = Arc::new(std::sync::atomic::AtomicBool::new(false));
    let total_games = Arc::new(AtomicUsize::new(0));
    let total_positions = Arc::new(AtomicUsize::new(0));
    let parse_errors = Arc::new(AtomicUsize::new(0));

    // Spawn worker threads that parse and replay games from the queue.
    let workers: Vec<_> = (0..n_threads)
        .map(|_| {
            let queue = Arc::clone(&queue);
            let done_loading = Arc::clone(&done_loading);
            let total_games = Arc::clone(&total_games);
            let total_positions = Arc::clone(&total_positions);
            let parse_errors = Arc::clone(&parse_errors);
            std::thread::spawn(move || {
                loop {
                    let item = queue.lock().expect("poisoned").pop();
                    match item {
                        Some((label, raw_pgn)) => {
                            let game = match parse_pgn_single_game(&raw_pgn) {
                                Ok(g) => g,
                                Err(e) => {
                                    eprintln!("Skipping {} (parse error: {})", label, e);
                                    parse_errors.fetch_add(1, Ordering::Relaxed);
                                    continue;
                                }
                            };
                            let positions = replay_one_game(&label, &game);
                            total_positions.fetch_add(positions, Ordering::Relaxed);
                            let g = total_games.fetch_add(1, Ordering::Relaxed) + 1;
                            if g % 100 == 0 {
                                eprintln!(
                                    "  {} games done, {} positions",
                                    g,
                                    total_positions.load(Ordering::Relaxed),
                                );
                            }
                        }
                        None if done_loading.load(Ordering::Relaxed) => break,
                        None => std::thread::yield_now(),
                    }
                }
            })
        })
        .collect();

    // Producer: read PGN files, split on blank lines into individual game texts.
    for path in &files {
        let filename = path.file_name().unwrap().to_string_lossy().to_string();
        let content = std::fs::read_to_string(path)
            .unwrap_or_else(|e| panic!("failed to read {}: {}", filename, e));

        // Split into individual games. In PGN, a blank line separates
        // headers from movetext, and another blank line separates games.
        // A new game begins when a '[' appears after a blank line following
        // movetext (i.e. a non-header line).
        let raw_games: Vec<String> = {
            let mut games = Vec::new();
            let mut current = String::new();
            let mut in_movetext = false;

            for line in content.lines() {
                let trimmed = line.trim();
                if trimmed.is_empty() {
                    if in_movetext {
                        // Blank line after movetext = end of game
                        if !current.is_empty() {
                            games.push(std::mem::take(&mut current));
                        }
                        in_movetext = false;
                        continue;
                    }
                    // Blank line between headers and movetext
                    current.push('\n');
                    continue;
                }
                if trimmed.starts_with('[') && trimmed.ends_with(']') {
                    in_movetext = false;
                } else {
                    in_movetext = true;
                }
                current.push_str(line);
                current.push('\n');
            }
            if !current.trim().is_empty() {
                games.push(current);
            }
            games
        };

        let raw_games_count = raw_games.len();
        for (gi, raw_text) in raw_games.into_iter().enumerate() {
            let label = format!("{}[game {}]", filename, gi);
            loop {
                let mut q = queue.lock().expect("poisoned");
                if q.len() < n_threads * 64 {
                    q.push((label, raw_text));
                    break;
                }
                drop(q);
                std::thread::yield_now();
            }
        }

        eprintln!("  loaded {} ({} games)", filename, raw_games_count);
    }

    done_loading.store(true, Ordering::Relaxed);

    for w in workers {
        w.join().expect("worker thread panicked");
    }

    eprintln!(
        "Lichess replay complete: {} positions across {} games from {} files ({} threads, {} parse errors)",
        total_positions.load(Ordering::Relaxed),
        total_games.load(Ordering::Relaxed),
        files.len(),
        n_threads,
        parse_errors.load(Ordering::Relaxed),
    );
}
