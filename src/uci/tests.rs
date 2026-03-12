use super::*;
use crate::color::Color;
use protocol::*;
use std::process::{Command, Stdio};

// ========================
// Protocol parser tests (no engine needed)
// ========================

#[test]
fn parse_bestmove_with_ponder() {
    assert_eq!(
        parse_bestmove_line("bestmove e2e4 ponder e7e5"),
        Some(("e2e4".to_string(), Some("e7e5".to_string())))
    );
}

#[test]
fn parse_bestmove_without_ponder() {
    assert_eq!(
        parse_bestmove_line("bestmove e2e4"),
        Some(("e2e4".to_string(), None))
    );
}

#[test]
fn parse_bestmove_only_keyword() {
    assert_eq!(parse_bestmove_line("bestmove"), None);
}

#[test]
fn parse_bestmove_rejects_non_bestmove() {
    assert_eq!(parse_bestmove_line("info depth 10"), None);
}

#[test]
fn parse_info_all_fields() {
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
fn parse_info_mate_score() {
    let info = parse_info_line("info depth 15 score mate 3 pv h5f7").expect("should parse");
    assert_eq!(info.score_cp, None);
    assert_eq!(info.score_mate, Some(3));
}

#[test]
fn parse_info_negative_mate() {
    let info = parse_info_line("info depth 10 score mate -2").expect("should parse");
    assert_eq!(info.score_mate, Some(-2));
    assert_eq!(info.score_cp, None);
}

#[test]
fn parse_info_missing_optional_fields() {
    let info = parse_info_line("info depth 1").expect("should parse");
    assert_eq!(info.depth, Some(1));
    assert_eq!(info.score_cp, None);
    assert_eq!(info.nodes, None);
    assert!(info.pv.is_empty());
}

#[test]
fn parse_info_rejects_non_info() {
    assert!(parse_info_line("bestmove e2e4").is_none());
}

// ========================
// Integration tests (require stockfish)
// ========================

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
fn uci_handshake() {
    skip_if_no_stockfish!();
    let engine = UciEngine::new("stockfish", &[]).expect("Failed to create UCI engine");
    assert!(engine.engine_name().is_some());
}

#[test]
fn engine_author_populated() {
    skip_if_no_stockfish!();
    let engine = UciEngine::new("stockfish", &[]).expect("Failed to create UCI engine");
    assert!(engine.engine_author().is_some());
}

#[test]
fn game_accessor() {
    skip_if_no_stockfish!();
    let engine = UciEngine::new("stockfish", &[]).expect("Failed to create UCI engine");
    // Initial game should be standard position
    assert_eq!(
        engine.game().clone().to_fen(),
        "rnbqkbnr/pppppppp/8/8/8/8/PPPPPPPP/RNBQKBNR w KQkq - 0 1"
    );
}

#[test]
fn set_option_and_is_ready() {
    skip_if_no_stockfish!();
    let mut engine = UciEngine::new("stockfish", &[]).expect("Failed to create UCI engine");
    engine.set_option("Hash", "32").expect("set_option failed");
    engine.is_ready().expect("is_ready failed after set_option");
}

#[test]
fn set_position_startpos_resets() {
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
fn set_position_fen() {
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
fn set_position_fen_invalid() {
    skip_if_no_stockfish!();
    let mut engine = UciEngine::new("stockfish", &[]).expect("Failed to create UCI engine");
    let result = engine.set_position_fen("not a valid fen");
    assert!(result.is_err());
}

#[test]
fn make_move_with_move_object() {
    skip_if_no_stockfish!();
    let mut engine = UciEngine::new("stockfish", &[]).expect("Failed to create UCI engine");
    let legal = engine.game().clone().legal_moves();
    let mv = legal.first().expect("should have legal moves");
    let ok = engine.make_move(mv).expect("make_move failed");
    assert!(ok);
    assert_eq!(engine.game().turn(), Color::Black);
}

#[test]
fn make_move_lan() {
    skip_if_no_stockfish!();
    let mut engine = UciEngine::new("stockfish", &[]).expect("Failed to create UCI engine");
    assert!(engine.make_move_lan("e2e4").expect("make_move_lan failed"));
    assert_eq!(engine.game().turn(), Color::Black);
}

#[test]
fn make_move_lan_invalid() {
    skip_if_no_stockfish!();
    let mut engine = UciEngine::new("stockfish", &[]).expect("Failed to create UCI engine");
    let result = engine.make_move_lan("z9z9");
    assert!(result.is_err());
}

#[test]
fn go_depth() {
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
fn go_movetime() {
    skip_if_no_stockfish!();
    let mut engine = UciEngine::new("stockfish", &[]).expect("Failed to create UCI engine");
    let result = engine.go_movetime(100).expect("go movetime failed");
    assert!(!result.best_move_lan.is_empty());
}

#[test]
fn go_clock() {
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
fn go_bestmove_depth_applies_move() {
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
fn go_bestmove_movetime_applies_move() {
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
fn bestmove_is_legal() {
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
fn search_result_has_info() {
    skip_if_no_stockfish!();
    let mut engine = UciEngine::new("stockfish", &[]).expect("Failed to create UCI engine");
    let result = engine.go_depth(10).expect("go depth failed");
    assert!(!result.info.is_empty());
    let last = result.info.last().unwrap();
    assert!(last.depth.is_some());
    assert!(last.score_cp.is_some() || last.score_mate.is_some());
}

#[test]
fn search_result_ponder_move() {
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
fn mate_in_one() {
    skip_if_no_stockfish!();
    let mut engine = UciEngine::new("stockfish", &[]).expect("Failed to create UCI engine");
    engine
        .set_position_fen("r1bqkb1r/pppp1ppp/2n2n2/4p2Q/2B1P3/8/PPPP1PPP/RNB1K1NR w KQkq - 4 4")
        .expect("set_position_fen failed");
    let result = engine.go_depth(10).expect("go depth failed");
    assert_eq!(result.best_move_lan, "h5f7", "Expected Qxf7#");
}

#[test]
fn make_moves_then_search() {
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
fn quit_succeeds() {
    skip_if_no_stockfish!();
    let mut engine = UciEngine::new("stockfish", &[]).expect("Failed to create UCI engine");
    engine.quit().expect("quit failed");
}
