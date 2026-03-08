use super::*;
use crate::color::Color;
use crate::outcome::GameOutcome;
use crate::pieces::{Piece, PieceType};
use crate::position::Position;
use crate::r#move::{Move, MoveFlags};

type Game8x8 = Game<{ nw_for_board(STANDARD_COLS as u8, STANDARD_ROWS as u8) }>;

#[test]
fn test_8x8_game_creation() {
    let game = Game8x8::standard();
    assert_eq!(game.board().width(), 8);
    assert_eq!(game.board().height(), 8);
    assert_eq!(game.turn(), Color::White);
    assert_eq!(game.fullmove_number(), 1);
    assert_eq!(game.halfmove_clock(), 0);
}

#[test]
fn test_8x8_game_initial_position() {
    let mut game = Game8x8::standard();
    let fen = game.to_fen();
    assert_eq!(
        fen,
        "rnbqkbnr/pppppppp/8/8/8/8/PPPPPPPP/RNBQKBNR w KQkq - 0 1"
    );
}

#[test]
fn test_8x8_game_king_tracking() {
    let game = Game8x8::standard();

    assert_eq!(game.white_king_pos, Position::new(4, 0));
    assert_eq!(game.black_king_pos, Position::new(4, 7));
}

#[test]
fn test_8x8_game_rook_attack_patterns() {
    let mut game = Game8x8::standard();
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
    let mut game = Game8x8::standard();
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
    let mut game = Game8x8::new(8, 8, valid_ep_fen, true).expect("Failed to parse FEN");

    assert_eq!(game.to_fen(), valid_ep_fen);
    assert_eq!(game.turn(), Color::White);
    assert_eq!(game.fullmove_number(), 3);
    assert_eq!(game.halfmove_clock(), 0);
}

#[test]
fn test_8x8_game_fen_parsing_invalid_en_passant() {
    let invalid_ep_fen = "rnbqkbnr/pppppppp/8/8/4P3/8/PPPP1PPP/RNBQKBNR b KQkq e3 0 1";
    let mut game = Game8x8::new(8, 8, invalid_ep_fen, true).expect("Failed to parse FEN");

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
    let mut game = Game8x8::standard();

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
    let mut game = Game8x8::standard();
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
    let mut game = Game8x8::standard();

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
    let mut game = Game8x8::standard();

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
    let mut game = Game8x8::new(8, 8, fen, false).expect("Failed to parse stalemate FEN");

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
    let mut game = Game8x8::standard();
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
    let mut game = Game8x8::standard();
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
    let mut game = Game8x8::standard();
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
    let mut game = Game8x8::standard();

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
    let mut game = Game8x8::standard();

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
    let mut game = Game8x8::standard();

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
    let mut game = Game8x8::standard();

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
    let game = Game8x8::standard();
    // 82 planes * 64 squares = 5248
    assert_eq!(
        crate::encode::get_total_actions(game.board().width(), game.board().height()),
        5248
    );
}
