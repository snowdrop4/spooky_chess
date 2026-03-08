use super::*;
use crate::bitboard::nw_for_board;
use crate::pieces::PieceType;
use crate::position::Position;
use crate::r#move::MoveFlags;

type Game10x10 = Game<{ nw_for_board(10, 10) }>;

#[test]
fn test_10x10_castling_kingside() {
    // 10x10 board: kings on col 5 (f-file), rooks on col 9 (j-file)
    // Row 0 = white back rank, row 9 = black back rank
    // King at f1, rook at j1, clear path between them
    let fen = "r3k4r/10/10/10/10/10/10/10/10/R3K4R w KQkq - 0 1";
    let mut game = Game10x10::new(10, 10, fen, true).expect("Failed to create 10x10 game");

    // White should be able to castle kingside (king e1 -> g1)
    let legal = game.legal_moves();
    let castle_ks = legal.iter().find(|m| {
        m.src == Position::new(4, 0)
            && m.dst == Position::new(6, 0)
            && m.flags.contains(MoveFlags::CASTLE)
    });
    assert!(
        castle_ks.is_some(),
        "White should be able to castle kingside on 10x10 board"
    );

    // Execute the castle
    let mv = game.move_from_lan("e1g1").unwrap();
    assert!(game.make_move(&mv), "Kingside castle should succeed");

    // King should be at g1 (col 6, row 0)
    assert_eq!(
        game.board()
            .get_piece(&Position::new(6, 0))
            .unwrap()
            .piece_type,
        PieceType::King
    );
    // Rook should have moved from j1 (col 9) to f1 (col 5)
    assert_eq!(
        game.board()
            .get_piece(&Position::new(5, 0))
            .unwrap()
            .piece_type,
        PieceType::Rook
    );
    // Original positions should be empty
    assert!(game.board().get_piece(&Position::new(4, 0)).is_none());
    assert!(game.board().get_piece(&Position::new(9, 0)).is_none());
}

#[test]
fn test_10x10_castling_queenside() {
    let fen = "r3k4r/10/10/10/10/10/10/10/10/R3K4R w KQkq - 0 1";
    let mut game = Game10x10::new(10, 10, fen, true).expect("Failed to create 10x10 game");

    let legal = game.legal_moves();
    let castle_qs = legal.iter().find(|m| {
        m.src == Position::new(4, 0)
            && m.dst == Position::new(2, 0)
            && m.flags.contains(MoveFlags::CASTLE)
    });
    assert!(
        castle_qs.is_some(),
        "White should be able to castle queenside on 10x10 board"
    );

    let mv = game.move_from_lan("e1c1").unwrap();
    assert!(game.make_move(&mv), "Queenside castle should succeed");

    // King at c1 (col 2)
    assert_eq!(
        game.board()
            .get_piece(&Position::new(2, 0))
            .unwrap()
            .piece_type,
        PieceType::King
    );
    // Rook moved from a1 (col 0) to d1 (col 3)
    assert_eq!(
        game.board()
            .get_piece(&Position::new(3, 0))
            .unwrap()
            .piece_type,
        PieceType::Rook
    );
}

#[test]
fn test_10x10_castling_unmake() {
    let fen = "r3k4r/10/10/10/10/10/10/10/10/R3K4R w KQkq - 0 1";
    let mut game = Game10x10::new(10, 10, fen, true).expect("Failed to create 10x10 game");

    let original_fen = game.to_fen();

    let mv = game.move_from_lan("e1g1").unwrap();
    game.make_move(&mv);
    game.unmake_move();

    assert_eq!(
        game.to_fen(),
        original_fen,
        "Unmake should restore state after kingside castle on 10x10"
    );
}

#[test]
fn test_10x10_castling_blocked() {
    // Place a piece between king and rook to block castling
    let fen = "r3k4r/10/10/10/10/10/10/10/10/R3KN3R w KQkq - 0 1";
    let mut game = Game10x10::new(10, 10, fen, true).expect("Failed to create 10x10 game");

    let legal = game.legal_moves();
    let castle_ks = legal.iter().find(|m| {
        m.src == Position::new(4, 0) && m.flags.contains(MoveFlags::CASTLE) && m.dst.col > m.src.col
    });
    assert!(
        castle_ks.is_none(),
        "Kingside castle should be blocked when piece is in the way"
    );
}

#[test]
fn test_10x10_en_passant_white() {
    // 10x10 board: white pawn on e6 (col 4, row 5), black pawn just double-pushed d8-d6
    // En passant target = d7 (col 3, row 6)
    let fen = "r3k4r/10/10/10/3pP5/10/10/10/10/R3K4R w KQkq d7 0 1";
    let mut game = Game10x10::new(10, 10, fen, true).expect("Failed to create 10x10 game");

    let legal = game.legal_moves();
    let ep = legal
        .iter()
        .find(|m| m.flags.contains(MoveFlags::EN_PASSANT));
    assert!(
        ep.is_some(),
        "White should be able to capture en passant on 10x10 board"
    );

    let ep_mv = ep.unwrap().clone();
    assert_eq!(ep_mv.src, Position::new(4, 5)); // e6
    assert_eq!(ep_mv.dst, Position::new(3, 6)); // d7

    game.make_move(&ep_mv);

    // White pawn should be at d7
    assert_eq!(
        game.board()
            .get_piece(&Position::new(3, 6))
            .unwrap()
            .piece_type,
        PieceType::Pawn
    );
    // Captured black pawn at d6 should be gone
    assert!(game.board().get_piece(&Position::new(3, 5)).is_none());
    // Original white pawn position should be empty
    assert!(game.board().get_piece(&Position::new(4, 5)).is_none());
}

#[test]
fn test_10x10_en_passant_black() {
    // Black pawn on f5 (col 5, row 4), white pawn just double-pushed to g5 (col 6, row 4)
    // En passant target = g4 (col 6, row 3)
    // FEN rows go top-to-bottom: row 9, 8, 7, 6, 5, 4, 3, 2, 1, 0
    let fen = "r3k4r/10/10/10/10/5pP3/10/10/10/R3K4R b KQkq g4 0 1";
    let mut game = Game10x10::new(10, 10, fen, true).expect("Failed to create 10x10 game");

    let legal = game.legal_moves();
    let ep = legal
        .iter()
        .find(|m| m.flags.contains(MoveFlags::EN_PASSANT));
    assert!(
        ep.is_some(),
        "Black should be able to capture en passant on 10x10 board"
    );

    let ep_mv = ep.unwrap().clone();
    assert_eq!(ep_mv.src, Position::new(5, 4)); // f5
    assert_eq!(ep_mv.dst, Position::new(6, 3)); // g4

    game.make_move(&ep_mv);

    // Black pawn at g4
    assert_eq!(
        game.board()
            .get_piece(&Position::new(6, 3))
            .unwrap()
            .piece_type,
        PieceType::Pawn
    );
    // Captured white pawn at g5 should be gone
    assert!(game.board().get_piece(&Position::new(6, 4)).is_none());
}

#[test]
fn test_10x10_en_passant_unmake() {
    let fen = "r3k4r/10/10/10/3pP5/10/10/10/10/R3K4R w KQkq d7 0 1";
    let mut game = Game10x10::new(10, 10, fen, true).expect("Failed to create 10x10 game");

    let original_fen = game.to_fen();

    let legal = game.legal_moves();
    let ep = legal
        .iter()
        .find(|m| m.flags.contains(MoveFlags::EN_PASSANT))
        .unwrap()
        .clone();
    game.make_move(&ep);
    game.unmake_move();

    assert_eq!(
        game.to_fen(),
        original_fen,
        "Unmake should restore state after en passant on 10x10"
    );
}

#[test]
fn test_10x10_en_passant_created_by_double_push() {
    // Verify that a double pawn push on a 10x10 board sets the en passant square correctly
    let fen = "r3k4r/10/10/10/10/10/10/10/4P5/R3K4R w KQkq - 0 1";
    let mut game = Game10x10::new(10, 10, fen, true).expect("Failed to create 10x10 game");

    let mv = game.move_from_lan("e2e4").unwrap();
    assert!(mv.flags.contains(MoveFlags::DOUBLE_PUSH));
    game.make_move(&mv);

    // En passant square should be e3 (col 4, row 2)
    assert_eq!(
        game.en_passant_square(),
        Some(Position::new(4, 2)),
        "En passant square should be set after double push on 10x10"
    );
}

#[test]
fn test_10x10_en_passant_fen_roundtrip() {
    // Verify en passant is correctly represented in FEN on non-standard board
    let fen = "r3k4r/10/10/10/3pP5/10/10/10/10/R3K4R w KQkq d7 0 1";
    let mut game = Game10x10::new(10, 10, fen, true).expect("Failed to create 10x10 game");

    assert_eq!(
        game.to_fen(),
        fen,
        "FEN roundtrip should preserve en passant on 10x10"
    );
}
