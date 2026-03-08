use super::*;
use crate::bitboard::nw_for_board;
use crate::board::{STANDARD_COLS, STANDARD_ROWS};
use crate::pieces::PieceType;
use crate::position::Position;
use crate::r#move::MoveFlags;

type StdGame = Game<{ nw_for_board(STANDARD_COLS as u8, STANDARD_ROWS as u8) }>;

#[test]
fn test_san_basic_pawn_moves() {
    let mut game = StdGame::standard();
    let mv = game.move_from_lan("e2e4").unwrap();
    assert_eq!(game.move_to_san(&mv), "e4");

    let mv2 = game.move_from_lan("d2d3").unwrap();
    assert_eq!(game.move_to_san(&mv2), "d3");
}

#[test]
fn test_san_knight_move() {
    let mut game = StdGame::standard();
    let mv = game.move_from_lan("g1f3").unwrap();
    assert_eq!(game.move_to_san(&mv), "Nf3");
}

#[test]
fn test_san_bishop_move() {
    // 1. e4 e5 2. Bc4
    let mut game = StdGame::standard();
    game.make_move(&game.move_from_lan("e2e4").unwrap());
    game.make_move(&game.move_from_lan("e7e5").unwrap());
    let mv = game.move_from_lan("f1c4").unwrap();
    assert_eq!(game.move_to_san(&mv), "Bc4");
}

#[test]
fn test_san_capture() {
    // 1. e4 d5 2. exd5
    let mut game = StdGame::standard();
    game.make_move(&game.move_from_lan("e2e4").unwrap());
    game.make_move(&game.move_from_lan("d7d5").unwrap());
    let mv = game.move_from_lan("e4d5").unwrap();
    assert_eq!(game.move_to_san(&mv), "exd5");
}

#[test]
fn test_san_piece_capture() {
    // 1. e4 e5 2. Bc4 Nc6 3. Bxf7+
    let mut game = StdGame::standard();
    game.make_move(&game.move_from_lan("e2e4").unwrap());
    game.make_move(&game.move_from_lan("e7e5").unwrap());
    game.make_move(&game.move_from_lan("f1c4").unwrap());
    game.make_move(&game.move_from_lan("b8c6").unwrap());
    let mv = game.move_from_lan("c4f7").unwrap();
    assert_eq!(game.move_to_san(&mv), "Bxf7+");
}

#[test]
fn test_san_disambiguation_file() {
    // Two rooks on same rank, different files
    let fen2 = "4k3/8/8/8/8/8/4K3/R6R w - - 0 1";
    let mut game = StdGame::new(8, 8, fen2, false).unwrap();
    let mv = game.move_from_lan("a1d1").unwrap();
    assert_eq!(game.move_to_san(&mv), "Rad1");
    let mv2 = game.move_from_lan("h1d1").unwrap();
    assert_eq!(game.move_to_san(&mv2), "Rhd1");
}

#[test]
fn test_san_disambiguation_rank() {
    // Two rooks on same file, different ranks
    let fen = "4k3/8/8/8/R7/8/8/R3K3 w - - 0 1";
    let mut game = StdGame::new(8, 8, fen, false).unwrap();
    let mv = game.move_from_lan("a1a2").unwrap();
    assert_eq!(game.move_to_san(&mv), "R1a2");
    let mv2 = game.move_from_lan("a4a2").unwrap();
    assert_eq!(game.move_to_san(&mv2), "R4a2");
}

#[test]
fn test_san_castling_kingside() {
    let fen = "r3k2r/pppppppp/8/8/8/8/PPPPPPPP/R3K2R w KQkq - 0 1";
    let mut game = StdGame::new(8, 8, fen, true).unwrap();
    let mv = game.move_from_lan("e1g1").unwrap();
    assert_eq!(game.move_to_san(&mv), "O-O");
}

#[test]
fn test_san_castling_queenside() {
    let fen = "r3k2r/pppppppp/8/8/8/8/PPPPPPPP/R3K2R w KQkq - 0 1";
    let mut game = StdGame::new(8, 8, fen, true).unwrap();
    let mv = game.move_from_lan("e1c1").unwrap();
    assert_eq!(game.move_to_san(&mv), "O-O-O");
}

#[test]
fn test_san_promotion() {
    let fen = "k7/4P3/8/8/8/8/8/4K3 w - - 0 1";
    let mut game = StdGame::new(8, 8, fen, false).unwrap();
    let mv = game.move_from_lan("e7e8q").unwrap();
    assert_eq!(game.move_to_san(&mv), "e8=Q+");
}

#[test]
fn test_san_promotion_capture() {
    let fen = "3n1k2/4P3/8/8/8/8/8/4K3 w - - 0 1";
    let mut game = StdGame::new(8, 8, fen, false).unwrap();
    let mv = game.move_from_lan("e7d8n").unwrap();
    assert_eq!(game.move_to_san(&mv), "exd8=N");
}

#[test]
fn test_san_checkmate() {
    // Fool's mate: 1. f3 e5 2. g4 Qh4#
    let mut game = StdGame::standard();
    game.make_move(&game.move_from_lan("f2f3").unwrap());
    game.make_move(&game.move_from_lan("e7e5").unwrap());
    game.make_move(&game.move_from_lan("g2g4").unwrap());
    let mv = game.move_from_lan("d8h4").unwrap();
    assert_eq!(game.move_to_san(&mv), "Qh4#");
}

#[test]
fn test_san_en_passant() {
    let fen = "4k3/8/8/3pP3/8/8/8/4K3 w - d6 0 1";
    let mut game = StdGame::new(8, 8, fen, false).unwrap();
    let mv = game.move_from_lan("e5d6").unwrap();
    assert_eq!(game.move_to_san(&mv), "exd6");
}

#[test]
fn test_san_from_basic() {
    let mut game = StdGame::standard();
    let mv = game.move_from_san("e4").unwrap();
    assert_eq!(mv.src, Position::new(4, 1));
    assert_eq!(mv.dst, Position::new(4, 3));
}

#[test]
fn test_san_from_knight() {
    let mut game = StdGame::standard();
    let mv = game.move_from_san("Nf3").unwrap();
    assert_eq!(mv.src, Position::new(6, 0));
    assert_eq!(mv.dst, Position::new(5, 2));
}

#[test]
fn test_san_from_castling() {
    let fen = "r3k2r/pppppppp/8/8/8/8/PPPPPPPP/R3K2R w KQkq - 0 1";
    let mut game = StdGame::new(8, 8, fen, true).unwrap();
    let mv = game.move_from_san("O-O").unwrap();
    assert!(mv.flags.contains(MoveFlags::CASTLE));
    assert!(mv.dst.col > mv.src.col);

    let mv2 = game.move_from_san("O-O-O").unwrap();
    assert!(mv2.flags.contains(MoveFlags::CASTLE));
    assert!(mv2.dst.col < mv2.src.col);
}

#[test]
fn test_san_from_castling_zeros() {
    let fen = "r3k2r/pppppppp/8/8/8/8/PPPPPPPP/R3K2R w KQkq - 0 1";
    let mut game = StdGame::new(8, 8, fen, true).unwrap();
    let mv = game.move_from_san("0-0").unwrap();
    assert!(mv.flags.contains(MoveFlags::CASTLE));
}

#[test]
fn test_san_from_promotion() {
    let fen = "k7/4P3/8/8/8/8/8/4K3 w - - 0 1";
    let mut game = StdGame::new(8, 8, fen, false).unwrap();
    let mv = game.move_from_san("e8=Q").unwrap();
    assert_eq!(mv.promotion, Some(PieceType::Queen));
    assert_eq!(mv.dst, Position::new(4, 7));
}

#[test]
fn test_san_from_with_check_suffix() {
    let mut game = StdGame::standard();
    // Should ignore the + suffix
    let mv = game.move_from_san("Nf3+").unwrap();
    assert_eq!(mv.dst, Position::new(5, 2));
}

#[test]
fn test_san_from_disambiguation() {
    let fen = "4k3/8/8/8/8/8/4K3/R6R w - - 0 1";
    let mut game = StdGame::new(8, 8, fen, false).unwrap();
    let mv = game.move_from_san("Rad1").unwrap();
    assert_eq!(mv.src, Position::new(0, 0)); // a1
    assert_eq!(mv.dst, Position::new(3, 0)); // d1
}

#[test]
fn test_san_from_error_invalid() {
    let mut game = StdGame::standard();
    assert!(game.move_from_san("Zz9").is_err());
    assert!(game.move_from_san("").is_err());
}

#[test]
fn test_san_roundtrip_all_legal_moves() {
    // From starting position, every legal move should roundtrip through SAN
    let mut game = StdGame::standard();
    let legal = game.legal_moves();
    for mv in &legal {
        let san = game.move_to_san(mv);
        let parsed = game.move_from_san(&san).unwrap_or_else(|e| {
            panic!(
                "Failed to parse SAN '{}' (from move {}): {}",
                san,
                mv.to_lan(),
                e
            )
        });
        assert_eq!(parsed.src, mv.src, "SAN roundtrip failed for {}", san);
        assert_eq!(parsed.dst, mv.dst, "SAN roundtrip failed for {}", san);
        assert_eq!(
            parsed.promotion, mv.promotion,
            "SAN roundtrip failed for {}",
            san
        );
    }
}

#[test]
fn test_san_roundtrip_midgame() {
    // Test roundtrip from a more complex position with captures, checks possible
    let fen = "r1bqkb1r/pppp1ppp/2n2n2/4p3/2B1P3/5N2/PPPP1PPP/RNBQK2R w KQkq - 4 4";
    let mut game = StdGame::new(8, 8, fen, true).unwrap();
    let legal = game.legal_moves();
    for mv in &legal {
        let san = game.move_to_san(mv);
        let parsed = game.move_from_san(&san).unwrap_or_else(|e| {
            panic!(
                "Failed to parse SAN '{}' (from {}): {}",
                san,
                mv.to_lan(),
                e
            )
        });
        assert_eq!(parsed.src, mv.src, "Roundtrip src failed for {}", san);
        assert_eq!(parsed.dst, mv.dst, "Roundtrip dst failed for {}", san);
    }
}

#[test]
fn test_san_roundtrip_random_games() {
    use rand::prelude::IndexedRandom;
    use rand::SeedableRng;

    let mut rng = rand::rngs::StdRng::seed_from_u64(123);

    for _game_num in 0..100 {
        let mut game = StdGame::standard();
        for _move_num in 0..100 {
            if game.is_over() {
                break;
            }
            let legal = game.legal_moves();
            if legal.is_empty() {
                break;
            }
            // Test roundtrip for all legal moves
            for mv in &legal {
                let san = game.move_to_san(mv);
                let parsed = game.move_from_san(&san).unwrap_or_else(|e| {
                    panic!(
                        "Game {}, move {}: Failed to parse SAN '{}' (from {}): {}",
                        _game_num,
                        _move_num,
                        san,
                        mv.to_lan(),
                        e
                    )
                });
                assert_eq!(parsed.src, mv.src);
                assert_eq!(parsed.dst, mv.dst);
                assert_eq!(parsed.promotion, mv.promotion);
            }
            let chosen = legal.choose(&mut rng).unwrap();
            game.make_move_unchecked(chosen);
        }
    }
}
