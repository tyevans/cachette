//! Writes a canvas to a file a person can look at.
//!
//! The viewer draws into a framebuffer. A framebuffer is not a file, so
//! nothing outside the window can see what the viewer drew. This module turns
//! a canvas into bytes.
//!
//! The format is binary PPM. Every image tool reads it, and it needs no
//! dependency. The example that writes a frame uses this, and so does a test
//! that must show a person the picture it got.
//!
//! There is one writer, not one for each caller. A second writer would be one
//! fact in two places, and nothing would fail when the copies disagreed.[^1]
//!
//! # References
//!
//! [^1]: Recurring Defect Shapes, shape 1. `.claude/rules/recurring-defects.md`

use std::io::Write;

use crate::paint::Canvas;

/// Writes a canvas as a binary PPM image.
///
/// Each pixel of the canvas holds one byte for each of red, green and blue,
/// in the low three bytes. The top byte is ignored.
///
/// # Errors
///
/// Returns the error the writer gave.
pub fn write_ppm(canvas: &Canvas, out: &mut impl Write) -> std::io::Result<()> {
    let (width, height) = (canvas.width(), canvas.height());
    write!(out, "P6\n{width} {height}\n255\n")?;
    let mut bytes = Vec::with_capacity(width * height * 3);
    for pixel in canvas.pixels() {
        bytes.push(((pixel >> 16) & 0xff) as u8);
        bytes.push(((pixel >> 8) & 0xff) as u8);
        bytes.push((pixel & 0xff) as u8);
    }
    out.write_all(&bytes)
}
