use std::fmt;
use Unit::*;

/// This is used for measurement

#[derive(Debug, Clone)]
pub enum Unit {
    /// The base unit
    Pt(f32),
    /// The height/width of one character (depending on context)
    Ch(f32),
    /// 1/100th of the height of the top-level element
    Vh(f32),
    /// 1/100th of the width of the top-level element
    Vw(f32),
    /// 1% of the height/width of the parent element (depending on context)
    Percent(f32),
    /// The sum of the contained `Unit`s
    Add(Box<Unit>, Box<Unit>),
    /// The difference of the `Unit`s
    Sub(Box<Unit>, Box<Unit>),
    /// The product of the `Unit` and factor
    Mul(Box<Unit>, f32),
    /// The quotient of the `Unit` and divisor
    Div(Box<Unit>, f32),
    /// The smaller of the two `Unit`s
    Min(Box<Unit>, Box<Unit>),
    /// The larger of the two `Unit`s
    Max(Box<Unit>, Box<Unit>),
}

impl Unit {
    /// Gets the value of the `Unit`, with the outputted value being in `pt`s
    pub fn value(&self, o: &Orientation, vh: &f32, vw: &f32, ph: &f32, pw: &f32, ch: &f32, cw: &f32) -> f32 {
        match self {
            Pt(a) => a.to_owned(),
            Ch(a) => match o { Orientation::Horizontal => a * cw, Orientation::Vertical => a * ch},
            Vh(a) => a * vh / 100.0,
            Vw(a) => a * vw / 100.0,
            Percent(a) => match o { Orientation::Horizontal => a * pw / 100.0, Orientation::Vertical => a * ph / 100.0},
            Add(a, b) => a.value(o, vh, vw, ph, pw, ch, cw) + b.value(o, vh, vw, ph, pw, ch, cw),
            Sub(a, b) => a.value(o, vh, vw, ph, pw, ch, cw) - b.value(o, vh, vw, ph, pw, ch, cw),
            Mul(a, b) => a.value(o, vh, vw, ph, pw, ch, cw) * b,
            Div(a, b) => a.value(o, vh, vw, ph, pw, ch, cw) / b,
            Min(a, b) => a.value(o, vh, vw, ph, pw, ch, cw).min(b.value(o, vh, vw, ph, pw, ch, cw)),
            Max(a, b) => a.value(o, vh, vw, ph, pw, ch, cw).max(b.value(o, vh, vw, ph, pw, ch, cw)),
        }
    }
}

impl fmt::Display for Unit {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Pt(a) => write!(f, "{}pt", a),
            Ch(a) => write!(f, "{}ch", a),
            Vh(a) => write!(f, "{}vh", a),
            Vw(a) => write!(f, "{}vw", a),
            Percent(a) => write!(f, "{}%", a),
            Add(a, b) => write!(f, "({} + {})", a, b),
            Sub(a, b) => write!(f, "({} - {})", a, b),
            Mul(a, b) => write!(f, "({} * {})", a, b),
            Div(a, b) => write!(f, "({} / {})", a, b),
            Min(a, b) => write!(f, "min({}, {})", a, b),
            Max(a, b) => write!(f, "max({}, {})", a, b),
        }
    }
}

/// Represents the orientation of where a `Unit` is being used
#[allow(missing_docs)]
pub enum Orientation {
    Horizontal,
    Vertical,
}