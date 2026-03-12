#![feature(generic_const_exprs)]
#![allow(incomplete_features)]

pub mod bitboard;
pub mod board;
pub mod color;
pub mod directions;
pub mod encode;
pub mod game;
pub mod r#move;
pub mod outcome;
pub mod pgn;
pub mod pieces;
pub mod position;
pub mod uci;

#[cfg(feature = "python")]
extern crate pyo3;

#[cfg(feature = "python")]
mod python;

#[cfg(feature = "python")]
use pyo3::prelude::*;

#[cfg(feature = "python")]
#[pymodule(gil_used = false)]
#[hotpath::measure]
fn spooky_chess(m: &Bound<'_, PyModule>) -> PyResult<()> {
    use color::Color;
    use python::*;
    m.add_class::<PyBoard>()?;
    m.add_class::<PyGame>()?;
    m.add_class::<PyMove>()?;
    m.add_class::<PyPiece>()?;
    m.add_class::<PyPosition>()?;
    m.add_class::<PyGameOutcome>()?;
    m.add_class::<PyPgnGame>()?;
    m.add_class::<PyUciEngine>()?;
    m.add_class::<PySearchResult>()?;
    m.add_function(wrap_pyfunction!(py_parse_pgn, m)?)?;
    m.add("WHITE", Color::White as i8)?;
    m.add("BLACK", Color::Black as i8)?;
    m.add("TOTAL_INPUT_PLANES", encode::TOTAL_INPUT_PLANES)?;
    Ok(())
}
