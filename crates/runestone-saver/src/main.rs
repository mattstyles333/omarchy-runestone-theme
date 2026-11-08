//! `runestone-saver`: Runestone as Omarchy's screensaver.
//!
//! A full-screen layer-shell overlay on every display: an obsidian field, the `script` rune
//! breathing in the middle of it, and — when Omarchy has branding text — a few dim lines of it
//! beneath. Any key, click, scroll or pointer movement ends it, as do `SIGTERM`, `SIGINT`,
//! `SIGHUP` and `SIGQUIT`.
//!
//! It is a drop-in for Omarchy's own screensaver. `install.sh` puts the binary in
//! `~/.local/bin`, writes a shim named `omarchy-launch-screensaver` beside it that runs this
//! binary, and puts `~/.local/bin` ahead of `$OMARCHY_PATH/bin` in the session PATH from
//! `~/.config/hypr/hyprland.lua`, so the idle service and the menus start Runestone without
//! touching anything Omarchy owns.
//!
//! ```sh
//! runestone-saver                                   # the screensaver
//! runestone-saver force                             # even with `omarchy toggle screensaver` off
//! runestone-saver --input notes.txt                 # draw this text instead of the branding
//! runestone-saver --headless --snap rune.png --at 3 --size 1920x1080 --scale 1
//! runestone-saver --headless --frames 90 --fps 30 --out /tmp/frames --size 1280x720
//! ```

use std::fs::{File, OpenOptions};
use std::io::Write;
use std::path::{Path, PathBuf};
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::OnceLock;
use std::time::Duration;

use rune::{
    el, text, Align, App, AppSpec, ConfiguredOverlay, Cx, Display, DisplayTarget, El, KeyEvent,
    KeyPolicy, OverlayInput, OverlayLevel, OverlayOptions, OverlayPlacement, OverlaySpec,
    RunOptions, TextStyle, Tx, WindowSpec,
};
use runestone::{palette, Scene};

/// The seed every render uses, as `runestone-render` does: the embers land in the same place.
const SEED: u64 = 0x5255_4e45; // "RUNE"

/// The size used before a layout lands, and the size of the headless window unless `--size`.
const PREVIEW: (f32, f32) = (1920.0, 1080.0);

/// The look's one clock is 16 s; the saver asks for a frame every 33 ms, about 30 fps, because a
/// screensaver has no call to burn a GPU at monitor rate.
const FRAME: Duration = Duration::from_millis(33);

/// Input before this many seconds is the hotkey that started the screensaver, not the user.
const ARM_AFTER: f64 = 0.5;

/// The branding file is a short mark, not a paragraph: the first few non-empty lines only.
const MAX_LINES: usize = 3;

/// The screensaver tree's own root; the scene inside it keeps its own `root` key.
const ROOT: &str = "saver";

/// Omarchy's off switch, as `omarchy toggle screensaver` writes it.
const TOGGLE: &str = ".local/state/omarchy/toggles/screensaver-off";

/// Omarchy's branding file for the screensaver.
const BRANDING: &str = ".config/omarchy/branding/screensaver.txt";

/// One instance at a time, keyed in `$XDG_RUNTIME_DIR`.
const LOCK: &str = "runestone-saver.lock";

/// The layer-shell namespace, so Omarchy's tools can find the surfaces.
const NAMESPACE: &str = "org.runestone.screensaver";

/// Both slots are the one bundled face: it carries the runes and just enough Latin for the
/// branding text, so nothing depends on the machine's fonts.
const FAMILIES: (&str, &str) = ("Noto Sans Runic", "Noto Sans Runic");

/// What the screensaver draws, fixed at launch: the size it was asked for and the branding
/// lines. Read by every overlay's view and by `init`.
struct Setup {
    size: (f32, f32),
    branding: Vec<String>,
}

static SETUP: OnceLock<Setup> = OnceLock::new();

/// A signal handler thread is not involved: the C handler sets this, and the next poll quits.
static TERMINATE: AtomicBool = AtomicBool::new(false);

fn setup() -> &'static Setup {
    SETUP.get_or_init(|| Setup {
        size: PREVIEW,
        branding: Vec::new(),
    })
}

fn main() {
    let argv: Vec<String> = std::env::args().skip(1).collect();
    let args = match Args::parse(argv) {
        Ok(Some(args)) => args,
        Ok(None) => {
            print!("{HELP}");
            return;
        }
        Err(error) => {
            eprintln!("runestone-saver: {error}");
            std::process::exit(2);
        }
    };
    if args.headless && !rune::is_headless() {
        eprintln!("runestone-saver: --headless needs --snap FILE or --frames N --out DIR");
        std::process::exit(2);
    }
    let branding = match load_branding(args.input.as_deref()) {
        Ok(lines) => lines,
        Err(error) => {
            eprintln!("runestone-saver: {error}");
            std::process::exit(1);
        }
    };
    if let Some(home) = std::env::var_os("HOME").map(PathBuf::from) {
        if refused_by_toggle(&home, args.force) {
            eprintln!(
                "runestone-saver: the screensaver is off (`omarchy toggle screensaver`); \
                 `runestone-saver force` starts it anyway"
            );
            return;
        }
    }
    // Held for the whole run: dropping the file releases the lock.
    let Some(_lock) = instance_lock(&runtime_dir()) else {
        eprintln!("runestone-saver: already running");
        return;
    };
    catch_termination();
    let _ = SETUP.set(Setup {
        size: args.size.unwrap_or(PREVIEW),
        branding,
    });
    // Clean errors, not a panic: nothing to draw on is Omarchy's session to notice, and the
    // process still exits like Omarchy's own screensaver does.
    if let Err(error) =
        rune::try_run_windows_with_args(RunOptions { accessory: true }, args.rune, launch)
    {
        eprintln!("runestone-saver: {error}");
        std::process::exit(1);
    }
}

/// Omarchy's own arguments first, then rune's, which it hands on unchanged: `--snap`, `--at`,
/// `--size`, `--scale`, `--frames`, `--fps`, `--out`, `--window` and the rest.
#[derive(Default)]
struct Args {
    force: bool,
    input: Option<PathBuf>,
    headless: bool,
    size: Option<(f32, f32)>,
    rune: Vec<String>,
}

impl Args {
    /// `None` asks for help. Unknown arguments are rune's and are passed through.
    fn parse(argv: Vec<String>) -> Result<Option<Args>, String> {
        let mut out = Args::default();
        let mut i = 0;
        while i < argv.len() {
            let arg = argv[i].clone();
            match arg.as_str() {
                "-h" | "--help" => return Ok(None),
                "force" | "--force" => out.force = true,
                "--headless" => out.headless = true,
                "--input" | "-i" | "--input-file" => {
                    i += 1;
                    let path = argv.get(i).ok_or("--input needs a path")?;
                    out.input = Some(PathBuf::from(path));
                }
                "--size" => {
                    i += 1;
                    let value = argv.get(i).ok_or("--size needs WIDTHxHEIGHT")?;
                    out.size = Some(parse_size(value)?);
                    out.rune.push("--size".to_owned());
                    out.rune.push(value.clone());
                }
                other => {
                    if let Some(value) = other.strip_prefix("--input=") {
                        out.input = Some(PathBuf::from(value));
                    } else if let Some(value) = other.strip_prefix("--size=") {
                        out.size = Some(parse_size(value)?);
                        out.rune.push("--size".to_owned());
                        out.rune.push(value.to_owned());
                    } else {
                        out.rune.push(other.to_owned());
                    }
                }
            }
            i += 1;
        }
        Ok(Some(out))
    }
}

fn parse_size(value: &str) -> Result<(f32, f32), String> {
    let (w, h) = value
        .split_once(['x', 'X'])
        .ok_or_else(|| format!("--size is WIDTHxHEIGHT, not {value:?}"))?;
    let size = |part: &str| -> Result<f32, String> {
        part.parse::<f32>()
            .ok()
            .filter(|n| *n > 0.0)
            .ok_or_else(|| format!("--size is WIDTHxHEIGHT, not {value:?}"))
    };
    Ok((size(w)?, size(h)?))
}

/// The branding lines: `--input` when given (and it must read), else Omarchy's
/// `screensaver.txt`, else nothing at all. First non-empty lines, trimmed.
fn load_branding(input: Option<&Path>) -> Result<Vec<String>, String> {
    let text = match input {
        Some(path) => std::fs::read_to_string(path)
            .map_err(|error| format!("cannot read --input {}: {error}", path.display()))?,
        None => std::env::var_os("HOME")
            .map(|home| std::fs::read_to_string(PathBuf::from(home).join(BRANDING)))
            .and_then(Result::ok)
            .unwrap_or_default(),
    };
    Ok(first_lines(&text, MAX_LINES))
}

fn first_lines(text: &str, max: usize) -> Vec<String> {
    text.lines()
        .map(str::trim)
        .filter(|line| !line.is_empty())
        .take(max)
        .map(str::to_owned)
        .collect()
}

/// `omarchy toggle screensaver` writes `screensaver-off`, and `force` starts anyway.
fn refused_by_toggle(home: &Path, force: bool) -> bool {
    !force && home.join(TOGGLE).exists()
}

fn runtime_dir() -> PathBuf {
    std::env::var_os("XDG_RUNTIME_DIR")
        .map(PathBuf::from)
        .unwrap_or_else(std::env::temp_dir)
}

/// The lock held for the run, or `None` when another instance has it. Where the lock can't be
/// taken at all (no writable runtime dir, a filesystem without locks) the saver runs: a lonely
/// screensaver is better than none.
fn instance_lock(dir: &Path) -> Option<File> {
    let unlocked = || OpenOptions::new().read(true).open("/dev/null").ok();
    let Ok(file) = OpenOptions::new()
        .read(true)
        .write(true)
        .create(true)
        .truncate(false)
        .open(dir.join(LOCK))
    else {
        return unlocked();
    };
    match file.try_lock() {
        Ok(()) => {}
        Err(std::fs::TryLockError::WouldBlock) => return None,
        Err(std::fs::TryLockError::Error(_)) => return unlocked(),
    }
    let _ = file.set_len(0);
    let _ = writeln!(&file, "{}", std::process::id());
    Some(file)
}

extern "C" fn on_terminate(_: libc::c_int) {
    TERMINATE.store(true, Ordering::Relaxed);
}

/// SIGTERM, SIGINT, SIGHUP and SIGQUIT quit like any key: the handler only sets a flag, and the
/// next poll (within a frame) asks the app to quit.
fn catch_termination() {
    for signal in [libc::SIGTERM, libc::SIGINT, libc::SIGHUP, libc::SIGQUIT] {
        // SAFETY: the handler stores one atomic, which signal-safety allows.
        unsafe {
            libc::signal(signal, on_terminate as *const () as libc::sighandler_t);
        }
    }
}

/// What each overlay keeps: how far into the loop it is, and whether input is the user's yet.
struct State {
    t: f64,
    armed: bool,
}

impl State {
    fn new() -> State {
        State {
            t: 0.0,
            armed: false,
        }
    }

    /// End the screensaver, once input can be the user's rather than the triggering hotkey.
    fn dismiss(&self, tx: &mut Tx) {
        if self.armed {
            tx.quit();
        }
    }
}

fn poll(s: &mut State, t: f64, tx: &mut Tx) -> bool {
    if TERMINATE.load(Ordering::Relaxed) {
        tx.quit();
    }
    s.t = t;
    s.armed |= t >= ARM_AFTER;
    // ~30 fps: a screensaver has no call to redraw at the monitor's rate.
    tx.wake_after(FRAME);
    true
}

fn on_key(_: &mut State, _: &KeyEvent, tx: &mut Tx) -> bool {
    tx.quit();
    true
}

fn view(s: &State, cx: &mut Cx<State>) -> El {
    let size = cx.layout_size(ROOT).map_or(setup().size, |[w, h]| (w, h));
    let mut root = el()
        .key(ROOT)
        .w_full()
        .h_full()
        .hit()
        .fill(palette::obsidian())
        .on_press_tx(|s: &mut State, tx| s.dismiss(tx))
        .on_pointer_move_local(|s: &mut State, _p, tx| s.dismiss(tx))
        .on_wheel(|s: &mut State, _w, tx| {
            s.dismiss(tx);
            true
        })
        .child(Scene::Script.view(s.t, size, SEED));
    if !setup().branding.is_empty() {
        root = root.child(branding(size));
    }
    root
}

/// The branding text: small, dim, centred under the rune.
fn branding(size: (f32, f32)) -> El {
    let lines = &setup().branding;
    let px = (size.1 * 0.018).clamp(10.0, 40.0);
    let style = TextStyle {
        line_height: px * 1.55,
        ..TextStyle::new(px, 400.0)
    };
    let width = size.0 * 0.72;
    let height = px * 1.55 * lines.len() as f32;
    el().absolute((size.0 - width) * 0.5, size.1 * 0.72)
        .size(width, height)
        .center()
        .child(
            text(lines.join("\n"))
                .font(style)
                .align(Align::Center)
                .color(palette::decode(palette::FOREGROUND_HEX).alpha(0.42)),
        )
}

fn spec() -> AppSpec<State> {
    AppSpec {
        poll: Some(poll),
        on_key: Some(on_key),
        fonts: &[include_bytes!(
            "../../../assets/fonts/NotoSansRunic-Regular.ttf"
        )],
        families: FAMILIES,
        ..AppSpec::new("Runestone", PREVIEW, State::new, view)
    }
}

fn overlay(display: DisplayTarget) -> ConfiguredOverlay<State> {
    OverlaySpec::new(NAMESPACE, PREVIEW, view)
        .options(OverlayOptions {
            display,
            placement: Some(OverlayPlacement::FullScreen),
            level: OverlayLevel::AboveStatus,
            input: OverlayInput::All,
            keys: KeyPolicy::OnDemand,
            ..Default::default()
        })
        .on_key(on_key)
        .poll(poll)
}

/// The windows to run: headless, one ordinary (offscreen) window drawing the same view; on a
/// desktop, one full-screen overlay per display and no window at all.
fn launch(displays: &[Display]) -> Vec<WindowSpec<State>> {
    if rune::is_headless() {
        return vec![WindowSpec::from_app("main", App::new(spec()))];
    }
    let mut app = App::new(spec()).windowless();
    let targets: Vec<DisplayTarget> = if displays.is_empty() {
        vec![DisplayTarget::Primary]
    } else {
        displays
            .iter()
            .map(|display| {
                display
                    .id
                    .clone()
                    .map_or(DisplayTarget::Primary, DisplayTarget::Id)
            })
            .collect()
    };
    let overlays: Vec<_> = targets
        .into_iter()
        .map(|target| app.overlay(overlay(target), State::new()))
        .collect();
    for handle in &overlays {
        handle.show();
    }
    // One key owner, even with several full-display surfaces: any key ends the screensaver.
    if let Some(first) = overlays.first() {
        first.request_keys();
    }
    vec![WindowSpec::from_app("saver", app)]
}

const HELP: &str = "\
usage: runestone-saver [force] [--input PATH] [--headless] [rune options]

  force               start even while the screensaver is off
                      (`omarchy toggle screensaver`) or another instance runs
  --input PATH        draw this file instead of ~/.config/omarchy/branding/screensaver.txt
  --headless          render without a display; needs --snap or --frames
  --snap FILE         write one PNG and exit
  --at SECONDS        the moment `--snap` draws (default 0)
  --size WxH          the frame's size in points (default 1920x1080)
  --scale N           pixels per point (default 2 on a desktop, 1 headless)
  --frames N --out DIR --fps N
                      write N frames into DIR and exit
  -h, --help          this text

A key, click, scroll or pointer movement ends it, as do SIGTERM, SIGINT, SIGHUP and SIGQUIT.
";

#[cfg(test)]
mod tests {
    use super::*;

    fn scratch(name: &str) -> PathBuf {
        let dir = std::env::temp_dir().join(format!(
            "runestone-saver-test-{}-{name}",
            std::process::id()
        ));
        let _ = std::fs::remove_dir_all(&dir);
        std::fs::create_dir_all(&dir).unwrap();
        dir
    }

    #[test]
    fn the_toggle_refuses_unless_forced() {
        let home = scratch("toggle");
        let toggle = home.join(TOGGLE);
        assert!(!refused_by_toggle(&home, false));
        std::fs::create_dir_all(toggle.parent().unwrap()).unwrap();
        std::fs::write(&toggle, b"").unwrap();
        assert!(refused_by_toggle(&home, false));
        assert!(!refused_by_toggle(&home, true));
        let _ = std::fs::remove_dir_all(&home);
    }

    #[test]
    fn one_instance_at_a_time() {
        let dir = scratch("lock");
        let first = instance_lock(&dir).expect("first instance takes the lock");
        assert!(instance_lock(&dir).is_none(), "second instance is refused");
        drop(first);
        assert!(instance_lock(&dir).is_some(), "released, it starts again");
        let _ = std::fs::remove_dir_all(&dir);
    }

    #[test]
    fn branding_is_the_first_non_empty_lines() {
        let text = "\n  Runestone  \n\n\nmatt styles\nfourth\nfifth\n";
        assert_eq!(
            first_lines(text, MAX_LINES),
            vec!["Runestone", "matt styles", "fourth"]
        );
        assert!(first_lines(" \n\t\n", MAX_LINES).is_empty());
    }

    #[test]
    fn size_is_width_by_height() {
        assert_eq!(parse_size("1920x1080"), Ok((1920.0, 1080.0)));
        assert!(parse_size("1920").is_err());
        assert!(parse_size("0x1080").is_err());
    }
}
