use super::*;
use crate::color::Color;
use crate::r#move::MoveFlags;
use crate::outcome::GameOutcome;
use crate::pieces::{Piece, PieceType};
use crate::position::Position;
use rstest::rstest;

macro_rules! board_size_tests {
    (
        $W:literal, $H:literal,
        start_fen: $start_fen:expr,
        castling_fen: $castling_fen:expr,
        castling_blocked_fen: $castling_blocked_fen:expr,
        king_col: $king_col:expr,
        ep_white_fen: $ep_white_fen:expr,
        ep_white_src: $ep_white_src:expr,
        ep_white_dst: $ep_white_dst:expr,
        ep_white_captured: $ep_white_captured:expr,
        ep_black_fen: $ep_black_fen:expr,
        ep_black_src: $ep_black_src:expr,
        ep_black_dst: $ep_black_dst:expr,
        ep_black_captured: $ep_black_captured:expr,
        ep_unmake_fen: $ep_unmake_fen:expr,
        ep_double_push_fen: $ep_double_push_fen:expr,
        ep_double_push_lan: $ep_double_push_lan:expr,
        ep_double_push_target: $ep_double_push_target:expr,
        ep_roundtrip_fen: $ep_roundtrip_fen:expr
    ) => {
        paste::paste! {
            mod [<tests_ $W x $H>] {
                use super::*;

                type G = Game<$W, $H>;

                // -------------------------------------------------------------
                // Creation & piece placement
                // -------------------------------------------------------------

                #[test]
                fn game_creation() {
                    let mut game = G::new($start_fen, true).expect("game_creation: failed to create game from start FEN");
                    assert_eq!(game.width(), $W);
                    assert_eq!(game.height(), $H);
                    assert_eq!(
                        game.get_piece(&game.white_king_pos).expect("game_creation: white king must exist at tracked position").piece_type,
                        PieceType::King,
                    );
                    assert_eq!(
                        game.get_piece(&game.black_king_pos).expect("game_creation: black king must exist at tracked position").piece_type,
                        PieceType::King,
                    );
                    let fen = game.to_fen();
                    assert!(!fen.is_empty());
                }

                #[test]
                fn piece_placement() {
                    let game = G::new($start_fen, true).expect("piece_placement: failed to create game from start FEN");
                    let white_pieces = game.pieces(Color::White);
                    let black_pieces = game.pieces(Color::Black);

                    assert!(!white_pieces.is_empty());
                    assert!(!black_pieces.is_empty());

                    let white_kings = white_pieces.iter()
                        .filter(|(_, p)| p.piece_type == PieceType::King).count();
                    let black_kings = black_pieces.iter()
                        .filter(|(_, p)| p.piece_type == PieceType::King).count();
                    assert_eq!(white_kings, 1);
                    assert_eq!(black_kings, 1);
                }

                // -------------------------------------------------------------
                // Attack Patterns
                // -------------------------------------------------------------

                #[test]
                fn rook_attack_patterns() {
                    let mut game = G::new($start_fen, false).expect("rook_attack_patterns: failed to create game from start FEN");
                    game.clear_board();
                    game.set_piece(
                        &Position::new(0, 0),
                        Some(Piece::new(PieceType::King, Color::White)),
                    );
                    game.set_piece(
                        &Position::new($W - 1, $H - 1),
                        Some(Piece::new(PieceType::King, Color::Black)),
                    );
                    game.white_king_pos = Position::new(0, 0);
                    game.black_king_pos = Position::new($W - 1, $H - 1);

                    // Rook at (2, 2) — valid on any board >= 5x5
                    let rook_pos = Position::new(2, 2);
                    game.set_piece(&rook_pos, Some(Piece::new(PieceType::Rook, Color::White)));

                    // Attacks along file and rank
                    assert!(game.is_square_attacked(&Position::new(2, 0), Color::White));
                    assert!(game.is_square_attacked(&Position::new(2, $H - 1), Color::White));
                    assert!(game.is_square_attacked(&Position::new(0, 2), Color::White));
                    assert!(game.is_square_attacked(&Position::new($W - 1, 2), Color::White));

                    // Does not attack diagonally
                    assert!(!game.is_square_attacked(&Position::new(3, 3), Color::White));

                    // Blocked path
                    game.set_piece(
                        &Position::new(2, 4),
                        Some(Piece::new(PieceType::Pawn, Color::Black)),
                    );
                    assert!(game.is_square_attacked(&Position::new(2, 4), Color::White));
                    assert!(!game.is_square_attacked(&Position::new(2, $H - 1), Color::White));
                }

                #[test]
                fn bishop_attack_patterns() {
                    let mut game = G::new($start_fen, false).expect("bishop_attack_patterns: failed to create game from start FEN");
                    game.clear_board();
                    // Kings placed off the diagonals from (2,2)
                    game.set_piece(
                        &Position::new(0, 1),
                        Some(Piece::new(PieceType::King, Color::White)),
                    );
                    game.set_piece(
                        &Position::new($W - 1, 1),
                        Some(Piece::new(PieceType::King, Color::Black)),
                    );
                    game.white_king_pos = Position::new(0, 1);
                    game.black_king_pos = Position::new($W - 1, 1);

                    // Bishop at (2, 2)
                    game.set_piece(
                        &Position::new(2, 2),
                        Some(Piece::new(PieceType::Bishop, Color::White)),
                    );

                    // Attacks diagonals
                    assert!(game.is_square_attacked(&Position::new(0, 0), Color::White));
                    assert!(game.is_square_attacked(&Position::new(4, 4), Color::White));
                    assert!(game.is_square_attacked(&Position::new(0, 4), Color::White));
                    assert!(game.is_square_attacked(&Position::new(4, 0), Color::White));

                    // Does not attack along file/rank
                    assert!(!game.is_square_attacked(&Position::new(2, 0), Color::White));
                    assert!(!game.is_square_attacked(&Position::new(4, 2), Color::White));

                    // Blocked path
                    game.set_piece(
                        &Position::new(3, 3),
                        Some(Piece::new(PieceType::Pawn, Color::Black)),
                    );
                    assert!(game.is_square_attacked(&Position::new(3, 3), Color::White));
                    assert!(!game.is_square_attacked(&Position::new(4, 4), Color::White));
                }

                // -------------------------------------------------------------
                // Insufficient Material & 50 Move rule
                // -------------------------------------------------------------

                #[rstest]
                #[case("k_vs_k", None)]
                #[case("kb_vs_k", Some((PieceType::Bishop, Color::White, Position::new(2, 2))))]
                fn insufficient_material(
                    #[case] _name: &str,
                    #[case] extra_piece: Option<(PieceType, Color, Position)>,
                ) {
                    let mut game = G::new($start_fen, true).expect("insufficient_material: failed to create game from start FEN");
                    game.clear_board();

                    game.set_piece(
                        &Position::new(0, 0),
                        Some(Piece::new(PieceType::King, Color::White)),
                    );
                    game.white_king_pos = Position::new(0, 0);
                    game.set_piece(
                        &Position::new($W - 1, $H - 1),
                        Some(Piece::new(PieceType::King, Color::Black)),
                    );
                    game.black_king_pos = Position::new($W - 1, $H - 1);

                    if let Some((pt, color, pos)) = extra_piece {
                        game.set_piece(&pos, Some(Piece::new(pt, color)));
                    }

                    assert!(game.is_insufficient_material());
                    assert!(game.is_over());
                    assert_eq!(game.outcome(), Some(GameOutcome::InsufficientMaterial));
                }

                #[test]
                fn fifty_move_rule() {
                    let mut game = G::new($start_fen, true).expect("fifty_move_rule: failed to create game from start FEN");
                    game.clear_board();

                    game.set_piece(
                        &Position::new(0, 0),
                        Some(Piece::new(PieceType::King, Color::White)),
                    );
                    game.white_king_pos = Position::new(0, 0);
                    game.set_piece(
                        &Position::new(1, 0),
                        Some(Piece::new(PieceType::Rook, Color::White)),
                    );
                    game.set_piece(
                        &Position::new($W - 1, $H - 1),
                        Some(Piece::new(PieceType::King, Color::Black)),
                    );
                    game.black_king_pos = Position::new($W - 1, $H - 1);

                    game.halfmove_clock = 150;
                    assert!(game.is_over());
                    assert_eq!(game.outcome(), Some(GameOutcome::FiftyMoveRule));
                }

                // -------------------------------------------------------------
                // Castling
                // -------------------------------------------------------------

                #[rstest]
                #[case("kingside", $king_col + 2, $king_col + 1)]
                #[case("queenside", $king_col - 2, $king_col - 1)]
                fn castling(
                    #[case] name: &str,
                    #[case] king_dst_col: usize,
                    #[case] rook_dst_col: usize,
                ) {
                    let mut game = G::new($castling_fen, true).expect("castling: failed to create game from castling FEN");

                    let legal = game.legal_moves();
                    let castle = legal.iter().find(|m| {
                        m.src == Position::new($king_col, 0)
                            && m.dst == Position::from_usize(king_dst_col, 0)
                            && m.flags.contains(MoveFlags::CASTLE)
                    });
                    assert!(
                        castle.is_some(),
                        "Should be able to castle {name} on {}x{} board", $W, $H,
                    );

                    let mv = castle.expect("castling: castle move must exist after is_some assertion").clone();
                    assert!(game.make_move(&mv));

                    assert_eq!(
                        game.get_piece(&Position::from_usize(king_dst_col, 0)).expect("castling: king must exist at destination after castling").piece_type,
                        PieceType::King,
                    );
                    assert_eq!(
                        game.get_piece(&Position::from_usize(rook_dst_col, 0)).expect("castling: rook must exist at destination after castling").piece_type,
                        PieceType::Rook,
                    );
                    assert!(game.get_piece(&Position::new($king_col, 0)).is_none());
                }

                #[test]
                fn castling_unmake() {
                    let mut game = G::new($castling_fen, true).expect("castling_unmake: failed to create game from castling FEN");
                    let original_fen = game.to_fen();

                    let legal = game.legal_moves();
                    let castle = legal.iter().find(|m| {
                        m.src == Position::new($king_col, 0)
                            && m.flags.contains(MoveFlags::CASTLE)
                            && m.dst.col > m.src.col
                    }).expect("castling_unmake: kingside castle move must exist").clone();
                    game.make_move(&castle);
                    game.unmake_move();

                    assert_eq!(game.to_fen(), original_fen);
                }

                #[test]
                fn castling_blocked() {
                    let mut game = G::new($castling_blocked_fen, true).expect("castling_blocked: failed to create game from blocked castling FEN");

                    let legal = game.legal_moves();
                    let castle_ks = legal.iter().find(|m| {
                        m.src == Position::new($king_col, 0)
                            && m.flags.contains(MoveFlags::CASTLE)
                            && m.dst.col > m.src.col
                    });
                    assert!(castle_ks.is_none(), "Kingside castle should be blocked");
                }

                // -------------------------------------------------------------
                // En-passant
                // -------------------------------------------------------------

                #[rstest]
                #[case("white",
                    $ep_white_fen,
                    $ep_white_src, $ep_white_dst, $ep_white_captured)]
                #[case("black",
                    $ep_black_fen,
                    $ep_black_src, $ep_black_dst, $ep_black_captured)]
                fn en_passant(
                    #[case] name: &str,
                    #[case] fen: &str,
                    #[case] expected_src: Position,
                    #[case] expected_dst: Position,
                    #[case] captured_pos: Position,
                ) {
                    let mut game = G::new(fen, true).expect("en_passant: failed to create game from en passant FEN");

                    let legal = game.legal_moves();
                    let ep = legal.iter().find(|m| m.flags.contains(MoveFlags::EN_PASSANT));
                    assert!(
                        ep.is_some(),
                        "{name} should be able to capture en passant on {}x{}", $W, $H,
                    );

                    let ep_mv = ep.expect("en_passant: en passant move must exist after is_some assertion").clone();
                    assert_eq!(ep_mv.src, expected_src);
                    assert_eq!(ep_mv.dst, expected_dst);

                    game.make_move(&ep_mv);

                    assert_eq!(
                        game.get_piece(&expected_dst).expect("en_passant: pawn must exist at en passant destination").piece_type,
                        PieceType::Pawn,
                    );
                    assert!(game.get_piece(&captured_pos).is_none());
                    assert!(game.get_piece(&expected_src).is_none());
                }

                #[test]
                fn en_passant_unmake() {
                    let mut game = G::new($ep_unmake_fen, true).expect("en_passant_unmake: failed to create game from FEN");
                    let original_fen = game.to_fen();

                    let legal = game.legal_moves();
                    let ep = legal.iter()
                        .find(|m| m.flags.contains(MoveFlags::EN_PASSANT))
                        .expect("en_passant_unmake: en passant move must exist").clone();
                    game.make_move(&ep);
                    game.unmake_move();

                    assert_eq!(game.to_fen(), original_fen);
                }

                #[test]
                fn en_passant_created_by_double_push() {
                    let mut game = G::new($ep_double_push_fen, true).expect("en_passant_created_by_double_push: failed to create game from FEN");

                    let mv = game.move_from_lan($ep_double_push_lan).expect("en_passant_created_by_double_push: failed to parse double push LAN move");
                    assert!(mv.flags.contains(MoveFlags::DOUBLE_PUSH));
                    game.make_move(&mv);

                    assert_eq!(
                        game.en_passant_square(),
                        Some($ep_double_push_target),
                    );
                }

                #[test]
                fn en_passant_fen_roundtrip() {
                    let mut game = G::new($ep_roundtrip_fen, true).expect("en_passant_fen_roundtrip: failed to create game from FEN");
                    assert_eq!(game.to_fen(), $ep_roundtrip_fen);
                }
            }
        }
    };
}

// -----------------------------------------------------------------------------
// 6x6
// -----------------------------------------------------------------------------

board_size_tests!(
    6, 6,
    start_fen: "rnbkqr/pppppp/6/6/PPPPPP/RNBKQR w - - 0 1",
    castling_fen: "5k/6/6/6/6/R1K2R w KQ - 0 1",
    castling_blocked_fen: "5k/6/6/6/6/R1KN1R w KQ - 0 1",
    king_col: 2,
    ep_white_fen: "r1k2r/6/6/2Pp2/6/R1K2R w KQkq d4 0 1",
    ep_white_src: Position::new(2, 2),
    ep_white_dst: Position::new(3, 3),
    ep_white_captured: Position::new(3, 2),
    ep_black_fen: "r1k2r/6/3pP1/6/6/R1K2R b KQkq e3 0 1",
    ep_black_src: Position::new(3, 3),
    ep_black_dst: Position::new(4, 2),
    ep_black_captured: Position::new(4, 3),
    ep_unmake_fen: "r1k2r/6/6/2Pp2/6/R1K2R w KQkq d4 0 1",
    ep_double_push_fen: "r1k2r/6/6/6/1P4/R1K2R w KQkq - 0 1",
    ep_double_push_lan: "b2b4",
    ep_double_push_target: Position::new(1, 2),
    ep_roundtrip_fen: "r1k2r/6/6/2Pp2/6/R1K2R w KQkq d4 0 1"
);

// -----------------------------------------------------------------------------
// 8x8
// -----------------------------------------------------------------------------

board_size_tests!(
    8, 8,
    start_fen: "rnbqkbnr/pppppppp/8/8/8/8/PPPPPPPP/RNBQKBNR w KQkq - 0 1",
    castling_fen: "r3k2r/8/8/8/8/8/8/R3K2R w KQkq - 0 1",
    castling_blocked_fen: "r3k2r/8/8/8/8/8/8/R3KN1R w KQkq - 0 1",
    king_col: 4,
    ep_white_fen: "r3k2r/8/8/3pP3/8/8/8/R3K2R w KQkq d6 0 1",
    ep_white_src: Position::new(4, 4),
    ep_white_dst: Position::new(3, 5),
    ep_white_captured: Position::new(3, 4),
    ep_black_fen: "r3k2r/8/8/8/3pP3/8/8/R3K2R b KQkq e3 0 1",
    ep_black_src: Position::new(3, 3),
    ep_black_dst: Position::new(4, 2),
    ep_black_captured: Position::new(4, 3),
    ep_unmake_fen: "r3k2r/8/8/3pP3/8/8/8/R3K2R w KQkq d6 0 1",
    ep_double_push_fen: "r3k2r/8/8/8/8/8/4P3/R3K2R w KQkq - 0 1",
    ep_double_push_lan: "e2e4",
    ep_double_push_target: Position::new(4, 2),
    ep_roundtrip_fen: "r3k2r/8/8/3pP3/8/8/8/R3K2R w KQkq d6 0 1"
);

// -----------------------------------------------------------------------------
// 10x10
// -----------------------------------------------------------------------------

board_size_tests!(
    10, 10,
    start_fen: "r3k4r/10/10/10/10/10/10/10/10/R3K4R w KQkq - 0 1",
    castling_fen: "r3k4r/10/10/10/10/10/10/10/10/R3K4R w KQkq - 0 1",
    castling_blocked_fen: "r3k4r/10/10/10/10/10/10/10/10/R3KN3R w KQkq - 0 1",
    king_col: 4,
    ep_white_fen: "r3k4r/10/10/3pP5/10/10/10/10/10/R3K4R w KQkq d8 0 1",
    ep_white_src: Position::new(4, 6),
    ep_white_dst: Position::new(3, 7),
    ep_white_captured: Position::new(3, 6),
    ep_black_fen: "r3k4r/10/10/10/10/10/5pP3/10/10/R3K4R b KQkq g3 0 1",
    ep_black_src: Position::new(5, 3),
    ep_black_dst: Position::new(6, 2),
    ep_black_captured: Position::new(6, 3),
    ep_unmake_fen: "r3k4r/10/10/3pP5/10/10/10/10/10/R3K4R w KQkq d8 0 1",
    ep_double_push_fen: "r3k4r/10/10/10/10/10/10/10/4P5/R3K4R w KQkq - 0 1",
    ep_double_push_lan: "e2e4",
    ep_double_push_target: Position::new(4, 2),
    ep_roundtrip_fen: "r3k4r/10/10/3pP5/10/10/10/10/10/R3K4R w KQkq d8 0 1"
);
