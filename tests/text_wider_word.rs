//! Wrapped text given less room than its longest word, as a hash, a path
//! or a long identifier gets in a narrow column.

use floem::headless::{HeadlessHarness, TestRoot};
use floem::kurbo::Size;
use floem::prelude::*;
use floem::text::{Attrs, AttrsList, LineHeightValue, OverflowWrap, TextLayout, TextWrapMode};
use floem::views::{Empty, Label, Stack, rich_text};
use floem::{HasViewId, ViewId};

/// Wider than any column here, and followed by enough short words to
/// fill several lines at a column's width and many more at one a line.
const TEXT: &str = "see 0123456789abcdef0123456789abcdef0123456789abcdef \
                    and then a few more short words to wrap after it";

const SIZE: f32 = 12.0;
const LEADING: f32 = 2.0;

fn attrs() -> AttrsList {
    AttrsList::new(
        Attrs::new()
            .font_size(SIZE)
            .line_height(LineHeightValue::Normal(LEADING)),
    )
}

/// `TEXT` laid out as a wrapped view lays it out at `width`.
fn laid_out(width: f32) -> Size {
    let mut layout = TextLayout::new();
    layout.set_text_wrap_mode(TextWrapMode::Wrap);
    layout.set_overflow_wrap(OverflowWrap::Normal);
    layout.set_text(TEXT, attrs(), None);
    layout.set_size(width, f32::MAX);
    layout.size()
}

/// The size of the taffy leaf a text view measures its text in.
fn text_node(id: ViewId) -> Size {
    let taffy = id.taffy();
    let taffy = taffy.borrow();
    let leaf = taffy.children(id.taffy_node()).unwrap()[0];
    let size = taffy.layout(leaf).unwrap().size;
    Size::new(size.width as f64, size.height as f64)
}

fn label() -> Label {
    Label::new(TEXT).style(|s| s.font_size(SIZE).line_height(LEADING).text_wrap())
}

fn assert_wraps_at(id: ViewId, width: f64) {
    let view = id.get_layout_rect();
    let node = text_node(id);
    let want = laid_out(width as f32).height;
    assert!(
        (view.width() - width).abs() < 0.5,
        "the view keeps its width: {view:?}"
    );
    assert!(
        node.width <= width + 0.5,
        "the text stays inside the view: text {node:?}, view {view:?}"
    );
    assert!(
        (view.height() - want).abs() < 0.5,
        "the view is as tall as its lines at its width ({want}), not {}",
        view.height()
    );
}

/// A label across a column narrower than its longest word lays its lines
/// out at the column, the word running past the edge on its own line.
/// Its text used to grow to the word, so every line broke at the word's
/// width, and the label took the height of the text broken after every
/// word, most of it empty below the lines.
#[test]
fn a_label_narrower_than_a_word_wraps_at_its_own_width() {
    let root = TestRoot::new();
    let label = label().style(|s| s.width_full());
    let id = label.view_id();
    let column = Stack::vertical((label,)).style(|s| s.width(150.0));
    let mut harness = HeadlessHarness::new_with_size(root, column, 400.0, 2000.0);
    harness.rebuild();
    harness.paint();
    assert_wraps_at(id, 150.0);
}

/// The same, grown to fill a row beside a fixed mark, as a list row is.
#[test]
fn a_label_grown_in_a_row_wraps_at_its_own_width() {
    let root = TestRoot::new();
    let label = label().style(|s| s.min_width(0.0).flex_grow(1.0_f32));
    let id = label.view_id();
    let row = Stack::horizontal((
        Empty::new().style(|s| s.size(6.0, 6.0).flex_shrink(0.0_f32)),
        label,
    ))
    .style(|s| s.width(160.0).gap(10.0).items_start());
    let column = Stack::vertical((row,)).style(|s| s.width(160.0));
    let mut harness = HeadlessHarness::new_with_size(root, column, 400.0, 2000.0);
    harness.rebuild();
    harness.paint();
    assert_wraps_at(id, 144.0);
}

/// Rich text lays its text out the same way.
#[test]
fn rich_text_narrower_than_a_word_wraps_at_its_own_width() {
    let root = TestRoot::new();
    let text = rich_text(TEXT.to_string(), attrs(), || (TEXT.to_string(), attrs()))
        .style(|s| s.width_full());
    let id = text.view_id();
    let column = Stack::vertical((text,)).style(|s| s.width(150.0));
    let mut harness = HeadlessHarness::new_with_size(root, column, 400.0, 2000.0);
    harness.rebuild();
    harness.paint();
    assert_wraps_at(id, 150.0);
}

/// A label sized to its content in a row too narrow for it stops at its
/// longest word, its minimum. At that width it is as tall as the text
/// laid out there, not as the text broken after every word, which is
/// how its minimum is measured.
#[test]
fn a_label_at_its_minimum_is_as_tall_as_its_lines_there() {
    let root = TestRoot::new();
    let label = label();
    let id = label.view_id();
    let row = Stack::horizontal((label,)).style(|s| s.width(60.0));
    let column = Stack::vertical((row,)).style(|s| s.width(60.0));
    let mut harness = HeadlessHarness::new_with_size(root, column, 400.0, 2000.0);
    harness.rebuild();
    harness.paint();
    let view = id.get_layout_rect();
    let want = laid_out(view.width() as f32).height;
    assert!(
        view.width() > 60.0,
        "the word holds the label open: {view:?}"
    );
    assert!(
        (view.height() - want).abs() < 0.5,
        "as tall as its lines at {} ({want}), not {}",
        view.width(),
        view.height()
    );
}
