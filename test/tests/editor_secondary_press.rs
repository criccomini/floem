//! A right press in the editor is not a left one: it starts no drag
//! selection and keeps the selection it lands in.
//!
//! Every mouse button comes from the primary pointer, so the editor took a
//! right press for a left press. The context menu the press opens swallows
//! the release, and the selection then followed the pointer until the next
//! click.

use floem::views::editor::Editor;
use floem::views::text_editor::text_editor;
use floem_test::prelude::*;
use serial_test::serial;

const TEXT: &str = "first line of text\nsecond line of text\nthird line of text\n";

/// The gutter takes the first 64 or so points of each row; the text starts
/// after it.
fn harness() -> (HeadlessHarness, Editor) {
    let root = TestRoot::new();
    let editor = text_editor(TEXT).style(|s| s.size(400.0, 200.0));
    let ed = editor.editor().clone();
    let harness = HeadlessHarness::new_with_size(root, editor, 400.0, 200.0);
    (harness, ed)
}

fn selection(ed: &Editor) -> Option<(usize, usize)> {
    ed.cursor
        .with_untracked(|c| c.get_selection())
        .filter(|(start, end)| start != end)
}

#[test]
#[serial]
fn left_press_then_move_selects() {
    let (mut harness, ed) = harness();
    harness.pointer_down(120.0, 5.0);
    harness.pointer_move(120.0, 45.0);
    assert!(
        selection(&ed).is_some(),
        "a left drag should select, or the tests below prove nothing"
    );
}

#[test]
#[serial]
fn right_press_then_move_selects_nothing() {
    let (mut harness, ed) = harness();
    harness.secondary_pointer_down(120.0, 5.0);
    harness.pointer_move(120.0, 45.0);
    assert!(
        !ed.active.get_untracked(),
        "a right press made the editor active"
    );
    assert_eq!(selection(&ed), None, "the pointer dragged a selection");
}

#[test]
#[serial]
fn right_press_inside_selection_keeps_it() {
    let (mut harness, ed) = harness();
    harness.pointer_down(70.0, 5.0);
    harness.pointer_move(300.0, 25.0);
    harness.pointer_up(300.0, 25.0);
    let selected = selection(&ed).expect("the drag selected");
    harness.secondary_pointer_down(120.0, 5.0);
    assert_eq!(selection(&ed), Some(selected));
}
