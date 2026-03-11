use crate::pieces::PieceType;
use crate::position::Position;
use bitflags::bitflags;

bitflags! {
    #[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
    pub struct MoveFlags: u8 {
        const CAPTURE = 0b00000001;
        const DOUBLE_PUSH = 0b00000010;
        const EN_PASSANT = 0b00000100;
        const CASTLE = 0b00001000;
        const PROMOTION = 0b00010000;
        const CHECK = 0b00100000;
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub struct Move {
    pub src: Position,
    pub dst: Position,
    pub flags: MoveFlags,
    pub promotion: Option<PieceType>,
}

#[hotpath::measure_all]
impl Move {
    pub fn from_position(src: Position, dst: Position, flags: MoveFlags) -> Self {
        Move {
            src,
            dst,
            flags,
            promotion: None,
        }
    }

    pub fn from_position_with_promotion(
        src: Position,
        dst: Position,
        flags: MoveFlags,
        promotion: PieceType,
    ) -> Self {
        Move {
            src,
            dst,
            flags: flags | MoveFlags::PROMOTION,
            promotion: Some(promotion),
        }
    }

    pub fn from_lan(lan: &str, board_width: usize, board_height: usize) -> Result<Self, String> {
        if lan.len() < 4 {
            return Err("Invalid LAN move".to_string());
        }

        let src = Position::from_algebraic(&lan[0..2])?;
        let dst = Position::from_algebraic(&lan[2..4])?;

        if !src.is_valid(board_width, board_height) || !dst.is_valid(board_width, board_height) {
            return Err("Move positions out of bounds".to_string());
        }

        let mut move_ = Move::from_position(src, dst, MoveFlags::empty());

        if lan.len() > 4 {
            let promo_char = lan
                .chars()
                .nth(4)
                .expect("Failed to get promotion character from LAN string");

            let promotion = PieceType::from_char(promo_char)
                .ok_or_else(|| "Invalid promotion piece".to_string())?;

            move_.promotion = Some(promotion);
            move_.flags |= MoveFlags::PROMOTION;
        }

        Ok(move_)
    }

    /// Returns `(rook_from, rook_to)` for a castling move given the board width.
    /// Kingside: rook starts at column `board_width - 1`, lands at `king_dst - 1`.
    /// Queenside: rook starts at column 0, lands at `king_dst + 1`.
    pub fn castling_rook_positions(&self, board_width: usize) -> (Position, Position) {
        debug_assert!(self.flags.contains(MoveFlags::CASTLE));
        if self.dst.col > self.src.col {
            // Kingside
            (
                Position::new(board_width - 1, self.src.row),
                Position::new(self.dst.col - 1, self.dst.row),
            )
        } else {
            // Queenside
            (
                Position::new(0, self.src.row),
                Position::new(self.dst.col + 1, self.dst.row),
            )
        }
    }

    pub fn to_lan(&self) -> String {
        let mut lan = format!("{}{}", self.src.to_algebraic(), self.dst.to_algebraic());

        if let Some(promo) = self.promotion {
            lan.push(promo.to_char());
        }

        lan
    }
}

#[hotpath::measure_all]
impl std::fmt::Display for Move {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.to_lan())
    }
}
