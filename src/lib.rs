// Copyright 2020 Liam Bloom
// SPDX-License-Identifier: Apache-2.0

//! # Graphic CLI
//! 
//! This library allows you to create a GUI in the command line.

// TODO: Move functions to Canvas if they can be
// TODO: Use mpsc to send data from layer to canvas

// TODO: To make the screen restore when done (like vim) use:
// crossterm::terminal::{EnterAlternateScreen, LeaveAlternateScreen}

#![warn(missing_docs)]

use std::{
    cell::RefCell, 
    io::{stdout, Read, Write}, 
    iter::Iterator, 
    ops::Deref,
    rc::Rc, 
    sync::{Once, atomic::{AtomicUsize, Ordering}}
};
use crossterm::{
    QueueableCommand, ExecutableCommand,
    tty::IsTty,
    style::{ContentStyle, StyledContent, PrintStyledContent, style},
    terminal, cursor,
};
use bmp;
//pub use rasterizer::{Rasterizer, Point};
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
}

impl Layer {
    /// Sets the color of one pixel of the layer
    pub fn plot(&mut self, p: IPoint, color: Color) -> Result<()> {
        //p = (p.0.floor(), p.1.floor());
        self.validate_iPoints(&[p])?;
        let r = Canvas::resolution();
        if p.0 > r.0 || p.1 > r.1 {
            panic!("Cannot draw outside of bounds");
        }
        let i = (p.1 as f32 * PX_SIZE.1) as usize * terminal::size().expect("Unable to get terminal size").0 as usize + (p.0 as f32 * PX_SIZE.0) as usize;
        //let i = iPoint_to_index(&p);
        self.changed.borrow_mut()[i] = true;
        let x = p.0 as f32 * PX_SIZE.0;
        let y = p.1 as f32 * PX_SIZE.1;
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
        Ok(())
    }

    fn resolution(&self) -> (usize, usize) {
        let res = Canvas::resolution();
        (res.0 as usize, res.0 as usize)
    }

    /// Draws a line connecting points `p0` and `p1`
    pub fn line(&mut self, p0: FPoint, p1: FPoint, color: Color) -> Result<()> {
        // Bresenham's Line algorithm with sub-pixel precision is here
        // https://stackoverflow.com/questions/41195973/how-to-use-bresenhams-line-drawing-algorithm-with-sub-pixel-bias
        // I'm not using it because having a high precision (a high `scale`), the loop
        // runs like a bazillion times, so it doesn't matter how efficient the algorithm
        // is, it's not that much faster than DDA. DDA is a slightly slower algorithm, 
        // but it's easier, and loops fewer times (far fewer with a high precision).
        // Maybe I could even modify DDA to keep track of how far off it is to be self
        // correcting (because DDA becomes offset from correct after a long distance)
        self.validate_fPoints(&[p0, p1])?;
        let dx = p1.0 - p0.0;
        let dy = p1.1 - p0.1;
        let steps;
        let mut x;
        let mut y;
        if dx.abs() > dy.abs() {
            x = p0.0.floor() + 0.5;
            y = p0.1 + (p0.0 - x) * (dy / dx);
            steps = dx.abs();
        } 
        else {
            y = p0.1.floor() + 0.5;
            x = p0.0 + (p0.1 - y) * (dx / dy);
            steps = dy.abs();
        }
        let x_step = dx / steps;
        let y_step = dy / steps;
        for _ in 0..=steps.round() as i32 {
            self.plot((x as u16, y as u16), color)?;
            x += x_step;
            y += y_step;
        }
        Ok(())
    }

    /// Draws and fills a rectangle to the layer
    pub fn fill_rect(&mut self, x: u16, y: u16, width: u16, height: u16, color: Color) -> Result<()> {
        self.validate_iPoints(&[(x, y), (width, height)])?;
        for y in y..(y + height) {
            for x in x..(x + width) {
                self.plot((x, y), color)?;
            }
        }
        Ok(())
    }

    pub fn draw_img(&mut self, x: u16, y: u16, src: &mut impl Read) -> Result<()> {
        let img = bmp::from_reader(src)?;
        for px in img.coordinates() {
            let px_color = img.get_pixel(px.0, px.1);
            self.plot((x + px.0 as u16, y + px.1 as u16), Color::Rgb { r: px_color.r, g: px_color.g, b: px_color.b })?;
        }
        Ok(())
    }

    fn validate_fPoints(&self, points: &[FPoint]) -> Result<()> {
        let resolution = self.resolution();
        let resolution = (resolution.0 as f32, resolution.1 as f32);
        for point in points {
            if point.0 < 0.0 || point.0.round() >= resolution.0 || point.1 < 0.0 || point.1.round() >= resolution.1 {
                return Err(ErrorKind::InvalidPoint(point.0, point.1))
            }
        }
        Ok(())
    }

    fn validate_iPoints(&self, points: &[IPoint]) -> Result<()> {
        let resolution = self.resolution();
        let resolution = (resolution.0 as u16, resolution.1 as u16);
        for point in points {
            if point.0 < 0 || point.0 >= resolution.0 || point.1 < 0 || point.1 >= resolution.1 {
                return Err(ErrorKind::InvalidPoint(point.0 as f32, point.1 as f32))
            }
        }
        Ok(())
    }
}

/// Point type
pub type FPoint = (f32, f32);
pub type IPoint = (u16, u16);

/*fn fPoint_to_index(p: &FPoint) -> usize {
    let r = Canvas::resolution();
    if p.0 > r.0.into() || p.1 > r.1.into() {
        panic!("Cannot draw outside of bounds");
    }
    (p.1 * PX_SIZE.1 * terminal::size().expect("Unable to get terminal size").0 as f32) as usize + (p.0 * PX_SIZE.0) as usize
}

fn iPoint_to_index(p: &IPoint) -> usize {
    let r = Canvas::resolution();
    if p.0 > r.0 || p.1 > r.1 {
        panic!("Cannot draw outside of bounds");
    }
    (p.1 as f32 * PX_SIZE.1 * terminal::size().expect("Unable to get terminal size").0 as f32) as usize + (p.0 as f32 * PX_SIZE.0) as usize
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