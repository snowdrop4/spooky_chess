#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
#[repr(i8)]
pub enum Color {
    White = 1,
    Black = -1,
}

#[hotpath::measure_all]
impl Color {
    #[inline]
    pub fn opposite(&self) -> Color {
        match self {
            Color::White => Color::Black,
            Color::Black => Color::White,
        }
    }

    #[inline]
    pub fn from_int(i: i8) -> Option<Color> {
        match i {
            1 => Some(Color::White),
            -1 => Some(Color::Black),
            _ => None,
        }
    }
}

#[hotpath::measure_all]
impl std::fmt::Display for Color {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let s = match self {
            Color::White => "White",
            Color::Black => "Black",
        };
        write!(f, "{}", s)
    }
}
