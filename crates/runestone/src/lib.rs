//! The Runestone look, as rune scenes.
//!
//! Obsidian black, Elder Futhark runes glowing in ember and aurora light. Three scenes —
//! [`Scene::Monolith`], [`Scene::Ring`] and [`Scene::Script`] — each a pure function of
//! `(t mod PERIOD, size, seed)`, so a rendered loop's last frame joins its first.
//!
//! [`palette`] is the one source of truth for every colour, and it also prints the `colors.toml`
//! this repository ships as an Omarchy theme: run `runestone-render --colors-toml`.

pub mod anim;
pub mod fx;
pub mod glyphs;
pub mod palette;
pub mod scenes;

pub use fx::RunestoneFx;
pub use scenes::Scene;

/// One full cycle of every scene, in seconds.
pub const PERIOD: f64 = 16.0;

/// How far through a cycle `t` is, in `0.0..1.0`. Exactly `0.0` again one `PERIOD` later.
pub fn phase(t: f64) -> f32 {
    (t / PERIOD).rem_euclid(1.0) as f32
}
