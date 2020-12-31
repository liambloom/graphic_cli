// Copyright 2020 Liam Bloom
// SPDX-License-Identifier: Apache-2.0

//! # Graphic CLI
//! 
//! This library allows you to create a GUI in the command line.

#![warn(missing_docs)]

use std::{
    io::{stdout, Write},
    ops::Deref, 
    sync::{Once, atomic::{AtomicUsize, Ordering}},
    rc::Rc,
    cell::RefCell,
    ops::RangeInclusive,
    iter::Iterator,
};
use crossterm::{
    QueueableCommand, ExecutableCommand,
    tty::IsTty,
    style::{ContentStyle, StyledContent, PrintStyledContent, style},
    terminal, cursor,
};
use num_traits::{Zero, NumRef};
#[cfg(unix)]
use libc::{winsize, ioctl, STDOUT_FILENO, TIOCGWINSZ};
#[cfg(unix)]
use lazy_static::lazy_static;

pub use crossterm::style::Color;

pub mod error;
use error::*;

#[cfg(windows)]
static PX_SIZE: (f32, f32) = (1.0, 0.5);

#[cfg(unix)]
lazy_static! {
    static ref PX_SIZE: (f32, f32) = {
        match get_winsize() {
            Ok(w) => {
                let ar = (w.ws_xpixel as f64 / w.ws_col as f64) / (w.ws_ypixel as f64 / w.ws_row as f64);
                if ar < 0.75 {
                    (1.0, 0.5)
                }
                else if ar > 1.333 {
                    (0.5, 1.0)
                }
                else {
                    (1.0, 1.0)
                }
            },
            Err(_) => (1.0, 0.5)
        }
    };
}

static CANVAS_COUNT: AtomicUsize = AtomicUsize::new(0);

/// The main element of this crate, the `Canvas` element draws to the canvas
#[derive(Clone, Debug)]
pub struct Canvas {
    /// This holds the layers within the canvas.
    pub layers: Vec<Layer>,
    changed: Rc<RefCell<Vec<bool>>>,
}

impl Canvas {
    /// Creates a new canvas
    pub fn new() -> Self {
        let mut out = stdout();
        CANVAS_COUNT.fetch_add(1, Ordering::Release);
        static ONCE: Once = Once::new();
        ONCE.call_once(|| {
            if !out.is_tty() {
                eprintln!("Stdout is not a terminal");
            }
        });
        out.execute(cursor::Hide).expect("Failed to hide the cursor");
        Self {
            layers: Vec::new(),
            changed: Rc::new(RefCell::new(vec![true; Layer::size()])),
        }
    }

    /// Creates a new `Layer`, adding it to `self.layers`
    /// TODO: Add way to add layers in other places (or make
    /// new_layer() public)
    pub fn add_layer(&mut self) -> &mut Layer {
        self.layers.push(self.new_layer());
        self.layers.last_mut().unwrap()
    }

    fn new_layer(&self) -> Layer {
        Layer {
            buf: vec![StyledContent::new(ContentStyle::new(), ' '); Layer::size()],
            changed: Rc::clone(&self.changed),
        }
    }

    /// Gets the resolution of a canvas
    pub fn resolution() -> (u16, u16) {
        let (cols, rows) = terminal::size().expect("Unable to get terminal size");
        ((cols as f32 / PX_SIZE.0) as u16, (rows as f32 * PX_SIZE.1) as u16)
    }

    /// Redraws changed parts of the canvas
    pub fn update(&self) -> Result<()> {
        self.update_inner(self.changed.borrow())
    }

    /// Redraws changed parts of the canvas, but only if it won't block
    pub fn try_update(&self) -> Result<()> {
        self.update_inner(self.changed.try_borrow()?)
    }

    fn update_inner(&self, changed: impl Deref<Target = Vec<bool>>) -> Result<()> {
        let cols = terminal::size().expect("Unable to get terminal size").0;
        let mut out = stdout();
        for (i, e) in changed.iter().enumerate() {
            if *e {
                out
                    .queue(cursor::MoveTo((i % cols as usize) as u16, (i / cols as usize) as u16))?
                    .queue(PrintStyledContent(self.char_at(i)))?;
            }
        }
        out.flush()?;
        Ok(())
    }

    fn char_at(&self, i: usize) -> StyledContent<char> {
        let mut c = StyledContent::new(ContentStyle::new(), ' ');
        for layer in self.layers.iter().rev() {
            underlay(&mut c, &layer.buf[i]);
            if !underlay_possible(&c) {
                break;
            }
        }
        c
    }
}

impl Drop for Canvas {
    fn drop(&mut self) {
        if CANVAS_COUNT.fetch_sub(1, Ordering::AcqRel) == 1 {
            stdout().execute(cursor::Show).expect("Failed to un-hide the cursor");
        }
    }
}

/// The layer holds image data within a canvas
#[derive(Clone, Debug)]
pub struct Layer {
    buf: Vec<StyledContent<char>>,
    changed: Rc<RefCell<Vec<bool>>>,
}

impl Layer {
    fn size() -> usize {
        let (cols, rows) = terminal::size().expect("Unable to get terminal size");
        rows as usize * cols as usize
    }

    /// Draws and fills a rectangle to the layer
    pub fn fill_rect(&mut self, x: u16, y: u16, width: u16, height: u16, color: Color) {
        for i in y..(y+height) {
            for j in x..(x+width) {
                self.plot((j.into(), i.into()), color);
            }
        }
    }

    /*/// Draws the outline of a rectangle to the layer
    pub fn draw_rect(&mut self, x: u16, y: u16, width: u16, height: u16, color: Color) {
        /*for i in x..(x+width) {
            self.set_px(i, y, color);
            self.set_px(i, y + height - 1, color);
        }
        for i in (y+1)..(y + height - 1) {
            self.set_px(x, i, color);
            self.set_px(x + width - 1, i, color);
        }*/
        self.draw_poly(&[(x, y), (x, y + height), (x + width, y + height), (x + width, y)], color)
    }*/

    /// Draws a line connecting `p1` and `p2`.
    pub fn draw_line<T>(&mut self, p1: &Point, p2: &Point, color: Color) {
        // TODO: Use bresenham's line algorithm (it's more efficient)
        let dx = p2.0 - p1.0;
        let dy = p2.1 - p1.1;
        //let p1 = (p1.0.round() as u16, p1.1.round() as u16);
        //let p2 = (p2.0.round() as u16, p2.1.round() as u16);
        if dy <= dx {
            let m = dy / dx;
            let range: RangeInclusive<u16> = 
                if dx > T::zero() { p1.x().into().round() as u16..=p2.x().into().round() as u16 }
                else { p2.x().into().round() as u16..=p1.x().into().round() as u16 };
            for x in range {
                self.plot(&(T::from(x), m * (T::from(x) - p1.x()) + p1.y()), color);
            }
        }
        else {
            let m = dx / dy;
            let range =
                if dy > T::zero() { p1.y().into().round() as u16..=p2.y().into().round() as u16 }
                else { p2.y().into().round() as u16..=p1.y().into().round() as u16 };
            for y in range {
                self.plot(&(m * (T::from(y) - p1.y()) + p1.x(), T::from(y)), color);
            }
        }
    }

    /*/// Outlines a polygon.
    pub fn draw_poly<T>(&mut self, points: &[impl Point<T>], color: Color)
        where T: NumRef + PartialOrd + PartialEq + Into<f32> + From<u16> + Zero + Copy,
              RangeInclusive<T>: Iterator<Item = T>,
    {
        if points.len() == 1 {
            self.set_px(&points[0], color);
        }
        else if points.len() > 1 {
            for i in 0..points.len()-1 {
                self.draw_line(&points[i], &points[i + 1], color);
            }
            self.draw_line(&points[points.len() - 1], &points[0], color);
        }
    }*/

    /*pub fn fill_poly(&mut self, points: &[Point], color: Color) {
        // It doesn't matter if this is convex, concave, or complex, because
        // I want to fill the outline, so there won't ever be bits cut off 
        // from other bits, and I can guarantee that any vertex will be in the
        // polygon, and therefore an acceptable place to start a flood fill
    }*/

    /// Sets the color of one pixel of the layer
    pub fn plot(&mut self, p: (f32, f32), color: Color) {
        let i = point_to_index(&p);
        self.changed.borrow_mut()[i] = true;
        let x = p.0 * PX_SIZE.0;
        let y = p.1 * PX_SIZE.1;
        overlay(&mut self.buf[i], &style(
        if PX_SIZE.0 == 0.5 {
                if x % 1.0 == 0.0 {
                    '▌'
                }
                else {
                    '▐'
                }
            }
            else if PX_SIZE.1 == 0.5 {
                if y % 1.0 == 0.0 {
                    '▀'
                }
                else {
                    '▄'
                }
            }
            else {
                '█'
            }
        )
            .with(color));
    }
}

type Point = (f32, f32);

fn point_to_index(p: &Point) -> usize {
    let r = Canvas::resolution();
    if p.0 > r.0.into() || p.1 > r.1.into() {
        panic!("Cannot draw outside of bounds");
    }
    (p.1 * PX_SIZE.1 * terminal::size().expect("Unable to get terminal size").0 as f32 + p.0 * PX_SIZE.0).round() as usize
}

/*impl<T> Point<T> for (T, T)
    where T: NumRef + PartialOrd + PartialEq + Into<f32> + Copy
{
    fn x(&self) -> T {
        self.0
    }

    fn y(&self) -> T {
        self.1
    }
}*/

/// Overlays c2 over c1, storing the result in c1
fn overlay(c1: &mut StyledContent<char>, c2: &StyledContent<char>) {
    if !underlay_possible(c2) {
        *c1 = *c2
    }
    else if c1.content() == c2.content() || ('─'..='╬').contains(c1.content()) {
        let s1 = c1.style_mut();
        let s2 = c2.style();
        if s2.foreground_color.is_some() {
            s1.foreground_color = s2.foreground_color;
        }
        if s2.background_color.is_some() {
            s1.background_color = s2.background_color;
        }
        //let lines = [['─', '━', '═'], ]
        if ![' ', '█'].contains(c2.content()) {
            todo!()
        }
    }
    // if c2 is ' ' or '', and it can be underlayed, c2 is entirely transparent,
    // so c1 stays the same
    else if ![' ', '█'].contains(c2.content()) {
        match if c1.content() < c2.content() { (c1.content(), c2.content()) } else { (c2.content(), c1.content()) } {
            ('▌', '▐') | ('▀', '▄') => (*c1.style_mut()).background_color = c2.style().foreground_color,
            //('─', '━') => *c1 = StyledContent::new(c1.style()),
            _ => *c1 = *c2
        }
    }
}

/// Overlays c1 over c2, storing the result in c1
fn underlay(c1: &mut StyledContent<char>, c2: &StyledContent<char>) {
    let mut c2 = *c2;
    overlay(&mut c2, c1);
    *c1 = c2;
}

fn underlay_possible(c: &StyledContent<char>) -> bool {
    match c.content() {
        '█' => c.style().foreground_color.is_none(),
        '▌' | '▐' | '▀' | '▄' => {
            let style = c.style();
            style.foreground_color.is_none() || style.background_color.is_none()
        },
        _ => c.style().background_color.is_none(),
    }
}

// Parts of the following function were taken from code written by Herman J. Radtke III
// It can be found at https://hermanradtke.com/2015/01/12/terminal-window-size-with-rust-ffi.html
// The original code is licensed under CC BY 4.0 (https://creativecommons.org/licenses/by/4.0/)
// Changes have been made to the code
//
// It is changed enough that I feel it would be acceptable to use, as I don't feel that two lines
// of code are enough to constitute a "creative work," especially since this is the only way to 
// use the ioctl function that I am aware of, but it took way to long to find this, so I'm putting
// a shoutout to hjr3 for posting the code.
#[cfg(unix)]
fn get_winsize() -> Result<winsize> {
    let w = winsize { ws_row: 0, ws_col: 0, ws_xpixel: 0, ws_ypixel: 0 };
    let r = unsafe { ioctl(STDOUT_FILENO, TIOCGWINSZ, &w) };

    debug_assert_eq!((w.ws_col, w.ws_row), terminal::size().unwrap());

    if r == 0 && w.ws_xpixel > 0 && w.ws_ypixel > 0 && w.ws_col > 0 && w.ws_row > 0 {
        Ok(w)
    } 
    else {
        Err(io::Error::from(io::ErrorKind::Other).into())
    }
}