#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
#[repr(i8)]
pub enum Color {
    White = 1,
    Black = -1,
}

impl Color {
    pub fn opposite(&self) -> Color {
        match self {
            Color::White => Color::Black,
            Color::Black => Color::White,
        }
    }

    pub fn from_int(i: i8) -> Option<Color> {
        match i {
            1 => Some(Color::White),
            -1 => Some(Color::Black),
            _ => None,
        }
    }
}
