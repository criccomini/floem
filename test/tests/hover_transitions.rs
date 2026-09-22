//! Tests for the `PointerEnter` and `PointerLeave` a pointer move sends.
//!
//! A move sends the views it left their `PointerLeave`, innermost first, and then
//! the views it entered their `PointerEnter`, outermost first. A view the pointer
//! stays over gets neither, even when the views around it change.

use std::{cell::RefCell, rc::Rc};

use floem_test::prelude::*;
use serial_test::serial;

type Log = Rc<RefCell<Vec<String>>>;

/// Records the view's `PointerEnter` and `PointerLeave` in `log` under `name`.
fn log_hover<V: IntoView>(view: V, name: &'static str, log: &Log) -> impl IntoView + use<V> {
    let enter_log = log.clone();
    let leave_log = log.clone();
    view.into_view()
        .on_event_cont(el::PointerEnter, move |_, _| {
            enter_log.borrow_mut().push(format!("enter {name}"));
        })
        .on_event_cont(el::PointerLeave, move |_, _| {
            leave_log.borrow_mut().push(format!("leave {name}"));
        })
}

#[test]
#[serial]
fn test_move_leaves_old_views_before_entering_new_ones() {
    // Two rows side by side, each holding a cell that fills it. Moving from one
    // cell to the other leaves the cell and its row, then enters the other row
    // and its cell.
    let root = TestRoot::new();
    let log: Log = Rc::default();

    let row = |row_name: &'static str, cell_name: &'static str| {
        let cell = log_hover(Empty::new().style(|s| s.size(50.0, 50.0)), cell_name, &log);
        log_hover(Container::new(cell), row_name, &log)
    };
    let view =
        Stack::new((row("row a", "cell a"), row("row b", "cell b"))).style(|s| s.size(100.0, 50.0));

    let mut harness = HeadlessHarness::new_with_size(root, view, 100.0, 50.0);

    harness.pointer_move(25.0, 25.0);
    log.borrow_mut().clear();

    harness.pointer_move(75.0, 25.0);
    assert_eq!(
        *log.borrow(),
        vec!["leave cell a", "leave row a", "enter row b", "enter cell b"],
        "A move should leave the old views from the innermost out, then enter the new \
         ones from the outermost in"
    );
}

#[test]
#[serial]
fn test_view_stays_hovered_when_pointer_crosses_an_ancestors_edge() {
    // An absolutely positioned box hangs out of its parent's right edge, and a view
    // fills the box. A hover path holds only the ancestors whose own box holds the
    // pointer, so crossing the parent's edge on the view drops the parent from the
    // path while the view stays in it.
    let root = TestRoot::new();
    let log: Log = Rc::default();

    let inner = Empty::new().style(|s| s.size_full());
    let inner_id = inner.view_id();
    let inner = log_hover(inner, "inner", &log);

    let overhang = Container::new(inner).style(|s| {
        s.absolute()
            .inset_left(50.0)
            .inset_top(0.0)
            .size(100.0, 50.0)
    });
    let parent = Container::new(overhang).style(|s| s.size(100.0, 100.0));
    let parent_id = parent.view_id();
    let parent = log_hover(parent, "parent", &log);

    let view = Container::new(parent).style(|s| s.size(200.0, 100.0));

    let mut harness = HeadlessHarness::new_with_size(root, view, 200.0, 100.0);

    // On the view, inside the parent's box.
    harness.pointer_move(75.0, 25.0);
    assert!(harness.is_hovered(parent_id));
    assert!(harness.is_hovered(inner_id));
    assert_eq!(*log.borrow(), vec!["enter parent", "enter inner"]);
    log.borrow_mut().clear();

    // On the view, past the parent's right edge.
    harness.pointer_move(125.0, 25.0);
    assert!(
        !harness.is_hovered(parent_id),
        "The parent's box no longer holds the pointer, so it should leave the hover path"
    );
    assert!(
        harness.is_hovered(inner_id),
        "The pointer is still on the view"
    );
    assert_eq!(
        *log.borrow(),
        vec!["leave parent"],
        "Only the parent should hear about the move; the view never stopped being hovered"
    );

    // Back inside the parent's box, still on the view.
    harness.pointer_move(75.0, 25.0);
    assert_eq!(*log.borrow(), vec!["leave parent", "enter parent"]);

    // Off the view and past the parent's edge: now the view is left.
    harness.pointer_move(175.0, 75.0);
    assert!(!harness.is_hovered(inner_id));
    assert_eq!(
        *log.borrow(),
        vec![
            "leave parent",
            "enter parent",
            "leave inner",
            "leave parent"
        ]
    );
}
