//! `script`: one large rune. It holds, dissolves into embers that drift up and away, and the next
//! rune of the futhark draws itself out of the dark where they were. Never two glyphs at once:
//! the old one is gone before the new one arrives, and the embers carry the change.

use super::backdrop::{add, aurora_plain, backdrop, foreground, halo, motes, vignette};
use super::Frame;
use crate::anim::{bump, smooth};
use crate::glyphs;
use crate::palette;
use rune::{el, text, El};

/// Where in a rune's slot each beat happens: it holds, dissolves, then the next one forms.
const HOLD: f32 = 0.40;
const DISSOLVE: f32 = 0.34;
const FORM_AT: f32 = 0.76;

pub fn view(f: &Frame) -> El {
    let slots = glyphs::COUNT as f32;
    let slot = 1.0 / slots;
    let index = ((f.p / slot).floor() as usize) % glyphs::COUNT;
    let local = (f.p / slot).fract();
    let out = 1.0 - smooth((local - HOLD) / DISSOLVE);
    let into = smooth((local - FORM_AT) / (1.0 - FORM_AT));
    let size = f.u(0.300);
    let cx = f.x(0.5);
    let cy = f.y(0.500);

    // The pool behind breathes three times a cycle, so it never looks mechanical.
    let breath = 0.55
        + 0.45
            * bump(f.p, 0.0, 1.0)
                .max(bump(f.p, 0.34, 1.0))
                .max(bump(f.p, 0.68, 1.0));
    // A flash as one rune lets go and the next one lands.
    let flash = bump(local, HOLD, DISSOLVE).max(bump(local, FORM_AT, 1.0 - FORM_AT));

    // The frame is a pure function of time, so it lands as computed: `instant_changes` keeps rune
    // from easing this tree's layout, opacity and transforms from the previous frame's, which is
    // what turns a layer that changed place into a rectangle drawn between the two.
    let mut root = add(f.root().instant_changes(), backdrop(f));
    // The aurora sits in the upper third and is drawn at full strength (`aurora_plain` scales its
    // own streaks down from there): the same field monolith has, behind the one rune.
    root = add(root, aurora_plain(f, 0.24, 1.5));
    root = root.child(halo(
        cx,
        cy,
        size * 0.58,
        size * 0.66,
        palette::ember_gold(),
        0.30 * breath,
    ));
    root = root.child(halo(
        cx,
        cy,
        size * 1.10,
        size * 1.25,
        palette::ember(),
        0.10 * breath,
    ));
    root = root.child(halo(
        cx,
        cy,
        size * 2.30,
        size * 2.60,
        palette::ember_deep(),
        0.06,
    ));
    root = root.child(halo(
        cx,
        cy,
        size * 0.80,
        size * 1.00,
        palette::ember_gold(),
        0.16 * flash,
    ));

    // The rune that is leaving, and the one arriving: their windows never overlap, so only one of
    // them is ever visible, and each fade ends with its glyph on `cy`, so the last frame of a slot
    // and the first of the next put the same glyph in the same place. Both slots are drawn every
    // frame, at zero alpha while their window is closed: the tree keeps one shape all loop long.
    // A child that comes and goes shifts the place of every layer after it, and rune answers a
    // moved element by moving its rect across frames — a scene-wide gradient inside an in-between
    // rect reads as a hard-edged translucent panel.
    root = root.child(rune_at(
        cx,
        cy - size * 0.045 * (1.0 - out),
        size,
        glyphs::glyph(index),
        out,
    ));
    root = root.child(rune_at(
        cx,
        cy + size * 0.045 * (1.0 - into),
        size,
        glyphs::glyph(index + 1),
        into,
    ));
    root = add(
        root,
        motes(
            f,
            0x534352,
            18,
            [0.42, 0.10, 0.58, 0.80],
            palette::ember_gold(),
            0.55,
        ),
    );
    root = add(root, dissolve(f, index, cx, cy, size, local));
    root = add(root, foreground(f, 0x534c));
    add(root, vignette(f))
}

/// One rune: its own glow, and the glyph itself, faded by `alpha`.
fn rune_at(cx: f32, cy: f32, size: f32, g: &glyphs::Glyph, alpha: f32) -> El {
    let box_side = size * 1.5;
    let style = glyphs::style(size, 500.0);
    // One colour, whatever the slot's alpha: a colour that changes between frames is a property
    // rune eases toward its new value, so a rune that lands pale would arrive orange and warm up
    // over the next few frames. The fade itself is `dim`, below.
    let colour = palette::ember_pale();
    el().absolute(cx - box_side * 0.5, cy - box_side * 0.5)
        .size(box_side, box_side)
        .center()
        .child(halo(
            box_side * 0.5,
            box_side * 0.5,
            size * 0.55,
            size * 0.85,
            palette::ember_gold(),
            0.55,
        ))
        .child(halo(
            box_side * 0.5,
            box_side * 0.5,
            size * 1.30,
            size * 1.90,
            palette::ember(),
            0.12,
        ))
        .child(text(g.rune).font(style).color(colour))
        // `dim`, not `opacity`: an element's opacity is a tweened property, so a slot asked to go
        // from one to zero keeps drawing its old glyph for several frames and two runes show at
        // once. `dim` multiplies the subtree's own paint and lands at once, and it adds no group
        // layer for a rectangle to be painted into.
        .dim(alpha.clamp(0.0, 1.0))
}

/// The embers the rune dissolves into: they rise out of the space the rune occupied, are seeded
/// per slot so each change looks its own, and are drawn on every frame, at zero alpha while
/// their window is shut.
fn dissolve(f: &Frame, index: usize, cx: f32, cy: f32, size: f32, local: f32) -> Vec<El> {
    let mut rng = f.rng(0xD15_5000 ^ index as u64);
    let mut motes = Vec::new();
    // Every mote is drawn every frame, at zero alpha outside the dissolve, for the same reason
    // both rune slots are: the tree's shape never changes from one frame to the next.
    let window =
        smooth((local - (HOLD * 0.8 - 0.06)) / 0.10) * smooth(((FORM_AT + 0.12) - local) / 0.12);
    let life = ((local - HOLD) / (FORM_AT + 0.12 - HOLD)).clamp(0.0, 1.0);
    for _ in 0..26 {
        let along = rng.unit();
        let spread = rng.range(-0.5, 0.5);
        let rise = rng.range(0.6, 1.8);
        // A mote starts somewhere on the rune and drifts up and out as `life` grows.
        let x =
            cx + size * (spread * 0.5 + 0.10 * (life * 3.0 + along * std::f32::consts::TAU).sin());
        let y = cy + size * (along - 0.5) * 0.9 - size * rise * life;
        let fade = (1.0 - life).powf(1.4) * (0.4 + 0.6 * along) * window;
        // Big enough to read as a disc. A mote two or three pixels across is a square whatever
        // shape it is drawn with, so an ember's radius comes from the frame, not from the rune.
        let s = f.u(rng.range(0.0026, 0.0056));
        // One element per ember: a pale heart falling off through gold to nothing. A radial
        // gradient is round at every size, where a filled box still shows its corners.
        motes.push(
            el().absolute(x - s * 4.0, y - s * 4.0)
                .size(s * 8.0, s * 8.0)
                .radial_gradient(
                    [0.5, 0.5],
                    s * 4.0,
                    [
                        (0.0, palette::ember_pale().alpha(0.92 * fade)),
                        (0.16, palette::ember_gold().alpha(0.62 * fade)),
                        (0.38, palette::ember().alpha(0.24 * fade)),
                        (0.70, palette::ember_deep().alpha(0.07 * fade)),
                        (1.0, palette::CLEAR),
                    ],
                ),
        );
    }
    motes
}
