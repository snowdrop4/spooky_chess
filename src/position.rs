use std::fmt;

#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub struct Position {
    pub col: usize,
    pub row: usize,
}

impl Position {
    pub fn new(col: usize, row: usize) -> Self {
        Position { col, row }
    }

    pub fn is_valid(&self, width: usize, height: usize) -> bool {
        self.col < width && self.row < height
    }

    pub fn to_algebraic(&self) -> String {
        if self.col < 26 {
            format!("{}{}", (b'a' + self.col as u8) as char, self.row + 1)
        } else {
            format!("{}-{}", self.col, self.row + 1)
        }
    }

    pub fn from_algebraic(s: &str) -> Result<Self, String> {
        if s.len() < 2 {
            return Err("Invalid position string".to_string());
        }

        let chars: Vec<char> = s.chars().collect();
        let col_char = chars[0];
        let row_str = &s[1..];

        let col = if col_char.is_ascii_lowercase() {
            (col_char as u8 - b'a') as usize
        } else {
            return Err("Invalid file character".to_string());
        };

        let row = row_str
            .parse::<usize>()
            .map_err(|_| "Invalid row number".to_string())?
            .saturating_sub(1);

        Ok(Position { col, row })
    }
}

impl fmt::Display for Position {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{}", self.to_algebraic())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_position_creation() {
        let pos = Position::new(3, 4);
        assert_eq!(pos.col, 3);
        assert_eq!(pos.row, 4);
    }

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
