//! Inline boxes added to an `AttrsList`: room in the text for something the
//! caller draws, laid out as part of the line.

use floem::text::{Attrs, AttrsList, InlineBox, TextLayout, TextWrapMode};

fn attrs_with_box(index: usize, width: f32, height: f32) -> AttrsList {
    let mut attrs = AttrsList::new(Attrs::new().font_size(14.0));
    attrs.add_inline_box(InlineBox {
        id: 7,
        index,
        width,
        height,
    });
    attrs
}

#[test]
fn a_box_takes_its_room_in_the_line_and_says_where_it_landed() {
    let plain =
        TextLayout::new_with_text("ab cd", AttrsList::new(Attrs::new().font_size(14.0)), None);
    let boxed = TextLayout::new_with_text("ab cd", attrs_with_box(3, 40.0, 10.0), None);

    let boxes = boxed.inline_boxes();
    assert_eq!(boxes.len(), 1);
    let placed = &boxes[0];
    assert_eq!(placed.id, 7);
    assert_eq!((placed.width, placed.height), (40.0, 10.0));
    // After "ab ", and the text after it is pushed along by its width.
    let before_cd = plain.cursor_point(3, floem::text::Affinity::Downstream).x;
    assert!(
        (f64::from(placed.x) - before_cd).abs() < 0.5,
        "{placed:?} {before_cd}"
    );
    assert!(
        (boxed.size().width - plain.size().width - 40.0).abs() < 0.5,
        "{} {}",
        boxed.size().width,
        plain.size().width
    );
    // It sits on the baseline.
    let metrics = boxed
        .parley_layout()
        .lines()
        .next()
        .unwrap()
        .metrics()
        .clone();
    assert!(
        (placed.y + placed.height - metrics.baseline).abs() < 0.01,
        "{placed:?} {metrics:?}"
    );
}

#[test]
fn a_tall_box_makes_its_line_taller() {
    let plain =
        TextLayout::new_with_text("ab cd", AttrsList::new(Attrs::new().font_size(14.0)), None);
    let boxed = TextLayout::new_with_text("ab cd", attrs_with_box(3, 10.0, 60.0), None);
    assert!(boxed.size().height > plain.size().height + 30.0);
}

#[test]
fn a_box_wraps_to_the_next_line_as_a_word_does() {
    let mut layout = TextLayout::new();
    layout.set_text_wrap_mode(TextWrapMode::Wrap);
    layout.set_text("ab cd", attrs_with_box(3, 40.0, 10.0), None);
    let one_line = layout.inline_boxes()[0].clone();
    let ab = layout.cursor_point(3, floem::text::Affinity::Upstream).x as f32;
    // Room for "ab " but not the box after it.
    layout.set_size(ab + 20.0, f32::MAX);
    assert!(layout.visual_line_count() >= 2);
    let wrapped = layout.inline_boxes()[0].clone();
    assert!(wrapped.y > one_line.y, "{one_line:?} {wrapped:?}");
    assert!(wrapped.x < 0.5, "{wrapped:?}");
}

#[test]
fn split_off_takes_the_boxes_past_the_split() {
    let mut attrs = attrs_with_box(1, 5.0, 5.0);
    attrs.add_inline_box(InlineBox {
        id: 8,
        index: 6,
        width: 5.0,
        height: 5.0,
    });
    let rest = attrs.split_off(4);
    assert_eq!(attrs.inline_boxes().len(), 1);
    assert_eq!(attrs.inline_boxes()[0].id, 7);
    assert_eq!(rest.inline_boxes().len(), 1);
    assert_eq!(
        (rest.inline_boxes()[0].id, rest.inline_boxes()[0].index),
        (8, 2)
    );
}

#[test]
fn a_selection_after_a_box_is_drawn_where_its_glyphs_are() {
    let layout = TextLayout::new_with_text("ab cd", attrs_with_box(3, 40.0, 10.0), None);
    let placed = layout.inline_boxes()[0].clone();
    // "d": a selection from byte 3 starts before the box, which sits there.
    let selection = layout.selection_from_byte_range(4, 5);
    let mut rects = Vec::new();
    layout.selection_geometry_with_line_metrics(&selection, |x0, _, x1, _| rects.push((x0, x1)));
    assert_eq!(rects.len(), 1);
    let (x0, x1) = rects[0];
    // The shading covers the glyph's ink, after the box, and does not
    // reach back over it.
    assert!(
        x0 >= f64::from(placed.x + placed.width) - 1.0,
        "{rects:?} {placed:?}"
    );
    assert!(x1 > x0);
}
