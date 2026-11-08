//! The cinematic layers rune-fx draws for the scenes: the aurora curtains behind everything and
//! the film grain over the whole frame.
//!
//! A scene marks a rect with [`effect`](rune::El::effect) and a tag from this module; the
//! compositor turns each tag into one `FxLayer` at the loop's clock. Every rune-fx effect is a
//! pure function of `(pixel, params, time, seed)`, so passing the phase times a whole turn of the
//! aurora keeps the loop seamless, and the grain is static (`fps: 0.0`) so every frame of a loop
//! carries the same texture.
//!
//! Wire it up with `App::compositor(RunestoneFx::default())`; without a compositor the scenes
//! still draw (the slots simply paint nothing).

use rune::compose::{Compositor, Frame};
use rune::render::{EffectPhase, EffectSite, Instance, KIND_EFFECT};
use rune_fx::{
    rgb, AuroraParams, Fx, FxFrame, FxLayer, FxOptions, FxRenderer, FxSlot, GrainParams, Shape,
};

use crate::palette;

/// The broad aurora curtain across the upper third, teal into blue.
pub const AURORA: u32 = 100;
/// A second, thinner curtain drifting against it, violet into teal.
pub const AURORA_DRIFT: u32 = 101;
/// Film grain over the whole frame.
pub const GRAIN: u32 = 110;

#[derive(Default)]
pub struct RunestoneFx {
    fx: Option<FxRenderer>,
}

/// The effects' clock: the loop's phase, in radians, so one cycle is exactly one turn.
fn clock(t: f64) -> f32 {
    crate::phase(t) * std::f32::consts::TAU
}

impl<S: 'static> Compositor<S> for RunestoneFx {
    fn render(&mut self, f: &mut Frame<'_, S>) {
        let (device, queue, size) = (f.device, f.queue, f.size);
        let now = clock(f.rt.t);
        let marks = &f.rt.marks;

        let mut slots: Vec<(u32, FxLayer, FxSlot)> = Vec::new();
        for (n, instance) in f.rt.instances.iter().enumerate() {
            if instance.params[1] != KIND_EFFECT {
                continue;
            }
            let mark = marks.get(n).copied().unwrap_or(0);
            let Some(layer) = layer_for(mark, instance, now) else {
                continue;
            };
            slots.push((
                n as u32,
                layer,
                FxSlot::default()
                    .scale(instance.params[3])
                    .clip(instance.clip, instance.fade),
            ));
        }

        let layers: Vec<FxLayer> = slots.iter().map(|(_, l, _)| *l).collect();
        let placements: Vec<FxSlot> = slots.iter().map(|(_, _, s)| *s).collect();
        let fx = self
            .fx
            .get_or_insert_with(|| FxRenderer::new(device, f.format, FxOptions { dither: true }));
        fx.set_srgb_blend(f.srgb_blend());
        if !layers.is_empty() {
            fx.prepare_with_slots(
                device,
                queue,
                &FxFrame {
                    viewport: [size.0, size.1],
                    scale: f.rt.env.scale,
                    layers: &layers,
                    backdrop: None,
                    sources: &[],
                },
                &placements,
            );
        }

        let clear = f.rt.clear_color();
        f.ui.prepare(device, queue, &mut f.rt.text.atlas, &f.rt.instances, size);
        f.ui.draw_with_backdrop(
            f.encoder,
            f.target,
            clear,
            &mut |phase, site: &EffectSite<'_>| {
                // Slots sit in the frame's own pass: an isolated layer (opacity, blur) would need
                // another format registered, and the scenes never put an effect slot inside one.
                if let EffectPhase::Draw(pass) = phase {
                    if site.in_layer {
                        return;
                    }
                    if let Some(k) = slots.iter().position(|(n, _, _)| *n == site.index) {
                        fx.draw_layer_in(pass, site.format, site.origin, k);
                    }
                }
            },
        );
    }
}

/// The layer one effect slot asks for, in the slot's own points. `element_data[0]` scales an
/// aurora's intensity, so one scene can light its sky harder than another.
fn layer_for(mark: u32, instance: &Instance, now: f32) -> Option<FxLayer> {
    let k = instance.params[3].max(0.001);
    let [x, y, w, h] = instance.rect.map(|v| v / k);
    let radius = instance.radii.iter().fold(0.0f32, |a, v| a.max(v.abs())) / k;
    let body = Shape::rounded(x, y, w, h, radius);
    let strength = if instance.uv[0] > 0.0 {
        instance.uv[0]
    } else {
        1.0
    };

    let layer = match mark {
        GRAIN => Fx::grain(GrainParams {
            amount: 0.12,
            fps: 0.0,
            feather: 0.0,
        })
        .on(body)
        .at(0.0)
        .seed(mark),
        AURORA => Fx::aurora(AuroraParams {
            a: rgb(palette::AURORA_TEAL_HEX),
            b: rgb(palette::AURORA_BLUE_HEX),
            c: rgb(palette::AURORA_VIOLET_HEX),
            scale: (h * 0.55).max(60.0),
            // One whole turn of the aurora per loop.
            speed: std::f32::consts::TAU / crate::PERIOD as f32,
            intensity: 0.085 * strength,
            feather: (h * 0.60).max(30.0),
        })
        .on(body)
        .at(now)
        .seed(mark),
        AURORA_DRIFT => Fx::aurora(AuroraParams {
            a: rgb(palette::AURORA_VIOLET_HEX),
            b: rgb(palette::AURORA_TEAL_HEX),
            c: rgb(palette::AURORA_BLUE_HEX),
            scale: (h * 0.34).max(40.0),
            speed: -(std::f32::consts::TAU / crate::PERIOD as f32),
            intensity: 0.05 * strength,
            feather: (h * 0.50).max(20.0),
        })
        .on(body)
        .at(now)
        .seed(mark),
        _ => return None,
    };
    Some(layer)
}
