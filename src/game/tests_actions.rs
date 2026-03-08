use rand::prelude::IndexedRandom;
use rand::SeedableRng;

#[test]
fn test_encode_decode_action_roundtrip() {
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
