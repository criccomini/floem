//! Tests for where the editor draws its caret.

use floem::prelude::*;
use floem::views::editor::core::cursor::CursorAffinity;
use floem::views::editor::view::cursor_caret;
use floem_test::prelude::*;

/// A caret at the start of a line is drawn inside the editor, whose
/// content clips at x = 0, rather than straddling that edge.
#[test]
fn test_caret_at_line_start_is_inside_the_editor() {
    let root = TestRoot::new();
    let editor = text_editor("abc\n\n").style(|s| s.size(200.0, 100.0));
    let ed = editor.editor().clone();
    let _harness = HeadlessHarness::new_with_size(root, editor, 200.0, 100.0);

    for (offset, what) in [(0, "a line's first column"), (5, "an empty last line")] {
        let caret = cursor_caret(&ed, offset, false, CursorAffinity::Backward);
        assert_eq!(caret.x, 0.0, "the caret on {what}");
        assert_eq!(caret.width, 2.0, "the caret on {what}");
    }

    // Past the first column the bar is still centred on the caret.
    let caret = cursor_caret(&ed, 3, false, CursorAffinity::Backward);
    assert!(caret.x > 0.0);
    assert_eq!(caret.width, 2.0);
}
