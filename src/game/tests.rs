use super::*;
use crate::bitboard::nw_for_board;
use crate::board::{STANDARD_COLS, STANDARD_ROWS};
use crate::color::Color;
use crate::outcome::GameOutcome;
use crate::pieces::{Piece, PieceType};
use crate::position::Position;
use crate::r#move::{Move, MoveFlags};

type StdGame = Game<{ nw_for_board(STANDARD_COLS as u8, STANDARD_ROWS as u8) }>;

// -----------------------------------------------------------------------------
// 8x8
// -----------------------------------------------------------------------------

#[test]
fn test_8x8_game_creation() {
    let game = StdGame::standard();
    assert_eq!(game.board().width(), 8);
    assert_eq!(game.board().height(), 8);
    assert_eq!(game.turn(), Color::White);
    assert_eq!(game.fullmove_number(), 1);
    assert_eq!(game.halfmove_clock(), 0);
}

#[test]
fn test_8x8_game_initial_position() {
    let mut game = StdGame::standard();
    let fen = game.to_fen();
    assert_eq!(
        fen,
        "rnbqkbnr/pppppppp/8/8/8/8/PPPPPPPP/RNBQKBNR w KQkq - 0 1"
    );
}

#[test]
fn test_8x8_game_king_tracking() {
    let game = StdGame::standard();

    assert_eq!(game.white_king_pos, Position::new(4, 0));
    assert_eq!(game.black_king_pos, Position::new(4, 7));
}

#[test]
fn test_8x8_game_rook_attack_patterns() {
    let mut game = StdGame::standard();
    game.board.clear();

    // Need kings on the board for a valid position
    game.board.set_piece(
        &Position::new(0, 0),
        Some(Piece::new(PieceType::King, Color::White)),
    );
    game.board.set_piece(
        &Position::new(7, 7),
        Some(Piece::new(PieceType::King, Color::Black)),
    );
    game.white_king_pos = Position::new(0, 0);
    game.black_king_pos = Position::new(7, 7);

    let rook = Piece::new(PieceType::Rook, Color::White);
    let rook_pos = Position::new(4, 4); // e5
    game.board.set_piece(&rook_pos, Some(rook));

    // Rook can attack along rows and cols (is_square_attacked checks if White attacks the square)
    assert!(game.is_square_attacked(&Position::new(4, 1), Color::White)); // e2
    assert!(game.is_square_attacked(&Position::new(4, 7), Color::White)); // e8
    assert!(game.is_square_attacked(&Position::new(1, 4), Color::White)); // b5
    assert!(game.is_square_attacked(&Position::new(7, 4), Color::White)); // h5

    // Cannot attack diagonally (only rook on board besides kings)
    assert!(!game.is_square_attacked(&Position::new(5, 5), Color::White)); // f6

    // Test blocked path
    let blocker = Piece::new(PieceType::Pawn, Color::Black);
    game.board.set_piece(&Position::new(4, 6), Some(blocker)); // e7

    assert!(!game.is_square_attacked(&Position::new(4, 7), Color::White)); // e8 (blocked)
    assert!(game.is_square_attacked(&Position::new(4, 6), Color::White)); // e7 (can capture)
}

#[test]
fn test_8x8_game_bishop_attack_patterns() {
    let mut game = StdGame::standard();
    game.board.clear();

    // Need kings on the board for a valid position
    game.board.set_piece(
        &Position::new(0, 2),
        Some(Piece::new(PieceType::King, Color::White)),
    );
    game.board.set_piece(
        &Position::new(7, 0),
        Some(Piece::new(PieceType::King, Color::Black)),
    );
    game.white_king_pos = Position::new(0, 2);
    game.black_king_pos = Position::new(7, 0);

    let bishop = Piece::new(PieceType::Bishop, Color::White);
    let bishop_pos = Position::new(4, 4); // e5
    game.board.set_piece(&bishop_pos, Some(bishop));

    // Bishop can attack diagonally
    assert!(game.is_square_attacked(&Position::new(0, 0), Color::White)); // a1
    assert!(game.is_square_attacked(&Position::new(7, 7), Color::White)); // h8
    assert!(game.is_square_attacked(&Position::new(1, 7), Color::White)); // b8
    assert!(game.is_square_attacked(&Position::new(7, 1), Color::White)); // h2

    // Cannot attack along rows/cols (only bishop+kings, kings are far away)
    assert!(!game.is_square_attacked(&Position::new(4, 1), Color::White)); // e2

    // Test blocked path
    let blocker = Piece::new(PieceType::Pawn, Color::Black);
    game.board.set_piece(&Position::new(6, 6), Some(blocker)); // g7

    assert!(!game.is_square_attacked(&Position::new(7, 7), Color::White)); // h8 (blocked)
    assert!(game.is_square_attacked(&Position::new(6, 6), Color::White)); // g7 (can capture)
}

#[test]
fn test_8x8_game_fen_parsing_valid_en_passant() {
    // Test with a valid en passant scenario
    let valid_ep_fen = "rnbqkbnr/ppp1pppp/8/3pP3/8/8/PPPP1PPP/RNBQKBNR w KQkq d6 0 3";
    let mut game = StdGame::new(8, 8, valid_ep_fen, true).expect("Failed to parse FEN");

    assert_eq!(game.to_fen(), valid_ep_fen);
    assert_eq!(game.turn(), Color::White);
    assert_eq!(game.fullmove_number(), 3);
    assert_eq!(game.halfmove_clock(), 0);
}

#[test]
fn test_8x8_game_fen_parsing_invalid_en_passant() {
    let invalid_ep_fen = "rnbqkbnr/pppppppp/8/8/4P3/8/PPPP1PPP/RNBQKBNR b KQkq e3 0 1";
    let mut game = StdGame::new(8, 8, invalid_ep_fen, true).expect("Failed to parse FEN");

    // Note: en passant square e3 is ignored because there's no enemy pawn that can capture
    assert_eq!(
        game.to_fen(),
        "rnbqkbnr/pppppppp/8/8/4P3/8/PPPP1PPP/RNBQKBNR b KQkq - 0 1"
    );
    assert_eq!(game.turn(), Color::Black);
    assert_eq!(game.fullmove_number(), 1);
    assert_eq!(game.halfmove_clock(), 0);
}

#[test]
fn test_8x8_game_move_making_basic() {
    let mut game = StdGame::standard();

    // Make a simple pawn move
    let e4_move = Move::from_position(Position::new(4, 1), Position::new(4, 3), MoveFlags::empty());
    let success = game.make_move(&e4_move);
    assert!(success, "Move should be successful");

    // Verify the move was made
    assert_eq!(game.board.get_piece(&Position::new(4, 1)), None);
    assert_eq!(
        game.board
            .get_piece(&Position::new(4, 3))
            .expect("Expected piece at position e4 after move")
            .piece_type,
        PieceType::Pawn
    );
}

#[test]
fn test_8x8_game_is_square_attacked_basic() {
    let mut game = StdGame::standard();
    game.board.clear();

    // Place a white rook at e5
    let rook = Piece::new(PieceType::Rook, Color::White);
    game.board.set_piece(&Position::new(4, 4), Some(rook)); // e5

    // The rook should attack squares along its row and col
    assert!(game.is_square_attacked(&Position::new(4, 0), Color::White)); // e1
    assert!(game.is_square_attacked(&Position::new(0, 4), Color::White)); // a5
    assert!(!game.is_square_attacked(&Position::new(5, 5), Color::White)); // f6 (diagonal)

    // Place a black king at e8
    let king = Piece::new(PieceType::King, Color::Black);
    game.board.set_piece(&Position::new(4, 7), Some(king)); // e8

    // The king should be attacked by the rook
    assert!(game.is_square_attacked(&Position::new(4, 7), Color::White));
}

#[test]
fn test_8x8_game_outcome_checkmate_white_wins() {
    // Scholar's mate - white wins
    // 1. e4 e5, 2. Bc4 Nc6, 3. Qh5 Nf6??, 4. Qxf7#
    let mut game = StdGame::standard();

    game.make_move(&Move::from_lan("e2e4", 8, 8).unwrap());
    game.make_move(&Move::from_lan("e7e5", 8, 8).unwrap());
    game.make_move(&Move::from_lan("f1c4", 8, 8).unwrap());
    game.make_move(&Move::from_lan("b8c6", 8, 8).unwrap());
    game.make_move(&Move::from_lan("d1h5", 8, 8).unwrap());
    game.make_move(&Move::from_lan("g8f6", 8, 8).unwrap());
    game.make_move(&Move::from_lan("h5f7", 8, 8).unwrap());

    assert!(game.is_checkmate());
    let outcome = game.outcome();
    assert_eq!(outcome, Some(GameOutcome::WhiteWin));
}

#[test]
fn test_8x8_game_outcome_checkmate_black_wins() {
    // Fool's mate - black wins
    let mut game = StdGame::standard();

    game.make_move(&Move::from_lan("f2f3", 8, 8).unwrap());
    game.make_move(&Move::from_lan("e7e5", 8, 8).unwrap());
    game.make_move(&Move::from_lan("g2g4", 8, 8).unwrap());
    game.make_move(&Move::from_lan("d8h4", 8, 8).unwrap());

    assert!(game.is_checkmate());
    let outcome = game.outcome();
    assert_eq!(outcome, Some(GameOutcome::BlackWin));
}

#[test]
fn test_8x8_game_outcome_stalemate() {
    // White king on a8, black queen on b6, black king on c7
    let fen = "K7/8/1q6/8/8/8/8/2k5 w - - 0 1";
    let mut game = StdGame::new(8, 8, fen, false).expect("Failed to parse stalemate FEN");

    assert!(
        !game.is_check(),
        "King should not be in check for stalemate"
    );
    let moves = game.legal_moves();
    assert!(
        moves.is_empty(),
        "King should have no legal moves for stalemate"
    );
    assert!(game.is_stalemate());
    let outcome = game.outcome();
    assert_eq!(outcome, Some(GameOutcome::Stalemate));
}

#[test]
fn test_8x8_game_outcome_insufficient_material() {
    let mut game = StdGame::standard();
    game.board.clear();

    // King vs King - insufficient material
    game.board.set_piece(
        &Position::new(4, 0),
        Some(Piece::new(PieceType::King, Color::White)),
    );
    game.board.set_piece(
        &Position::new(4, 7),
        Some(Piece::new(PieceType::King, Color::Black)),
    );

    assert!(game.is_insufficient_material());
    assert!(game.is_over());
    let outcome = game.outcome();
    assert_eq!(outcome, Some(GameOutcome::InsufficientMaterial));
}

#[test]
fn test_8x8_game_outcome_insufficient_material_bishop() {
    let mut game = StdGame::standard();
    game.board.clear();

    // King + Bishop vs King - insufficient material
    game.board.set_piece(
        &Position::new(4, 0),
        Some(Piece::new(PieceType::King, Color::White)),
    );
    game.board.set_piece(
        &Position::new(2, 2),
        Some(Piece::new(PieceType::Bishop, Color::White)),
    );
    game.board.set_piece(
        &Position::new(4, 7),
        Some(Piece::new(PieceType::King, Color::Black)),
    );

    assert!(game.is_insufficient_material());
    assert!(game.is_over());
    let outcome = game.outcome();
    assert_eq!(outcome, Some(GameOutcome::InsufficientMaterial));
}

#[test]
fn test_8x8_game_outcome_fifty_move_rule() {
    let mut game = StdGame::standard();
    game.board.clear();

    // Set up a simple position with just kings and a rook
    game.board.set_piece(
        &Position::new(4, 0),
        Some(Piece::new(PieceType::King, Color::White)),
    );
    game.board.set_piece(
        &Position::new(0, 0),
        Some(Piece::new(PieceType::Rook, Color::White)),
    );
    game.board.set_piece(
        &Position::new(4, 7),
        Some(Piece::new(PieceType::King, Color::Black)),
    );

    // Manually set halfmove clock to trigger fifty-move rule (150 half-moves = 75 full moves)
    game.halfmove_clock = 150;

    assert!(game.is_over());
    let outcome = game.outcome();
    assert_eq!(outcome, Some(GameOutcome::FiftyMoveRule));
}

#[test]
fn test_8x8_game_halfmove_clock_reset_on_pawn_move() {
    let mut game = StdGame::standard();

    // Make some non-pawn moves to increase halfmove clock
    game.make_move(&Move::from_lan("g1f3", 8, 8).unwrap());
    game.make_move(&Move::from_lan("g8f6", 8, 8).unwrap());
    game.make_move(&Move::from_lan("f3g1", 8, 8).unwrap());
    game.make_move(&Move::from_lan("f6g8", 8, 8).unwrap());

    assert_eq!(game.halfmove_clock, 4);

    // Make a pawn move - should reset halfmove clock
    game.make_move(&Move::from_lan("e2e4", 8, 8).unwrap());
    assert_eq!(game.halfmove_clock, 0);
}

#[test]
fn test_8x8_game_halfmove_clock_reset_on_capture() {
    let mut game = StdGame::standard();

    // Set up a position where a capture is possible
    game.make_move(&Move::from_lan("e2e4", 8, 8).unwrap());
    game.make_move(&Move::from_lan("d7d5", 8, 8).unwrap());

    assert_eq!(game.halfmove_clock, 0); // Both were pawn moves

    // Make some knight moves to increase halfmove clock
    game.make_move(&Move::from_lan("g1f3", 8, 8).unwrap());
    game.make_move(&Move::from_lan("b8c6", 8, 8).unwrap());
    assert_eq!(game.halfmove_clock, 2);

    // Make a capture - should reset halfmove clock
    game.make_move(&Move::from_lan("e4d5", 8, 8).unwrap()); // Capture pawn
    assert_eq!(game.halfmove_clock, 0);
}

#[test]
fn test_8x8_game_castling_rights_methods() {
    let mut game = StdGame::standard();

    // Initial position should have all castling rights
    assert!(game.castling_rights().has_kingside(Color::White));
    assert!(game.castling_rights().has_queenside(Color::White));
    assert!(game.castling_rights().has_kingside(Color::Black));
    assert!(game.castling_rights().has_queenside(Color::Black));

    // Move white king
    game.make_move(&Move::from_lan("e2e3", 8, 8).unwrap());
    game.make_move(&Move::from_lan("e7e6", 8, 8).unwrap());
    game.make_move(&Move::from_lan("e1e2", 8, 8).unwrap());

    // White should lose all castling rights
    assert!(!game.castling_rights().has_kingside(Color::White));
    assert!(!game.castling_rights().has_queenside(Color::White));
    // Black should still have both
    assert!(game.castling_rights().has_kingside(Color::Black));
    assert!(game.castling_rights().has_queenside(Color::Black));
}

#[test]
fn test_8x8_game_castling_rights_rook_move() {
    let mut game = StdGame::standard();

    // Clear path for rook movement
    game.board.clear();
    game.board.set_piece(
        &Position::new(4, 0),
        Some(Piece::new(PieceType::King, Color::White)),
    );
    game.board.set_piece(
        &Position::new(4, 7),
        Some(Piece::new(PieceType::King, Color::Black)),
    );
    game.board.set_piece(
        &Position::new(0, 0),
        Some(Piece::new(PieceType::Rook, Color::White)),
    );
    game.board.set_piece(
        &Position::new(7, 0),
        Some(Piece::new(PieceType::Rook, Color::White)),
    );

    // Move queenside rook
    game.make_move(&Move::from_lan("a1a2", 8, 8).unwrap());

    // Should lose queenside castling only
    assert!(game.castling_rights().has_kingside(Color::White));
    assert!(!game.castling_rights().has_queenside(Color::White));
}

#[test]
fn test_8x8_total_actions_standard() {
    let game = StdGame::standard();
    // 82 planes * 64 squares = 5248
    assert_eq!(
        crate::encode::get_total_actions(game.board().width(), game.board().height()),
        5248
    );
}

// -----------------------------------------------------------------------------
// 6x6
// -----------------------------------------------------------------------------

#[test]
fn test_6x6_game_board_sizes() {
    // Create custom FENs for different board sizes
    let fen_6x6 = "rnbkqr/pppppp/6/6/PPPPPP/RNBKQR w - - 0 1";

    let mut game: Game<{ nw_for_board(6, 6) }> =
        Game::new(6, 6, fen_6x6, true).expect("Failed to create 6x6 game");
    assert_eq!(game.board().width(), 6);
    assert_eq!(game.board().height(), 6);

    assert_eq!(
        game.board()
            .get_piece(&game.white_king_pos)
            .unwrap()
            .piece_type,
        PieceType::King
    );
    assert_eq!(
        game.board()
            .get_piece(&game.black_king_pos)
            .unwrap()
            .piece_type,
        PieceType::King
    );

    // Check that tracked positions match actual king positions
    // Should be able to generate FEN
    let fen = game.to_fen();
    assert!(!fen.is_empty());
}

#[test]
fn test_6x6_game_piece_placement() {
    let fen_6x6 = "rnbkqr/pppppp/6/6/PPPPPP/RNBKQR w - - 0 1";
    let game: Game<{ nw_for_board(6, 6) }> =
        Game::new(6, 6, fen_6x6, true).expect("Failed to create 6x6 game");
    let white_pieces = game.board().pieces(Color::White);
    let black_pieces = game.board().pieces(Color::Black);

    // Should have pieces placed
    assert!(!white_pieces.is_empty());
    assert!(!black_pieces.is_empty());

    // Should have exactly one king each
    let white_kings: Vec<_> = white_pieces
        .iter()
        .filter(|(_, piece)| piece.piece_type == PieceType::King)
        .collect();
    let black_kings: Vec<_> = black_pieces
        .iter()
        .filter(|(_, piece)| piece.piece_type == PieceType::King)
        .collect();

    assert_eq!(white_kings.len(), 1);
    assert_eq!(black_kings.len(), 1);
}

// -----------------------------------------------------------------------------
// 10x10
// -----------------------------------------------------------------------------

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

// -----------------------------------------------------------------------------
// Actions
// -----------------------------------------------------------------------------

#[test]
fn test_encode_decode_action_roundtrip() {
    use rand::prelude::IndexedRandom;
    use rand::SeedableRng;

    let mut rng = rand::rngs::StdRng::seed_from_u64(42);

    for _game_num in 0..500 {
        let mut game = StdGame::standard();
        for _move_num in 0..200 {
            if game.is_over() {
                break;
            }
            let legal_moves = game.legal_moves();
            if legal_moves.is_empty() {
                break;
            }

            for mv in &legal_moves {
                let action = game.encode_action(mv).expect("Failed to encode action");
                let decoded = game.decode_action(action).expect("Failed to decode action");

                assert_eq!(
                    decoded.src,
                    mv.src,
                    "src mismatch for move {} (action {})",
                    mv.to_lan(),
                    action
                );
                assert_eq!(
                    decoded.dst,
                    mv.dst,
                    "dst mismatch for move {} (action {})",
                    mv.to_lan(),
                    action
                );
                assert_eq!(
                    decoded.promotion,
                    mv.promotion,
                    "promotion mismatch for move {} (action {})",
                    mv.to_lan(),
                    action
                );
            }

            let chosen = legal_moves.choose(&mut rng).unwrap();
            game.make_move_unchecked(chosen);
        }
    }
}

#[test]
fn test_8x8_apply_action_roundtrip() {
    let mut game = StdGame::standard();
    // e2e4 as an action
    let mv = game.move_from_lan("e2e4").unwrap();
    let action = game.encode_action(&mv).unwrap();
    assert!(game.apply_action(action));
    assert_eq!(game.turn(), Color::Black);

    // Verify the pawn moved
    assert!(game.board().get_piece(&Position::new(4, 1)).is_none());
    assert!(game.board().get_piece(&Position::new(4, 3)).is_some());
}
