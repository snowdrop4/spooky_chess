use super::*;
use crate::color::Color;
use crate::position::Position;

use rand::SeedableRng;
use rand::prelude::IndexedRandom;
use rand::rngs::SmallRng;

type Game8x8 = Game<8, 8>;

#[test]
fn encode_decode_action_roundtrip() {
    let mut rng = SmallRng::seed_from_u64(42);

    for _game_num in 0..500 {
        let mut game = Game8x8::standard();
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

            let chosen = legal_moves
                .choose(&mut rng)
                .expect("encode_decode_action_roundtrip: legal moves list must not be empty");
            game.make_move_unchecked(chosen);
        }
    }
}

#[test]
fn apply_action_roundtrip() {
    let mut game = Game8x8::standard();

    // e2e4 as an action
    let mv = game
        .move_from_lan("e2e4")
        .expect("apply_action_roundtrip: failed to parse e2e4 LAN move");
    let action = game
        .encode_action(&mv)
        .expect("apply_action_roundtrip: failed to encode action for e2e4");
    assert!(game.apply_action(action));
    assert_eq!(game.turn(), Color::Black);

    // Verify the pawn moved
    assert!(game.board().get_piece(&Position::new(4, 1)).is_none());
    assert!(game.board().get_piece(&Position::new(4, 3)).is_some());
}
