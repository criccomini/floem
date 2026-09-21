use floem::headless::{HeadlessHarness, TestRoot};
use floem::kurbo::Point;
use floem::reactive::SignalWith;
use floem::ui_events::pointer::PointerState;
use floem::views::editor::Editor;
use floem::views::editor::core::cursor::CursorAffinity;
use floem::views::{Decorators, text_editor};

const TEXT: &str = "the quick brown fox\njumps over\nthe lazy dog\n";

/// An editor over `TEXT`, laid out so points map to offsets.
fn editor() -> (HeadlessHarness, Editor) {
    let root = TestRoot::new();
    let view = text_editor(TEXT).style(|s| s.size(400.0, 200.0));
    let editor = view.editor().clone();
    let mut harness = HeadlessHarness::new_with_size(root, view, 400.0, 200.0);
    harness.rebuild();
    harness.paint();
    (harness, editor)
}

/// The pointer over the middle of the character at `offset`, pressed
/// `count` times.
fn pointer_at(ed: &Editor, offset: usize, count: u8) -> PointerState {
    let line = ed.line_of_offset(offset);
    let left = ed.line_point_of_offset(offset, CursorAffinity::Forward).x;
    let right = ed
        .line_point_of_offset(offset + 1, CursorAffinity::Backward)
        .x;
    let line_height = f64::from(ed.line_height(line));
    let point = Point::new(
        (left + right) / 2.0,
        line as f64 * line_height + line_height / 2.0,
    );
    let mut state = PointerState::default();
    state.position.x = point.x;
    state.position.y = point.y;
    state.count = count;
    state
}

/// The one selected range, anchor end first.
fn selection(ed: &Editor) -> (usize, usize) {
    ed.cursor.with(|c| (c.start_offset(), c.offset()))
}

/// A double click on macOS reaches the editor as a press, then a move with
/// the button down, often without the pointer leaving the character. The
/// move used to drag the selection's end back under the pointer, cutting
/// the word off there.
#[test]
fn a_double_click_selects_the_whole_word_through_the_move_that_follows_it() {
    let (_harness, ed) = editor();
    let quick = pointer_at(&ed, 6, 2);
    ed.pointer_down_primary(&quick);
    assert_eq!(selection(&ed), (4, 9), "the press selects the word");

    ed.pointer_move(&quick);
    assert_eq!(
        selection(&ed),
        (4, 9),
        "and a move within it keeps the word"
    );

    ed.pointer_up(&quick);
    assert_eq!(selection(&ed), (4, 9));
}

#[test]
fn dragging_after_a_double_click_grows_the_selection_by_words() {
    let (_harness, ed) = editor();
    ed.pointer_down_primary(&pointer_at(&ed, 6, 2));

    ed.pointer_move(&pointer_at(&ed, 11, 0));
    assert_eq!(
        selection(&ed),
        (4, 15),
        "to the right, to the end of the word under the pointer"
    );

    ed.pointer_move(&pointer_at(&ed, 1, 0));
    assert_eq!(
        selection(&ed),
        (9, 0),
        "to the left, to the start of the word there, anchored at the pressed word's end"
    );

    ed.pointer_move(&pointer_at(&ed, 5, 0));
    assert_eq!(
        selection(&ed),
        (4, 9),
        "back on the pressed word, just that"
    );

    ed.pointer_move(&pointer_at(&ed, 21, 0));
    assert_eq!(selection(&ed), (4, 25), "and across a line, the same");
}

#[test]
fn dragging_after_a_triple_click_grows_the_selection_by_lines() {
    let (_harness, ed) = editor();
    ed.pointer_down_primary(&pointer_at(&ed, 22, 3));
    assert_eq!(selection(&ed), (20, 31), "the press selects the line");

    ed.pointer_move(&pointer_at(&ed, 25, 0));
    assert_eq!(selection(&ed), (20, 31), "a move along it changes nothing");

    ed.pointer_move(&pointer_at(&ed, 33, 0));
    assert_eq!(
        selection(&ed),
        (20, 44),
        "down, the next line comes in whole"
    );

    ed.pointer_move(&pointer_at(&ed, 6, 0));
    assert_eq!(selection(&ed), (31, 0), "up, the line above does");
}

#[test]
fn dragging_after_a_click_still_selects_by_characters() {
    let (_harness, ed) = editor();
    ed.pointer_down_primary(&pointer_at(&ed, 6, 1));
    assert_eq!(selection(&ed), (6, 6));

    ed.pointer_move(&pointer_at(&ed, 11, 0));
    assert_eq!(selection(&ed), (6, 11));
}
