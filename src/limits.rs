pub const MIN_BOARD_DIM: usize = 6;
pub const MAX_BOARD_DIM: usize = 16;

#[inline]
pub const fn board_dimension_is_valid(dimension: usize) -> bool {
    dimension >= MIN_BOARD_DIM && dimension <= MAX_BOARD_DIM
}

pub fn validate_board_dimensions(width: usize, height: usize) -> Result<(), String> {
    if !board_dimension_is_valid(width) {
        return Err(format!(
            "Board width must be between {} and {}",
            MIN_BOARD_DIM, MAX_BOARD_DIM
        ));
    }
    if !board_dimension_is_valid(height) {
        return Err(format!(
            "Board height must be between {} and {}",
            MIN_BOARD_DIM, MAX_BOARD_DIM
        ));
    }
    Ok(())
}
