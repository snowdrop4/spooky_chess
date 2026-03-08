use super::*;
use crate::bitboard::nw_for_board;
use crate::color::Color;
use crate::pieces::PieceType;

type Game6x6 = Game<{ nw_for_board(6, 6) }>;

#[test]
fn test_6x6_game_board_sizes() {
    // Create custom FENs for different board sizes
    let fen_6x6 = "rnbkqr/pppppp/6/6/PPPPPP/RNBKQR w - - 0 1";

    let mut game = Game6x6::new(6, 6, fen_6x6, true).expect("Failed to create 6x6 game");
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
    let game = Game6x6::new(6, 6, fen_6x6, true).expect("Failed to create 6x6 game");
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
