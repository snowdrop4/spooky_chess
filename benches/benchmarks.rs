use criterion::{criterion_group, criterion_main, Criterion};
use rand::prelude::IndexedRandom;
use rand::rngs::StdRng;
use rand::SeedableRng;
use spooky_chess::encode::encode_game_planes;
use spooky_chess::game::StandardGame;
use std::hint::black_box;

/// Play ~20 random moves on a fresh game to create a realistic mid-game position.
/// Uses a fixed seed for reproducibility across benchmark runs.
fn setup_midgame() -> StandardGame {
    let mut game = StandardGame::standard();
    let mut rng = StdRng::seed_from_u64(42);
    for _ in 0..20 {
        let moves = game.legal_moves();
        if moves.is_empty() {
            break;
        }
        let mv = moves.choose(&mut rng).unwrap();
        game.make_move(mv);
    }
    game
}

// ---------------------------------------------------------------------------
// Microbenchmarks
// ---------------------------------------------------------------------------

fn bench_legal_moves(c: &mut Criterion) {
    let game = setup_midgame();
    c.bench_function("legal_moves", |b| b.iter(|| black_box(game.legal_moves())));
}

fn bench_make_move(c: &mut Criterion) {
    let game = setup_midgame();
    let moves = game.legal_moves();
    let mv = *moves.first().unwrap();
    c.bench_function("make_move", |b| {
        b.iter_batched(
            || game.clone(),
            |mut g| {
                black_box(g.make_move(&mv));
            },
            criterion::BatchSize::SmallInput,
        )
    });
}

fn bench_make_unmake(c: &mut Criterion) {
    let game = setup_midgame();
    let moves = game.legal_moves();
    let mv = *moves.first().unwrap();
    c.bench_function("make_unmake", |b| {
        b.iter_batched(
            || game.clone(),
            |mut g| {
                g.make_move(&mv);
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
    let game = setup_midgame();
    c.bench_function("outcome", |b| b.iter(|| black_box(game.outcome())));
}

// ---------------------------------------------------------------------------
// Integration benchmarks
// ---------------------------------------------------------------------------

fn bench_random_playout(c: &mut Criterion) {
    c.bench_function("random_playout", |b| {
        b.iter(|| {
            let mut game = StandardGame::standard();
            let mut rng = StdRng::seed_from_u64(123);
            while !game.is_over() {
                let moves = game.legal_moves();
                if moves.is_empty() {
                    break;
                }
                let mv = moves.choose(&mut rng).unwrap();
                game.make_move(mv);
            }
            black_box(game.outcome())
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
                let mv = moves.first().unwrap();
                g.make_move(mv);
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
criterion_group!(
    name = playouts;
    config = Criterion::default().sample_size(10_000);
    targets =
        bench_random_playout,
);
criterion_main!(benches, playouts);
