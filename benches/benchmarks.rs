#![feature(generic_const_exprs)]
#![allow(incomplete_features)]

use criterion::{Criterion, criterion_group, criterion_main};
use rand::SeedableRng;
use rand::prelude::IndexedRandom;
use rand::rngs::SmallRng;
use spooky_chess::encode::encode_game_planes;
use spooky_chess::game::StandardGame;
use spooky_chess::outcome::TurnState;
use spooky_chess::uci::UciEngine;
use std::hint::black_box;

/// Play ~20 random moves on a fresh game to create a realistic mid-game position.
/// Uses a fixed seed for reproducibility across benchmark runs.
fn setup_midgame() -> StandardGame {
    let mut game = StandardGame::standard();
    let mut rng = SmallRng::seed_from_u64(42);
    for _ in 0..20 {
        let moves = game.legal_moves();
        if moves.is_empty() {
            break;
        }
        let mv = moves
            .choose(&mut rng)
            .expect("setup_midgame: legal moves must not be empty for random choice");
        game.make_move_unchecked(mv);
    }
    game
}

// ---------------------------------------------------------------------------
// Microbenchmarks
// ---------------------------------------------------------------------------

fn bench_legal_moves(c: &mut Criterion) {
    let mut game = setup_midgame();
    c.bench_function("legal_moves", |b| b.iter(|| black_box(game.legal_moves())));
}

fn bench_make_move(c: &mut Criterion) {
    let mut game = setup_midgame();
    let moves = game.legal_moves();
    let mv = *moves
        .first()
        .expect("bench_make_move: legal moves must not be empty");
    c.bench_function("make_move", |b| {
        b.iter_batched(
            || game.clone(),
            |mut g| {
                black_box(g.make_move_unchecked(&mv));
            },
            criterion::BatchSize::SmallInput,
        )
    });
}

fn bench_make_unmake(c: &mut Criterion) {
    let mut game = setup_midgame();
    let moves = game.legal_moves();
    let mv = *moves
        .first()
        .expect("bench_make_unmake: legal moves must not be empty");
    c.bench_function("make_unmake", |b| {
        b.iter_batched(
            || game.clone(),
            |mut g| {
                g.make_move_unchecked(&mv);
                black_box(g.unmake_move());
            },
            criterion::BatchSize::SmallInput,
        )
    });
}

fn bench_encode_game_planes(c: &mut Criterion) {
    let game = setup_midgame();
    c.bench_function("encode_game_planes", |b| {
        b.iter_batched(
            || game.clone(),
            |mut g| black_box(encode_game_planes(&mut g)),
            criterion::BatchSize::SmallInput,
        )
    });
}

fn bench_outcome(c: &mut Criterion) {
    let mut game = setup_midgame();
    c.bench_function("outcome", |b| b.iter(|| black_box(game.outcome())));
}

// ---------------------------------------------------------------------------
// Integration benchmarks
// ---------------------------------------------------------------------------

fn bench_random_playout(c: &mut Criterion) {
    c.bench_function("random_playout", |b| {
        b.iter(|| {
            let mut game = StandardGame::standard();
            let mut rng = SmallRng::seed_from_u64(123);
            loop {
                match game.turn_state() {
                    TurnState::Over(outcome) => break black_box(Some(outcome)),
                    TurnState::Ongoing(moves) => {
                        let mv = moves
                            .choose(&mut rng)
                            .expect("bench_random_playout: legal moves must not be empty");
                        game.make_move_unchecked(mv);
                    }
                }
            }
        })
    });
}

fn bench_self_play_step(c: &mut Criterion) {
    let game = setup_midgame();
    c.bench_function("self_play_step", |b| {
        b.iter_batched(
            || game.clone(),
            |mut g| {
                let moves = g.legal_moves();
                let _planes = encode_game_planes(&mut g);
                let mv = moves
                    .first()
                    .expect("bench_self_play_step: legal moves must not be empty");
                g.make_move_unchecked(mv);
                black_box(&g);
            },
            criterion::BatchSize::SmallInput,
        )
    });
}

criterion_group!(
    name = benches;
    config = Criterion::default().sample_size(100_000);
    targets =
        bench_legal_moves,
        bench_make_move,
        bench_make_unmake,
        bench_encode_game_planes,
        bench_outcome,
        bench_self_play_step,
);
fn bench_random_playout_stockfish(c: &mut Criterion) {
    c.bench_function("random_playout_stockfish_depth4", |b| {
        b.iter(|| {
            let mut engine = UciEngine::new("stockfish", &[]).expect("failed to spawn stockfish");
            engine.set_position_startpos();
            let mut rng = SmallRng::seed_from_u64(456);
            loop {
                match engine.game().clone().turn_state() {
                    TurnState::Over(outcome) => break black_box(Some(outcome)),
                    TurnState::Ongoing(moves) => {
                        // Ask stockfish to evaluate at depth 4
                        let result = engine.go_depth(4).expect("stockfish go_depth failed");
                        black_box(&result);
                        // Play a random move (not stockfish's best) to keep games varied
                        let mv = moves.choose(&mut rng).expect(
                            "bench_random_playout_stockfish: legal moves must not be empty",
                        );
                        engine.make_move(mv).expect("make_move failed");
                    }
                }
            }
        })
    });
}

criterion_group!(
    name = playouts;
    config = Criterion::default().sample_size(10_000);
    targets =
        bench_random_playout,
);
criterion_group!(
    name = stockfish_playouts;
    config = Criterion::default().sample_size(10);
    targets =
        bench_random_playout_stockfish,
);
criterion_main!(benches, playouts, stockfish_playouts);
