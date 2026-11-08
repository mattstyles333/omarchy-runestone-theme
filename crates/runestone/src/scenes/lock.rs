//! The lock screen mark: the word *rune* in Elder Futhark, glowing, on nothing at all.
//!
//! This is not one of the three wallpapers. It is the image Omarchy keeps as a theme's
//! `unlock.png` — a transparent PNG the lock screen draws over itself — so it has no backdrop
//! and every pixel that is not a rune or its glow stays clear.

use super::{Frame, ROOT};
use crate::glyphs;
use crate::palette;
use rune::{el, text, El};

/// The runes that spell it out: r, u, n, e.
const WORD: [&str; 4] = ["ᚱ", "ᚢ", "ᚾ", "ᛖ"];

pub fn view(f: &Frame) -> El {
    let size = f.u(0.60);
    let style = glyphs::style(size, 500.0);
    let pitch = size * 1.05;
    let width = pitch * WORD.len() as f32;
    let left = f.x(0.5) - width * 0.5;
    let centre = f.y(0.5);

    let mut root = el().key(ROOT).w_full().h_full();
    for (i, rune) in WORD.iter().enumerate() {
        let cx = left + pitch * (i as f32 + 0.5);
        root = root
            .child(super::halo(
                cx,
                centre,
                size * 0.55,
                size * 0.80,
                palette::ember_gold(),
                0.48,
            ))
            .child(super::halo(
                cx,
                centre,
                size * 1.15,
                size * 1.60,
                palette::ember(),
                0.11,
            ))
            .child(
                el().absolute(cx - size * 0.75, centre - size * 0.75)
                    .size(size * 1.5, size * 1.5)
                    .center()
                    .child(text(*rune).font(style).color(palette::ember_pale())),
            );
    }
    // A hairline of ember under the word, the way a carved stone is ruled.
    root.child(
        el().absolute(f.x(0.5) - width * 0.32, centre + size * 0.72)
            .size(width * 0.64, f.u(0.010))
            .radius(f.u(0.005))
            .fill(palette::ember().alpha(0.55)),
    )
}
