//! Glyph rasterization with a faux bold that grows every glyph.
//!
//! swash's `Render::embolden` moves each point of an outline outward, and
//! which way is outward depends on the outline's winding. swash takes that
//! winding from the signed area of all the outline's points strung together
//! as one polygon, so the edges bridging one contour to the next count too.
//! Where a small contour comes first, as the dot of SF Mono's `i` or the
//! accent of an `Í`, those edges can flip the sign, and swash then moves
//! every point inward: the bold thins the glyph by as much as it thickens
//! the ones around it. Here the winding is summed contour by contour, as
//! FreeType's `FT_Outline_Get_Orientation` does, and the emboldening itself
//! is swash's (a port of FreeType's `FT_Outline_EmboldenXY`), unchanged.

use swash::scale::image::{Content, Image};
use swash::scale::outline::Outline;
use swash::scale::{Render, Scaler, Source, StrikeWith};
use swash::zeno::{Format, Mask, Origin, Point, Transform, Vector, Verb};

/// Renders a glyph as `Render` does from `ColorOutline(0)`,
/// `ColorBitmap(BestFit)` and `Outline`, in that order, into an alpha mask
/// offset by `offset`, with a faux bold of `embolden` and `transform`
/// applied to the outline. swash emboldens no color glyph, so those still
/// go through `Render`.
pub(crate) fn render_glyph(
    scaler: &mut Scaler,
    glyph_id: u16,
    offset: Vector,
    embolden: f32,
    transform: Option<Transform>,
) -> Option<Image> {
    let color = Render::new(&[
        Source::ColorOutline(0),
        Source::ColorBitmap(StrikeWith::BestFit),
    ])
    .format(Format::Alpha)
    .offset(offset)
    .transform(transform)
    .render(scaler, glyph_id);
    if color.is_some() {
        return color;
    }
    if !scaler.has_outlines() {
        return None;
    }
    let mut outline = scaler.scale_outline(glyph_id)?;
    if embolden != 0.0 {
        embolden_outline(&mut outline, embolden);
    }
    if let Some(transform) = &transform {
        outline.transform(transform);
    }
    let mut image = Image::new();
    image.placement = Mask::new(outline.path())
        .format(Format::Alpha)
        .origin(Origin::BottomLeft)
        .offset(offset)
        .render_offset(offset)
        .inspect(|format, width, height| {
            image.data.resize(format.buffer_size(width, height), 0);
        })
        .render_into(&mut image.data[..], None);
    image.content = Content::Mask;
    image.source = Source::Outline;
    Some(image)
}

/// Widens every stroke of `outline` by twice `strength` pixels, the glyph
/// growing up and to the right, as swash's `Outline::embolden` does.
fn embolden_outline(outline: &mut Outline, strength: f32) {
    let verbs = outline.verbs().to_vec();
    embolden(outline.points_mut(), &verbs, strength);
}

fn embolden(points: &mut [Point], verbs: &[Verb], strength: f32) {
    let contours = contours(verbs);
    let winding = winding(points, &contours);
    for contour in contours {
        embolden_contour(&mut points[contour], winding, strength, strength);
    }
}

/// The range of `points` each contour of a path with `verbs` holds,
/// delimited as swash's `Outline::embolden` delimits them.
fn contours(verbs: &[Verb]) -> Vec<std::ops::Range<usize>> {
    let mut contours = Vec::new();
    let mut start = 0;
    let mut end = 0;
    for verb in verbs {
        match verb {
            Verb::MoveTo | Verb::Close => {
                if end > start {
                    contours.push(start..end);
                }
                start = end;
                if *verb == Verb::MoveTo {
                    end += 1;
                }
            }
            Verb::LineTo => end += 1,
            Verb::QuadTo => end += 2,
            Verb::CurveTo => end += 3,
        }
    }
    if end > start {
        contours.push(start..end);
    }
    contours
}

/// 1 where the contours, taken together, run counterclockwise with y up,
/// else 0: swash's convention, from the sum of each contour's own signed
/// area, closing every contour on its own first point.
fn winding(points: &[Point], contours: &[std::ops::Range<usize>]) -> u8 {
    let mut area = 0.0;
    for contour in contours {
        let points = &points[contour.clone()];
        let mut prev = points[points.len() - 1];
        for cur in points {
            area += (cur.y - prev.y) * (cur.x + prev.x);
            prev = *cur;
        }
    }
    if area > 0.0 { 1 } else { 0 }
}

/// swash 0.2's `embolden` for one contour (Apache-2.0 OR MIT), itself a
/// port of FreeType's `FT_Outline_EmboldenXY`.
fn embolden_contour(points: &mut [Point], winding: u8, x_strength: f32, y_strength: f32) {
    if points.is_empty() {
        return;
    }
    let last = points.len() - 1;
    let mut i = last;
    let mut j = 0;
    let mut k = !0;
    let mut out_len;
    let mut in_len = 0.;
    let mut anchor_len = 0.;
    let mut anchor = Point::ZERO;
    let mut out;
    let mut in_ = Point::ZERO;
    while j != i && i != k {
        if j != k {
            out = points[j] - points[i];
            out_len = out.length();
            if out_len == 0. {
                j = if j < last { j + 1 } else { 0 };
                continue;
            } else {
                let s = 1. / out_len;
                out.x *= s;
                out.y *= s;
            }
        } else {
            out = anchor;
            out_len = anchor_len;
        }
        if in_len != 0. {
            if k == !0 {
                k = i;
                anchor = in_;
                anchor_len = in_len;
            }
            let mut d = (in_.x * out.x) + (in_.y * out.y);
            let shift = if d > -0.9396 {
                d += 1.;
                let mut sx = in_.y + out.y;
                let mut sy = in_.x + out.x;
                if winding == 0 {
                    sx = -sx;
                } else {
                    sy = -sy;
                }
                let mut q = (out.x * in_.y) - (out.y * in_.x);
                if winding == 0 {
                    q = -q;
                }
                let l = in_len.min(out_len);
                if x_strength * q <= l * d {
                    sx = sx * x_strength / d;
                } else {
                    sx = sx * l / q;
                }
                if y_strength * q <= l * d {
                    sy = sy * y_strength / d;
                } else {
                    sy = sy * l / q;
                }
                Point::new(sx, sy)
            } else {
                Point::ZERO
            };

            while i != j {
                points[i].x += x_strength + shift.x;
                points[i].y += y_strength + shift.y;
                i = if i < last { i + 1 } else { 0 };
            }
        } else {
            i = j;
        }
        in_ = out;
        in_len = out_len;
        j = if j < last { j + 1 } else { 0 };
    }
}

#[cfg(test)]
mod tests {
    use swash::FontRef;
    use swash::scale::image::Image;
    use swash::scale::{Render, ScaleContext, Source, StrikeWith};
    use swash::zeno::{Format, Point, Vector, Verb};

    use super::{contours, embolden, render_glyph, winding};

    const FIRA_SANS: &[u8] = include_bytes!("../../examples/webgpu/fonts/FiraSans-Medium.ttf");

    /// A square dot above a bar, both counterclockwise, the dot first:
    /// the shape of an `i` whose dot leads its outline.
    fn dot_over_bar() -> (Vec<Point>, Vec<Verb>) {
        let points = [
            (4.0, 10.0),
            (6.0, 10.0),
            (6.0, 12.0),
            (4.0, 12.0),
            (10.0, 0.0),
            (10.0, 1.0),
            (0.0, 1.0),
            (0.0, 0.0),
        ]
        .map(|(x, y)| Point::new(x, y))
        .to_vec();
        let square = [
            Verb::MoveTo,
            Verb::LineTo,
            Verb::LineTo,
            Verb::LineTo,
            Verb::Close,
        ];
        (points, [square, square].concat())
    }

    fn signed_area(points: &[Point]) -> f32 {
        let mut prev = points[points.len() - 1];
        let mut area = 0.0;
        for cur in points {
            area += (cur.y - prev.y) * (cur.x + prev.x);
            prev = *cur;
        }
        area / 2.0
    }

    fn render(font: FontRef, ch: char, x_bin: u8, embolden: f32, ours: bool) -> Image {
        let mut context = ScaleContext::new();
        let mut scaler = context.builder(font).size(24.0).build();
        let glyph_id = font.charmap().map(ch);
        let offset = Vector::new(f32::from(x_bin) / 4.0, 0.0);
        if ours {
            render_glyph(&mut scaler, glyph_id, offset, embolden, None).unwrap()
        } else {
            Render::new(&[
                Source::ColorOutline(0),
                Source::ColorBitmap(StrikeWith::BestFit),
                Source::Outline,
            ])
            .format(Format::Alpha)
            .offset(offset)
            .embolden(embolden)
            .render(&mut scaler, glyph_id)
            .unwrap()
        }
    }

    fn ink(image: &Image) -> u32 {
        image.data.iter().map(|&a| u32::from(a)).sum()
    }

    #[test]
    fn winding_ignores_the_edges_between_contours() {
        let (points, verbs) = dot_over_bar();
        // Strung together as one polygon, as swash takes them, the points
        // wind clockwise, which is what thinned the glyph.
        assert!(signed_area(&points) < 0.0);
        assert_eq!(winding(&points, &contours(&verbs)), 1);
    }

    #[test]
    fn embolden_grows_every_contour() {
        let (mut points, verbs) = dot_over_bar();
        embolden(&mut points, &verbs, 0.5);
        // swash's faux bold widens a stroke by twice its strength.
        let areas = [signed_area(&points[..4]), signed_area(&points[4..])];
        assert!((areas[0] - 3.0 * 3.0).abs() < 1e-4, "{areas:?}");
        assert!((areas[1] - 11.0 * 2.0).abs() < 1e-4, "{areas:?}");
    }

    #[test]
    fn a_glyph_led_by_a_small_contour_gets_bolder() {
        let font = FontRef::from_index(FIRA_SANS, 0).unwrap();
        let plain = ink(&render(font, '±', 0, 0.0, true));
        assert!(ink(&render(font, '±', 0, 0.5, true)) > plain);
        // swash thins it, which is why this module exists.
        assert!(ink(&render(font, '±', 0, 0.5, false)) < plain);
    }

    #[test]
    fn a_glyph_swash_emboldens_right_comes_out_as_it_did() {
        let font = FontRef::from_index(FIRA_SANS, 0).unwrap();
        for ch in (' '..='~').chain(['é', 'ü', 'ß']) {
            for x_bin in 0..4 {
                let ours = render(font, ch, x_bin, 0.2, true);
                let swash = render(font, ch, x_bin, 0.2, false);
                let place = |image: &Image| {
                    let p = image.placement;
                    (p.left, p.top, p.width, p.height)
                };
                assert_eq!(place(&ours), place(&swash), "{ch:?} in bin {x_bin}");
                assert_eq!(ours.data, swash.data, "{ch:?} in bin {x_bin}");
            }
        }
    }
}
