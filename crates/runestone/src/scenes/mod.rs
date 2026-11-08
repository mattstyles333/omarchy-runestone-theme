//! The three scenes, and the shared layers they are built from: the obsidian base, the aurora
//! behind it, the vignette in front of it, and the embers drifting through it.
//!
//! Every scene is [`Scene::view`]`(t, size, seed)`, a pure function of the cycle's phase, the
//! size it is drawn at and a seed: the same inputs always paint the same picture.

mod backdrop;
pub mod lock;
mod monolith;
mod ring;
mod script;

pub use backdrop::{aurora, backdrop, embers, halo, ribbon, vignette};

use crate::anim::Rng;
use crate::phase;
use rune::{Cx, El};

/// One scene of the wallpaper, named the way `--scene` names it.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Scene {
    /// A carved standing stone, its runes igniting one after another.
    Monolith,
    /// The 24 runes on a slowly turning ring, seen in perspective.
    Ring,
    /// One large rune morphing into the next through the futhark.
    Script,
}

impl Scene {
    /// Every scene, in the order `--scene` accepts them.
    pub const ALL: [(&'static str, Scene); 3] = [
        ("monolith", Scene::Monolith),
        ("ring", Scene::Ring),
        ("script", Scene::Script),
    ];

    pub fn from_name(name: &str) -> Option<Scene> {
        Self::ALL.iter().find(|(n, _)| *n == name).map(|(_, s)| *s)
    }

    pub fn name(self) -> &'static str {
        match self {
            Scene::Monolith => "monolith",
            Scene::Ring => "ring",
            Scene::Script => "script",
        }
    }

    /// The scene at `t` seconds into the cycle, drawn at `size` points.
    pub fn view(self, t: f64, size: (f32, f32), seed: u64) -> El {
        let f = Frame {
            w: size.0.max(1.0),
            h: size.1.max(1.0),
            p: phase(t),
            seed,
        };
        match self {
            Scene::Monolith => monolith::view(&f),
            Scene::Ring => ring::view(&f),
            Scene::Script => script::view(&f),
        }
    }

    /// The same scene, from a rune `AppSpec` view: the window's own size if it has been laid out
    /// (a preview window the user resized), else the size the app was built for.
    pub fn element<S: 'static>(self, t: f64, size: (f32, f32), seed: u64, cx: &mut Cx<S>) -> El {
        let size = cx.layout_size(ROOT).map_or(size, |[w, h]| (w, h));
        self.view(t, size, seed)
    }
}

/// The root element's key, and how a view reads the size it is actually drawn at.
pub const ROOT: &str = "root";

/// What a scene is drawn into: a size in points, the cycle's phase, and the seed that fixes the
/// embers and any scatter.
pub struct Frame {
    pub w: f32,
    pub h: f32,
    /// The cycle's phase, `0.0..1.0`.
    pub p: f32,
    pub seed: u64,
}

impl Frame {
    /// `frac` of the width.
    pub fn x(&self, frac: f32) -> f32 {
        frac * self.w
    }

    /// `frac` of the height.
    pub fn y(&self, frac: f32) -> f32 {
        frac * self.h
    }

    /// A length as a fraction of the height: the measure every scene is drawn in, so 16:9 stills
    /// and 16:9 loops are the same picture at different resolutions.
    pub fn u(&self, frac: f32) -> f32 {
        frac * self.h
    }

    /// A seeded stream, salted so two uses in one scene never share a sequence.
    pub fn rng(&self, salt: u64) -> Rng {
        Rng::new(self.seed.wrapping_add(salt.wrapping_mul(0x9E37_79B9)))
    }

    /// The root of every scene: the full frame, filled with the obsidian base.
    pub fn root(&self) -> El {
        rune::el()
            .key(ROOT)
            .w_full()
            .h_full()
            .fill(crate::palette::obsidian())
    }
}
