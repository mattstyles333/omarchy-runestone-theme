//! `ring`: the 24 Elder Futhark runes on a slowly turning ring seen in perspective, the rune at
//! the front of the ellipse the brightest, aurora ribbons above.

use super::backdrop::{add, aurora, backdrop, embers, grain, halo, vignette};
use super::Frame;
use crate::anim::smooth;
use crate::glyphs;
use crate::palette::{self, mix};
use rune::{canvas, col, el, text, El, LineCap, Shape};
use std::f32::consts::TAU;

/// Slots the ring turns through in one cycle. Two whole slots keeps the motion calm and the
/// picture identical one cycle later.
const SLOTS_PER_CYCLE: f32 = 2.0;

pub fn view(f: &Frame) -> El {
    let cx = f.x(0.5);
    let cy = f.y(0.550);
    let rx = f.u(0.480);
    let ry = f.u(0.155);
    let turn = f.p * SLOTS_PER_CYCLE * TAU / glyphs::COUNT as f32;

    let mut root = add(f.root(), backdrop(f));
    root = add(root, aurora(f, 0.26, 1.05));
    // The pool of light the ring stands in, and the ellipse it rides on.
    root = root.child(halo(
        cx,
        cy + f.u(0.02),
        rx * 0.62,
        ry * 2.1,
        palette::ember_deep(),
        0.06,
    ));
    root = add(root, ellipse(f, cx, cy, rx, ry));

    // Back to front, so a nearer rune paints over a farther one.
    let mut order: Vec<(f32, usize)> = (0..glyphs::COUNT).map(|i| (angle(i, turn), i)).collect();
    order.sort_by(|a, b| {
        a.0.sin()
            .partial_cmp(&b.0.sin())
            .unwrap_or(std::cmp::Ordering::Equal)
    });
    for (theta, i) in order {
        let (sin, cos) = theta.sin_cos();
        // 0 at the back of the ring, 1 at the front.
        let depth = (sin + 1.0) * 0.5;
        let near = smooth(depth);
        let x = cx + rx * cos;
        let y = cy + ry * sin;
        let size = f.u(0.026 + 0.030 * near.powf(1.25));
        let heat = 0.18 + 0.82 * near.powf(2.0);
        root = root.child(slot(f, x, y, size, glyphs::glyph(i), heat));
    }

    root = add(
        root,
        embers(
            f,
            0x52494e,
            20,
            [0.22, 0.24, 0.78, 0.88],
            palette::ember_gold(),
            0.5,
        ),
    );
    root = add(root, vignette(f));
    add(root, grain(f))
}

fn angle(i: usize, turn: f32) -> f32 {
    i as f32 * TAU / glyphs::COUNT as f32 + turn
}

/// The ring's own line: a hairline ellipse, brightest at the front, drawn in the frame's own
/// coordinates so it lands exactly on the runes.
fn ellipse(f: &Frame, cx: f32, cy: f32, rx: f32, ry: f32) -> Vec<El> {
    let mut faint = Shape::new();
    let mut mid = Shape::new();
    let mut bright = Shape::new();
    let steps = 180;
    for k in 0..steps {
        let a0 = k as f32 / steps as f32 * TAU;
        let a1 = (k + 1) as f32 / steps as f32 * TAU;
        let at = |a: f32| [cx + rx * a.cos(), cy + ry * a.sin()];
        let depth = (a0.sin() + 1.0) * 0.5;
        if depth > 0.72 {
            bright = bright.line(
                at(a0),
                at(a1),
                2.2,
                LineCap::Round,
                palette::ember_gold().alpha(0.16),
            );
        } else if depth > 0.40 {
            mid = mid.line(
                at(a0),
                at(a1),
                1.6,
                LineCap::Round,
                palette::ember().alpha(0.09),
            );
        } else {
            faint = faint.line(
                at(a0),
                at(a1),
                1.2,
                LineCap::Round,
                palette::ember_deep().alpha(0.08),
            );
        }
    }
    vec![
        canvas(faint.view_box(f.w, f.h))
            .absolute(0.0, 0.0)
            .size(f.w, f.h),
        canvas(mid.view_box(f.w, f.h))
            .absolute(0.0, 0.0)
            .size(f.w, f.h),
        canvas(bright.view_box(f.w, f.h))
            .absolute(0.0, 0.0)
            .size(f.w, f.h),
    ]
}

/// One rune on the ring: a glow that grows with how near it is, over the rune itself.
fn slot(_f: &Frame, cx: f32, cy: f32, size: f32, g: &glyphs::Glyph, heat: f32) -> El {
    let side = size * 1.9;
    let style = glyphs::style(size, 500.0);
    let colour = mix(palette::ember_deep(), palette::ember_pale(), heat);
    col()
        .absolute(cx - side * 0.5, cy - side * 0.5)
        .size(side, side)
        .child(halo(
            side * 0.5,
            side * 0.5,
            size * 1.5,
            size * 1.5,
            palette::ember(),
            0.10 * heat,
        ))
        .child(halo(
            side * 0.5,
            side * 0.5,
            size * 0.85,
            size * 0.85,
            palette::ember_gold(),
            0.40 * heat,
        ))
        .child(
            el().size(side, side)
                .center()
                .child(text(g.rune).font(style).color(colour)),
        )
}
