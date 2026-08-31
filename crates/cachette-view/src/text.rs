//! The glyphs the head-up display writes with.
//!
//! A framebuffer holds pixels and nothing else. It has no font, so the viewer
//! must carry one. This module holds the one the viewer uses, and it holds
//! only the lookup. The painting is on the canvas, where every other pixel
//! write already is.
//!
//! # Why a compiled-in bitmap font
//!
//! The viewer needs one size of one face, in one colour, for short lines of
//! plain text. A scalable font brings a rasteriser, a hinting model and a
//! font file that the binary must find at run time. None of that serves a
//! label that reads "tick".
//!
//! A bitmap font is a table. It compiles into the binary, it needs no file on
//! disk, and it draws with a bit test. The table this module uses is public
//! domain, and the crate that carries it has no dependency of its own. The
//! licence audit accepts it.[^1]
//!
//! The cost is that the text has one size. The viewer scales a glyph by a
//! whole number when it wants a larger one, which keeps every edge on a pixel
//! boundary.
//!
//! # References
//!
//! [^1]: The licence and advisory audit. `deny.toml`

use font8x8::legacy::BASIC_LEGACY;

/// The width of one glyph cell in pixels, before any scale.
pub const GLYPH_WIDTH: i32 = 8;

/// The height of one glyph cell in pixels, before any scale.
pub const GLYPH_HEIGHT: i32 = 8;

/// Returns the rows of one glyph, or the rows of a space.
///
/// Each row is a byte. The lowest bit is the leftmost pixel. A character
/// outside the table draws as a space, so an unexpected byte leaves a gap
/// rather than a wrong picture.
#[must_use]
pub fn glyph(character: char) -> [u8; 8] {
    let code = character as usize;
    if code < BASIC_LEGACY.len() {
        BASIC_LEGACY[code]
    } else {
        [0; 8]
    }
}

/// Returns the width of a line in pixels, at the given scale.
#[must_use]
pub fn width_of(line: &str, scale: i32) -> i32 {
    line.chars().count() as i32 * GLYPH_WIDTH * scale
}
