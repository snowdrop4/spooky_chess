use crate::game::Game;
use crate::r#move::Move;
use serde::{Deserialize, Deserializer, Serialize, Serializer};

/// Serialize Game as FEN string
impl Serialize for Game {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        let fen = self.to_fen();
        serializer.serialize_str(&fen)
    }
}

/// Deserialize Game from FEN string
impl<'de> Deserialize<'de> for Game {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        let fen = String::deserialize(deserializer)?;
        // Assume castling is enabled by default for deserialized games
        Game::new(8, 8, &fen, true).map_err(serde::de::Error::custom)
    }
}

/// Serialize Move as LAN string
impl Serialize for Move {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        let lan = self.to_lan();
        serializer.serialize_str(&lan)
    }
}

/// Deserialize Move from LAN string
impl<'de> Deserialize<'de> for Move {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        let lan = String::deserialize(deserializer)?;
        // Assume standard 8x8 board
        Move::from_lan(&lan, 8, 8).map_err(serde::de::Error::custom)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::position::Position;
    use crate::r#move::MoveFlags;

    #[test]
    fn test_game_serde() {
        let game = Game::standard();
        let fen_before = game.to_fen();

        // Serialize to JSON
        let json = serde_json::to_string(&game).unwrap();

        // Deserialize back
        let game2: Game = serde_json::from_str(&json).unwrap();
        let fen_after = game2.to_fen();

        assert_eq!(fen_before, fen_after);
    }

    #[test]
    fn test_move_serde() {
        let move_ = Move::from_position(
            Position::new(4, 1),
            Position::new(4, 3),
            MoveFlags::DOUBLE_PUSH,
        );
        let lan_before = move_.to_lan();

        // Serialize to JSON
        let json = serde_json::to_string(&move_).unwrap();

        // Deserialize back
        let move2: Move = serde_json::from_str(&json).unwrap();
        let lan_after = move2.to_lan();

        assert_eq!(lan_before, lan_after);
        assert_eq!(move_.src, move2.src);
        assert_eq!(move_.dst, move2.dst);
    }

    #[test]
    fn test_move_with_promotion_serde() {
        use crate::pieces::PieceType;

        let move_ = Move::from_position_with_promotion(
            Position::new(4, 6),
            Position::new(4, 7),
            MoveFlags::PROMOTION,
            PieceType::Queen,
        );
        let lan_before = move_.to_lan();

        // Serialize to JSON
        let json = serde_json::to_string(&move_).unwrap();
        assert!(json.contains("e7e8q"));

        // Deserialize back
        let move2: Move = serde_json::from_str(&json).unwrap();
        let lan_after = move2.to_lan();

        assert_eq!(lan_before, lan_after);
        assert_eq!(move_.promotion, move2.promotion);
    }

    #[test]
    fn test_game_roundtrip() {
        let mut game = Game::standard();

        // Make some moves
        let moves_lan = vec!["e2e4", "e7e5", "g1f3", "b8c6"];
        for lan in moves_lan {
            let mv = Move::from_lan(lan, 8, 8).unwrap();
            game.make_move(&mv);
        }

        // Serialize
        let json = serde_json::to_string(&game).unwrap();

        // Deserialize
        let game2: Game = serde_json::from_str(&json).unwrap();

        // Check they're the same
        assert_eq!(game.to_fen(), game2.to_fen());
    }

    #[test]
    fn test_bincode_game() {
        let game = Game::standard();
        let fen_before = game.to_fen();

        // Serialize with bincode
        let encoded = bincode::serialize(&game).unwrap();

        // Deserialize
        let game2: Game = bincode::deserialize(&encoded).unwrap();
        let fen_after = game2.to_fen();

        assert_eq!(fen_before, fen_after);
    }

    #[test]
    fn test_bincode_move() {
        let move_ = Move::from_lan("e2e4", 8, 8).unwrap();
        let lan_before = move_.to_lan();

        // Serialize with bincode
        let encoded = bincode::serialize(&move_).unwrap();

        // Deserialize
        let move2: Move = bincode::deserialize(&encoded).unwrap();
        let lan_after = move2.to_lan();

        assert_eq!(lan_before, lan_after);
    }
}
