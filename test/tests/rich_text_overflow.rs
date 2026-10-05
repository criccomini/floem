//! Rich text wraps a line wider than the view unless it is told to keep
//! to one line, as a label with an ellipsis does.

use floem::prelude::*;
use floem::style::{NoWrapOverflow, TextOverflow};
use floem::text::{Attrs, AttrsList};
use floem::views::rich_text;
use floem_test::TestRoot;
use floem_test::prelude::*;
use serial_test::serial;

const TEXT: &str = "a line of rich text far too long for the narrow box it is in";

/// The height rich text of `TEXT` takes in a box 80px wide, with
/// `overflow` when there is one, and the height of one line of it.
fn heights(overflow: Option<TextOverflow>) -> (f64, f64) {
    let attrs = || AttrsList::new(Attrs::new().font_size(14.0));
    let height = |text: &'static str, overflow: Option<TextOverflow>| {
        let root = TestRoot::new();
        let view = rich_text(text.to_string(), attrs(), move || {
            (text.to_string(), attrs())
        });
        let view = match overflow {
            Some(overflow) => view.text_overflow(overflow),
            None => view,
        };
        let view = view.style(|s| s.width(80.0));
        let id = view.view_id();
        let boxed = Container::new(view).style(|s| s.size(200.0, 200.0).flex_col().items_start());
        let mut harness = HeadlessHarness::new_with_size(root, boxed, 200.0, 200.0);
        harness.rebuild();
        id.get_layout().expect("laid out").size.height as f64
    };
    (height(TEXT, overflow), height("a", None))
}

#[test]
#[serial]
fn rich_text_wraps_a_long_line_by_default() {
    let (height, line) = heights(None);
    assert!(height > line * 1.5, "{height} is one line of {line}");
}

#[test]
#[serial]
fn rich_text_cut_short_with_an_ellipsis_stays_one_line() {
    let (height, line) = heights(Some(TextOverflow::NoWrap(NoWrapOverflow::Ellipsis)));
    assert!(
        (height - line).abs() < 0.5,
        "{height} is not one line of {line}"
    );
}
