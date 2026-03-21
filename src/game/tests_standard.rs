use super::*;
use crate::color::Color;
use crate::r#move::{Move, MoveFlags};
use crate::outcome::{GameOutcome, TurnState};
use crate::pieces::{Piece, PieceType};
use crate::position::Position;
use rstest::rstest;

type Game8x8 = Game<8, 8>;

#[test]
fn initial_position() {
    let mut game = Game8x8::standard();
    let fen = game.to_fen();
    assert_eq!(
        fen,
        "rnbqkbnr/pppppppp/8/8/8/8/PPPPPPPP/RNBQKBNR w KQkq - 0 1"
    );
}

#[test]
fn king_tracking() {
    let game = Game8x8::standard();
    assert_eq!(game.white_king_pos, Position::new(4, 0));
    assert_eq!(game.black_king_pos, Position::new(4, 7));
}

#[test]
fn fen_parsing_valid_en_passant() {
    let valid_ep_fen = "rnbqkbnr/ppp1pppp/8/3pP3/8/8/PPPP1PPP/RNBQKBNR w KQkq d6 0 3";
    let mut game = Game8x8::new(valid_ep_fen, true).expect("Failed to parse FEN");

    assert_eq!(game.to_fen(), valid_ep_fen);
    assert_eq!(game.turn(), Color::White);
    assert_eq!(game.fullmove_number(), 3);
    assert_eq!(game.halfmove_clock(), 0);
}

#[test]
fn fen_parsing_invalid_en_passant() {
    let invalid_ep_fen = "rnbqkbnr/pppppppp/8/8/4P3/8/PPPP1PPP/RNBQKBNR b KQkq e3 0 1";
    let mut game = Game8x8::new(invalid_ep_fen, true).expect("Failed to parse FEN");

    assert_eq!(
        game.to_fen(),
        "rnbqkbnr/pppppppp/8/8/4P3/8/PPPP1PPP/RNBQKBNR b KQkq - 0 1"
    );
    assert_eq!(game.turn(), Color::Black);
    assert_eq!(game.fullmove_number(), 1);
    assert_eq!(game.halfmove_clock(), 0);
}

#[test]
fn move_making_basic() {
    let mut game = Game8x8::standard();

    let e4_move = Move::from_position(Position::new(4, 1), Position::new(4, 3), MoveFlags::empty());
    let success = game.make_move(&e4_move);
    assert!(success, "Move should be successful");

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
fn outcome_checkmate_white_wins() {
    let mut game = Game8x8::standard();

    game.make_move(
        &Move::from_lan("e2e4", 8, 8).expect("outcome_checkmate_white_wins: failed to parse e2e4"),
    );
    game.make_move(
        &Move::from_lan("e7e5", 8, 8).expect("outcome_checkmate_white_wins: failed to parse e7e5"),
    );
    game.make_move(
        &Move::from_lan("f1c4", 8, 8).expect("outcome_checkmate_white_wins: failed to parse f1c4"),
    );
    game.make_move(
        &Move::from_lan("b8c6", 8, 8).expect("outcome_checkmate_white_wins: failed to parse b8c6"),
    );
    game.make_move(
        &Move::from_lan("d1h5", 8, 8).expect("outcome_checkmate_white_wins: failed to parse d1h5"),
    );
    game.make_move(
        &Move::from_lan("g8f6", 8, 8).expect("outcome_checkmate_white_wins: failed to parse g8f6"),
    );
    game.make_move(
        &Move::from_lan("h5f7", 8, 8).expect("outcome_checkmate_white_wins: failed to parse h5f7"),
    );

    assert!(game.is_checkmate());
    assert_eq!(game.outcome(), Some(GameOutcome::WhiteWin));
}

#[test]
fn outcome_checkmate_black_wins() {
    let mut game = Game8x8::standard();

    game.make_move(
        &Move::from_lan("f2f3", 8, 8).expect("outcome_checkmate_black_wins: failed to parse f2f3"),
    );
    game.make_move(
        &Move::from_lan("e7e5", 8, 8).expect("outcome_checkmate_black_wins: failed to parse e7e5"),
    );
    game.make_move(
        &Move::from_lan("g2g4", 8, 8).expect("outcome_checkmate_black_wins: failed to parse g2g4"),
    );
    game.make_move(
        &Move::from_lan("d8h4", 8, 8).expect("outcome_checkmate_black_wins: failed to parse d8h4"),
    );

    assert!(game.is_checkmate());
    assert_eq!(game.outcome(), Some(GameOutcome::BlackWin));
}

#[test]
fn outcome_stalemate() {
    let fen = "K7/8/1q6/8/8/8/8/2k5 w - - 0 1";
    let mut game = Game8x8::new(fen, false).expect("Failed to parse stalemate FEN");

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
    assert_eq!(game.outcome(), Some(GameOutcome::Stalemate));
}

#[test]
fn turn_state_ongoing_returns_legal_moves() {
    let mut game = Game8x8::standard();
    let legal = game.legal_moves();

    match game.turn_state() {
        TurnState::Ongoing(moves) => assert_eq!(moves, legal),
        TurnState::Over(outcome) => panic!("expected ongoing turn state, got {outcome:?}"),
    }
}

#[test]
fn turn_state_stalemate_returns_outcome() {
    let fen = "K7/8/1q6/8/8/8/8/2k5 w - - 0 1";
    let mut game = Game8x8::new(fen, false).expect("Failed to parse stalemate FEN");

    match game.turn_state() {
        TurnState::Over(outcome) => assert_eq!(outcome, GameOutcome::Stalemate),
        TurnState::Ongoing(moves) => {
            panic!("expected terminal turn state, got {} moves", moves.len())
        }
    }
}

#[test]
fn halfmove_clock_reset_on_pawn_move() {
    let mut game = Game8x8::standard();

    game.make_move(
        &Move::from_lan("g1f3", 8, 8)
            .expect("halfmove_clock_reset_on_pawn_move: failed to parse g1f3"),
    );
    game.make_move(
        &Move::from_lan("g8f6", 8, 8)
            .expect("halfmove_clock_reset_on_pawn_move: failed to parse g8f6"),
    );
    game.make_move(
        &Move::from_lan("f3g1", 8, 8)
            .expect("halfmove_clock_reset_on_pawn_move: failed to parse f3g1"),
    );
    game.make_move(
        &Move::from_lan("f6g8", 8, 8)
            .expect("halfmove_clock_reset_on_pawn_move: failed to parse f6g8"),
    );

    assert_eq!(game.halfmove_clock, 4);

    game.make_move(
        &Move::from_lan("e2e4", 8, 8)
            .expect("halfmove_clock_reset_on_pawn_move: failed to parse e2e4"),
    );
    assert_eq!(game.halfmove_clock, 0);
}

#[test]
fn halfmove_clock_reset_on_capture() {
    let mut game = Game8x8::standard();

    game.make_move(
        &Move::from_lan("e2e4", 8, 8)
            .expect("halfmove_clock_reset_on_capture: failed to parse e2e4"),
    );
    game.make_move(
        &Move::from_lan("d7d5", 8, 8)
            .expect("halfmove_clock_reset_on_capture: failed to parse d7d5"),
    );
    assert_eq!(game.halfmove_clock, 0);

    game.make_move(
        &Move::from_lan("g1f3", 8, 8)
            .expect("halfmove_clock_reset_on_capture: failed to parse g1f3"),
    );
    game.make_move(
        &Move::from_lan("b8c6", 8, 8)
            .expect("halfmove_clock_reset_on_capture: failed to parse b8c6"),
    );
    assert_eq!(game.halfmove_clock, 2);

    game.make_move(
        &Move::from_lan("e4d5", 8, 8)
            .expect("halfmove_clock_reset_on_capture: failed to parse e4d5"),
    );
    assert_eq!(game.halfmove_clock, 0);
}

#[test]
fn castling_rights_methods() {
    let mut game = Game8x8::standard();

    assert!(game.castling_rights().has_kingside(Color::White));
    assert!(game.castling_rights().has_queenside(Color::White));
    assert!(game.castling_rights().has_kingside(Color::Black));
    assert!(game.castling_rights().has_queenside(Color::Black));

    game.make_move(
        &Move::from_lan("e2e3", 8, 8).expect("castling_rights_methods: failed to parse e2e3"),
    );
    game.make_move(
        &Move::from_lan("e7e6", 8, 8).expect("castling_rights_methods: failed to parse e7e6"),
    );
    game.make_move(
        &Move::from_lan("e1e2", 8, 8).expect("castling_rights_methods: failed to parse e1e2"),
    );

    assert!(!game.castling_rights().has_kingside(Color::White));
    assert!(!game.castling_rights().has_queenside(Color::White));
    assert!(game.castling_rights().has_kingside(Color::Black));
    assert!(game.castling_rights().has_queenside(Color::Black));
}

#[test]
fn castling_rights_rook_move() {
    let mut game = Game8x8::standard();

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

    game.make_move(
        &Move::from_lan("a1a2", 8, 8).expect("castling_rights_rook_move: failed to parse a1a2"),
    );

    assert!(game.castling_rights().has_kingside(Color::White));
    assert!(!game.castling_rights().has_queenside(Color::White));
}

#[rstest]
#[case("k_vs_k", None)]
#[case("kb_vs_k", Some((PieceType::Bishop, Color::White, Position::new(2, 2))))]
fn insufficient_material(
    #[case] _name: &str,
    #[case] extra_piece: Option<(PieceType, Color, Position)>,
) {
    let mut game = Game8x8::standard();
    game.board.clear();

    game.board.set_piece(
        &Position::new(4, 0),
        Some(Piece::new(PieceType::King, Color::White)),
    );
    game.white_king_pos = Position::new(4, 0);
    game.board.set_piece(
        &Position::new(4, 7),
        Some(Piece::new(PieceType::King, Color::Black)),
    );
    game.black_king_pos = Position::new(4, 7);

    if let Some((pt, color, pos)) = extra_piece {
        game.board.set_piece(&pos, Some(Piece::new(pt, color)));
    }

    assert!(game.is_insufficient_material());
    assert!(game.is_over());
    assert_eq!(game.outcome(), Some(GameOutcome::InsufficientMaterial));
}

#[test]
fn fifty_move_rule() {
    let mut game = Game8x8::standard();
    game.board.clear();

    game.board.set_piece(
        &Position::new(4, 0),
        Some(Piece::new(PieceType::King, Color::White)),
    );
    game.white_king_pos = Position::new(4, 0);
    game.board.set_piece(
        &Position::new(0, 0),
        Some(Piece::new(PieceType::Rook, Color::White)),
    );
    game.board.set_piece(
        &Position::new(4, 7),
        Some(Piece::new(PieceType::King, Color::Black)),
    );
    game.black_king_pos = Position::new(4, 7);

    game.halfmove_clock = 150;

    assert!(game.is_over());
    assert_eq!(game.outcome(), Some(GameOutcome::FiftyMoveRule));
}

#[test]
fn total_actions_standard() {
    let game = Game8x8::standard();
    assert_eq!(
        crate::encode::get_total_actions(game.board().width(), game.board().height()),
        5248
    );
}
