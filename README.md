# Runestone

An Omarchy theme called **Runestone**: obsidian black, Elder Futhark runes glowing in ember and
aurora light, embers drifting through it, and nothing in a hurry.

Everything here is drawn by [rune](https://github.com/anomalyco/rune), a Rust GPU UI framework.
The wallpapers are not photographs and not a render farm's guess: they are three scenes written
as Rust views, each a pure function of the loop's phase, so a 16 second cycle ends exactly where
it began. The stills, the live loops and the previews are all rendered from that one source.

## The three scenes

| | |
|---|---|
| **monolith** | A dark carved standing stone, centre-left, its runes igniting one after another from the crown down and cooling back to coal, embers rising past it, aurora faint behind. |
| **ring** | The 24 Elder Futhark runes on a slowly turning ring seen in perspective, the rune at the front of the ellipse the brightest, aurora ribbons above. |
| **script** | One large rune, morphing into the next through the futhark on an eased breath, an ember trail drifting off it. The most minimal of the three. |

Each scene ships as a still (`backgrounds/<scene>.webp`, 3840×2160) and as a live wallpaper
(`backgrounds/<scene>.mp4`, 2560×1440, 30 fps, 16 s, seamless). Omarchy's OWE wallpaper engine
plays a video dropped beside the stills, so a theme's `backgrounds/` can carry both.

## Install

```sh
omarchy theme install https://github.com/mattstyles333/omarchy-runestone-theme.git
```

Then pick *Runestone* in the theme menu, and choose a background in Appearance → Background:
the stills for calm, the loops for an ember-lit desktop.

## Screensaver

`runestone-saver` is the theme's screensaver: a full-screen layer-shell overlay on every
display, an obsidian field with the `script` rune breathing in the middle of it, and Omarchy's
branding text — `~/.config/omarchy/branding/screensaver.txt`, or `--input FILE` — small and dim
beneath. It draws at about 30 fps, not at the monitor's rate.

Any key, click, scroll or pointer movement ends it, as do `SIGTERM`, `SIGINT`, `SIGHUP` and
`SIGQUIT`. It runs one instance at a time, and stays away while `omarchy toggle screensaver`
leaves the `screensaver-off` flag set; `runestone-saver force` starts it anyway.

```sh
./install.sh          # build, install to ~/.local/bin, and hook Omarchy
./install.sh --dry-run
./uninstall.sh        # put Omarchy's own screensaver back
```

`install.sh` needs Rust, and on Arch:

```sh
pacman -S --needed rust libxkbcommon wayland vulkan-icd-loader
```

### How it hooks in, without editing Omarchy

Nothing Omarchy owns is touched, and `omarchy update` does not undo it.

- Omarchy starts its screensaver by running `omarchy-launch-screensaver`: from the idle service
  (`shell/plugins/services/idle/Service.qml`) and from the menus. That script only
  `exec`s `/usr/lib/omarchy-screensaver/omarchy-screensaver`
  (`bin/omarchy-launch-screensaver`).
- Omarchy's `default/hypr/envs.lua` prepends `$OMARCHY_PATH/bin` to the session PATH, so a
  `~/.local/bin` script would lose to it.
- `install.sh` therefore writes a shim named `omarchy-launch-screensaver` into `~/.local/bin`
  (it runs `runestone-saver`, and falls back to Omarchy's binary if the saver is gone), and
  appends a marked block to `~/.config/hypr/hyprland.lua` — the user config Omarchy's own
  template labels *"personal overrides"*, loaded after the defaults — that puts `~/.local/bin`
  first on PATH. This is the override [omarchy-screensaver-panel](https://github.com/brianirish/omarchy-screensaver-panel)
  uses for the same reason, with `~/.local/bin` in place of `/usr/local/bin`.

`uninstall.sh` removes exactly those three things — the binary, the shim, the marked block —
and leaves everything else as it was.

### Options

```
runestone-saver [force] [--input PATH] [options]
```

Rune's tooling flags work too, so the screensaver can be rendered without a display:

```sh
cargo run --release -p runestone-saver -- --headless --snap /tmp/saver.png --at 3 --size 1920x1080 --scale 1
cargo run --release -p runestone-saver -- --headless --frames 90 --fps 30 --out /tmp/frames --size 1280x720
```

## What an Omarchy theme may contain

This repository ships the files `omarchy-theme-set` stages for a theme installed from a repo:

- `colors.toml` — the palette every app is generated from. **Generated**, from
  `crates/runestone/src/palette.rs`, so the desktop's chrome and the wallpapers cannot drift.
- `icons.theme` — the icon set to pair with it.
- `btop.theme` — btop's own colours.
- `backgrounds/` — three stills and three live loops.
- `preview.png` (2560×1440), `preview-unlock.png` (1920×1080) and `unlock.png` (800×188,
  transparent) — what the theme picker and the lock screen show.

There is no `shell.toml`: no theme Omarchy ships carries one, and the shell's colours come from
`colors.toml`. Hyprland, terminal and VSCode configs are deliberately absent — a theme installed
from a git repo has them dropped anyway, because they would run code.

## Re-rendering

```sh
scripts/render.sh            # colours, stills, loops, previews
scripts/render.sh --check    # the same, then measure that each loop closes on itself
scripts/render.sh --stills   # stills and previews only
```

It builds `runestone-render`, renders each scene headlessly with rune's own tooling flags
(`--snap`, `--frames`, `--size`, `--scale`), and encodes with ffmpeg (WebP q95 for stills, H.264
yuv420p CRF 17 for loops). Needs Rust, ffmpeg and a Vulkan driver; on a machine with no GPU,
Mesa's lavapipe will do (slowly).

Watch a scene without rendering anything:

```sh
cargo run --release -p runestone-render -- --scene ring      # a preview window, Esc to quit
cargo run --release -p runestone-render -- --scene script --snap /tmp/script.png --at 4 --size 1920x1080 --scale 1
```

## Layout

```
crates/runestone          the look: palette, the futhark, the three scenes as pure views
crates/runestone-render   one rune app: stills, loops, previews, or a window
crates/runestone-saver    the screensaver: the same scenes as an Omarchy drop-in, one overlay per display
assets/fonts              Noto Sans Runic (SIL OFL 1.1, licence alongside)
scripts/render.sh         regenerate every asset
install.sh uninstall.sh   put the screensaver in place, and take it away again
```

`crates/runestone/src/palette.rs` is the single source of truth for colour: the scenes read it,
and `--colors-toml` prints `colors.toml` from it. `PERIOD` (16 s) is the one clock the scenes run
on; change it and re-render to change the loop length everywhere.

## Licence

MIT, © 2026 Matt Styles. The bundled Noto Sans Runic is under the SIL Open Font License 1.1; see
`assets/fonts/OFL.txt`.
