//! The cards the window draws over the map.
//!
//! # Why a card and not a panel
//!
//! The panel grew to thirteen sections and became longer than the window. It
//! cut at the foot and said so, and a watcher could not reach the rows below
//! the notice.[^1] Adding a section made the cut worse, and ordering the
//! sections bought one placement and could not buy a second.[^2]
//!
//! **The window shows what changes moment to moment. The record of a moment
//! goes to the inspection path.**[^9] A quantity that a watcher reads once, or
//! reads occasionally, does not earn a place on the glass. The panel still
//! exists and still holds every section, and a person reaches it with one
//! command that renders it to an image.[^3]
//!
//! The map fills the window. The cards sit over it and cover a small part of
//! it, so a watcher reads the world and the numbers at once.
//!
//! # What a card may say
//!
//! A card states what the panel states. It reads the same readout, which is
//! taken once against the world and the finished canvas, so no number here
//! can disagree with the same number in the panel.[^4] The glass adds no
//! reader of its own and starts no pass over the world.[^5]
//!
//! # The reference layer
//!
//! Three things a watcher checks occasionally rather than continuously: which
//! faction each colour is, which ground each colour is, where the camera sits,
//! and what the step costs. They appear while a key is held and they vanish
//! when it is released.
//!
//! The product record asks that the window name every colour it draws.[^6] A
//! legend on a key is the window naming its colours, so the record holds. The
//! reason to put it on a key is that a hidden layer takes no space from the
//! map. The record was amended to describe the window as it now is, and the
//! register holds the reasoning.[^7]
//!
//! The key holds no state between frames. The caller passes what the keyboard
//! says, in the same way it passes the camera, and nothing reaches the
//! engine.[^8]
//!
//! # References
//!
//! [^1]: Backlog item 0133. `docs/backlog/complete/0133-let-a-watcher-reach-a-panel-longer-than-the-window.md`
//! [^2]: Decisions register, DEC-078. `docs/DECISIONS.md`
//! [^3]: The panel picture recipe. `justfile`
//! [^4]: ADR-0070, the head-up display reports what the drawing pass read, decision D1. `docs/adrs/accepted/adr-0070-the-head-up-display-reports-what-the-drawing-pass-read.md`
//! [^5]: ADR-0070, the head-up display reports what the drawing pass read, decision D2. `docs/adrs/accepted/adr-0070-the-head-up-display-reports-what-the-drawing-pass-read.md`
//! [^6]: PRD-0005, a watcher can tell what is happening and why. `docs/product/shipped/prd-0005-a-watcher-can-tell-what-is-happening-and-why.md`
//! [^7]: Decisions register, DEC-084. `docs/DECISIONS.md`
//! [^8]: ADR-0067, the viewer reads the world and never writes to it, decision D2. `docs/adrs/accepted/adr-0067-the-viewer-reads-the-world-and-never-writes-to-it.md`
//! [^9]: ADR-0093, the window shows what changes, and the record of a moment goes to the inspection path, decisions D1 to D5. `docs/adrs/draft/adr-0093-the-window-shows-what-changes.md`

use cachette_core::resource::ResourceKind;
use cachette_core::terrain::KIND_COUNT;
use cachette_core::NeedCondition;

use crate::hud::KINDS;
use crate::hud::{
    accumulated, fraction, grouped, name_of, option_name, resource_name, ChoiceReadout, Readout,
    TileReadout,
};
use crate::paint::{faction_colour, kind_colour, Canvas, COLOURED_FACTIONS};
use crate::text;

/// The gap between the window edge and a card, in pixels.
const MARGIN: i32 = 14;

/// The gap between a card edge and its text, in pixels.
const PAD: i32 = 9;

/// The distance between one row of a card and the next, in pixels.
const LINE: i32 = 12;

/// The gap between one card and the next in a stack, in pixels.
const GAP: i32 = 8;

/// The gap between the widest label of a card and its value column.
const COLUMN_GAP: i32 = 18;

/// The side of a colour swatch, in pixels.
const SWATCH: i32 = 8;

/// The gap between a swatch and the label beside it, in pixels.
const SWATCH_GAP: i32 = 6;

/// The colour of a card, mixed over the map.
///
/// A card sits over the world and lets a little of it through, so a watcher
/// sees that the map continues behind the numbers. The weight is high, because
/// a card must stay legible over bright ground as well as over water.
///
/// The palette is the viewer's own. No record binds it.
const CARD: u32 = 0x0006_0a0e;

/// How much of the card colour covers the map under it.
const CARD_WEIGHT: u8 = 236;

/// The colour of a card edge.
const EDGE: u32 = 0x0031_4a57;

/// The colour of a card heading.
const HEADING: u32 = 0x0074_a6ba;

/// The colour of a label.
const LABEL: u32 = 0x0072_868f;

/// The colour of a value.
const VALUE: u32 = 0x00e6_f0f5;

/// The colour of the line that says where the rest of the numbers are.
const HINT: u32 = 0x0059_6b74;

/// What the window draws over the map.
///
/// The caller chooses. The window draws the cards, and the picture that a
/// person reads without a display draws the whole panel, so nothing the panel
/// holds is lost when the window stops showing it.[^1]
///
/// # References
///
/// [^1]: Decisions register, DEC-084. `docs/DECISIONS.md`
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Overlay {
    /// The cards, over the map. What the window shows.
    ///
    /// The reference layer names the colours, says where the camera sits and
    /// states the cost. It appears while the watcher holds a key.
    Glass {
        /// Whether the watcher is asking for the reference layer.
        reference: bool,
    },
    /// The whole panel, every section. What a rendered picture holds.
    Panel,
}

/// One row of a card.
struct Row {
    /// The name of the quantity.
    label: String,
    /// The quantity.
    value: String,
    /// A colour the row stands for, drawn as a swatch before the label.
    swatch: Option<u32>,
}

impl Row {
    /// Builds a row with no swatch.
    fn new(label: &str, value: String) -> Self {
        Self {
            label: label.to_string(),
            value,
            swatch: None,
        }
    }

    /// Builds a row that names a colour.
    fn coloured(colour: u32, label: String, value: String) -> Self {
        Self {
            label,
            value,
            swatch: Some(colour),
        }
    }

    /// Returns the width the label takes, including any swatch.
    fn label_width(&self) -> i32 {
        let swatch = if self.swatch.is_some() {
            SWATCH + SWATCH_GAP
        } else {
            0
        };
        swatch + text::width_of(&self.label, 1)
    }
}

/// One card.
///
/// A card is a heading and a list of rows. Its size follows its content, so a
/// card never states a rectangle it did not fill and never cuts a row.
struct Card {
    /// The name of the card.
    heading: &'static str,
    /// The rows, in the order they are drawn.
    rows: Vec<Row>,
}

impl Card {
    /// Returns the width of the card in pixels.
    ///
    /// The width follows the widest row and the heading, so no value is ever
    /// cut. The panel cuts a value that overruns its column, because the panel
    /// has a fixed width. A card has no fixed width and needs no cut.
    fn width(&self) -> i32 {
        let widest_label = self.rows.iter().map(Row::label_width).max().unwrap_or(0);
        let widest_value = self
            .rows
            .iter()
            .map(|row| text::width_of(&row.value, 1))
            .max()
            .unwrap_or(0);
        let body = widest_label + COLUMN_GAP + widest_value;
        let heading = text::width_of(self.heading, 1);
        PAD * 2 + body.max(heading)
    }

    /// Returns the height of the card in pixels.
    fn height(&self) -> i32 {
        PAD * 2 + LINE + LINE * self.rows.len() as i32
    }
}

/// Where a card sits in the window.
///
/// The cards stack away from the corner they are anchored to, so a card that
/// grows pushes its neighbour toward the middle and never off the glass.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum Anchor {
    /// The top left corner, stacking downward.
    TopLeft,
    /// The top right corner, stacking downward.
    TopRight,
    /// The bottom left corner, stacking upward from above the hint line.
    BottomLeft,
    /// The bottom right corner, stacking upward.
    BottomRight,
}

/// Builds the cards the glass holds, in order.
///
/// **This is the whole content of the glass.** Nothing else states what the
/// cards say. The painting walks this list and the reading walks it too, so
/// the two cannot disagree.[^1]
///
/// # References
///
/// [^1]: Recurring Defect Shapes, shape 1. `.claude/rules/recurring-defects.md`
fn cards(readout: &Readout, reference: bool) -> Vec<(Anchor, Card)> {
    let mut cards = vec![(Anchor::TopLeft, world_card(readout))];
    // **The card appears only when something is being carried.** A card that
    // said "carrying 0" on every frame of every run would take space from the
    // map to report nothing, and a watcher would learn to skip it. The count
    // is what decides, so the card cannot outlive the behaviour it
    // reports.[^10]
    //
    // [^10]: Testing Rules, section 2a. `.claude/rules/testing.md`
    if readout.units_carrying() > 0 {
        cards.push((Anchor::TopLeft, load_card(readout)));
    }
    // **The seats hold still, so they do not earn a place in the window.** A
    // pass opens them once and fills them in the same tick, and the count
    // reads the same on every frame after that. The test is what the quantity
    // does and not how interesting it is, so this fails it.[^11] The seats are
    // in the panel instead, which is the path that no window height
    // bounds.[^12]
    //
    // [^11]: ADR-0093, the window shows what changes, decision D1. `docs/adrs/draft/adr-0093-the-window-shows-what-changes.md`
    // [^12]: ADR-0093, the window shows what changes, decision D3. `docs/adrs/draft/adr-0093-the-window-shows-what-changes.md`
    // **The card appears only once somebody has been promoted.** Nothing
    // promoted anybody until this pass existed, and a card that reported
    // "characters 0" on every frame of every run would say nothing and go on
    // saying nothing if the pass broke.[^12]
    //
    // [^12]: Testing Rules, section 2a. `.claude/rules/testing.md`
    if readout.characters() > 0 {
        cards.push((Anchor::TopLeft, character_card(readout)));
    }
    if let Some(choice) = readout.choice() {
        cards.push((Anchor::BottomRight, unit_card(&choice)));
    }
    if reference {
        cards.push((Anchor::TopRight, view_card(readout)));
        cards.push((Anchor::TopRight, cost_card(readout)));
        cards.push((Anchor::BottomLeft, colour_card(readout)));
    }
    cards
}

/// Returns each card and the rectangle it occupies, in drawing order.
///
/// A rectangle is a left, top, width and height in pixels. The width and the
/// height follow the content, so a card never states a rectangle it did not
/// fill.
fn placed(readout: &Readout, reference: bool, canvas: &Canvas) -> Vec<((i32, i32), Card)> {
    let (width, height) = (canvas.width() as i32, canvas.height() as i32);
    // One pen for each corner. A stack grows away from its corner.
    let mut down_left = MARGIN;
    let mut down_right = MARGIN;
    let mut up_left = height - MARGIN - text::GLYPH_HEIGHT - GAP;
    let mut up_right = height - MARGIN;

    let mut out = Vec::new();
    for (anchor, card) in cards(readout, reference) {
        let (card_width, card_height) = (card.width(), card.height());
        let at = match anchor {
            Anchor::TopLeft => {
                let at = (MARGIN, down_left);
                down_left += card_height + GAP;
                at
            }
            Anchor::TopRight => {
                let at = (width - MARGIN - card_width, down_right);
                down_right += card_height + GAP;
                at
            }
            Anchor::BottomLeft => {
                up_left -= card_height;
                let at = (MARGIN, up_left);
                up_left -= GAP;
                at
            }
            Anchor::BottomRight => {
                up_right -= card_height;
                let at = (width - MARGIN - card_width, up_right);
                up_right -= GAP;
                at
            }
        };
        out.push((at, card));
    }
    out
}

/// Returns the rectangle of each card the glass draws.
///
/// A caller reads this to check that every card sits inside the window. The
/// panel states a rectangle that a short window cuts, and says so on its last
/// line.[^1] A card is sized by its content and placed against a corner, so it
/// has no such case, and this is how a test reads that.
///
/// # References
///
/// [^1]: Backlog item 0133. `docs/backlog/complete/0133-let-a-watcher-reach-a-panel-longer-than-the-window.md`
#[must_use]
pub fn card_bounds(
    readout: &Readout,
    reference: bool,
    canvas: &Canvas,
) -> Vec<(i32, i32, i32, i32)> {
    placed(readout, reference, canvas)
        .into_iter()
        .map(|((x, y), card)| (x, y, card.width(), card.height()))
        .collect()
}

/// Returns what the glass says, as one line for each heading and row.
///
/// A heading gives its own name. A row gives its label and its value,
/// separated by a colon. The reading walks the same card list the painting
/// walks, so a test that reads this reads what a watcher sees.[^1]
///
/// # References
///
/// [^1]: Recurring Defect Shapes, shape 1. `.claude/rules/recurring-defects.md`
#[must_use]
pub fn says(readout: &Readout, reference: bool) -> Vec<String> {
    let mut out = Vec::new();
    for (_, card) in cards(readout, reference) {
        out.push(card.heading.to_string());
        for row in &card.rows {
            out.push(format!("{}: {}", row.label, row.value));
        }
    }
    out
}

/// Draws the cards over the map.
///
/// Call this after the world pass. The readout is taken once against the world
/// and the finished canvas, and the cards are a function of the readout
/// alone.[^1]
///
/// # References
///
/// [^1]: ADR-0070, the head-up display reports what the drawing pass read, decision D1. `docs/adrs/accepted/adr-0070-the-head-up-display-reports-what-the-drawing-pass-read.md`
pub fn draw(readout: &Readout, canvas: &mut Canvas, reference: bool) {
    for ((x, y), card) in placed(readout, reference, canvas) {
        paint(canvas, x, y, &card);
    }
    hint(canvas, reference);
}

/// Draws one card at a position.
fn paint(canvas: &mut Canvas, x: i32, y: i32, card: &Card) {
    let (width, height) = (card.width(), card.height());
    canvas.shade(x, y, width, height, CARD, CARD_WEIGHT);
    outline(canvas, x, y, width, height);

    let left = x + PAD;
    let right = x + width - PAD;
    canvas.write(left, y + PAD, card.heading, 1, HEADING);

    let mut pen = y + PAD + LINE;
    for row in &card.rows {
        let mut text_left = left;
        if let Some(colour) = row.swatch {
            canvas.block(left, pen, SWATCH, SWATCH, colour);
            text_left += SWATCH + SWATCH_GAP;
        }
        canvas.write(text_left, pen, &row.label, 1, LABEL);
        let start = right - text::width_of(&row.value, 1);
        canvas.write(start, pen, &row.value, 1, VALUE);
        pen += LINE;
    }
}

/// Draws the edge of a card.
fn outline(canvas: &mut Canvas, x: i32, y: i32, width: i32, height: i32) {
    canvas.block(x, y, width, 1, EDGE);
    canvas.block(x, y + height - 1, width, 1, EDGE);
    canvas.block(x, y, 1, height, EDGE);
    canvas.block(x + width - 1, y, 1, height, EDGE);
}

/// Returns the card that says how the world is going.
///
/// Five rows, and every one of them moves. The tick moves each frame. The
/// population moves as units are born and die. The two shortage rows move as a
/// store empties. The food moves as a crowd works a deposit, as the deposit
/// recovers, and as the watcher scrolls.
///
/// **The rows a watcher checks rather than watches are not here.** The tile
/// address, the ground, the count of units in the window and the count of
/// tiles drawn all sit behind the key. Each is a number a person reads once to
/// orient themselves, and none of them tells a watcher that something is
/// happening.[^1] [^2]
///
/// # References
///
/// [^1]: Decisions register, DEC-084. `docs/DECISIONS.md`
/// [^2]: ADR-0093, the window shows what changes, decisions D1 and D2. `docs/adrs/draft/adr-0093-the-window-shows-what-changes.md`
/// The card that says what the drawn units are hauling and where they live.
///
/// **Both numbers are counts of the window.** The drawing asked at every unit
/// it painted, on the loop that already ran, so neither starts a pass over the
/// arena.[^1]
///
/// A kind that nobody is carrying gets no row. The demonstration world hauls
/// food and nothing else, so two rows of a permanent zero would say that the
/// card is broken rather than that the world carries no wood.
///
/// # References
///
/// [^1]: ADR-0070, the head-up display reports what the drawing pass read, decision D1. `docs/adrs/accepted/adr-0070-the-head-up-display-reports-what-the-drawing-pass-read.md`
fn load_card(readout: &Readout) -> Card {
    let drawn = readout.soldiers_painted();
    let mut rows = vec![Row::new(
        "carrying",
        format!("{} of {}", readout.units_carrying(), drawn),
    )];
    for kind in ResourceKind::ALL {
        let held = readout.carried_by_kind()[kind as usize];
        if held > 0 {
            rows.push(Row::new(resource_name(kind), grouped(u64::from(held))));
        }
    }
    // The home count holds still as well. Every live unit has one, so the row
    // reads "n of n" on every frame, and it belongs in the panel rather than
    // on the glass for the same reason the seats do.[^11]
    if readout.rationings() > 0 {
        rows.push(Row::new(
            "sites rationed",
            grouped(u64::from(readout.rationings())),
        ));
        // The shortfall is in accumulator units, so it is formatted and
        // never shown raw. A raw one would state a quantity sixty-five
        // thousand times the real one.
        rows.push(Row::new(
            "  short by",
            accumulated(readout.rationed_short()),
        ));
    }
    Card {
        heading: "WHAT THEY CARRY",
        rows,
    }
}

/// How long a promotion stays on the glass, in ticks.
///
/// The log holds one frame. A card driven by the log alone would show a
/// promotion for one frame and forget it, and at the rate the demonstration
/// runs that is under a tenth of a second. The birth tick of the newest
/// character is stored, so the card can hold the moment for a while after it
/// without the viewer keeping a memory of its own.
const PROMOTION_HOLD: u64 = 90;

/// The card that says a soldier became a character, and why.
///
/// **This is the moment, not the number.** A watcher sees that somebody was
/// promoted, which faction they were, and the deeds that earned it, while it
/// is fresh. The running count sits under it so that the card still says
/// something once the moment has passed.
///
/// The deeds come from the log of the step that just ran, so they appear on
/// the frame of the promotion and not after it. The faction and the age come
/// from the character, which stores its birth tick, so those stay.[^1]
///
/// # References
///
/// [^1]: ADR-0067, the viewer reads the world and never writes to it, decision D2. `docs/adrs/accepted/adr-0067-the-viewer-reads-the-world-and-never-writes-to-it.md`
fn character_card(readout: &Readout) -> Card {
    let mut rows = Vec::new();
    if let Some((faction, birth)) = readout.newest_character() {
        let age = readout.tick().saturating_sub(birth);
        if age <= PROMOTION_HOLD {
            rows.push(Row::new("just promoted", format!("faction {}", faction.0)));
            if let Some(deeds) = readout.promoted_deeds() {
                rows.push(Row::new("  for deeds", grouped(deeds)));
            }
            rows.push(Row::new("  ticks ago", grouped(age)));
        }
    }
    rows.push(Row::new(
        "characters in world",
        grouped(u64::from(readout.characters())),
    ));
    Card {
        heading: "THE CHARACTERS",
        rows,
    }
}

fn world_card(readout: &Readout) -> Card {
    let mut rows = vec![
        Row::new("tick", grouped(readout.tick())),
        Row::new(
            "people in world",
            grouped(u64::from(readout.soldiers_live())),
        ),
        Row::new("ended", grouped(readout.units_ended() as u64)),
        Row::new("short", grouped(u64::from(readout.units_short()))),
    ];
    if let Some(tile) = readout.tile() {
        rows.push(Row::new("food here", food_of(&tile)));
    }
    Card {
        heading: "THE WORLD",
        rows,
    }
}

/// Returns what is left of the food a tile carries, of what the ground gave.
///
/// A tile the ground gave nothing of returns a word and not a pair of zeroes.
/// A reader cannot tell a drained deposit from ground that never carried
/// one.[^1]
///
/// # References
///
/// [^1]: ADR-0070, the head-up display reports what the drawing pass read, decision D2. `docs/adrs/accepted/adr-0070-the-head-up-display-reports-what-the-drawing-pass-read.md`
fn food_of(tile: &TileReadout) -> String {
    let gave = tile.generated(ResourceKind::Food);
    if gave == 0 {
        return "none".to_string();
    }
    format!("{} of {}", tile.stock(ResourceKind::Food), gave)
}

/// Returns the card that says why the nearest unit chose what it chose.
///
/// This is the only per-entity statement of cause the engine can make, and it
/// changes on every frame the unit chooses on.[^1] The card names the winning
/// option and its score. The four option rows stay in the panel, because a
/// watcher reads them when they are asking why, and not while they watch.
///
/// # References
///
/// [^1]: ADR-0064, a unit chooses by scoring a small fixed option set, decision D2. `docs/adrs/accepted/adr-0064-a-unit-chooses-by-scoring-a-small-fixed-option-set.md`
fn unit_card(choice: &ChoiceReadout) -> Card {
    let focus = choice.focus();
    let mut rows = vec![Row::coloured(
        faction_colour(focus.faction()),
        format!("faction {}", focus.faction().0),
        match focus.condition() {
            None => "-".to_string(),
            Some(NeedCondition::Fed) => "fed".to_string(),
            Some(NeedCondition::Short) => "short".to_string(),
            Some(NeedCondition::Starved) => "starved".to_string(),
        },
    )];
    match choice.explanation() {
        // The engine says nothing about this unit. The card says that, rather
        // than printing a zero a reader cannot tell from a real score.
        None => rows.push(Row::new("chose", "-".to_string())),
        Some(answer) => {
            rows.push(Row::new(
                "chose",
                format!("{} {}", option_name(answer.best), best_score(&answer)),
            ));
            rows.push(Row::new("needs", fraction(Some(answer.need))));
        }
    }
    Card {
        heading: "NEAREST UNIT",
        rows,
    }
}

/// Returns the score of the option the choice selected.
///
/// Returns a dash when every option fell below the floor. A unit that chose
/// nothing has no winning score, and a zero would read as one.[^1]
///
/// # References
///
/// [^1]: ADR-0070, the head-up display reports what the drawing pass read, decision D2. `docs/adrs/accepted/adr-0070-the-head-up-display-reports-what-the-drawing-pass-read.md`
fn best_score(answer: &cachette_core::ChoiceExplanation) -> String {
    match answer.scores.get(answer.best as usize) {
        None => "-".to_string(),
        Some(score) => fraction(Some(*score)),
    }
}

/// Returns the card that names every colour the window draws.
///
/// The product record asks that the window name every colour it draws, so that
/// a developer can point at a unit and say which faction it belongs to.[^1]
/// This card is that naming. It appears while the watcher holds the key.
///
/// The counts beside each name are counts of the window, which the drawing pass
/// produced while it painted.[^2]
///
/// # References
///
/// [^1]: PRD-0005, a watcher can tell what is happening and why. `docs/product/shipped/prd-0005-a-watcher-can-tell-what-is-happening-and-why.md`
/// [^2]: ADR-0070, the head-up display reports what the drawing pass read, decision D1. `docs/adrs/accepted/adr-0070-the-head-up-display-reports-what-the-drawing-pass-read.md`
fn colour_card(readout: &Readout) -> Card {
    let mut rows = Vec::with_capacity(COLOURED_FACTIONS + KIND_COUNT);
    for (slot, count) in readout
        .by_faction()
        .iter()
        .enumerate()
        .take(legend_rows(readout))
    {
        rows.push(Row::coloured(
            faction_colour(cachette_core::FactionId(slot as u16)),
            format!("faction {slot}"),
            grouped(u64::from(*count)),
        ));
    }
    for (ordinal, count) in readout.by_kind().iter().enumerate() {
        let kind = KINDS[ordinal];
        rows.push(Row::coloured(
            kind_colour(kind),
            name_of(kind).to_string(),
            grouped(u64::from(*count)),
        ));
    }
    Card {
        heading: "COLOURS IN THE WINDOW",
        rows,
    }
}

/// Returns the number of faction rows the legend draws.
///
/// The count follows the world, not the colour table. A legend sized by the
/// table names a colour that no faction uses, and a reader then looks for a
/// faction that is not there.
///
/// A faction beyond the colour table shares a colour with an earlier one, so
/// the legend stops at the table rather than naming one colour twice.
fn legend_rows(readout: &Readout) -> usize {
    (readout.factions() as usize).clamp(1, COLOURED_FACTIONS)
}

/// Returns the card that says where the camera sits.
///
/// The product record asks that the window state where the person is looking,
/// as a tile address, and how much of the world the window covers.[^1] A
/// watcher checks this occasionally rather than continuously, so it sits behind
/// the key.
///
/// # References
///
/// [^1]: PRD-0005, a watcher can tell what is happening and why. `docs/product/shipped/prd-0005-a-watcher-can-tell-what-is-happening-and-why.md`
fn view_card(readout: &Readout) -> Card {
    let (across, down) = readout.extent_shown();
    let mut rows = vec![
        Row::new(
            "centre tile",
            format!("q {}  r {}", readout.centre().q, readout.centre().r),
        ),
        Row::new("showing", format!("{across} x {down} tiles")),
        Row::new("tiles drawn", grouped(u64::from(readout.tiles_painted()))),
        // The scope is in the label of both rows, not only of this one. The
        // product record asks that a reader cannot mistake one count for the
        // other, and the two now sit on different layers, so the label is the
        // only thing that separates them.[^2]
        //
        // [^2]: PRD-0005, a watcher can tell what is happening and why. `docs/product/shipped/prd-0005-a-watcher-can-tell-what-is-happening-and-why.md`
        Row::new(
            "people in window",
            grouped(u64::from(readout.soldiers_painted())),
        ),
    ];
    if let Some(tile) = readout.tile() {
        rows.push(Row::new("ground here", name_of(tile.kind()).to_string()));
    }
    Card {
        heading: "WHERE YOU ARE LOOKING",
        rows,
    }
}

/// Returns the card that says what the frame cost.
///
/// The product record asks that the window state the cost of the step and the
/// cost of the drawing as two separate numbers, while the run continues.[^1]
/// Both are here, and the heading says which machine they describe. Neither is
/// evidence about the target platform.[^2]
///
/// # References
///
/// [^1]: PRD-0005, a watcher can tell what is happening and why. `docs/product/shipped/prd-0005-a-watcher-can-tell-what-is-happening-and-why.md`
/// [^2]: Project orientation, the target platform. `CLAUDE.md`
fn cost_card(readout: &Readout) -> Card {
    Card {
        heading: "COST ON THIS MACHINE",
        rows: vec![
            Row::new(
                "step",
                Readout::cost_in_millis(readout.step_mean(), readout.steps_measured()),
            ),
            Row::new(
                "draw",
                Readout::cost_in_millis(readout.draw_mean(), readout.frames_measured()),
            ),
            Row::new(
                "rate",
                if readout.steps_measured() == 0 {
                    crate::hud::NOT_MEASURED.to_string()
                } else {
                    format!("{:.1} a second", readout.rate())
                },
            ),
        ],
    }
}

/// What the window says while the reference layer is hidden.
const CLOSED_HINT: &str = "hold tab for more    just inspect writes the rest";

/// What the window says while the reference layer is shown.
const OPEN_HINT: &str = "release tab    just inspect writes the rest";

/// Returns the line that says where the rest of the numbers are.
///
/// The line names a command. A caller reads it to check that the command
/// exists, because a name in the window and a recipe in the build file are one
/// fact in two places, and nothing fails when they disagree.[^1]
///
/// # References
///
/// [^1]: Recurring Defect Shapes, shape 1. `.claude/rules/recurring-defects.md`
#[must_use]
pub const fn hint_line(reference: bool) -> &'static str {
    if reference {
        OPEN_HINT
    } else {
        CLOSED_HINT
    }
}

/// The name of the command that renders every number the window does not show.
///
/// The hint line holds it and the build file defines it. A test compares the
/// two, so a rename that reaches one and not the other fails.[^1]
///
/// # References
///
/// [^1]: Recurring Defect Shapes, shape 1. `.claude/rules/recurring-defects.md`
pub const DETAIL_COMMAND: &str = "inspect";

/// Writes the line that says where the rest of the numbers are.
///
/// **A watcher must be able to find what the window stopped showing.** The
/// panel holds every section still, and one command renders it. A window that
/// dropped thirteen sections in silence would be the failure the panel record
/// names for a number that is absent without saying so.[^1]
///
/// # References
///
/// [^1]: ADR-0070, the head-up display reports what the drawing pass read, decision D2. `docs/adrs/accepted/adr-0070-the-head-up-display-reports-what-the-drawing-pass-read.md`
fn hint(canvas: &mut Canvas, reference: bool) {
    let line = hint_line(reference);
    let y = canvas.height() as i32 - MARGIN - text::GLYPH_HEIGHT;
    // The line sits on the map, and the map is any colour. A strip behind it
    // keeps it legible over bright ground as well as over water, in the same
    // way a card stays legible.
    canvas.shade(
        MARGIN - 4,
        y - 3,
        text::width_of(line, 1) + 8,
        text::GLYPH_HEIGHT + 6,
        CARD,
        CARD_WEIGHT,
    );
    canvas.write(MARGIN, y, line, 1, HINT);
}
