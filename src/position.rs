use std::fmt;

#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub struct Position {
    pub col: u8,
    pub row: u8,
}

#[hotpath::measure_all]
impl Position {
    pub fn new(col: u8, row: u8) -> Self {
        Position { col, row }
    }

    pub(crate) fn from_usize(col: usize, row: usize) -> Self {
        Position {
            col: u8::try_from(col).expect("Position::from_usize: col exceeds u8"),
            row: u8::try_from(row).expect("Position::from_usize: row exceeds u8"),
        }
    }

    #[inline]
    pub fn is_valid(&self, width: usize, height: usize) -> bool {
        usize::from(self.col) < width && usize::from(self.row) < height
    }

    #[inline]
    pub fn to_index(&self, width: usize) -> usize {
        usize::from(self.row) * width + usize::from(self.col)
    }

    #[inline]
    pub fn from_index(index: usize, width: usize) -> Position {
        Position {
            col: u8::try_from(index % width).expect("Position::from_index: col exceeds u8"),
            row: u8::try_from(index / width).expect("Position::from_index: row exceeds u8"),
        }
    }

    pub fn to_algebraic(&self) -> String {
        if self.col < 26 {
            format!("{}{}", (b'a' + self.col) as char, usize::from(self.row) + 1)
        } else {
            format!("{}-{}", self.col, usize::from(self.row) + 1)
        }
    }

    pub fn from_algebraic(s: &str) -> Result<Self, String> {
        if s.len() < 2 {
            return Err("Invalid position string".to_string());
        }

        let col_char = s.as_bytes()[0] as char;
        let row_str = &s[1..];

        let col = if col_char.is_ascii_lowercase() {
            col_char as u8 - b'a'
        } else {
            return Err("Invalid file character".to_string());
        };

        let row_num = row_str
            .parse::<u16>()
            .map_err(|_| "Invalid row number".to_string())?;
        if row_num == 0 {
            return Err("Invalid row number".to_string());
        }
        let row = u8::try_from(row_num - 1).map_err(|_| "Invalid row number".to_string())?;

        Ok(Position { col, row })
    }
}

#[hotpath::measure_all]
impl fmt::Display for Position {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{}", self.to_algebraic())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_position_algebraic() {
        let pos = Position::new(0, 0);
        assert_eq!(pos.to_algebraic(), "a1");

        let pos = Position::new(7, 7);
        assert_eq!(pos.to_algebraic(), "h8");

        let pos = Position::new(4, 3);
        assert_eq!(pos.to_algebraic(), "e4");
    }

    #[test]
    fn test_position_from_algebraic() {
        let pos = Position::from_algebraic("a1")
            .expect("Failed to create position from algebraic notation 'a1'");
        assert_eq!(pos.col, 0);
        assert_eq!(pos.row, 0);

        let pos = Position::from_algebraic("e4")
            .expect("Failed to create position from algebraic notation 'e4'");
        assert_eq!(pos.col, 4);
        assert_eq!(pos.row, 3);

        let pos = Position::from_algebraic("h8")
            .expect("Failed to create position from algebraic notation 'h8'");
        assert_eq!(pos.col, 7);
        assert_eq!(pos.row, 7);
    }

    #[test]
    fn test_position_validity() {
        let pos = Position::new(3, 4);
        assert!(pos.is_valid(8, 8));
        assert!(!pos.is_valid(4, 4));

        let pos = Position::new(7, 7);
        assert!(pos.is_valid(8, 8));
        assert!(!pos.is_valid(7, 7));
    }
}
