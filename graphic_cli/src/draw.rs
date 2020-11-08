use std::convert::Into;
use crossterm::style::{Color, StyledContent, style};

#[derive(Debug, Clone)]
pub struct Canvas {
    pub a: Vec<Vec<StyledContent<char>>>,
}

impl Canvas {
    pub fn new(rows: usize, cols: usize) -> Self {
        Self {
            a: vec![vec![style(' ').on(Color::White); cols]; rows],
        }
    }
}

// 1-bit color (2 possibilities)
/*enum BW {
    Black,
    White,
    Transparent,
}

// 3-bit color (8 possibilities)
enum Colors8 {
    Black,
    Red,
    Green,
    Yellow,
    Blue,
    Magenta,
    Cyan,
    White,
    Transparent,
    Inverse,
}

// 4-bit color (16 possibilities)
enum ANSI {
    Normal(Colors8),
    Bright(Colors8),
}

// 8-bit color (256 possibilities)
struct Colors256 {
    code: u8,
}

// 24-bit true color (16,777,215 possibilities)
struct RGB {
    r: u8,
    g: u8,
    b: u8,
}

pub trait Color {
    //#[cfg(feature = "tty")]
    fn as_crossterm() -> crossterm::style::Color;
}*/