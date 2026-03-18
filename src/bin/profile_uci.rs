#![feature(generic_const_exprs)]
#![allow(incomplete_features)]

use rand::SeedableRng;
use rand::prelude::IndexedRandom;
use rand::rngs::SmallRng;
use spooky_chess::uci::UciEngine;

#[hotpath::measure]
fn play_random_game_with_stockfish(rng: &mut SmallRng) -> spooky_chess::outcome::GameOutcome {
    let mut engine = UciEngine::new("stockfish", &[]).expect("failed to spawn stockfish");
    engine.set_position_startpos();

    loop {
        if engine.is_over() {
            return engine
                .game()
                .clone()
                .outcome()
                .expect("game is over but outcome() returned None");
        }

        let moves = engine.legal_moves();

        // Ask stockfish to evaluate at depth 4
        engine.go_depth(4).expect("stockfish go_depth failed");

        // Play a random move to keep games varied
        let mv = moves
            .choose(rng)
            .expect("play_random_game_with_stockfish: legal moves list must not be empty");
        engine.make_move(mv).expect("make_move failed");
    }
}

#[hotpath::main(limit = 0)]
fn main() {
    let num_games = 50;
    let mut white_wins = 0;
    let mut black_wins = 0;
    let mut draws = 0;

    let mut rng = SmallRng::seed_from_u64(0xDEAD_BEEF);

    for _i in 0..num_games {
        let outcome = play_random_game_with_stockfish(&mut rng);

        match outcome.winner() {
            Some(spooky_chess::color::Color::White) => white_wins += 1,
            Some(spooky_chess::color::Color::Black) => black_wins += 1,
            None => draws += 1,
        }
    }

    println!("\nResults after {} games:", num_games);
    println!(
        "  White wins: {} ({:.1}%)",
        white_wins,
        white_wins as f64 / num_games as f64 * 100.0
    );
    println!(
        "  Black wins: {} ({:.1}%)",
        black_wins,
        black_wins as f64 / num_games as f64 * 100.0
    );
    println!(
        "  Draws:      {} ({:.1}%)",
        draws,
        draws as f64 / num_games as f64 * 100.0
    );
}
