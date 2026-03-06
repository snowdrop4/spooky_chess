use rand::seq::IndexedRandom;
use spooky_chess::game::Game;

#[hotpath::measure]
fn play_random_game() -> spooky_chess::outcome::GameOutcome {
    let mut game = Game::<1>::standard();
    let mut rng = rand::rng();

    loop {
        if let Some(outcome) = game.outcome() {
            return outcome;
        }

        let moves = game.legal_moves();
        let mv = moves.choose(&mut rng).unwrap();
        game.make_move_unchecked(mv);
    }
}

#[hotpath::main(limit = 0)]
fn main() {
    let num_games = 50;
    let mut white_wins = 0;
    let mut black_wins = 0;
    let mut draws = 0;

    for i in 0..num_games {
        let outcome = play_random_game();

        match outcome.winner() {
            Some(spooky_chess::color::Color::White) => white_wins += 1,
            Some(spooky_chess::color::Color::Black) => black_wins += 1,
            None => draws += 1,
        }

        if (i + 1) % 10 == 0 {
            println!("Played {}/{} games", i + 1, num_games);
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
