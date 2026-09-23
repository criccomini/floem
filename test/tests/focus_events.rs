//! Tests for the order of `FocusLost` and `FocusGained`.
//!
//! When focus moves, the view that lost it hears so before the view that gained it,
//! so a handler that turns something off on `FocusLost` and on on `FocusGained`
//! (IME input, a caret) ends on.

use std::{cell::RefCell, rc::Rc};

use floem::event::Event;
use floem_test::prelude::*;
use serial_test::serial;
use ui_events::keyboard::{Code, Key, KeyState, KeyboardEvent, Location, Modifiers, NamedKey};

type Log = Rc<RefCell<Vec<String>>>;

/// Records the view's `FocusGained` and `FocusLost` in `log` under `name`.
fn log_focus<V: IntoView>(view: V, name: &'static str, log: &Log) -> impl IntoView + use<V> {
    let gained_log = log.clone();
    let lost_log = log.clone();
    view.into_view()
        .on_event_cont(el::FocusGained, move |_, _| {
            gained_log.borrow_mut().push(format!("gained {name}"));
        })
        .on_event_cont(el::FocusLost, move |_, _| {
            lost_log.borrow_mut().push(format!("lost {name}"));
        })
}

fn press_tab(harness: &mut HeadlessHarness) {
    harness.dispatch_event(Event::Key(KeyboardEvent {
        key: Key::Named(NamedKey::Tab),
        code: Code::Tab,
        modifiers: Modifiers::default(),
        location: Location::Standard,
        is_composing: false,
        repeat: false,
        state: KeyState::Down,
    }));
}

#[test]
#[serial]
fn test_focus_moving_on_is_lost_before_it_is_gained() {
    let root = TestRoot::new();
    let log: Log = Rc::default();

    let a = Empty::new().style(|s| s.size(50.0, 50.0).keyboard_navigable());
    let b = Empty::new().style(|s| s.size(50.0, 50.0).keyboard_navigable());
    let b_id = b.view_id();
    let view = Stack::new((log_focus(a, "a", &log), log_focus(b, "b", &log)))
        .style(|s| s.size(100.0, 50.0));

    let mut harness = HeadlessHarness::new_with_size(root, view, 100.0, 50.0);

    harness.click(25.0, 25.0);
    assert_eq!(*log.borrow(), vec!["gained a"]);

    harness.click(75.0, 25.0);
    assert!(harness.is_focused(b_id));
    assert_eq!(
        *log.borrow(),
        vec!["gained a", "lost a", "gained b"],
        "The view that lost focus should hear so before the one that gained it"
    );
}

#[test]
#[serial]
fn test_view_refocused_by_keyboard_ends_focused() {
    // A clicked view focused again by Tab (it is the only stop, so Tab wraps to it)
    // is refocused for keyboard navigation: it loses focus and gains it again.
    let root = TestRoot::new();
    let log: Log = Rc::default();

    let a = Empty::new().style(|s| s.size(50.0, 50.0).keyboard_navigable());
    let a_id = a.view_id();
    let view = Container::new(log_focus(a, "a", &log)).style(|s| s.size(100.0, 50.0));

    let mut harness = HeadlessHarness::new_with_size(root, view, 100.0, 50.0);

    harness.click(25.0, 25.0);
    press_tab(&mut harness);
    assert!(harness.is_focused(a_id));
    assert_eq!(
        *log.borrow(),
        vec!["gained a", "lost a", "gained a"],
        "A view refocused in place should end on FocusGained"
    );
}
