//! Tests for the time limit on a press whose view hides before the release.
//!
//! A view that hides itself when pressed leaves the release to whatever lies under it,
//! so the press and the release land on different views. That pair clicks the pressed
//! view only when the release comes within 100ms of the press; otherwise the click
//! goes to the two views' common ancestor. A press and a release on one view click it
//! however long the press is held.
//!
//! This file holds a single test so that it makes the first press and the first release
//! in its process. The press and the release used to be timed on two clocks, each
//! started by its own first use, so every later click was timed short by however long
//! the process's first click was held. The test opens with a held click for that
//! reason: a quick one would start the two clocks only milliseconds apart, and the
//! press held 150ms after it would still be timed at nearly 150ms.

use std::{thread::sleep, time::Duration};

use floem_test::prelude::*;

#[test]
fn test_press_on_a_view_that_hides_is_timed_from_the_press() {
    let root = TestRoot::new();
    let tracker = ClickTracker::new();
    let hidden = RwSignal::new(false);

    let hider = tracker
        .track_named("hider", Empty::new())
        .on_event_cont(el::PointerDown, move |_, _| hidden.set(true))
        .style(move |s| s.size(50.0, 50.0).apply_if(hidden.get(), |s| s.hide()));
    let view = Stack::new((
        tracker
            .track_named("plain", Empty::new())
            .style(|s| s.size(50.0, 50.0)),
        tracker
            .track_named("parent", Container::new(hider))
            .style(|s| s.size(50.0, 50.0)),
    ))
    .style(|s| s.size(100.0, 50.0));

    let mut harness = HeadlessHarness::new_with_size(root, view, 100.0, 50.0);

    // The process's first click: held 150ms and released on the view it pressed.
    harness.pointer_down(25.0, 25.0);
    sleep(Duration::from_millis(150));
    harness.pointer_up(25.0, 25.0);
    assert_eq!(
        tracker.clicked_names(),
        vec!["plain"],
        "A press and a release on one view should click it however long the press is held"
    );

    // Pressing the hider hides it, so the release lands on its parent.
    harness.pointer_down(75.0, 25.0);
    sleep(Duration::from_millis(150));
    harness.pointer_up(75.0, 25.0);
    assert_eq!(
        tracker.clicked_names(),
        vec!["plain", "parent"],
        "A press held 150ms on a view that then hid should click the parent the release \
         landed on, not the hidden view"
    );
}
