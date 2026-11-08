//! The small maths every scene animates with: seamless wave and pulse helpers over one cycle's
//! phase, and a seeded random stream for the embers and the grain of the layout.
//!
//! Everything here is periodic in `phase` (`0.0..1.0`), so a scene built on it renders the same
//! picture at `t = 0` and `t = PERIOD`.

use std::f32::consts::TAU;

/// Smoothstep: an S-curve from 0 at `x <= 0` to 1 at `x >= 1`.
pub fn smooth(x: f32) -> f32 {
    let x = x.clamp(0.0, 1.0);
    x * x * (3.0 - 2.0 * x)
}

/// 0 → 1 across `[start, start + len]` of the cycle, then 1.
pub fn ramp(p: f32, start: f32, len: f32) -> f32 {
    if len <= 0.0 {
        return 1.0;
    }
    smooth((p - start) / len)
}

/// 1 → 0 across `[start, start + len]` of the cycle.
pub fn fall(p: f32, start: f32, len: f32) -> f32 {
    1.0 - ramp(p, start, len)
}

/// A single breath: 0 before `start`, up and back to 0 by `start + len`, eased at both ends.
pub fn bump(p: f32, start: f32, len: f32) -> f32 {
    let x = ((p - start) / len).clamp(0.0, 1.0);
    if x <= 0.0 || x >= 1.0 {
        return 0.0;
    }
    (x * TAU / 2.0).sin().powi(2)
}

/// A wave in `-1.0..1.0` that completes `cycles` whole oscillations per cycle of `p`.
pub fn wave(p: f32, cycles: f32, offset: f32) -> f32 {
    (p * cycles * TAU + offset).sin()
}

/// A wave in `0.0..1.0`, the convenient one for driving opacity.
pub fn sway(p: f32, cycles: f32, offset: f32) -> f32 {
    0.5 + 0.5 * wave(p, cycles, offset)
}

/// SplitMix64: a tiny deterministic stream, so a seed picks the same embers on every render.
pub struct Rng(u64);

impl Rng {
    pub fn new(seed: u64) -> Rng {
        Rng(seed ^ 0x9E37_79B9_7F4A_7C15)
    }

    pub fn next_u64(&mut self) -> u64 {
        self.0 = self.0.wrapping_add(0x9E37_79B9_7F4A_7C15);
        let mut z = self.0;
        z = (z ^ (z >> 30)).wrapping_mul(0xBF58_476D_1CE4_E5B9);
        z = (z ^ (z >> 27)).wrapping_mul(0x94D0_49BB_1331_11EB);
        z ^ (z >> 31)
    }

    /// A number in `0.0..1.0`.
    pub fn unit(&mut self) -> f32 {
        (self.next_u64() >> 40) as f32 / 16_777_216.0
    }

    /// A number in `lo..hi`.
    pub fn range(&mut self, lo: f32, hi: f32) -> f32 {
        lo + (hi - lo) * self.unit()
    }
}
