use crate::color::Color;
use crate::directions::{KNIGHT_DELTAS, direction_index};
use crate::game::Game;
use crate::r#move::Move;
use crate::pieces::PieceType;

/// Number of planes for piece positions (6 for WHITE + 6 for BLACK)
pub const PIECE_PLANES: usize = 6 + 6;

/// Number of constant planes (2 repetitions + 1 color + 1 total move + 4 castling + 1 no-progress)
pub const CONSTANT_PLANES: usize = 2 + 1 + 1 + 4 + 1;

/// Number of positions in the game history to encode
pub const HISTORY_LENGTH: usize = 8;

/// Total number of input planes for the neural network
pub const TOTAL_INPUT_PLANES: usize = (HISTORY_LENGTH * PIECE_PLANES) + CONSTANT_PLANES;

/// Number of directions for horizontal/vertical/diagonal moves (N, NE, E, SE, S, SW, W, NW)
pub const NUM_DIRECTIONS: usize = 8;

/// Number of knight move patterns
pub const NUM_KNIGHT_DELTAS: usize = 8;

/// Number of underpromotion directions (left diagonal, straight, right diagonal)
pub const NUM_UNDERPROMO_DIRECTIONS: usize = 3;

/// Number of underpromotion piece types (knight, bishop, rook - excluding queen)
pub const NUM_UNDERPROMO_PIECES: usize = 3;

/// Number of promotion move directions (forward, backward)
pub const NUM_PROMOTION_ORIENTATIONS: usize = 2;

/// Normalization divisor for fullmove number in the NN input planes.
const FULLMOVE_SCALE: f32 = 100.0;

/// Normalization divisor for halfmove clock (no-progress count) in the NN input planes.
const HALFMOVE_SCALE: f32 = 50.0;

/// Encode the full game state into a flat f32 array for efficient transfer to Python/numpy
/// Returns (flat_data, num_planes, height, width), where flat_data is in row-major order
#[hotpath::measure]
pub fn encode_game_planes<const W: usize, const H: usize>(
    game: &mut Game<W, H>,
) -> (Vec<f32>, usize, usize, usize)
where
    [(); (W * H).div_ceil(64)]:,
{
    let num_planes = TOTAL_INPUT_PLANES;
    let board_size = H * W;
    let total_size = num_planes * board_size;
    let mut data = vec![0.0f32; total_size];

    let perspective = game.turn();
    let opponent = perspective.opposite();

    let history_len = game.move_count();
    let steps_back = (HISTORY_LENGTH - 1).min(history_len);

    let moves_to_replay: Vec<Move> = game.move_history()[(history_len - steps_back)..]
        .iter()
        .map(|e| e.mv)
        .collect();

    // T=0: current position
    fill_chess_planes::<W, H>(&mut data, game, perspective, 0);

    // T=1..steps_back: walk backward through history
    for t in 1..=steps_back {
        game.unmake_move();
        fill_chess_planes::<W, H>(&mut data, game, perspective, t);
    }

    // Replay saved moves to restore game state
    for mv in &moves_to_replay {
        game.make_move_unchecked(mv);
    }

    debug_assert_eq!(
        game.move_count(),
        history_len,
        "game state not fully restored after encode: move_count {} != original {}",
        game.move_count(),
        history_len,
    );

    // Constant plane layout (relative to constant_start):
    const PLANE_REPETITION_1: usize = 0;
    const PLANE_REPETITION_2: usize = 1;
    const PLANE_COLOR: usize = 2;
    const PLANE_MOVE_COUNT: usize = 3;
    const PLANE_P1_KINGSIDE: usize = 4;
    const PLANE_P1_QUEENSIDE: usize = 5;
    const PLANE_P2_KINGSIDE: usize = 6;
    const PLANE_P2_QUEENSIDE: usize = 7;
    const PLANE_NO_PROGRESS: usize = 8;

    let constant_start = HISTORY_LENGTH * PIECE_PLANES;

    // Repetition count planes - zeros for now (PLANE_REPETITION_1, PLANE_REPETITION_2)
    let _ = (PLANE_REPETITION_1, PLANE_REPETITION_2);

    // Color plane
    let color_value = if perspective == Color::White {
        1.0
    } else {
        0.0
    };
    fill_constant_plane(
        &mut data,
        constant_start + PLANE_COLOR,
        color_value,
        board_size,
    );

    // Total move count plane
    let move_count = game.fullmove_number() as f32 / FULLMOVE_SCALE;
    fill_constant_plane(
        &mut data,
        constant_start + PLANE_MOVE_COUNT,
        move_count,
        board_size,
    );

    // Castling rights (4 planes)
    let castling_rights = game.castling_rights();

    let p1_kingside = if castling_rights.has_kingside(perspective) {
        1.0
    } else {
        0.0
    };
    fill_constant_plane(
        &mut data,
        constant_start + PLANE_P1_KINGSIDE,
        p1_kingside,
        board_size,
    );

    let p1_queenside = if castling_rights.has_queenside(perspective) {
        1.0
    } else {
        0.0
    };
    fill_constant_plane(
        &mut data,
        constant_start + PLANE_P1_QUEENSIDE,
        p1_queenside,
        board_size,
    );

    let p2_kingside = if castling_rights.has_kingside(opponent) {
        1.0
    } else {
        0.0
    };
    fill_constant_plane(
        &mut data,
        constant_start + PLANE_P2_KINGSIDE,
        p2_kingside,
        board_size,
    );

    let p2_queenside = if castling_rights.has_queenside(opponent) {
        1.0
    } else {
        0.0
    };
    fill_constant_plane(
        &mut data,
        constant_start + PLANE_P2_QUEENSIDE,
        p2_queenside,
        board_size,
    );

    // No-progress count plane
    let no_progress = game.halfmove_clock() as f32 / HALFMOVE_SCALE;
    fill_constant_plane(
        &mut data,
        constant_start + PLANE_NO_PROGRESS,
        no_progress,
        board_size,
    );

    (data, num_planes, H, W)
}

#[hotpath::measure]
fn fill_constant_plane(data: &mut [f32], plane: usize, value: f32, board_size: usize) {
    let offset = plane * board_size;
    data[offset..offset + board_size].fill(value);
}

#[inline]
fn piece_type_plane_index(pt: PieceType) -> usize {
    match pt {
        PieceType::Pawn => 0,
        PieceType::Knight => 1,
        PieceType::Bishop => 2,
        PieceType::Rook => 3,
        PieceType::Queen => 4,
        PieceType::King => 5,
    }
}

#[hotpath::measure]
fn fill_chess_planes<const W: usize, const H: usize>(
    data: &mut [f32],
    game: &Game<W, H>,
    perspective: Color,
    t: usize,
) where
    [(); (W * H).div_ceil(64)]:,
{
    let board_size = H * W;
    debug_assert!(
        t < HISTORY_LENGTH,
        "history timestep t={} exceeds HISTORY_LENGTH={}",
        t,
        HISTORY_LENGTH,
    );
    let base_plane = t * PIECE_PLANES;

    for (pos, piece) in game.pieces_iter(perspective) {
        let plane_idx = piece_type_plane_index(piece.piece_type);
        let offset = (base_plane + plane_idx) * board_size;
        let idx = pos.to_index(W);
        debug_assert!(
            idx < board_size,
            "piece position index {} exceeds board_size {}",
            idx,
            board_size,
        );
        data[offset + idx] = 1.0;
    }

    for (pos, piece) in game.pieces_iter(perspective.opposite()) {
        let plane_idx = piece_type_plane_index(piece.piece_type);
        let offset = (base_plane + 6 + plane_idx) * board_size;
        let idx = pos.to_index(W);
        debug_assert!(
            idx < board_size,
            "piece position index {} exceeds board_size {}",
            idx,
            board_size,
        );
        data[offset + idx] = 1.0;
    }
}

/// Encode a move as a full action index (plane * board_size + src_index)
#[hotpath::measure]
pub fn encode_action(move_: &Move, width: usize, height: usize) -> Option<usize> {
    debug_assert!(
        usize::from(move_.src.col) < width && usize::from(move_.src.row) < height,
        "encode_action: move src ({},{}) out of bounds for {}x{} board",
        move_.src.col,
        move_.src.row,
        width,
        height,
    );
    debug_assert!(
        usize::from(move_.dst.col) < width && usize::from(move_.dst.row) < height,
        "encode_action: move dst ({},{}) out of bounds for {}x{} board",
        move_.dst.col,
        move_.dst.row,
        width,
        height,
    );
    let plane = encode_move_plane(move_, width, height)?;
    let board_size = width * height;
    let src_index = usize::from(move_.src.row) * width + usize::from(move_.src.col);
    Some(plane * board_size + src_index)
}

/// Get the total number of action indices for a given board size
#[hotpath::measure]
pub fn get_total_actions(width: usize, height: usize) -> usize {
    get_move_planes_count(width, height) * width * height
}

/// Encode a move as a plane index for the policy head
/// Move planes encode the movement pattern:
/// - Horizontal/vertical/diagonal moves, for all non-knight pieces,
///   in 8 directions (N, NE, E, SE, S, SW, W, NW) up to max distance
/// - L-shaped moves for knights, in 8 directions
/// - Underpromotions (3 directions × 3 piece types, excluding queen)
#[hotpath::measure]
pub(crate) fn encode_move_plane(move_: &Move, width: usize, height: usize) -> Option<usize> {
    let src = move_.src;
    let dst = move_.dst;
    let dx = dst.col as i32 - src.col as i32;
    let dy = dst.row as i32 - src.row as i32;

    let max_distance = width.max(height) - 1;

    // L-shaped moves for knights
    for (i, &(kdx, kdy)) in KNIGHT_DELTAS.iter().enumerate() {
        if dx == kdx && dy == kdy {
            let knight_planes_start = NUM_DIRECTIONS * max_distance;
            return Some(knight_planes_start + i);
        }
    }

    // Underpromotions (only for non-queen promotions)
    // Note: underpromotions are forward by 1 row only (dy = ±1 depending on perspective)
    if let Some(promo) = move_.promotion
        && promo != PieceType::Queen
        && dy.abs() == 1
    {
        let direction_idx = if dx == -1 {
            0 // left diagonal
        } else if dx == 0 {
            1 // straight
        } else if dx == 1 {
            2 // right diagonal
        } else {
            return None;
        };

        let piece_idx = match promo {
            PieceType::Knight => 0,
            PieceType::Bishop => 1,
            PieceType::Rook => 2,
            _ => return None,
        };

        // Store which direction (forward/backward) in the encoding
        let knight_planes_start = NUM_DIRECTIONS * max_distance;
        let underpromo_planes_start = knight_planes_start + NUM_KNIGHT_DELTAS;
        let dir_offset = if dy > 0 {
            0
        } else {
            NUM_UNDERPROMO_DIRECTIONS * NUM_UNDERPROMO_PIECES
        };
        return Some(
            underpromo_planes_start
                + dir_offset
                + direction_idx * NUM_UNDERPROMO_PIECES
                + piece_idx,
        );
    }

    // Horizontal/vertical/diagonal moves for all non-knight pieces
    // Verify it's actually a straight/diagonal move (not an arbitrary direction)
    let is_straight_or_diagonal = (dx == 0) != (dy == 0)  // straight
        || (dx.abs() == dy.abs() && dx != 0); // diagonal

    let direction = if is_straight_or_diagonal {
        direction_index(dx, dy)
    } else {
        None
    };

    direction.and_then(|dir| {
        let distance = dx.abs().max(dy.abs()) as usize;
        if distance > 0 && distance <= max_distance {
            Some(dir * max_distance + (distance - 1))
        } else {
            None
        }
    })
}

/// Decode a plane index back to move deltas
/// Returns (dx, dy, promotion) for the given plane index and board dimensions
#[hotpath::measure]
pub(crate) fn decode_move_plane(
    plane_idx: usize,
    width: usize,
    height: usize,
) -> Option<(i32, i32, Option<PieceType>)> {
    let max_distance = width.max(height) - 1;
    let straight_diagonal_planes = NUM_DIRECTIONS * max_distance;
    let knight_planes_start = straight_diagonal_planes;
    let underpromo_planes_start = knight_planes_start + NUM_KNIGHT_DELTAS;

    if plane_idx < straight_diagonal_planes {
        // Horizontal/vertical/diagonal moves for all non-knight pieces
        let direction = plane_idx / max_distance;
        let distance = (plane_idx % max_distance) + 1;

        let (dx, dy) = match direction {
            0 => (0, distance as i32),                     // N
            1 => (distance as i32, distance as i32),       // NE
            2 => (distance as i32, 0),                     // E
            3 => (distance as i32, -(distance as i32)),    // SE
            4 => (0, -(distance as i32)),                  // S
            5 => (-(distance as i32), -(distance as i32)), // SW
            6 => (-(distance as i32), 0),                  // W
            7 => (-(distance as i32), distance as i32),    // NW
            _ => return None,
        };

        Some((dx, dy, None))
    } else if plane_idx < underpromo_planes_start {
        // L-shaped moves for knights
        let knight_idx = plane_idx - knight_planes_start;
        KNIGHT_DELTAS
            .get(knight_idx)
            .map(|&(dx, dy)| (dx, dy, None))
    } else {
        // Underpromotion
        let underpromo_idx = plane_idx - underpromo_planes_start;
        let total_underpromo_planes =
            NUM_UNDERPROMO_DIRECTIONS * NUM_UNDERPROMO_PIECES * NUM_PROMOTION_ORIENTATIONS;
        if underpromo_idx < total_underpromo_planes {
            let forward_underpromo_planes = NUM_UNDERPROMO_DIRECTIONS * NUM_UNDERPROMO_PIECES;
            let dy = if underpromo_idx < forward_underpromo_planes {
                1
            } else {
                -1
            };
            let idx_within_direction = underpromo_idx % forward_underpromo_planes;
            let direction_idx = idx_within_direction / NUM_UNDERPROMO_PIECES;
            let piece_idx = idx_within_direction % NUM_UNDERPROMO_PIECES;

            let dx = match direction_idx {
                0 => -1, // left diagonal
                1 => 0,  // straight
                2 => 1,  // right diagonal
                _ => return None,
            };

            let promo = match piece_idx {
                0 => Some(PieceType::Knight),
                1 => Some(PieceType::Bishop),
                2 => Some(PieceType::Rook),
                _ => return None,
            };

            Some((dx, dy, promo))
        } else {
            None
        }
    }
}

/// Get the total number of move policy planes for a given board dimensions
#[hotpath::measure]
pub fn get_move_planes_count(width: usize, height: usize) -> usize {
    let max_distance = width.max(height) - 1;
    let straight_diagonal_planes = NUM_DIRECTIONS * max_distance;
    let knight_planes = NUM_KNIGHT_DELTAS;
    let underpromo_planes =
        NUM_UNDERPROMO_DIRECTIONS * NUM_UNDERPROMO_PIECES * NUM_PROMOTION_ORIENTATIONS;

    straight_diagonal_planes + knight_planes + underpromo_planes
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::position::Position;

    fn get_plane_value(
        data: &[f32],
        plane: usize,
        row: usize,
        col: usize,
        height: usize,
        width: usize,
    ) -> f32 {
        data[plane * height * width + row * width + col]
    }

    #[test]
    fn test_standard_game_encode_initial_position() {
        let mut game = Game::standard();
        let (data, num_planes, height, width) = encode_game_planes(&mut game);

        // Should have TOTAL_INPUT_PLANES planes
        assert_eq!(num_planes, TOTAL_INPUT_PLANES);
        assert_eq!(height, 8);
        assert_eq!(width, 8);
        assert_eq!(data.len(), num_planes * height * width);

        // Check white pawns (plane 0) - should be on row 1
        for col in 0..8 {
            assert_eq!(
                get_plane_value(&data, 0, 1, col, height, width),
                1.0,
                "White pawn at row 1, col {}",
                col
            );
        }

        // Check white king (plane 5) at e1 (col 4, row 0)
        assert_eq!(
            get_plane_value(&data, 5, 0, 4, height, width),
            1.0,
            "White king at e1"
        );
    }

    #[test]
    fn test_standard_game_encode_game() {
        let mut game = Game::standard();
        let (data, num_planes, height, width) = encode_game_planes(&mut game);

        // Should have TOTAL_INPUT_PLANES planes
        assert_eq!(num_planes, TOTAL_INPUT_PLANES);
        assert_eq!(height, 8);
        assert_eq!(width, 8);
        assert_eq!(data.len(), num_planes * height * width);

        // Color plane should be all 1.0 (white's turn)
        let color_plane_idx = HISTORY_LENGTH * PIECE_PLANES + 2; // After board history and repetitions
        assert_eq!(
            get_plane_value(&data, color_plane_idx, 0, 0, height, width),
            1.0
        );
    }

    #[test]
    fn test_encode_move_plane_horizontal_vertical() {
        use crate::r#move::MoveFlags;

        // Test vertical move (rook moving north)
        let move_north =
            Move::from_position(Position::new(3, 0), Position::new(3, 4), MoveFlags::empty());
        let encoded = encode_move_plane(&move_north, 8, 8);
        assert_eq!(encoded, Some(3)); // North direction, distance 4

        // Test horizontal move (rook moving east)
        let move_east =
            Move::from_position(Position::new(0, 3), Position::new(5, 3), MoveFlags::empty());
        let encoded = encode_move_plane(&move_east, 8, 8);
        assert_eq!(encoded, Some(2 * 7 + 4)); // East direction, distance 5
    }

    #[test]
    fn test_encode_move_plane_diagonal() {
        use crate::r#move::MoveFlags;

        // Test diagonal move (bishop moving NE)
        let move_ne =
            Move::from_position(Position::new(1, 1), Position::new(4, 4), MoveFlags::empty());
        let encoded = encode_move_plane(&move_ne, 8, 8);
        assert_eq!(encoded, Some(7 + 2)); // NE direction, distance 3

        // Test diagonal move (bishop moving SW)
        let move_sw =
            Move::from_position(Position::new(5, 5), Position::new(3, 3), MoveFlags::empty());
        let encoded = encode_move_plane(&move_sw, 8, 8);
        assert_eq!(encoded, Some(5 * 7 + 1)); // SW direction, distance 2
    }

    #[test]
    fn test_encode_move_plane_knight() {
        use crate::r#move::MoveFlags;

        // Test knight move (1, 2)
        let move_knight =
            Move::from_position(Position::new(3, 3), Position::new(4, 5), MoveFlags::empty());
        let encoded = encode_move_plane(&move_knight, 8, 8);
        assert_eq!(encoded, Some(8 * 7)); // First knight pattern

        // Test knight move (2, -1)
        let move_knight2 =
            Move::from_position(Position::new(3, 3), Position::new(5, 2), MoveFlags::empty());
        let encoded = encode_move_plane(&move_knight2, 8, 8);
        assert_eq!(encoded, Some(8 * 7 + 2)); // Third knight pattern
    }

    #[test]
    fn test_encode_move_plane_underpromotion() {
        use crate::r#move::MoveFlags;

        // Test straight underpromotion to knight (forward)
        let move_promo = Move::from_position_with_promotion(
            Position::new(3, 6),
            Position::new(3, 7),
            MoveFlags::PROMOTION,
            PieceType::Knight,
        );
        let encoded = encode_move_plane(&move_promo, 8, 8);
        assert_eq!(encoded, Some((8 * 7 + 8) + 3)); // Forward, straight, knight

        // Test diagonal underpromotion to rook (forward)
        let move_promo2 = Move::from_position_with_promotion(
            Position::new(3, 6),
            Position::new(4, 7),
            MoveFlags::PROMOTION,
            PieceType::Rook,
        );
        let encoded = encode_move_plane(&move_promo2, 8, 8);
        assert_eq!(encoded, Some((8 * 7 + 8) + 2 * 3 + 2)); // Forward, right diagonal, rook

        // Test straight underpromotion to bishop (backward)
        let move_promo3 = Move::from_position_with_promotion(
            Position::new(3, 1),
            Position::new(3, 0),
            MoveFlags::PROMOTION,
            PieceType::Bishop,
        );
        let encoded = encode_move_plane(&move_promo3, 8, 8);
        assert_eq!(encoded, Some(8 * 7 + 8 + 9 + 3 + 1)); // Backward, straight, bishop
    }

    #[test]
    fn test_encode_move_plane_queen_promotion() {
        use crate::r#move::MoveFlags;

        // Queen promotions should use regular straight/diagonal encoding
        let move_promo = Move::from_position_with_promotion(
            Position::new(3, 6),
            Position::new(3, 7),
            MoveFlags::PROMOTION,
            PieceType::Queen,
        );
        let encoded = encode_move_plane(&move_promo, 8, 8);
        assert_eq!(encoded, Some(0)); // North direction, distance 1
    }

    #[test]
    fn test_decode_move_plane_horizontal_vertical() {
        // North, distance 4
        let decoded = decode_move_plane(3, 8, 8);
        assert_eq!(decoded, Some((0, 4, None)));

        // East, distance 5
        let decoded = decode_move_plane(2 * 7 + 4, 8, 8);
        assert_eq!(decoded, Some((5, 0, None)));

        // South, distance 2
        let decoded = decode_move_plane(4 * 7 + 1, 8, 8);
        assert_eq!(decoded, Some((0, -2, None)));
    }

    #[test]
    fn test_decode_move_plane_diagonal() {
        // NE, distance 3
        let decoded = decode_move_plane(7 + 2, 8, 8);
        assert_eq!(decoded, Some((3, 3, None)));

        // SW, distance 2
        let decoded = decode_move_plane(5 * 7 + 1, 8, 8);
        assert_eq!(decoded, Some((-2, -2, None)));
    }

    #[test]
    fn test_decode_move_plane_knight() {
        // First knight pattern (1, 2)
        let decoded = decode_move_plane(8 * 7, 8, 8);
        assert_eq!(decoded, Some((1, 2, None)));

        // Third knight pattern (2, -1)
        let decoded = decode_move_plane(8 * 7 + 2, 8, 8);
        assert_eq!(decoded, Some((2, -1, None)));
    }

    #[test]
    fn test_decode_move_plane_underpromotion() {
        // Forward, straight, knight
        let decoded = decode_move_plane(8 * 7 + 8 + 3, 8, 8);
        assert_eq!(decoded, Some((0, 1, Some(PieceType::Knight))));

        // Forward, right diagonal, rook
        let decoded = decode_move_plane(8 * 7 + 8 + 2 * 3 + 2, 8, 8);
        assert_eq!(decoded, Some((1, 1, Some(PieceType::Rook))));

        // Backward, straight, bishop
        let decoded = decode_move_plane(8 * 7 + 8 + 9 + 3 + 1, 8, 8);
        assert_eq!(decoded, Some((0, -1, Some(PieceType::Bishop))));
    }

    #[test]
    fn test_encode_decode_roundtrip() {
        use crate::r#move::MoveFlags;

        let moves = vec![
            Move::from_position(Position::new(0, 0), Position::new(0, 5), MoveFlags::empty()),
            Move::from_position(Position::new(2, 2), Position::new(5, 5), MoveFlags::empty()),
            Move::from_position(Position::new(3, 3), Position::new(4, 5), MoveFlags::empty()),
            Move::from_position_with_promotion(
                Position::new(3, 6),
                Position::new(3, 7),
                MoveFlags::PROMOTION,
                PieceType::Bishop,
            ),
        ];

        for move_ in moves {
            let encoded = encode_move_plane(&move_, 8, 8).expect("Failed to encode move");
            let (dx, dy, promo) = decode_move_plane(encoded, 8, 8).expect("Failed to decode");

            assert_eq!(dx, move_.dst.col as i32 - move_.src.col as i32);
            assert_eq!(dy, move_.dst.row as i32 - move_.src.row as i32);
            assert_eq!(promo, move_.promotion.filter(|&p| p != PieceType::Queen));
        }
    }

    #[test]
    fn test_get_move_planes_count() {
        // For 2x2 board: (8 * 1) + 8 + 18 = 34
        assert_eq!(get_move_planes_count(2, 2), 34);

        // For 6x6 board: (8 * 5) + 8 + 18 = 66
        assert_eq!(get_move_planes_count(6, 6), 66);

        // For 8x8 board: (8 * 7) + 8 + 18 = 82
        assert_eq!(get_move_planes_count(8, 8), 82);
    }

    #[test]
    fn test_get_total_actions() {
        // For 8x8 board: 82 * 64 = 5248
        assert_eq!(get_total_actions(8, 8), 5248);

        // For 6x6 board: 66 * 36 = 2376
        assert_eq!(get_total_actions(6, 6), 2376);
    }

    #[test]
    fn test_fuzz_move_encoding_random_games() {
        use rand::SeedableRng;
        use rand::prelude::IndexedRandom;
        use rand::rngs::SmallRng;
        use std::sync::Arc;
        use std::sync::atomic::{AtomicU64, Ordering};
        use std::thread;

        let num_games = 5_000;
        let num_threads = std::thread::available_parallelism()
            .map(|n| n.get())
            .unwrap_or(1);
        let games_per_thread = num_games / num_threads;

        let total_moves_played = Arc::new(AtomicU64::new(0));
        let total_moves_tested = Arc::new(AtomicU64::new(0));

        let mut handles = vec![];

        for thread_id in 0..num_threads {
            let moves_played = Arc::clone(&total_moves_played);
            let moves_tested = Arc::clone(&total_moves_tested);

            let handle = thread::spawn(move || {
                let mut rng = SmallRng::seed_from_u64(thread_id as u64);
                let mut thread_moves_played = 0u64;
                let mut thread_moves_tested = 0u64;

                for _game_num in 0..games_per_thread {
                    let mut game = Game::standard();
                    let max_moves = 200;

                    for _move_num in 0..max_moves {
                        if game.is_over() {
                            break;
                        }

                        let legal_moves = game.legal_moves();
                        if legal_moves.is_empty() {
                            break;
                        }

                        // Test encoding for all legal moves
                        let width = 8;
                        let height = 8;
                        let total_actions = get_total_actions(width, height);
                        let mut seen_actions = std::collections::HashSet::new();

                        for move_ in &legal_moves {
                            // Test plane encoding
                            let encoded = encode_move_plane(move_, width, height);
                            assert!(
                                encoded.is_some(),
                                "Failed to encode move {} in position {}",
                                move_.to_lan(),
                                game.to_fen()
                            );

                            let plane_idx = encoded.expect(
                                "test_fuzz_move_encoding_random_games: failed to encode move plane",
                            );
                            let decoded = decode_move_plane(plane_idx, width, height);
                            assert!(
                                decoded.is_some(),
                                "Failed to decode plane {} for move {}",
                                plane_idx,
                                move_.to_lan()
                            );

                            let (dx, dy, promo) = decoded.expect(
                                "test_fuzz_move_encoding_random_games: failed to decode move plane",
                            );

                            // Verify deltas
                            let expected_dx = move_.dst.col as i32 - move_.src.col as i32;
                            let expected_dy = move_.dst.row as i32 - move_.src.row as i32;

                            assert_eq!(
                                dx,
                                expected_dx,
                                "Move {}: decoded dx {} != expected {}",
                                move_.to_lan(),
                                dx,
                                expected_dx
                            );
                            assert_eq!(
                                dy,
                                expected_dy,
                                "Move {}: decoded dy {} != expected {}",
                                move_.to_lan(),
                                dy,
                                expected_dy
                            );

                            // Verify promotion (queen promotions should decode as None)
                            if let Some(move_promo) = move_.promotion {
                                if move_promo != PieceType::Queen {
                                    assert_eq!(
                                        promo,
                                        Some(move_promo),
                                        "Move {}: decoded promotion {:?} != expected {:?}",
                                        move_.to_lan(),
                                        promo,
                                        Some(move_promo)
                                    );
                                } else {
                                    assert_eq!(
                                        promo,
                                        None,
                                        "Move {}: queen promotion should decode as None, got {:?}",
                                        move_.to_lan(),
                                        promo
                                    );
                                }
                            } else {
                                assert_eq!(
                                    promo,
                                    None,
                                    "Move {}: expected no promotion, got {:?}",
                                    move_.to_lan(),
                                    promo
                                );
                            }

                            // Test full action encoding
                            let action = encode_action(move_, width, height);
                            assert!(
                                action.is_some(),
                                "Failed to encode action for move {} in position {}",
                                move_.to_lan(),
                                game.to_fen()
                            );
                            let action_idx = action.expect("test_fuzz_move_encoding_random_games: failed to encode full action");
                            assert!(
                                action_idx < total_actions,
                                "Action index {} out of range (total: {}) for move {}",
                                action_idx,
                                total_actions,
                                move_.to_lan()
                            );

                            // Verify no action collisions
                            assert!(
                                seen_actions.insert(action_idx),
                                "Action collision: action {} for move {} in position {}",
                                action_idx,
                                move_.to_lan(),
                                game.to_fen()
                            );

                            // Verify action roundtrip: decode action back to src/dst
                            let decoded_plane = action_idx / (width * height);
                            let src_index = action_idx % (width * height);
                            let decoded_src_col = src_index % width;
                            let decoded_src_row = src_index / width;
                            assert_eq!(
                                decoded_src_col,
                                usize::from(move_.src.col),
                                "Action roundtrip: src_col mismatch for move {}",
                                move_.to_lan()
                            );
                            assert_eq!(
                                decoded_src_row,
                                usize::from(move_.src.row),
                                "Action roundtrip: src_row mismatch for move {}",
                                move_.to_lan()
                            );
                            assert_eq!(
                                decoded_plane,
                                plane_idx,
                                "Action roundtrip: plane mismatch for move {}",
                                move_.to_lan()
                            );

                            thread_moves_tested += 1;
                        }

                        // Make a random move
                        let chosen_move = legal_moves.choose(&mut rng).expect(
                            "test_fuzz_move_encoding_random_games: legal moves must not be empty",
                        );
                        game.make_move_unchecked(chosen_move);

                        thread_moves_played += 1;
                    }
                }

                moves_played.fetch_add(thread_moves_played, Ordering::Relaxed);
                moves_tested.fetch_add(thread_moves_tested, Ordering::Relaxed);
            });

            handles.push(handle);
        }

        for handle in handles {
            handle
                .join()
                .expect("test_fuzz_move_encoding_random_games: worker thread panicked");
        }

        let final_moves_played = total_moves_played.load(Ordering::Relaxed);
        let final_moves_tested = total_moves_tested.load(Ordering::Relaxed);

        println!(
            "\nMove Encoding Fuzz Test (Rust):\n  Games: {}\n  Threads: {}\n  Moves played: {}\n  Moves tested: {}",
            num_games, num_threads, final_moves_played, final_moves_tested
        );

        assert!(final_moves_played > 0, "No moves were played");
        assert!(final_moves_tested > 0, "No moves were tested");
    }
}
