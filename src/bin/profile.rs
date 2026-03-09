#![feature(generic_const_exprs)]
#![allow(incomplete_features)]

use rand::rngs::SmallRng;
use rand::seq::IndexedRandom;
use rand::SeedableRng;
use spooky_chess::game::Game;

#[hotpath::measure]
fn play_random_game(rng: &mut SmallRng) -> spooky_chess::outcome::GameOutcome {
    let mut game = Game::<8, 8>::standard();

    loop {
        if let Some(outcome) = game.outcome() {
            return outcome;
        }

        let moves = game.legal_moves();
        let mv = moves.choose(rng).unwrap();
        game.make_move_unchecked(mv);
    }
}

#[hotpath::main(limit = 0)]
fn main() {
    let num_games = 200;
    let mut white_wins = 0;
    let mut black_wins = 0;
    let mut draws = 0;

    let mut rng = SmallRng::seed_from_u64(0xDEAD_BEEF);

    for _i in 0..num_games {
        let outcome = play_random_game(&mut rng);

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
