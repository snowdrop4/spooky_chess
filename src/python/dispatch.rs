use crate::game::Game;

/// Generates the cartesian product of W and H ranges, then invokes $mac with all (W, H) pairs.
macro_rules! cartesian_dispatch {
    ($mac:ident, [$($w:literal),*], $hs:tt) => {
        cartesian_dispatch!(@acc $mac, $hs, [] ; $($w),*);
    };
    // Base case: no more W values, invoke the target macro with accumulated pairs.
    (@acc $mac:ident, $hs:tt, [$($pairs:tt)*] ; ) => {
        $mac!($($pairs)*);
    };
    // Recursive case: peel off one W, expand all H for it, then continue.
    (@acc $mac:ident, [$($h:literal),*], [$($pairs:tt)*] ; $w:literal $(, $rest:literal)*) => {
        cartesian_dispatch!(@acc $mac, [$($h),*], [$($pairs)* $(($w, $h),)*] ; $($rest),*);
    };
}

macro_rules! define_wh_dispatch {
    ($(($w:literal, $h:literal)),* $(,)?) => {
        paste::paste! {
            #[derive(Clone)]
            pub(super) enum GameInner {
                $( [<W $w H $h>](Game<$w, $h>), )*
            }

            macro_rules! dispatch_game {
                ($self_:expr, $g:ident => $body:expr) => {
                    match $self_ {
                        $( GameInner::[<W $w H $h>]($g) => $body, )*
                    }
                };
            }

            pub(super) fn make_game_inner(width: usize, height: usize, fen: &str, castling_enabled: bool) -> Result<GameInner, String> {
                match (width, height) {
                    $( ($w, $h) => Ok(GameInner::[<W $w H $h>](Game::new(fen, castling_enabled)?)), )*
                    _ => Err(format!("Unsupported board size: {}x{}", width, height)),
                }
            }

            pub(super) fn make_standard_game_inner() -> GameInner {
                GameInner::W8H8(Game::standard())
            }

        }
    }
}

cartesian_dispatch!(
    define_wh_dispatch,
    [5, 6, 7, 8, 9, 10, 11, 12, 13, 14, 15, 16],
    [5, 6, 7, 8, 9, 10, 11, 12, 13, 14, 15, 16]
);
