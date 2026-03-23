use super::*;

macro_rules! tournament_pgn {
    ($file:literal) => {
        include_str!(concat!("../../pgn/example/", $file))
    };
}

macro_rules! annotated_pgn {
    ($file:literal) => {
        include_str!(concat!("../../pgn/annotated_games/", $file))
    };
}

// ---------------------------------------------------------------------------
// Tournament Games
// ---------------------------------------------------------------------------

#[test]
fn test_parse_games_pgn() {
    let pgn = tournament_pgn!("games.pgn");
    let games = parse_pgn(pgn).expect("test_parse_games_pgn: failed to parse PGN file");
    assert_eq!(games.len(), 56);
    for game in &games {
        assert!(!game.moves.is_empty());
        assert_ne!(game.result, PgnResult::Unknown);
    }
}

#[test]
fn test_scholars_mate() {
    let pgn = tournament_pgn!("scholars_mate.pgn");
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
    let pgn = tournament_pgn!("multi_game.pgn");
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
    let pgn = tournament_pgn!("fen_start.pgn");
    let game =
        parse_pgn_single_game(pgn).expect("test_fen_start_position: failed to parse FEN start PGN");
    assert_eq!(game.moves.len(), 1);
    assert_eq!(game.result, PgnResult::Unknown);
    assert_eq!(
        game.starting_fen(),
        Some("rnbqkbnr/pppppppp/8/8/4P3/8/PPPP1PPP/RNBQKBNR b KQkq e3 0 1")
    );
    assert_eq!(
        game.starting_game()
            .expect("test_fen_start_position: failed to build start game")
            .to_fen(),
        "rnbqkbnr/pppppppp/8/8/4P3/8/PPPP1PPP/RNBQKBNR b KQkq - 0 1"
    );
}

#[test]
fn test_castling_uppercase_o() {
    let pgn = tournament_pgn!("castling_uppercase.pgn");
    let game = parse_pgn_single_game(pgn)
        .expect("test_castling_uppercase_o: failed to parse castling uppercase PGN");
    assert_eq!(game.moves.len(), 8);
}

#[test]
fn test_castling_zero() {
    let pgn = tournament_pgn!("castling_zero.pgn");
    let game =
        parse_pgn_single_game(pgn).expect("test_castling_zero: failed to parse castling zero PGN");
    assert_eq!(game.moves.len(), 8);
}

#[test]
fn test_promotion_with_equals() {
    let pgn = tournament_pgn!("promotion_equals.pgn");
    let game = parse_pgn_single_game(pgn)
        .expect("test_promotion_with_equals: failed to parse promotion equals PGN");
    assert_eq!(game.moves.len(), 1);
}

#[test]
fn test_promotion_without_equals() {
    let pgn = tournament_pgn!("promotion_no_equals.pgn");
    let game = parse_pgn_single_game(pgn)
        .expect("test_promotion_without_equals: failed to parse promotion no-equals PGN");
    assert_eq!(game.moves.len(), 1);
}

#[test]
fn test_comments_and_annotations_skipped() {
    let pgn = tournament_pgn!("annotated.pgn");
    let mut game = parse_pgn_single_game(pgn)
        .expect("test_comments_and_annotations_skipped: failed to parse annotated PGN");
    assert_eq!(game.moves.len(), 7);
    assert!(game.final_game.is_checkmate());
}

#[test]
fn test_invalid_move() {
    let pgn = tournament_pgn!("invalid_move.pgn");
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
    let pgn = tournament_pgn!("draw.pgn");
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

#[test]
fn test_standard_start_helpers() {
    let pgn = tournament_pgn!("scholars_mate.pgn");
    let game =
        parse_pgn_single_game(pgn).expect("test_standard_start_helpers: failed to parse PGN");
    assert_eq!(game.starting_fen(), None);
    assert_eq!(
        game.starting_game()
            .expect("test_standard_start_helpers: failed to build start game")
            .to_fen(),
        "rnbqkbnr/pppppppp/8/8/8/8/PPPPPPPP/RNBQKBNR w KQkq - 0 1"
    );
}

// ---------------------------------------------------------------------------
// Annotated Games
// ---------------------------------------------------------------------------

/// Helper: parse a PGN file via the iterator, count how many games parse
/// successfully and how many fail, and return (ok, err, total_moves).
fn parse_annotated_file(pgn: &str) -> (usize, usize, usize) {
    let iter = PgnIter::new(pgn.to_string()).expect("failed to create PGN iterator");
    let mut ok = 0;
    let mut err = 0;
    let mut total_moves = 0;
    for result in iter {
        match result {
            Ok(game) => {
                ok += 1;
                total_moves += game.moves.len();
            }
            Err(_) => {
                err += 1;
            }
        }
    }
    (ok, err, total_moves)
}

macro_rules! annotated_game_test {
    ($name:ident, $file:literal, xfail) => {
        #[test]
        #[should_panic(expected = "failed to parse")]
        fn $name() {
            let pgn = annotated_pgn!($file);
            let (ok, err, total_moves) = parse_annotated_file(pgn);
            assert_eq!(
                err, 0,
                "{}: {} games failed to parse ({} succeeded)",
                $file, err, ok
            );
            assert!(
                total_moves > 0,
                "{}: expected at least some moves across all games",
                $file
            );
        }
    };
    ($name:ident, $file:literal) => {
        #[test]
        fn $name() {
            let pgn = annotated_pgn!($file);
            let (ok, err, total_moves) = parse_annotated_file(pgn);
            assert_eq!(
                err, 0,
                "{}: {} games failed to parse ({} succeeded)",
                $file, err, ok
            );
            assert!(
                total_moves > 0,
                "{}: expected at least some moves across all games",
                $file
            );
        }
    };
}

annotated_game_test!(test_annotated_gm_games, "GM_games.pgn", xfail);
annotated_game_test!(test_annotated_setone, "annotatedsetone.pgn");
annotated_game_test!(test_annotated_settwo, "annotatedsettwo.pgn");
annotated_game_test!(test_annotated_bali02, "bali02.pgn");
annotated_game_test!(
    test_annotated_electronic_campfire,
    "electronic_campfire.pgn"
);
annotated_game_test!(test_annotated_great_masters, "great_masters.pgn");
annotated_game_test!(test_annotated_hartwig, "hartwig.pgn");
annotated_game_test!(test_annotated_hayes, "hayes.pgn");
annotated_game_test!(test_annotated_immortal_games, "immortal_games.pgn", xfail);
annotated_game_test!(test_annotated_kk, "kk.pgn");
annotated_game_test!(test_annotated_kramnik, "kramnik.pgn");
annotated_game_test!(test_annotated_linares_2001, "linares_2001.pgn");
annotated_game_test!(test_annotated_linares_2002, "linares_2002.pgn");
annotated_game_test!(test_annotated_moscow64, "moscow64.pgn");
annotated_game_test!(test_annotated_perle, "perle.pgn");
annotated_game_test!(test_annotated_polgar, "polgar.pgn");
annotated_game_test!(test_annotated_pon_korch, "pon_korch.pgn");
annotated_game_test!(test_annotated_russian_chess, "russian_chess.pgn");
annotated_game_test!(test_annotated_scca, "scca.pgn");
annotated_game_test!(test_annotated_schiller, "schiller.pgn");
annotated_game_test!(test_annotated_semicomm, "semicomm.pgn");
annotated_game_test!(test_annotated_top_games, "top_games.pgn");
annotated_game_test!(test_annotated_vc_2001, "vc_2001.pgn");
