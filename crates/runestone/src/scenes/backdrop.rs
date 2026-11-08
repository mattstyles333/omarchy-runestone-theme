//! The layers every scene shares: the obsidian base with its depth, the aurora behind the
//! subject, the embers drifting through it, and the vignette and grain that close the frame.
//!
//! Every helper here returns the elements it draws as a flat list. Nothing is wrapped in a
//! container element: a container that is itself laid out in flow carries its absolutely
//! positioned children with it, which shows up as a rectangle of its bounds painted off its
//! place.

use super::Frame;
use crate::anim::{sway, wave};
use crate::fx;
use crate::palette::{self, CLEAR};
use rune::{effect, el, El};

/// The base: obsidian, lifted a little where the light falls and dropped to true black in the
/// corners. Two soft pools, so it reads as depth rather than a flat fill.
pub fn backdrop(f: &Frame) -> Vec<El> {
    vec![
        el().absolute(0.0, 0.0).size(f.w, f.h).radial_gradient(
            [0.42, 0.46],
            [f.w * 0.85, f.h * 0.95],
            [
                (0.0, palette::stone_lit().alpha(0.10)),
                (0.30, palette::stone().alpha(0.05)),
                (0.70, CLEAR),
                (1.0, CLEAR),
            ],
        ),
        el().absolute(0.0, 0.0).size(f.w, f.h).radial_gradient(
            [0.50, 1.04],
            [f.w * 0.62, f.h * 0.42],
            [
                (0.0, palette::aurora_teal().alpha(0.03)),
                (0.45, palette::aurora_violet().alpha(0.02)),
                (1.0, CLEAR),
            ],
        ),
    ]
}

/// One soft streak of aurora: an elongated pool of light, turned a few degrees off horizontal.
pub struct Ribbon {
    pub cx: f32,
    pub cy: f32,
    pub w: f32,
    pub h: f32,
    /// Radians off horizontal.
    pub angle: f32,
    pub tint: rune::Color,
    pub strength: f32,
}

pub fn ribbon(r: Ribbon) -> El {
    let Ribbon {
        cx,
        cy,
        w,
        h,
        angle,
        tint,
        strength,
    } = r;
    el().absolute(cx - w * 0.5, cy - h * 0.5)
        .size(w, h)
        .rotate(angle)
        .radial_gradient(
            [0.5, 0.5],
            [w * 0.5, h * 0.5],
            [
                (0.0, tint.alpha(strength)),
                (0.16, tint.alpha(strength * 0.80)),
                (0.34, tint.alpha(strength * 0.52)),
                (0.54, tint.alpha(strength * 0.27)),
                (0.74, tint.alpha(strength * 0.09)),
                (1.0, CLEAR),
            ],
        )
}

/// The aurora behind a scene: a faint gradient body so the sky is never flat, and two rune-fx
/// curtains over it, one broad and one drifting against it, both centred on `y`.
pub fn aurora(f: &Frame, y: f32, strength: f32) -> Vec<El> {
    let breathe = 0.75 + 0.25 * sway(f.p, 1.0, 0.0);
    vec![
        ribbon(Ribbon {
            cx: f.x(0.50) + f.u(0.06) * wave(f.p, 1.0, 0.0),
            cy: f.y(y) + f.u(0.02) * wave(f.p, 2.0, 1.1),
            w: f.w * 1.15,
            h: f.u(0.50),
            angle: -0.16 + 0.03 * wave(f.p, 1.0, 0.6),
            tint: palette::aurora_teal(),
            strength: 0.030 * strength * breathe,
        }),
        ribbon(Ribbon {
            cx: f.x(0.56) + f.u(0.08) * wave(f.p, 1.0, 2.4),
            cy: f.y(y + 0.09) + f.u(0.03) * wave(f.p, 2.0, 0.3),
            w: f.w * 0.85,
            h: f.u(0.40),
            angle: 0.22 + 0.03 * wave(f.p, 1.0, 1.9),
            tint: palette::aurora_violet(),
            strength: 0.024 * strength * breathe,
        }),
        // The curtains themselves: rune-fx draws these at the loop's phase.
        effect(fx::AURORA)
            .absolute(f.x(0.5) - f.w * 0.52, f.y(y) - f.u(0.46))
            .size(f.w * 1.04, f.u(0.78))
            .clip_fade(0.0, 0.0, 0.0, f.u(0.22))
            .effect_data([strength, 0.0, 0.0, 0.0]),
        effect(fx::AURORA_DRIFT)
            .absolute(f.x(0.5) - f.w * 0.42, f.y(y + 0.08) - f.u(0.30))
            .size(f.w * 0.84, f.u(0.52))
            .clip_fade(0.0, 0.0, 0.0, f.u(0.16))
            .effect_data([strength, 0.0, 0.0, 0.0]),
    ]
}

/// The aurora without rune-fx: the same two colours, built as a band of tall soft streaks that
/// drift against each other. Used by the scenes that do not take rune-fx layers.
pub fn aurora_plain(f: &Frame, y: f32, strength: f32) -> Vec<El> {
    let mut layers = Vec::new();
    let breathe = 0.75 + 0.25 * sway(f.p, 1.0, 0.0);
    let mut rng = f.rng(0xA0_2A);
    // The body of the curtain, then its streaks.
    layers.push(ribbon(Ribbon {
        cx: f.x(0.50) + f.u(0.06) * wave(f.p, 1.0, 0.0),
        cy: f.y(y) + f.u(0.02) * wave(f.p, 2.0, 1.1),
        w: f.w * 1.15,
        h: f.u(0.46),
        angle: -0.14 + 0.03 * wave(f.p, 1.0, 0.6),
        tint: palette::aurora_teal(),
        strength: 0.075 * strength * breathe,
    }));
    for i in 0..16 {
        let k = i as f32 / 15.0;
        let drift = rng.range(-1.0, 1.0);
        let tall = rng.range(0.55, 1.0);
        let tint = if i % 3 == 0 {
            palette::aurora_violet()
        } else if i % 3 == 1 {
            palette::aurora_teal()
        } else {
            palette::aurora_blue()
        };
        layers.push(ribbon(Ribbon {
            cx: f.x(0.06 + 0.88 * k)
                + f.u(0.05) * drift * wave(f.p, 1.0, k * std::f32::consts::TAU),
            cy: f.y(y) + f.u(0.055) * wave(f.p, 2.0, k * 3.1),
            w: f.u(0.10) + f.u(0.03) * rng.unit(),
            h: f.u(0.42) * tall,
            angle: rng.range(-0.22, 0.22),
            tint,
            strength: (0.020 + 0.014 * rng.unit()) * strength * breathe,
        }));
    }
    layers
}

/// A handful of big, out-of-focus motes in front of everything: the parallax layer that makes a
/// still frame feel like it has depth.
pub fn foreground(f: &Frame, salt: u64) -> Vec<El> {
    let mut rng = f.rng(salt);
    let mut motes = Vec::new();
    for _ in 0..5 {
        let lap = ((f.p + rng.unit()) % 1.0 + 1.0) % 1.0;
        let x = f.x(rng.range(0.05, 0.95))
            + f.u(0.03) * wave(f.p, 1.0, rng.unit() * std::f32::consts::TAU);
        let y = f.y(1.02) - lap * f.u(1.05);
        let s = f.u(rng.range(0.010, 0.022));
        let fade = ((y / f.h) * std::f32::consts::PI).sin().max(0.0);
        motes.push(
            el().absolute(x - s * 2.0, y - s * 2.0)
                .size(s * 4.0, s * 4.0)
                .radial_gradient(
                    [0.5, 0.5],
                    s * 2.0,
                    [
                        (0.0, palette::ember().alpha(0.20 * fade)),
                        (0.35, palette::ember_deep().alpha(0.12 * fade)),
                        (0.70, palette::ember_deep().alpha(0.04 * fade)),
                        (1.0, CLEAR),
                    ],
                ),
        );
        // The mote's own soft centre, so it never reads as a flat disc either.
        motes.push(
            el().absolute(x - s * 0.55, y - s * 0.55)
                .size(s * 1.1, s * 1.1)
                .radial_gradient(
                    [0.5, 0.5],
                    s * 0.55,
                    [
                        (0.0, palette::ember_pale().alpha(0.30 * fade)),
                        (1.0, CLEAR),
                    ],
                ),
        );
    }
    motes
}

/// The vignette: the frame closes in toward black, and the floor falls away.
pub fn vignette(f: &Frame) -> Vec<El> {
    vec![
        el().absolute(0.0, 0.0).size(f.w, f.h).radial_gradient(
            [0.5, 0.48],
            [f.w * 0.72, f.h * 0.78],
            [
                (0.0, CLEAR),
                (0.52, CLEAR),
                (0.78, palette::BLACK.alpha(0.30)),
                (1.0, palette::BLACK.alpha(0.88)),
            ],
        ),
        el().absolute(0.0, 0.0).size(f.w, f.h).linear_gradient(
            0.0,
            [
                (0.0, palette::BLACK.alpha(0.85)),
                (0.14, palette::BLACK.alpha(0.35)),
                (0.30, CLEAR),
            ],
        ),
        el().absolute(0.0, 0.0).size(f.w, f.h).linear_gradient(
            std::f32::consts::PI,
            [(0.0, palette::BLACK.alpha(0.55)), (0.10, CLEAR)],
        ),
    ]
}

/// Film grain over the whole frame, the last thing a scene adds.
pub fn grain(f: &Frame) -> Vec<El> {
    vec![effect(fx::GRAIN).absolute(0.0, 0.0).size(f.w, f.h)]
}

/// A soft pool of light: the glow under a rune, or the light it throws on what is behind it.
pub fn halo(cx: f32, cy: f32, radius: f32, radius_y: f32, tint: rune::Color, strength: f32) -> El {
    let (w, h) = (radius * 2.0, radius_y * 2.0);
    el().absolute(cx - radius, cy - radius_y)
        .size(w, h)
        .radial_gradient(
            [0.5, 0.5],
            [radius, radius_y],
            [
                (0.0, tint.alpha(strength)),
                (0.22, tint.alpha(strength * 0.52)),
                (0.52, tint.alpha(strength * 0.17)),
                (0.78, tint.alpha(strength * 0.04)),
                (1.0, CLEAR),
            ],
        )
}

/// Embers as single elements: a soft pool each, rising through `band`, wrapping once per cycle.
/// One element per mote, sized by its own radius, so nothing can inherit a container's bounds.
pub fn motes(
    f: &Frame,
    salt: u64,
    count: usize,
    band: [f32; 4],
    tint: rune::Color,
    strength: f32,
) -> Vec<El> {
    let mut rng = f.rng(salt);
    let mut out = Vec::new();
    let [x0, y0, x1, y1] = band;
    let (top, span) = (f.y(y0), f.y(y1 - y0));
    for _ in 0..count {
        let start = rng.unit();
        let laps = 1.0 + (rng.unit() * 2.0).floor();
        let drift = rng.range(-1.0, 1.0);
        let cycles = 1.0 + (rng.unit() * 2.0).floor();
        let brightness = rng.range(0.35, 1.0);
        let r = (f.u(rng.range(0.0016, 0.0042)) * 3.6).clamp(0.5, f.u(0.02));
        let y =
            top + (span + f.u(0.06)) - ((start + f.p * laps).rem_euclid(1.0)) * (span + f.u(0.06));
        let x = f.x(x0 + (x1 - x0) * rng.unit())
            + f.u(0.045) * drift * wave(f.p, cycles, start * std::f32::consts::TAU);
        let across = ((y - top) / span).clamp(0.0, 1.0);
        let fade = (across * std::f32::consts::PI).sin().max(0.0);
        let a = strength * brightness * fade;
        out.push(
            el().absolute(x - r, y - r)
                .size(r * 2.0, r * 2.0)
                .radial_gradient(
                    [0.5, 0.5],
                    r,
                    [
                        (0.0, palette::ember_pale().alpha(0.85 * a)),
                        (0.25, tint.alpha(0.40 * a)),
                        (0.60, tint.alpha(0.10 * a)),
                        (1.0, CLEAR),
                    ],
                ),
        );
    }
    out
}

/// Embers: slow motes rising out of `(x0, y0, x1, y1)`, each on its own drift and speed, each
/// wrapping exactly once round its band per cycle so the loop closes.
pub fn embers(
    f: &Frame,
    salt: u64,
    count: usize,
    band: [f32; 4],
    tint: rune::Color,
    strength: f32,
) -> Vec<El> {
    let mut rng = f.rng(salt);
    let mut motes = Vec::new();
    let [x0, y0, x1, y1] = band;
    let (top, span) = (f.y(y0), f.y(y1 - y0));
    for _ in 0..count {
        let start = rng.unit();
        // Whole laps per cycle: the motes are where they started at p = 0 and p = 1.
        let laps = 1.0 + (rng.unit() * 2.0).floor();
        let drift = rng.range(-1.0, 1.0);
        let cycles = 1.0 + (rng.unit() * 2.0).floor();
        let brightness = rng.range(0.35, 1.0);
        let size = f.u(rng.range(0.0016, 0.0042));
        let y =
            top + (span + f.u(0.06)) - ((start + f.p * laps).rem_euclid(1.0)) * (span + f.u(0.06));
        let x = f.x(x0 + (x1 - x0) * rng.unit())
            + f.u(0.045) * drift * wave(f.p, cycles, start * std::f32::consts::TAU);

        // A mote is brightest mid-band and fades out at both ends of its band, which hides the wrap.
        let across = ((y - top) / span).clamp(0.0, 1.0);
        let fade = (across * std::f32::consts::PI).sin().max(0.0);

        motes.push(halo(
            x,
            y,
            size * 3.6,
            size * 3.6,
            tint,
            strength * brightness * fade * 0.55,
        ));
        motes.push(
            el().absolute(x - size, y - size)
                .size(size * 2.0, size * 2.0)
                .radius(size)
                .fill(palette::ember_pale().alpha(strength * brightness * fade * 0.9)),
        );
    }
    motes
}

/// Appends a flat list of layers to `root`.
pub fn add(mut root: El, layers: Vec<El>) -> El {
    for layer in layers {
        root = root.child(layer);
    }
    root
}
