use floem::headless::{HeadlessHarness, TestRoot};
use floem::reactive::{SignalGet, SignalUpdate};
use floem::style::recalc::StyleReason;
use floem::views::editor::Editor;
use floem::views::editor::core::cursor::CursorAffinity;
use floem::views::{Decorators, text_editor};

/// An editor ten lines high over two hundred lines.
fn editor() -> (HeadlessHarness, Editor, f64) {
    let root = TestRoot::new();
    let text: String = (0..200).map(|i| format!("line {i}\n")).collect();
    let view = text_editor(text).style(|s| s.size(400.0, 200.0));
    let editor = view.editor().clone();
    let mut harness = HeadlessHarness::new_with_size(root, view, 400.0, 200.0);
    harness.rebuild();
    harness.paint();
    let line_height = f64::from(editor.line_height(0));
    (harness, editor, line_height)
}

fn put_caret_on_line(harness: &mut HeadlessHarness, ed: &Editor, line: usize) {
    let offset = ed.offset_of_line(line);
    ed.cursor
        .update(|c| c.set_offset(offset, CursorAffinity::Backward, false, false));
    harness.rebuild();
    harness.paint();
}

/// The caret is brought into view when it moves. A restyle of the editor,
/// which a hover, a resize or a theme change brings, is no move, and the
/// view stays where the user scrolled it, caret out of sight or not.
#[test]
fn a_restyle_leaves_the_view_where_it_was_scrolled_to() {
    let (mut harness, ed, line_height) = editor();
    put_caret_on_line(&mut harness, &ed, 5);

    // A few lines past the caret, so it is just out of view above.
    harness.scroll_down(200.0, 100.0, 12.0 * line_height);
    harness.rebuild();
    harness.paint();
    let scrolled = ed.viewport.get().y0;
    assert!(
        scrolled > 6.0 * line_height,
        "the caret is not out of view: the view starts at {scrolled}"
    );

    let content = ed.editor_view_id.get().expect("the editor's content view");
    content.request_style(StyleReason::full_recalc());
    harness.rebuild();
    harness.paint();
    assert_eq!(
        ed.viewport.get().y0,
        scrolled,
        "a restyle scrolled the caret back into view"
    );

    // A move of the caret still brings it into view.
    put_caret_on_line(&mut harness, &ed, 6);
    assert!(
        ed.viewport.get().y0 <= 6.0 * line_height,
        "the moved caret is not in view: the view starts at {}",
        ed.viewport.get().y0
    );
}
