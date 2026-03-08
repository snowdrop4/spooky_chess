pub const ORTHOGONAL: [(i32, i32); 4] = [(0, 1), (0, -1), (1, 0), (-1, 0)];

pub const DIAGONAL: [(i32, i32); 4] = [(1, 1), (1, -1), (-1, 1), (-1, -1)];

pub const ALL_DIRS: [(i32, i32); 8] = [
    (0, 1),
    (0, -1),
    (1, 0),
    (-1, 0),
    (1, 1),
    (1, -1),
    (-1, 1),
    (-1, -1),
];

/// Map a (dx, dy) direction to a plane index (0..7) for move encoding.
/// Order: N=0, NE=1, E=2, SE=3, S=4, SW=5, W=6, NW=7
pub fn direction_index(dx: i32, dy: i32) -> Option<usize> {
    match (dx.signum(), dy.signum()) {
        (0, 1) => Some(0),   // N
        (1, 1) => Some(1),   // NE
        (1, 0) => Some(2),   // E
        (1, -1) => Some(3),  // SE
        (0, -1) => Some(4),  // S
        (-1, -1) => Some(5), // SW
        (-1, 0) => Some(6),  // W
        (-1, 1) => Some(7),  // NW
        _ => None,
    }
}

/// L-shaped knight move offsets as (col_delta, row_delta).
/// The ordering is stable and used by the encode module for plane indices.
pub const KNIGHT_DELTAS: [(i32, i32); 8] = [
    (1, 2),
    (2, 1),
    (2, -1),
    (1, -2),
    (-1, -2),
    (-2, -1),
    (-2, 1),
    (-1, 2),
];
