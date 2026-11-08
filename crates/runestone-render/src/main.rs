//! `runestone-render`: the Runestone scenes as one rune app.
//!
//! It is an ordinary Rune app, so it opens a preview window on a real desktop and takes Rune's
//! tooling flags for headless stills and loops:
//!
//! ```sh
//! runestone-render                                   # preview window, monolith
//! runestone-render --scene ring                      # preview window, ring
//! runestone-render --scene script --snap out.png --at 3.5 --size 3840x2160 --scale 1
//! runestone-render --scene monolith --frames 480 --fps 30 --size 2560x1440 --scale 1 --out frames/
//! runestone-render --colors-toml > colors.toml
//! ```
//!
//! `scripts/render.sh` runs all of that for you and encodes the result.

use rune::{App, AppSpec, Cx, El, Key, KeyEvent, Tx, WindowOptions};
use runestone::{scenes, Scene};

/// The seed every render uses: the embers land in the same place on every machine.
const SEED: u64 = 0x5255_4e45; // "RUNE"

/// The preview window's size on a desktop; headless runs pass `--size` instead.
const PREVIEW: (f32, f32) = (1280.0, 720.0);

/// The size the run was asked for, read once in `main` and used until the first layout lands.
static SIZE: std::sync::OnceLock<(f32, f32)> = std::sync::OnceLock::new();

/// What the app draws: one of the three wallpapers, or the lock screen's mark.
#[derive(Clone, Copy, PartialEq, Eq)]
enum Shot {
    Wallpaper(Scene),
    Unlock,
}

struct State {
    shot: Shot,
    /// Seconds into the loop; the poll supplies it, virtual when headless.
    t: f64,
    /// The size the app was asked for, so the very first frame (which has no layout yet) is
    /// already drawn at it: `--size` when headless, the preview window's own otherwise.
    size: (f32, f32),
}

fn view(s: &State, cx: &mut Cx<State>) -> El {
    let size = cx.layout_size(scenes::ROOT).map_or(s.size, |[w, h]| (w, h));
    match s.shot {
        Shot::Wallpaper(scene) => scene.view(s.t, size, SEED),
        Shot::Unlock => scenes::lock::view(&scenes::Frame {
            w: size.0,
            h: size.1,
            p: 0.0,
            seed: SEED,
        }),
    }
}

fn init() -> State {
    State {
        shot: Shot::Wallpaper(Scene::Monolith),
        t: 0.0,
        size: SIZE.get().copied().unwrap_or(PREVIEW),
    }
}

fn poll(s: &mut State, t: f64, _: &mut Tx) -> bool {
    s.t = t;
    true
}

fn on_key(_: &mut State, key: &KeyEvent, tx: &mut Tx) -> bool {
    match key.key {
        Key::Escape => {
            tx.quit();
            true
        }
        _ => false,
    }
}

fn main() {
    if std::env::args().any(|a| a == "--colors-toml") {
        print!("{}", runestone::palette::colors_toml());
        return;
    }
    // Rune's own flags, read once here so the first frame can be drawn at the size asked for.
    let args = rune::tooling::Args::parse();
    let size = args.size.unwrap_or(PREVIEW);
    let _ = SIZE.set(size);
    // `--scene unlock` is the one asset that has to stay transparent.
    let unlock = args.scene.as_deref() == Some("unlock");

    let spec = AppSpec {
        scenes: &[
            ("monolith", |s: &mut State| {
                s.shot = Shot::Wallpaper(Scene::Monolith)
            }),
            ("ring", |s: &mut State| {
                s.shot = Shot::Wallpaper(Scene::Ring)
            }),
            ("script", |s: &mut State| {
                s.shot = Shot::Wallpaper(Scene::Script)
            }),
            ("unlock", |s: &mut State| s.shot = Shot::Unlock),
        ],
        fonts: &[include_bytes!(
            "../../../assets/fonts/NotoSansRunic-Regular.ttf"
        )],
        poll: Some(poll),
        on_key: Some(on_key),
        ..AppSpec::new("Runestone", PREVIEW, init, view)
    };
    App::new(spec)
        .compositor(runestone::RunestoneFx::default())
        .window_options(WindowOptions {
            transparent: unlock,
            min_size: Some((320.0, 180.0)),
            ..Default::default()
        })
        .run();
}
