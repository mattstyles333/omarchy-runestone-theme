//! `monolith`: a dark carved standing stone, left of centre, its runes igniting one after another
//! from the crown down and cooling back to coal, embers rising from the runes that are lit, mist
//! and aurora behind it.

use super::backdrop::{add, aurora_plain, backdrop, halo, motes, vignette};
use super::Frame;
use crate::anim::bump;
use crate::glyphs::{self, Glyph};
use crate::palette::{self, mix, BLACK};
use rune::{canvas, el, text, El, LineCap, Path, Shape};

/// How many runes are cut into the stone, and how long each one burns.
const RUNES: usize = 9;
const IGNITE_START: f32 = 0.05;
const IGNITE_STEP: f32 = 0.072;
const BURN: f32 = 0.19;

/// How hot rune `i` is at phase `p`: the scene and anything following a rune's heat read this.
pub fn heat(i: usize, p: f32) -> f32 {
    let start = IGNITE_START + i as f32 * IGNITE_STEP;
    // A coal keeps a little of the light after the burn passes.
    bump(p, start, BURN).max(bump(p, start, BURN * 1.9) * 0.22)
}

pub fn view(f: &Frame) -> El {
    let stone_w = f.u(0.215);
    let stone_h = f.u(0.660);
    let stone_x = f.x(0.310) - stone_w * 0.5;
    let stone_y = f.y(0.520) - stone_h * 0.5;
    let foot = stone_y + stone_h;

    let mut root = add(f.root(), backdrop(f));
    root = add(root, aurora_plain(f, 0.24, 1.0));
    // The ground: a plane the stone stands on, the mist across it, and the stone's own shadow.
    root = root.child(ground(f, foot));
    root = add(root, mist(f, foot));
    root = root.child(halo(
        stone_x + stone_w * 0.55,
        foot + f.u(0.010),
        stone_w * 1.35,
        f.u(0.030),
        BLACK,
        0.80,
    ));
    root = root.child(stone(f, stone_x, stone_y, stone_w, stone_h));

    // The runes, crown to foot, each cut into the stone.
    let inset_y = stone_y + stone_h * 0.085;
    let span = stone_h * 0.80;
    let pitch = span / RUNES as f32;
    let rune_size = f.u(0.058);
    for i in 0..RUNES {
        let hot = heat(i, f.p);
        let cx = stone_x + stone_w * 0.5 + f.u(0.004) * (i as f32 * 0.7).sin();
        let cy = inset_y + pitch * (i as f32 + 0.5);
        root = root.child(carved(f, cx, cy, rune_size, glyphs::glyph(i * 2 + 1), hot));
    }

    root = add(
        root,
        motes(
            f,
            0x4d4f,
            22,
            [0.14, 0.46, 0.58, 0.95],
            palette::ember_gold(),
            0.85,
        ),
    );
    root = add(
        root,
        motes(
            f,
            0x4e4d,
            8,
            [0.18, 0.62, 0.52, 1.00],
            palette::ember(),
            0.55,
        ),
    );
    root = add(
        root,
        motes(
            f,
            0x7b1e,
            5,
            [0.10, 0.55, 0.70, 1.00],
            palette::ember_pale(),
            0.50,
        ),
    );
    add(root, vignette(f))
}

/// The silhouette's outline in unit space: a rough, slightly tapered menhir with a chipped
/// crown. Asymmetric on purpose, so it reads as a stone rather than a shape.
const OUTLINE: [[f32; 2]; 14] = [
    [0.10, 1.00],
    [0.04, 0.86],
    [0.02, 0.62],
    [0.06, 0.38],
    [0.13, 0.16],
    [0.24, 0.045],
    [0.44, 0.005],
    [0.62, 0.02],
    [0.74, 0.075],
    [0.84, 0.21],
    [0.93, 0.46],
    [0.97, 0.74],
    [0.92, 0.92],
    [0.80, 1.00],
];

/// One point of the outline in the frame's points.
fn at(x: f32, y: f32, w: f32, h: f32, p: [f32; 2]) -> [f32; 2] {
    [x + w * p[0], y + h * p[1]]
}

/// The stone itself: an irregular slab of dark rock, lit by the aurora along its left edge and
/// warmed from inside wherever its runes are burning.
fn stone(f: &Frame, x: f32, y: f32, w: f32, h: f32) -> El {
    let mut body = Path::new();
    body = body.move_to(at(x, y, w, h, OUTLINE[0]));
    for p in &OUTLINE[1..] {
        body = body.line_to(at(x, y, w, h, *p));
    }
    let body = body.close();

    // The lit edge, and the shadow side, follow the outline: a real rim, not a rectangle's.
    let mut rim = Path::new();
    rim = rim.move_to(at(x, y, w, h, OUTLINE[0]));
    for p in &OUTLINE[1..6] {
        rim = rim.line_to(at(x, y, w, h, *p));
    }
    let mut shade = Path::new();
    shade = shade.move_to(at(x, y, w, h, OUTLINE[9]));
    for p in &OUTLINE[10..] {
        shade = shade.line_to(at(x, y, w, h, *p));
    }

    let frame = |shape: Shape| {
        canvas(shape.view_box(f.w, f.h))
            .absolute(0.0, 0.0)
            .size(f.w, f.h)
    };

    el().absolute(0.0, 0.0)
        .size(f.w, f.h)
        // The body, in its own flat tone: most of this stone's light comes from its runes.
        .child(frame(Shape::new().fill_path(&body, palette::stone())))
        // A cool pool of aurora light on the crown, and the foot falling into shadow: both are
        // soft pools, so neither can spill past the outline as a lit rectangle.
        .child(frame(Shape::new().stroke_path(
            &rim,
            f.u(0.0040),
            LineCap::Round,
            palette::aurora_teal().alpha(0.26),
        )))
        .child(frame(Shape::new().stroke_path(
            &rim,
            f.u(0.0013),
            LineCap::Round,
            palette::stone_edge().alpha(0.50),
        )))
        .child(frame(Shape::new().stroke_path(
            &shade,
            f.u(0.0030),
            LineCap::Round,
            BLACK.alpha(0.70),
        )))
}

/// One rune cut into the stone: a groove in shadow with a light catching its lower lip, and, when
/// `hot` is up, light coming out of the cut itself.
fn carved(_f: &Frame, cx: f32, cy: f32, size: f32, g: &Glyph, hot: f32) -> El {
    let box_side = size * 1.7;
    let style = glyphs::style(size, 500.0);
    let colour = mix(
        palette::ember_deep().alpha(0.55),
        palette::ember_pale(),
        hot,
    );
    let mut cell = el()
        .absolute(cx - box_side * 0.5, cy - box_side * 0.5)
        .size(box_side, box_side);

    if hot > 0.01 {
        // Light from inside the cut, then a halo that spills onto the face.
        cell = cell.child(halo(
            box_side * 0.5,
            box_side * 0.5,
            size * 1.45,
            size * 2.10,
            palette::ember(),
            0.26 * hot,
        ));
        cell = cell.child(halo(
            box_side * 0.5,
            box_side * 0.5,
            size * 0.60,
            size * 1.00,
            palette::ember_gold(),
            0.95 * hot,
        ));
    }

    if std::env::var_os("RS_NOTEXT").is_some() {
        return cell;
    }
    // The cut: the groove's shadow to the lower right, a hair of light on its upper lip, then the
    // glyph. A cold rune reads as a groove; a hot one as the groove filled with light.
    cell = cell.child(
        el().absolute(1.1, 1.5)
            .size(box_side, box_side)
            .center()
            .child(text(g.rune).font(style).color(BLACK.alpha(0.60))),
    );
    cell = cell.child(
        el().absolute(-0.8, -1.0)
            .size(box_side, box_side)
            .center()
            .child(
                text(g.rune)
                    .font(style)
                    .color(palette::stone_lit().alpha(0.30)),
            ),
    );
    cell.child(
        el().size(box_side, box_side)
            .center()
            .child(text(g.rune).font(style).color(colour)),
    )
}

/// The ground: a plane falling away from the stone's foot, so it stands somewhere.
fn ground(f: &Frame, foot: f32) -> El {
    el().absolute(0.0, foot - f.u(0.01))
        .size(f.w, f.h - foot + f.u(0.01))
        .linear_gradient(
            0.0,
            [
                (0.0, palette::obsidian_deep().alpha(1.0)),
                (0.35, palette::obsidian_deep().alpha(0.55)),
                (1.0, BLACK.alpha(0.0)),
            ],
        )
}

/// Mist lying across the base: a few wide, soft pools drifting against each other.
fn mist(f: &Frame, foot: f32) -> Vec<El> {
    let mut motes = Vec::new();
    for i in 0..4 {
        let k = i as f32;
        let cx = f.x(0.34)
            + f.u(0.20) * (f.p * std::f32::consts::TAU + k * 1.7).sin()
            + (k - 1.5) * f.u(0.09);
        let cy = foot + f.u(0.012) + (k - 1.5) * f.u(0.006);
        let wide = f.u(0.30 - k * 0.03);
        let tint = if i % 2 == 0 {
            palette::aurora_teal().alpha(0.055)
        } else {
            palette::stone_edge().alpha(0.07)
        };
        motes.push(halo(cx, cy, wide, f.u(0.022), tint, 1.0));
    }
    motes
}
