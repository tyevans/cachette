//! The standard every panel of the viewer follows.
//!
//! # Why this is in Rust and not in the control plane
//!
//! The viewer lays the panel out. The control plane owns the loop, the camera
//! and the pixel memory, and it asks the engine for one frame. It places no
//! glyph and it measures no text.[^1] A layout standard written in Python
//! would therefore describe work that nothing in Python performs, which is a
//! capability nobody invokes.[^2]
//!
//! # What this module holds
//!
//! It holds the geometry of a panel, the colours a panel writes with, the one
//! writer that cuts text to a right edge, the line kinds a panel is built
//! from, and the list of panels the viewer knows.
//!
//! **Every declaration here is the only one.** The head-up display reads its
//! width, its padding and its line height from this module. A second copy of
//! any of them would be one fact in two places, with nothing to fail when the
//! copies drift.[^3]
//!
//! # The cut
//!
//! A panel is a rectangle of fixed width. Text longer than that rectangle used
//! to run over the map, because only one line kind cut its text and the other
//! kinds wrote from the left with no right bound.[^4]
//!
//! One writer now takes a right edge and cuts to it. A caller cannot ask it to
//! write past that edge. The bound is derived from the panel width and from
//! the glyph table, so no author has to know a character count.
//!
//! # What a panel may read
//!
//! A panel reads what the engine already holds, at a bounded number of
//! addresses. It starts no pass over the world, and its cost never follows the
//! extent of the world or the population.[^5] A number the panel cannot
//! afford is stated as absent, never as a zero a reader cannot tell from a
//! true one.[^6]
//!
//! # References
//!
//! [^1]: ADR-0094, the caller owns the camera and the pixels, decision D1. `docs/adrs/draft/adr-0094-the-caller-owns-the-camera-and-the-pixels.md`
//! [^2]: Recurring Defect Shapes, shape 3. `.claude/rules/recurring-defects.md`
//! [^3]: Recurring Defect Shapes, shape 1. `.claude/rules/recurring-defects.md`
//! [^4]: Backlog item 0300. `docs/backlog/complete/0300-cut-every-panel-line-to-the-width-of-the-panel.md`
//! [^5]: ADR-0070, the head-up display reports what the drawing pass read, decision D1. `docs/adrs/accepted/adr-0070-the-head-up-display-reports-what-the-drawing-pass-read.md`
//! [^6]: ADR-0070, the head-up display reports what the drawing pass read, decision D2. `docs/adrs/accepted/adr-0070-the-head-up-display-reports-what-the-drawing-pass-read.md`

pub mod characters;
pub mod determinism;
pub mod events;
pub mod inspector;
pub mod market;
pub mod statistics;
pub mod weather;

use cachette_core::{Axial, World};

use crate::paint::{Camera, Canvas, Focus};
use crate::text;

/// The gap between the frame edge and a panel, in pixels.
pub const MARGIN: i32 = 14;

/// The width of a panel, in pixels.
pub const WIDTH: i32 = 268;

/// The gap between the panel edge and its text, in pixels.
pub const PAD: i32 = 13;

/// The distance between one line of text and the next, in pixels.
pub const LINE: i32 = 12;

/// The height of a title line, in pixels.
pub const TITLE_LINE: i32 = 18;

/// The height of a hairline between two sections, in pixels.
pub const RULE_LINE: i32 = 8;

/// The offset of the value column from the panel text edge, in pixels.
pub const VALUE_COLUMN: i32 = 96;

/// The height of a share bar, in pixels.
pub const BAR_HEIGHT: i32 = 5;

/// The gap between one panel of the deck and the next, in pixels.
pub const GAP: i32 = 8;

/// The colour of a panel, mixed over the world.
pub const PANEL: u32 = 0x0009_0e12;

/// How much of the panel colour covers the world under it.
pub const PANEL_WEIGHT: u8 = 224;

/// The colour of a panel edge and of the rules between sections.
pub const EDGE: u32 = 0x0027_3a44;

/// The colour of a section heading.
pub const HEADING: u32 = 0x0074_a6ba;

/// The colour of a label.
pub const LABEL: u32 = 0x0069_7d87;

/// The colour of a value.
pub const VALUE: u32 = 0x00d6_e4ea;

/// The colour of a title.
pub const TITLE: u32 = 0x00e8_c84a;

/// The width of the swatch a legend line draws, in pixels.
pub const SWATCH: i32 = text::GLYPH_HEIGHT;

/// The gap between a swatch and the label beside it, in pixels.
pub const SWATCH_GAP: i32 = 6;

/// The colour of the mark that says the panel cut a line.
///
/// **The mark is what makes a cut visible to the person watching.** A cut line
/// states something other than what it was given, and it does so in
/// silence.[^1] The check that finds a cut had one caller and it was a test, so
/// no run ever asked whether the panel cut something.[^2] The drawing now asks
/// on every line of every frame, and it paints this mark when the answer is
/// yes.
///
/// # References
///
/// [^1]: Backlog item 0300. `docs/backlog/complete/0300-cut-every-panel-line-to-the-width-of-the-panel.md`
/// [^2]: Backlog item 0072. `docs/backlog/complete/0072-run-the-panel-fit-check-in-the-drawing-pass.md`
pub const CUT_MARK: u32 = 0x00e0_5a3c;

/// The width and the height of the mark that says the panel cut a line.
pub const CUT_MARK_SIZE: i32 = 4;

/// Returns the pixel room a panel gives its text.
#[must_use]
pub const fn text_room() -> i32 {
    WIDTH - PAD * 2
}

/// Returns the pixel room a panel gives a value in the value column.
#[must_use]
pub const fn value_room() -> i32 {
    text_room() - VALUE_COLUMN
}

/// Reports whether a text fits the room it is given.
///
/// A test calls this on a string of its own, to prove that the check can
/// answer no. A check with no proven failure mode is decoration.[^1]
///
/// # References
///
/// [^1]: Testing Rules, a determinism test must be able to fail. `.claude/rules/testing.md`
#[must_use]
pub fn fits(line: &str, room: i32, scale: i32) -> bool {
    text::width_of(line, scale) <= room
}

/// Returns as much of a text as the room holds, in whole glyphs.
///
/// The bound is derived from the room and from the glyph table. Nobody counts
/// characters, so nobody can count them wrongly.
#[must_use]
pub fn fit(line: &str, room: i32, scale: i32) -> String {
    let cells = (room / (text::GLYPH_WIDTH * scale.max(1))).max(0);
    // The value is held at zero first, so the conversion cannot fail.
    let cells = usize::try_from(cells).unwrap_or(0);
    line.chars().take(cells).collect()
}

/// Writes a text from the left, and never past the right edge.
///
/// **This is the only writer a panel uses.** A caller supplies the edge and
/// the text is cut to it, so no text a panel holds can reach the map.[^1]
///
/// Returns whether the text was cut. A cut is still a defect, because the
/// panel then states something other than what it was given, and a test reads
/// the same answer to find one.
///
/// # References
///
/// [^1]: Backlog item 0300. `docs/backlog/complete/0300-cut-every-panel-line-to-the-width-of-the-panel.md`
pub fn write_fitted(
    canvas: &mut Canvas,
    left: i32,
    right: i32,
    pen: i32,
    line: &str,
    scale: i32,
    colour: u32,
) -> bool {
    // The mark takes room from the text, so the cut is measured against what
    // is left. A text that fits without the mark still fits, because the mark
    // is only drawn when the text does not.
    let shown = fit(line, right - left, scale);
    let cut = shown.chars().count() != line.chars().count();
    if cut {
        let shown = fit(line, right - left - CUT_MARK_SIZE - 2, scale);
        canvas.write(left, pen, &shown, scale, colour);
        mark_the_cut(canvas, right, pen);
    } else {
        canvas.write(left, pen, &shown, scale, colour);
    }
    cut
}

/// Paints the mark that says the panel cut a line.
///
/// **This runs on the drawing path, on every line of every frame.** A check
/// that only a test invokes is a capability nobody invokes, and this project
/// records that shape.[^1] The mark is visible to the person watching, which
/// is what the panel is for.
///
/// # References
///
/// [^1]: Recurring Defect Shapes, shape 3. `.claude/rules/recurring-defects.md`
fn mark_the_cut(canvas: &mut Canvas, right: i32, pen: i32) {
    canvas.block(
        right - CUT_MARK_SIZE,
        pen + 2,
        CUT_MARK_SIZE,
        CUT_MARK_SIZE,
        CUT_MARK,
    );
}

/// Writes a text against the right edge, and never past the left one.
///
/// The text is cut to the room between the two edges first, so a long value
/// cannot walk left over the label beside it.
pub fn write_against_right(
    canvas: &mut Canvas,
    left: i32,
    right: i32,
    pen: i32,
    line: &str,
    colour: u32,
) -> bool {
    let room = right - left;
    let whole = fit(line, room, 1);
    let cut = whole.chars().count() != line.chars().count();
    // The mark sits against the right edge, so a cut value stops short of it.
    let shown = if cut {
        fit(line, room - CUT_MARK_SIZE - 2, 1)
    } else {
        whole
    };
    let edge = if cut {
        right - CUT_MARK_SIZE - 2
    } else {
        right
    };
    let start = left.max(edge - text::width_of(&shown, 1));
    canvas.write(start, pen, &shown, 1, colour);
    if cut {
        mark_the_cut(canvas, right, pen);
    }
    cut
}

/// One line of a panel.
///
/// **A panel is a list of these and nothing else states what it holds.** The
/// height is summed from the list and the painting walks the list, so the two
/// cannot disagree.[^1]
///
/// # References
///
/// [^1]: Recurring Defect Shapes, shape 1. `.claude/rules/recurring-defects.md`
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum Line {
    /// The name of the panel.
    Title(String),
    /// A dim line that runs the width of the panel.
    Note(String),
    /// A hairline between two sections.
    Rule,
    /// The name of a section.
    Heading(String),
    /// A label on the left and a value against the right edge.
    Row(String, String),
    /// A colour swatch, a label, and a value against the right edge.
    Swatch(u32, String, String),
}

impl Line {
    /// Builds a note.
    #[must_use]
    pub fn note(text: impl Into<String>) -> Self {
        Self::Note(text.into())
    }

    /// Builds a section heading.
    #[must_use]
    pub fn heading(text: impl Into<String>) -> Self {
        Self::Heading(text.into())
    }

    /// Builds a row of a label and a value.
    #[must_use]
    pub fn row(label: impl Into<String>, value: impl Into<String>) -> Self {
        Self::Row(label.into(), value.into())
    }

    /// Builds a row with a colour swatch before the label.
    #[must_use]
    pub fn swatch(colour: u32, label: impl Into<String>, value: impl Into<String>) -> Self {
        Self::Swatch(colour, label.into(), value.into())
    }

    /// Returns the height this line occupies, in pixels.
    #[must_use]
    pub const fn height(&self) -> i32 {
        match self {
            Self::Title(_) => TITLE_LINE,
            Self::Note(_) | Self::Heading(_) | Self::Row(_, _) | Self::Swatch(_, _, _) => LINE,
            Self::Rule => RULE_LINE,
        }
    }

    /// Returns what this line says, as one string, or nothing for a rule.
    ///
    /// A test reads this rather than the pixels, so a test reads what a
    /// watcher sees.
    #[must_use]
    pub fn says(&self) -> Option<String> {
        match self {
            Self::Rule => None,
            Self::Title(text) | Self::Note(text) | Self::Heading(text) => Some(text.clone()),
            Self::Row(label, value) | Self::Swatch(_, label, value) => {
                Some(format!("{label}: {value}"))
            }
        }
    }

    /// Reports whether the panel has to cut this line to draw it.
    ///
    /// A cut line states something other than what it was given. The drawing
    /// cuts rather than escaping the panel, and this is how a test sees that a
    /// cut happened.
    #[must_use]
    pub fn is_cut(&self) -> bool {
        match self {
            Self::Rule => false,
            Self::Title(text) => !fits(text, text_room(), 2),
            Self::Note(text) | Self::Heading(text) => !fits(text, text_room(), 1),
            // A label and a value share one row. Each is bounded by the panel
            // edge, so neither can reach the map. The pair does not fit when
            // the two together are wider than the row, because the value is
            // written against the right edge and would then sit over the
            // label.
            Self::Row(label, value) => {
                !fits(value, value_room(), 1)
                    || text::width_of(label, 1) + text::width_of(value, 1) > text_room()
            }
            Self::Swatch(_, label, value) => {
                !fits(value, value_room(), 1)
                    || SWATCH + SWATCH_GAP + text::width_of(label, 1) + text::width_of(value, 1)
                        > text_room()
            }
        }
    }
}

/// What a panel is given to read.
///
/// It holds the world, the camera, the size of the frame, the unit the drawing
/// pass focused on, and the tile the watcher pointed at. A panel reads through
/// this and never writes.[^1]
///
/// # References
///
/// [^1]: ADR-0067, the viewer reads the world and never writes to it, decision D1. `docs/adrs/accepted/adr-0067-the-viewer-reads-the-world-and-never-writes-to-it.md`
#[derive(Clone, Copy)]
pub struct View<'a> {
    /// The world, borrowed shared.
    pub world: &'a World,
    /// The camera the frame was drawn with.
    pub camera: Camera,
    /// The width of the frame, in pixels.
    pub frame_width: usize,
    /// The height of the frame, in pixels.
    pub frame_height: usize,
    /// The unit the drawing pass fixed on, when it found one.
    pub focus: Option<Focus>,
    /// The tile the watcher pointed at, when the watcher pointed at one.
    pub pointer: Option<Axial>,
}

/// A panel of the deck.
///
/// **A panel is one file.** It states its own name and its own lines, and it
/// appears in the list this module holds. Nothing else has to change when a
/// panel joins the deck.
///
/// A panel reads what the engine already holds, at a bounded number of
/// addresses. Its cost never follows the extent of the world or the
/// population.[^1]
///
/// # References
///
/// [^1]: ADR-0070, the head-up display reports what the drawing pass read, decision D1. `docs/adrs/accepted/adr-0070-the-head-up-display-reports-what-the-drawing-pass-read.md`
pub trait Panel: Sync {
    /// Returns the name a caller selects this panel by.
    ///
    /// The name is lower case and holds no space, because a caller types it.
    fn name(&self) -> &'static str;

    /// Returns the title the panel writes at its head.
    fn title(&self) -> &'static str;

    /// Returns the lines the panel holds, in order.
    ///
    /// This is the whole content of the panel. Nothing else states what it
    /// says.
    fn lines(&self, view: &View<'_>) -> Vec<Line>;
}

/// The panels the viewer knows, in the order the deck stacks them.
///
/// **This list is the registration.** A panel that is not here is not drawn,
/// and a caller cannot name it.
#[must_use]
pub fn registered() -> &'static [&'static (dyn Panel + 'static)] {
    &[
        &statistics::Statistics,
        &events::Events,
        &characters::Characters,
        &inspector::Inspector,
        &determinism::Determinism,
        &weather::Weather,
        &market::Market,
    ]
}

/// The panels a frame draws.
///
/// A set is a bit for each registered panel, so it is a small copied value and
/// a caller may hold one between frames.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct Set(u32);

impl Set {
    /// The set that draws no panel.
    pub const EMPTY: Self = Self(0);

    /// Returns the set that draws every registered panel.
    #[must_use]
    pub fn all() -> Self {
        let count = u32::try_from(registered().len()).unwrap_or(0);
        Self(if count >= 32 {
            u32::MAX
        } else {
            (1u32 << count) - 1
        })
    }

    /// Reports whether the set holds no panel.
    #[must_use]
    pub const fn is_empty(self) -> bool {
        self.0 == 0
    }

    /// Returns the set with the panel of this name added.
    ///
    /// Returns `None` when no registered panel carries the name. A caller that
    /// asked for a panel that does not exist is refused rather than given a
    /// frame with nothing on it, because those two look the same.
    #[must_use]
    pub fn with(self, name: &str) -> Option<Self> {
        let at = registered().iter().position(|panel| panel.name() == name)?;
        Some(Self(self.0 | (1u32 << at)))
    }

    /// Reports whether the set holds the panel at this position.
    #[must_use]
    pub const fn holds(self, at: usize) -> bool {
        at < 32 && (self.0 >> at) & 1 == 1
    }
}

/// Returns the height a list of lines needs, in pixels.
#[must_use]
pub fn height_of(lines: &[Line]) -> i32 {
    lines.iter().map(Line::height).sum::<i32>() + PAD * 2
}

/// What a panel says when the frame was too short to hold it.
pub const CUT_NOTICE: &str = "frame too short. panel cut.";

/// Returns as many lines as a rectangle of this height holds.
///
/// **The last line says the panel was cut.** A number that is missing and says
/// so is a number a reader knows to look elsewhere for. A number that is
/// missing in silence is the failure the record forbids.[^1]
///
/// # References
///
/// [^1]: ADR-0070, the head-up display reports what the drawing pass read, decision D2. `docs/adrs/accepted/adr-0070-the-head-up-display-reports-what-the-drawing-pass-read.md`
#[must_use]
pub fn cut_to_fit(lines: Vec<Line>, room: i32) -> Vec<Line> {
    let notice = Line::note(CUT_NOTICE);
    let whole: i32 = lines.iter().map(Line::height).sum();
    if whole <= room {
        return lines;
    }
    let mut kept = Vec::with_capacity(lines.len());
    let mut used = notice.height();
    for line in lines {
        let height = line.height();
        if used + height > room {
            break;
        }
        used += height;
        kept.push(line);
    }
    kept.push(notice);
    kept
}

/// Draws one panel at a place, and returns the height it took.
///
/// The painting walks the same list the height was summed from, so the panel
/// cannot paint past the rectangle it states.
pub fn draw_one(canvas: &mut Canvas, left: i32, top: i32, lines: &[Line]) -> i32 {
    let height = height_of(lines);
    canvas.shade(left, top, WIDTH, height, PANEL, PANEL_WEIGHT);
    outline(canvas, left, top, WIDTH, height);

    let text_left = left + PAD;
    let text_right = left + WIDTH - PAD;
    let mut pen = top + PAD;
    for line in lines {
        paint(canvas, text_left, text_right, pen, line);
        pen += line.height();
    }
    height
}

/// Draws the panel edge.
fn outline(canvas: &mut Canvas, x: i32, y: i32, width: i32, height: i32) {
    canvas.block(x, y, width, 1, EDGE);
    canvas.block(x, y + height - 1, width, 1, EDGE);
    canvas.block(x, y, 1, height, EDGE);
    canvas.block(x + width - 1, y, 1, height, EDGE);
}

/// Paints one line at the given position.
///
/// Every branch writes through the fitted writer, so no branch can write past
/// the right edge.
fn paint(canvas: &mut Canvas, left: i32, right: i32, pen: i32, line: &Line) {
    match line {
        Line::Title(text) => {
            write_fitted(canvas, left, right, pen, text, 2, TITLE);
        }
        Line::Note(text) => {
            write_fitted(canvas, left, right, pen, text, 1, LABEL);
        }
        Line::Heading(text) => {
            write_fitted(canvas, left, right, pen, text, 1, HEADING);
        }
        Line::Rule => canvas.block(left, pen + 3, right - left, 1, EDGE),
        // A label is bounded by the panel edge and not by the value column.
        // The bound that matters is the one that stops text reaching the map,
        // and a label wider than the column is still legible, because the
        // value is written against the right edge. The pair is reported as a
        // cut when the two together are wider than the row, which is the case
        // where the value would sit over the label.
        Line::Row(label, value) => {
            let column = left + VALUE_COLUMN;
            write_fitted(canvas, left, right, pen, label, 1, LABEL);
            write_against_right(canvas, column, right, pen, value, VALUE);
        }
        Line::Swatch(colour, label, value) => {
            canvas.block(left, pen, SWATCH, SWATCH, *colour);
            let text_left = left + SWATCH + SWATCH_GAP;
            let column = left + VALUE_COLUMN;
            write_fitted(canvas, text_left, right, pen, label, 1, LABEL);
            write_against_right(canvas, column, right, pen, value, VALUE);
        }
    }
}

/// Draws the panels a set names, down the right side of the frame.
///
/// The deck stacks from the top. A panel that would run past the foot of the
/// frame is cut, and the cut says so.
///
/// Returns what each drawn panel held, in the order it was drawn, so a test
/// reads what a watcher sees rather than reading the pixels.
pub fn draw_deck(view: &View<'_>, set: Set, canvas: &mut Canvas) -> Vec<Vec<Line>> {
    let mut drawn = Vec::new();
    if set.is_empty() {
        return drawn;
    }
    let left = i32::try_from(canvas.width()).unwrap_or(i32::MAX) - MARGIN - WIDTH;
    let foot = i32::try_from(canvas.height()).unwrap_or(i32::MAX) - MARGIN;
    let mut top = MARGIN;

    for (at, panel) in registered().iter().enumerate() {
        if !set.holds(at) {
            continue;
        }
        let room = foot - top - PAD * 2;
        if room < LINE {
            break;
        }
        let mut lines = vec![Line::Title(panel.title().to_string())];
        lines.extend(panel.lines(view));
        let lines = cut_to_fit(lines, room);
        top += draw_one(canvas, left, top, &lines) + GAP;
        drawn.push(lines);
    }
    drawn
}

/// Returns what a set of panels says, one string for each line.
///
/// This is what a test asserts on. It is taken before the cut, so it says what
/// the panels hold and not what fitted in the frame they were last drawn at.
#[must_use]
pub fn says(view: &View<'_>, set: Set) -> Vec<String> {
    let mut said = Vec::new();
    for (at, panel) in registered().iter().enumerate() {
        if !set.holds(at) {
            continue;
        }
        said.push(panel.title().to_string());
        said.extend(panel.lines(view).iter().filter_map(Line::says));
    }
    said
}

/// Returns every line of a set of panels that the drawing has to cut.
///
/// A cut line states something other than what it was given, and it does so
/// silently. This is how a test sees the cut. A test that only checked the
/// panel edge would pass because of the cut rather than in spite of it.
#[must_use]
pub fn lines_that_do_not_fit(view: &View<'_>, set: Set) -> Vec<String> {
    let mut bad = Vec::new();
    for (at, panel) in registered().iter().enumerate() {
        if !set.holds(at) {
            continue;
        }
        for line in panel.lines(view) {
            if line.is_cut() {
                if let Some(text) = line.says() {
                    bad.push(text);
                }
            }
        }
    }
    bad
}
