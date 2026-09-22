//! Tests for the time limit on a press and a release that land on different views.
//!
//! A press on one view and a release on another count as a click on the first only
//! when the pointer moved at most 5px and the release came within 100ms of the press.
//!
//! This file holds a single test so that it makes the first press and the first release
//! in its process. The press and the release used to be timed on two clocks, each
//! started by its own first use, so the first click in a process always passed the
//! time check and every later one was timed short by however long the first was held.

use std::{thread::sleep, time::Duration};

use floem_test::prelude::*;

#[test]
fn test_press_and_release_on_different_views_are_timed_on_one_clock() {
    let root = TestRoot::new();
    let tracker = ClickTracker::new();

    let view = Stack::new((
        tracker
            .track_named("a", Empty::new())
            .style(|s| s.size(50.0, 50.0)),
        tracker
            .track_named("b", Empty::new())
            .style(|s| s.size(50.0, 50.0)),
    ))
    .style(|s| s.size(100.0, 50.0));

    let mut harness = HeadlessHarness::new_with_size(root, view, 100.0, 50.0);

    // The process's first click: press on a, hold 300ms, release on b 4px away.
    harness.pointer_down(48.0, 25.0);
    sleep(Duration::from_millis(300));
    harness.pointer_up(52.0, 25.0);
    assert_eq!(
        tracker.click_count(),
        0,
        "The first click in a process should not pass the time check when held 300ms"
    );

    // A later one held 150ms: under the first click's hold, but over the limit.
    harness.pointer_down(48.0, 25.0);
    sleep(Duration::from_millis(150));
    harness.pointer_up(52.0, 25.0);
    assert_eq!(
        tracker.click_count(),
        0,
        "A press held 150ms should not click, however long an earlier one was held"
    );

    // Released at once, the same press and release click a.
    harness.pointer_down(48.0, 25.0);
    harness.pointer_up(52.0, 25.0);
    assert_eq!(
        tracker.clicked_names(),
        vec!["a"],
        "A press released within 100ms and 5px should click the view it pressed"
    );
}
