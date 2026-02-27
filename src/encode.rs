use crate::color::Color;
use crate::game::Game;
use crate::pieces::PieceType;
use crate::position::Position;
use crate::r#move::Move;

/// Number of planes for piece positions (6 for WHITE + 6 for BLACK)
const PIECE_PLANES: usize = 6 + 6;

/// Number of constant planes (2 repetitions + 1 color + 1 total move + 4 castling + 1 no-progress)
const CONSTANT_PLANES: usize = 2 + 1 + 1 + 4 + 1;

/// Number of positions in the game history to encode
pub const HISTORY_LENGTH: usize = 8;

/// Total number of input planes for the neural network
pub const TOTAL_INPUT_PLANES: usize = (HISTORY_LENGTH * PIECE_PLANES) + CONSTANT_PLANES;

/// Number of directions for horizontal/vertical/diagonal moves (N, NE, E, SE, S, SW, W, NW)
const NUM_DIRECTIONS: usize = 8;

/// Number of knight move patterns
const NUM_KNIGHT_MOVES: usize = 8;

/// Number of underpromotion directions (left diagonal, straight, right diagonal)
const NUM_UNDERPROMO_DIRECTIONS: usize = 3;

/// Number of underpromotion piece types (knight, bishop, rook - excluding queen)
const NUM_UNDERPROMO_PIECES: usize = 3;

/// Number of promotion move directions (forward, backward)
const NUM_PROMOTION_ORIENTATIONS: usize = 2;

/// Encode the full game state into a flat f32 array for efficient transfer to Python/numpy
/// Returns (flat_data, num_planes, height, width), where flat_data is in row-major order
pub fn encode_game_planes<const NW: usize>(game: &mut Game<NW>) -> (Vec<f32>, usize, usize, usize) {
    let width = game.board().width();
    let height = game.board().height();
    let num_planes = TOTAL_INPUT_PLANES;
    let board_size = height * width;
    let total_size = num_planes * board_size;
    let mut data = vec![0.0f32; total_size];

    let perspective = game.turn();
    let opponent = perspective.opposite();

    let history_len = game.move_count();
    let steps_back = (HISTORY_LENGTH - 1).min(history_len);

    let all_moves = game.move_history();
    let moves_to_replay: Vec<Move> = all_moves[(history_len - steps_back)..].to_vec();

    // T=0: current position
    fill_chess_planes(&mut data, game, perspective, 0, width, height);

    // T=1..steps_back: walk backward through history
    for t in 1..=steps_back {
        game.unmake_move();
        fill_chess_planes(&mut data, game, perspective, t, width, height);
    }

    // Replay saved moves to restore game state
    for mv in &moves_to_replay {
        game.make_move(mv);
    }

    // Constant planes start at index: HISTORY_LENGTH * PIECE_PLANES
    let constant_start = HISTORY_LENGTH * PIECE_PLANES;

    // Repetition count planes (2 planes) - zeros for now

    // Color plane
    let color_plane = constant_start + 2;
    let color_value = if perspective == Color::White {
        1.0
    } else {
        0.0
    };
    fill_constant_plane(&mut data, color_plane, color_value, board_size);

    // Total move count plane
    let move_plane = constant_start + 3;
    let move_count = game.fullmove_number() as f32 / 100.0;
    fill_constant_plane(&mut data, move_plane, move_count, board_size);

    // Castling rights (4 planes)
    let castling_rights = game.castling_rights();

    let p1_kingside = if castling_rights.has_kingside(perspective) {
        1.0
    } else {
        0.0
    };
    fill_constant_plane(&mut data, constant_start + 4, p1_kingside, board_size);

    let p1_queenside = if castling_rights.has_queenside(perspective) {
        1.0
    } else {
        0.0
    };
    fill_constant_plane(&mut data, constant_start + 5, p1_queenside, board_size);

    let p2_kingside = if castling_rights.has_kingside(opponent) {
        1.0
    } else {
        0.0
    };
    fill_constant_plane(&mut data, constant_start + 6, p2_kingside, board_size);

    let p2_queenside = if castling_rights.has_queenside(opponent) {
        1.0
    } else {
        0.0
    };
    fill_constant_plane(&mut data, constant_start + 7, p2_queenside, board_size);

    // No-progress count plane
    let no_progress = game.halfmove_clock() as f32 / 50.0;
    fill_constant_plane(&mut data, constant_start + 8, no_progress, board_size);

    (data, num_planes, height, width)
}

fn fill_constant_plane(data: &mut [f32], plane: usize, value: f32, board_size: usize) {
    let offset = plane * board_size;
    for i in 0..board_size {
        data[offset + i] = value;
    }
}

fn fill_chess_planes<const NW: usize>(
    data: &mut [f32],
    game: &Game<NW>,
    perspective: Color,
    t: usize,
    width: usize,
    height: usize,
) {
    let opponent = perspective.opposite();
    let board_size = height * width;
    let base_plane = t * PIECE_PLANES;

    let piece_types = [
        PieceType::Pawn,
        PieceType::Knight,
        PieceType::Bishop,
        PieceType::Rook,
        PieceType::Queen,
        PieceType::King,
    ];

    for (piece_idx, piece_type) in piece_types.iter().enumerate() {
        let own_offset = (base_plane + piece_idx) * board_size;
        let opp_offset = (base_plane + 6 + piece_idx) * board_size;
        for row in 0..height {
            for col in 0..width {
                let pos = Position::new(col, row);
                if let Some(piece) = game.board().get_piece(&pos) {
                    let idx = row * width + col;
                    if piece.piece_type == *piece_type {
                        if piece.color == perspective {
                            data[own_offset + idx] = 1.0;
                        } else if piece.color == opponent {
                            data[opp_offset + idx] = 1.0;
                        }
                    }
                }
            }
        }
    }
}

/// Encode a move as a plane index for the policy head
/// Move planes encode the movement pattern:
/// - Horizontal/vertical/diagonal moves, for all non-knight pieces,
///   in 8 directions (N, NE, E, SE, S, SW, W, NW) up to max distance
/// - L-shaped moves for knights, in 8 directions
/// - Underpromotions (3 directions × 3 piece types, excluding queen)
pub fn encode_move(move_: &Move, width: usize, height: usize) -> Option<usize> {
    let src = move_.src;
    let dst = move_.dst;
    let dx = dst.col as i32 - src.col as i32;
    let dy = dst.row as i32 - src.row as i32;

    // L-shaped moves for knights
    let knight_deltas = [
        (1, 2),
        (2, 1),
        (2, -1),
        (1, -2),
        (-1, -2),
        (-2, -1),
        (-2, 1),
        (-1, 2),
    ];

    let max_distance = width.max(height) - 1;

    for (i, &(kdx, kdy)) in knight_deltas.iter().enumerate() {
        if dx == kdx && dy == kdy {
            let knight_planes_start = NUM_DIRECTIONS * max_distance;
            return Some(knight_planes_start + i);
        }
    }

    // Underpromotions (only for non-queen promotions)
    // Note: underpromotions are forward by 1 row only (dy = ±1 depending on perspective)
    if let Some(promo) = move_.promotion {
        if promo != PieceType::Queen && dy.abs() == 1 {
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
            let underpromo_planes_start = knight_planes_start + NUM_KNIGHT_MOVES;
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
    }

    // Horizontal/vertical/diagonal moves for all non-knight pieces
    let direction = if dx == 0 && dy > 0 {
        Some(0) // North
    } else if dx > 0 && dy > 0 && dx == dy {
        Some(1) // NE
    } else if dx > 0 && dy == 0 {
        Some(2) // East
    } else if dx > 0 && dy < 0 && dx == -dy {
        Some(3) // SE
    } else if dx == 0 && dy < 0 {
        Some(4) // South
    } else if dx < 0 && dy < 0 && dx == dy {
        Some(5) // SW
    } else if dx < 0 && dy == 0 {
        Some(6) // West
    } else if dx < 0 && dy > 0 && -dx == dy {
        Some(7) // NW
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
pub fn decode_move_plane(
    plane_idx: usize,
    width: usize,
    height: usize,
) -> Option<(i32, i32, Option<PieceType>)> {
    let max_distance = width.max(height) - 1;
    let straight_diagonal_planes = NUM_DIRECTIONS * max_distance;
    let knight_planes_start = straight_diagonal_planes;
    let underpromo_planes_start = knight_planes_start + NUM_KNIGHT_MOVES;

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
        let knight_deltas = [
            (1, 2),
            (2, 1),
            (2, -1),
            (1, -2),
            (-1, -2),
            (-2, -1),
            (-2, 1),
            (-1, 2),
        ];

        knight_deltas
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
pub fn get_move_planes_count(width: usize, height: usize) -> usize {
    let max_distance = width.max(height) - 1;
    let straight_diagonal_planes = NUM_DIRECTIONS * max_distance;
    let knight_planes = NUM_KNIGHT_MOVES;
    let underpromo_planes =
        NUM_UNDERPROMO_DIRECTIONS * NUM_UNDERPROMO_PIECES * NUM_PROMOTION_ORIENTATIONS;

    straight_diagonal_planes + knight_planes + underpromo_planes
}

/// Decode a plane index and source position to a Move
/// Returns the decoded move if valid
pub fn decode_move_from_plane(
    plane_idx: usize,
    src_col: usize,
    src_row: usize,
    width: usize,
    height: usize,
) -> Option<Move> {
    let (dx, dy, promo) = decode_move_plane(plane_idx, width, height)?;

    // Calculate destination
    let dst_col = (src_col as i32 + dx) as usize;
    let dst_row = (src_row as i32 + dy) as usize;

    // Check bounds
    if dst_col >= width || dst_row >= height {
        return None;
    }

    let src = Position::new(src_col, src_row);
    let dst = Position::new(dst_col, dst_row);

    let flags = if promo.is_some() {
        use crate::r#move::MoveFlags;
        MoveFlags::PROMOTION
    } else {
        use crate::r#move::MoveFlags;
        MoveFlags::empty()
    };

    let move_ = if let Some(promo_piece) = promo {
        Move::from_position_with_promotion(src, dst, flags, promo_piece)
    } else {
        Move::from_position(src, dst, flags)
    };

    Some(move_)
}

#[cfg(test)]
mod tests {
    use super::*;

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
    fn test_encode_move_horizontal_vertical() {
        use crate::r#move::MoveFlags;

        // Test vertical move (rook moving north)
        let move_north =
            Move::from_position(Position::new(3, 0), Position::new(3, 4), MoveFlags::empty());
        let encoded = encode_move(&move_north, 8, 8);
        assert_eq!(encoded, Some(3)); // North direction, distance 4

        // Test horizontal move (rook moving east)
        let move_east =
            Move::from_position(Position::new(0, 3), Position::new(5, 3), MoveFlags::empty());
        let encoded = encode_move(&move_east, 8, 8);
        assert_eq!(encoded, Some(2 * 7 + 4)); // East direction, distance 5
    }

    #[test]
    fn test_encode_move_diagonal() {
        use crate::r#move::MoveFlags;

        // Test diagonal move (bishop moving NE)
        let move_ne =
            Move::from_position(Position::new(1, 1), Position::new(4, 4), MoveFlags::empty());
        let encoded = encode_move(&move_ne, 8, 8);
        assert_eq!(encoded, Some(7 + 2)); // NE direction, distance 3

        // Test diagonal move (bishop moving SW)
        let move_sw =
            Move::from_position(Position::new(5, 5), Position::new(3, 3), MoveFlags::empty());
        let encoded = encode_move(&move_sw, 8, 8);
        assert_eq!(encoded, Some(5 * 7 + 1)); // SW direction, distance 2
    }

    #[test]
    fn test_encode_move_knight() {
        use crate::r#move::MoveFlags;

        // Test knight move (1, 2)
        let move_knight =
            Move::from_position(Position::new(3, 3), Position::new(4, 5), MoveFlags::empty());
        let encoded = encode_move(&move_knight, 8, 8);
        assert_eq!(encoded, Some(8 * 7)); // First knight pattern

        // Test knight move (2, -1)
        let move_knight2 =
            Move::from_position(Position::new(3, 3), Position::new(5, 2), MoveFlags::empty());
        let encoded = encode_move(&move_knight2, 8, 8);
        assert_eq!(encoded, Some(8 * 7 + 2)); // Third knight pattern
    }

    #[test]
    fn test_encode_move_underpromotion() {
        use crate::r#move::MoveFlags;

        // Test straight underpromotion to knight (forward)
        let move_promo = Move::from_position_with_promotion(
            Position::new(3, 6),
            Position::new(3, 7),
            MoveFlags::PROMOTION,
            PieceType::Knight,
        );
        let encoded = encode_move(&move_promo, 8, 8);
        assert_eq!(encoded, Some((8 * 7 + 8) + 3)); // Forward, straight, knight

        // Test diagonal underpromotion to rook (forward)
        let move_promo2 = Move::from_position_with_promotion(
            Position::new(3, 6),
            Position::new(4, 7),
            MoveFlags::PROMOTION,
            PieceType::Rook,
        );
        let encoded = encode_move(&move_promo2, 8, 8);
        assert_eq!(encoded, Some((8 * 7 + 8) + 2 * 3 + 2)); // Forward, right diagonal, rook

        // Test straight underpromotion to bishop (backward)
        let move_promo3 = Move::from_position_with_promotion(
            Position::new(3, 1),
            Position::new(3, 0),
            MoveFlags::PROMOTION,
            PieceType::Bishop,
        );
        let encoded = encode_move(&move_promo3, 8, 8);
        assert_eq!(encoded, Some(8 * 7 + 8 + 9 + 3 + 1)); // Backward, straight, bishop
    }

    #[test]
    fn test_encode_move_queen_promotion() {
        use crate::r#move::MoveFlags;

        // Queen promotions should use regular straight/diagonal encoding
        let move_promo = Move::from_position_with_promotion(
            Position::new(3, 6),
            Position::new(3, 7),
            MoveFlags::PROMOTION,
            PieceType::Queen,
        );
        let encoded = encode_move(&move_promo, 8, 8);
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
            let encoded = encode_move(&move_, 8, 8).expect("Failed to encode move");
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
    fn test_decode_move_from_plane() {
        // Test straight move (north)
        let plane_idx = 3; // North, distance 4
        let decoded = decode_move_from_plane(plane_idx, 3, 0, 8, 8).unwrap();
        assert_eq!(decoded.src.col, 3);
        assert_eq!(decoded.src.row, 0);
        assert_eq!(decoded.dst.col, 3);
        assert_eq!(decoded.dst.row, 4);

        // Test knight move
        let plane_idx = 8 * 7; // First knight pattern (1, 2)
        let decoded = decode_move_from_plane(plane_idx, 3, 3, 8, 8).unwrap();
        assert_eq!(decoded.src.col, 3);
        assert_eq!(decoded.src.row, 3);
        assert_eq!(decoded.dst.col, 4);
        assert_eq!(decoded.dst.row, 5);

        // Test out of bounds
        let plane_idx = 3; // North, distance 4
        let decoded = decode_move_from_plane(plane_idx, 3, 7, 8, 8); // Would go off board
        assert!(decoded.is_none());
    }

    #[test]
    fn test_encode_decode_roundtrip_from_plane() {
        use crate::r#move::MoveFlags;

        let test_moves = vec![
            (Position::new(0, 0), Position::new(0, 5)),
            (Position::new(2, 2), Position::new(5, 5)),
            (Position::new(3, 3), Position::new(4, 5)), // Knight move
        ];

        for (src, dst) in test_moves {
            let original_move = Move::from_position(src, dst, MoveFlags::empty());
            let encoded = encode_move(&original_move, 8, 8).expect("Failed to encode");

            let decoded =
                decode_move_from_plane(encoded, src.col, src.row, 8, 8).expect("Failed to decode");

            assert_eq!(decoded.src.col, original_move.src.col);
            assert_eq!(decoded.src.row, original_move.src.row);
            assert_eq!(decoded.dst.col, original_move.dst.col);
            assert_eq!(decoded.dst.row, original_move.dst.row);
        }
    }

    #[test]
    fn test_fuzz_move_encoding_random_games() {
        use rand::prelude::IndexedRandom;
        use rand::SeedableRng;
        use std::sync::atomic::{AtomicU64, Ordering};
        use std::sync::Arc;
        use std::thread;

        let num_games = 5_000;
        let num_threads = num_cpus::get();
        let games_per_thread = num_games / num_threads;

        let total_moves_played = Arc::new(AtomicU64::new(0));
        let total_moves_tested = Arc::new(AtomicU64::new(0));

        let mut handles = vec![];

        for thread_id in 0..num_threads {
            let moves_played = Arc::clone(&total_moves_played);
            let moves_tested = Arc::clone(&total_moves_tested);

            let handle = thread::spawn(move || {
                let mut rng = rand::rngs::StdRng::seed_from_u64(thread_id as u64);
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
                        for move_ in &legal_moves {
                            let width = game.board().width();
                            let height = game.board().height();
                            let encoded = encode_move(move_, width, height);
                            assert!(
                                encoded.is_some(),
                                "Failed to encode move {} in position {}",
                                move_.to_lan(),
                                game.to_fen()
                            );

                            let plane_idx = encoded.unwrap();
                            let decoded = decode_move_plane(plane_idx, width, height);
                            assert!(
                                decoded.is_some(),
                                "Failed to decode plane {} for move {}",
                                plane_idx,
                                move_.to_lan()
                            );

                            let (dx, dy, promo) = decoded.unwrap();

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

                            thread_moves_tested += 1;
                        }

                        // Make a random move
                        let chosen_move = legal_moves.choose(&mut rng).unwrap();
                        let success = game.make_move(chosen_move);
                        assert!(success, "Failed to make move {}", chosen_move.to_lan());

                        thread_moves_played += 1;
                    }
                }

                moves_played.fetch_add(thread_moves_played, Ordering::Relaxed);
                moves_tested.fetch_add(thread_moves_tested, Ordering::Relaxed);
            });

            handles.push(handle);
        }

        for handle in handles {
            handle.join().unwrap();
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
