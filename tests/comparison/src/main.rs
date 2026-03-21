#![feature(generic_const_exprs)]
#![allow(incomplete_features)]

use rand::SeedableRng;
use rand::prelude::IndexedRandom;
use rand::rngs::SmallRng;
use spooky_chess::game::StandardGame;
use spooky_chess::outcome::TurnState;
use std::time::Instant;

fn simulate_game(rng: &mut SmallRng, moves_count: usize) -> usize {
    let mut game = StandardGame::standard();
    let mut moves_made = 0;

    while moves_made < moves_count {
        match game.turn_state() {
            TurnState::Over(_) => break,
            TurnState::Ongoing(legal_moves) => {
                game.make_move_unchecked(
                    legal_moves
                        .choose(rng)
                        .expect("simulate_game: legal moves list must not be empty"),
                );
                moves_made += 1;
            }
        }
    }

    moves_made
}

fn main() {
    let game_count = 50000;
    let move_count = 100;

    let mut rng = SmallRng::seed_from_u64(0xDEAD_BEEF);
    let mut moves_made = 0;

    let start = Instant::now();

    for _ in 0..game_count {
        moves_made += simulate_game(&mut rng, move_count);
    }

    let elapsed = start.elapsed();
    let moves_per_s = moves_made as f64 / elapsed.as_secs_f64();

    println!("{game_count} random game playouts");
    println!("  spooky_chess (Rust Bindings):");
    println!("    moves:   {moves_made}");
    println!("    time:    {elapsed:.2?}");
    println!("    moves/s: {moves_per_s:.2}");
}
