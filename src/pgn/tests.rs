use super::*;

macro_rules! pgn {
    ($file:literal) => {
        include_str!(concat!("../../pgn/example/", $file))
    };
}

#[test]
fn test_parse_games_pgn() {
    let pgn = pgn!("games.pgn");
    let games = parse_pgn(pgn).expect("test_parse_games_pgn: failed to parse PGN file");
    assert_eq!(games.len(), 56);
    for game in &games {
        assert!(!game.moves.is_empty());
        assert_ne!(game.result, PgnResult::Unknown);
    }
}

#[test]
fn test_scholars_mate() {
    let pgn = pgn!("scholars_mate.pgn");
    let mut game =
        parse_pgn_single_game(pgn).expect("test_scholars_mate: failed to parse scholars mate PGN");
    assert_eq!(game.headers.white(), Some("Player1"));
    assert_eq!(game.headers.black(), Some("Player2"));
    assert_eq!(game.headers.event(), Some("Test"));
    assert_eq!(game.result, PgnResult::WhiteWin);
    assert_eq!(game.moves.len(), 7);
    assert!(game.final_game.is_checkmate());
}

#[test]
fn test_multi_game() {
    let pgn = pgn!("multi_game.pgn");
    let mut games = parse_pgn(pgn).expect("test_multi_game: failed to parse multi-game PGN");
    assert_eq!(games.len(), 2);

    assert_eq!(games[0].headers.white(), Some("A"));
    assert_eq!(games[0].result, PgnResult::WhiteWin);
    assert_eq!(games[0].moves.len(), 7);

    assert_eq!(games[1].headers.white(), Some("C"));
    assert_eq!(games[1].result, PgnResult::BlackWin);
    assert_eq!(games[1].moves.len(), 4);
    assert!(games[1].final_game.is_checkmate());
}

#[test]
fn test_fen_start_position() {
    let pgn = pgn!("fen_start.pgn");
    let game =
        parse_pgn_single_game(pgn).expect("test_fen_start_position: failed to parse FEN start PGN");
    assert_eq!(game.moves.len(), 1);
    assert_eq!(game.result, PgnResult::Unknown);
}

#[test]
fn test_castling_uppercase_o() {
    let pgn = pgn!("castling_uppercase.pgn");
    let game = parse_pgn_single_game(pgn)
        .expect("test_castling_uppercase_o: failed to parse castling uppercase PGN");
    assert_eq!(game.moves.len(), 8);
}

#[test]
fn test_castling_zero() {
    let pgn = pgn!("castling_zero.pgn");
    let game =
        parse_pgn_single_game(pgn).expect("test_castling_zero: failed to parse castling zero PGN");
    assert_eq!(game.moves.len(), 8);
}

#[test]
fn test_promotion_with_equals() {
    let pgn = pgn!("promotion_equals.pgn");
    let game = parse_pgn_single_game(pgn)
        .expect("test_promotion_with_equals: failed to parse promotion equals PGN");
    assert_eq!(game.moves.len(), 1);
}

#[test]
fn test_promotion_without_equals() {
    let pgn = pgn!("promotion_no_equals.pgn");
    let game = parse_pgn_single_game(pgn)
        .expect("test_promotion_without_equals: failed to parse promotion no-equals PGN");
    assert_eq!(game.moves.len(), 1);
}

#[test]
fn test_comments_and_annotations_skipped() {
    let pgn = pgn!("annotated.pgn");
    let mut game = parse_pgn_single_game(pgn)
        .expect("test_comments_and_annotations_skipped: failed to parse annotated PGN");
    assert_eq!(game.moves.len(), 7);
    assert!(game.final_game.is_checkmate());
}

#[test]
fn test_invalid_move() {
    let pgn = pgn!("invalid_move.pgn");
    match parse_pgn_single_game(pgn) {
        Err(PgnError::InvalidMove { san, .. }) => {
            assert_eq!(san, "Qh8");
        }
        Err(other) => panic!("Expected InvalidMove, got: {:?}", other),
        Ok(_) => panic!("Expected error, got Ok"),
    }
}

#[test]
fn test_draw_result() {
    let pgn = pgn!("draw.pgn");
    let game = parse_pgn_single_game(pgn).expect("test_draw_result: failed to parse draw PGN");
    assert_eq!(game.result, PgnResult::Draw);
}

#[test]
fn test_empty_pgn() {
    let games = parse_pgn("").expect("test_empty_pgn: failed to parse empty PGN string");
    assert!(games.is_empty());
}

#[test]
fn test_headers_case_insensitive() {
    let headers = PgnHeaders {
        pairs: vec![("White".to_string(), "Kasparov".to_string())],
    };
    assert_eq!(headers.get("white"), Some("Kasparov"));
    assert_eq!(headers.get("WHITE"), Some("Kasparov"));
}
