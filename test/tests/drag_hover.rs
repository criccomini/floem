//! Tests for the `DragTargetEnter` and `DragTargetLeave` a drag sends.
//!
//! A drag enters and leaves drop targets as the pointer moves over them. A target the
//! pointer stays on gets neither, even when the views around it under the pointer
//! change.

use std::{cell::RefCell, rc::Rc};

use floem_test::prelude::*;
use serial_test::serial;

type Log = Rc<RefCell<Vec<String>>>;

/// Records the view's `DragTargetEnter` and `DragTargetLeave` in `log` under `name`.
fn log_drag_over<V: IntoView>(view: V, name: &'static str, log: &Log) -> impl IntoView + use<V> {
    let enter_log = log.clone();
    let leave_log = log.clone();
    view.into_view()
        .on_event_cont(el::DragTargetEnter, move |_, _| {
            enter_log.borrow_mut().push(format!("enter {name}"));
        })
        .on_event_cont(el::DragTargetLeave, move |_, _| {
            leave_log.borrow_mut().push(format!("leave {name}"));
        })
}

#[test]
#[serial]
fn test_drop_target_stays_entered_when_pointer_crosses_an_ancestors_edge() {
    // A draggable box on the left. To its right, a parent with an absolutely
    // positioned box hanging out of its right edge, filled by a drop target. Dragging
    // across the parent's edge on the target drops the parent from the views under
    // the pointer while the target stays under it.
    let root = TestRoot::new();
    let log: Log = Rc::default();

    let source = Empty::new().style(|s| s.size(40.0, 100.0)).draggable();

    let target = log_drag_over(Empty::new().style(|s| s.size_full()), "target", &log);
    let overhang = Container::new(target).style(|s| {
        s.absolute()
            .inset_left(50.0)
            .inset_top(0.0)
            .size(100.0, 50.0)
    });
    let parent = log_drag_over(
        Container::new(overhang).style(|s| s.size(100.0, 100.0)),
        "parent",
        &log,
    );

    let view = Stack::new((source, parent)).style(|s| s.size(250.0, 100.0));

    let mut harness = HeadlessHarness::new_with_size(root, view, 250.0, 100.0);

    // Press on the source and move past the drag threshold.
    harness.pointer_down(20.0, 20.0);
    harness.pointer_move(30.0, 20.0);

    // Onto the target, inside the parent's box (x 40..140; the target is x 90..190).
    harness.pointer_move(120.0, 20.0);
    assert_eq!(*log.borrow(), vec!["enter parent", "enter target"]);
    log.borrow_mut().clear();

    // Still on the target, past the parent's right edge.
    harness.pointer_move(160.0, 20.0);
    assert_eq!(
        *log.borrow(),
        vec!["leave parent"],
        "Only the parent should hear about the move; the target is still under the pointer"
    );

    // Off the target: now it is left.
    harness.pointer_move(230.0, 80.0);
    assert_eq!(*log.borrow(), vec!["leave parent", "leave target"]);

    harness.pointer_up(230.0, 80.0);
}
