use floem::event::Event;
use floem::headless::{HeadlessHarness, TestRoot};
use floem::reactive::{RwSignal, SignalGet, SignalWith};
use floem::ui_events::keyboard::{Code, Key, KeyState, KeyboardEvent, Modifiers, NamedKey};
use floem::views::editor::Editor;
use floem::views::{Button, Decorators, Stack, TextInput, text_editor};
use floem::{HasViewId, IntoView, ViewId};

const TEXT: &str = "the quick brown fox";

/// `view` between two keyboard-navigable buttons, so Alt with an arrow has
/// somewhere to send the focus, with the keyboard on `focus`.
fn between_buttons(root: TestRoot, view: impl IntoView, focus: ViewId) -> HeadlessHarness {
    let button =
        |label: &'static str| Button::new(label).style(|s| s.keyboard_navigable().width(80.0));
    let row =
        Stack::horizontal((button("left"), view, button("right"))).style(|s| s.size(600.0, 200.0));
    let mut harness = HeadlessHarness::new_with_size(root, row, 600.0, 200.0);
    harness.rebuild();
    harness.paint();
    focus.request_focus();
    harness.rebuild();
    assert!(harness.is_focused(focus), "the box takes the keyboard");
    harness
}

fn key(key: Key, code: Code, modifiers: Modifiers) -> Event {
    Event::Key(KeyboardEvent {
        state: KeyState::Down,
        key,
        code,
        location: Default::default(),
        modifiers,
        repeat: false,
        is_composing: false,
    })
}

fn alt_arrow(name: NamedKey) -> Event {
    let code = match name {
        NamedKey::ArrowLeft => Code::ArrowLeft,
        NamedKey::ArrowRight => Code::ArrowRight,
        NamedKey::ArrowUp => Code::ArrowUp,
        NamedKey::ArrowDown => Code::ArrowDown,
        _ => unreachable!(),
    };
    key(Key::Named(name), code, Modifiers::ALT)
}

fn offset(ed: &Editor) -> usize {
    ed.cursor.with(|c| c.offset())
}

/// Alt with an arrow moves the editor's cursor by a word, and is also the
/// key floem's default action walks the focus with. The cursor used to
/// move and then the focus left for a neighbour, taking the next keystroke.
#[test]
fn the_editor_keeps_the_keyboard_through_alt_arrows() {
    let root = TestRoot::new();
    let view = text_editor(TEXT).style(|s| s.width(300.0).height(200.0));
    let ed = view.editor().clone();
    let id = ed.editor_view_id.get().expect("the editor's own view");
    let mut harness = between_buttons(root, view, id);
    assert_eq!(offset(&ed), 0);

    harness.dispatch_event(alt_arrow(NamedKey::ArrowLeft));
    harness.rebuild();
    assert_eq!(offset(&ed), 0, "nothing to move to at the start");
    assert!(harness.is_focused(id), "and the focus stays put");

    harness.dispatch_event(alt_arrow(NamedKey::ArrowRight));
    harness.rebuild();
    assert_eq!(offset(&ed), 3, "to the end of the word");
    assert!(harness.is_focused(id));

    harness.dispatch_event(alt_arrow(NamedKey::ArrowRight));
    harness.rebuild();
    assert_eq!(offset(&ed), 9, "and of the next");
    assert!(harness.is_focused(id));

    harness.dispatch_event(alt_arrow(NamedKey::ArrowLeft));
    harness.rebuild();
    assert_eq!(offset(&ed), 4, "back to the start of the word");
    assert!(harness.is_focused(id));
}

#[test]
fn the_text_input_keeps_the_keyboard_through_alt_arrows() {
    let root = TestRoot::new();
    let buffer = RwSignal::new(TEXT.to_string());
    let view = TextInput::new(buffer).style(|s| s.width(300.0));
    let id = view.view_id();
    let mut harness = between_buttons(root, view, id);

    // The cursor starts at the front, with no word behind it: the input
    // has nothing to do with the key, and still keeps it.
    harness.dispatch_event(alt_arrow(NamedKey::ArrowLeft));
    harness.rebuild();
    assert!(
        harness.is_focused(id),
        "nothing to move to, and the focus stays put"
    );

    harness.dispatch_event(alt_arrow(NamedKey::ArrowRight));
    harness.rebuild();
    assert!(harness.is_focused(id));
    harness.dispatch_event(alt_arrow(NamedKey::ArrowRight));
    harness.rebuild();
    assert!(harness.is_focused(id));
    harness.dispatch_event(key(
        Key::Character("!".into()),
        Code::Digit1,
        Modifiers::empty(),
    ));
    harness.rebuild();
    assert_eq!(
        buffer.get(),
        "the quick! brown fox",
        "typed at the second word's end"
    );

    harness.dispatch_event(alt_arrow(NamedKey::ArrowLeft));
    harness.rebuild();
    assert!(harness.is_focused(id));
    harness.dispatch_event(key(
        Key::Character("?".into()),
        Code::Slash,
        Modifiers::empty(),
    ));
    harness.rebuild();
    assert_eq!(
        buffer.get(),
        "the ?quick! brown fox",
        "typed at that word's start"
    );

    // An arrow the single-line input has no movement for is still its own.
    harness.dispatch_event(alt_arrow(NamedKey::ArrowUp));
    harness.rebuild();
    assert!(harness.is_focused(id));
}
