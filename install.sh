#!/usr/bin/env bash
# Install the Runestone screensaver and point Omarchy's screensaver at it.
#
#   ./install.sh                 build, install to $HOME/.local/bin, hook Omarchy
#   ./install.sh --dry-run       print every change instead of making it
#   ./install.sh --binary PATH   install a prebuilt runestone-saver
#   ./install.sh --prefix DIR    install somewhere other than $HOME/.local/bin
#
# Nothing outside $HOME is touched and no Omarchy file is edited. The hook is a shim named
# `omarchy-launch-screensaver` in the install directory, plus a PATH line appended to
# ~/.config/hypr/hyprland.lua, which runs after Omarchy's defaults. uninstall.sh reverts it.
set -euo pipefail

dry_run=0
prefix=""
prebuilt=""

usage() {
  sed -n '2,11p' "${BASH_SOURCE[0]}" | sed 's/^# \{0,1\}//'
}

while (($#)); do
  case "$1" in
    --dry-run) dry_run=1; shift ;;
    --prefix|--binary)
      [[ -n ${2:-} ]] || { printf 'install.sh: %s needs a value\n' "$1" >&2; exit 2; }
      if [[ $1 == --prefix ]]; then prefix=$2; else prebuilt=$2; fi
      shift 2 ;;
    -h|--help) usage; exit 0 ;;
    *) printf 'install.sh: unknown argument %s\n' "$1" >&2; exit 2 ;;
  esac
done

note() { printf '%s\n' "$*"; }

home="${HOME:?HOME is not set}"
here="$(cd -- "$(dirname -- "${BASH_SOURCE[0]}")" && pwd)"
bin_dir="${prefix:-$home/.local/bin}"
saver="$bin_dir/runestone-saver"
shim="$bin_dir/omarchy-launch-screensaver"
hyprland="$home/.config/hypr/hyprland.lua"
start='-- >>> runestone-saver >>>'
end='-- <<< runestone-saver <<<'

shim_text() {
  cat <<EOF
#!/usr/bin/env bash
# Runestone's screensaver, installed by omarchy-runestone-theme's install.sh.
# Omarchy's own omarchy-launch-screensaver is untouched: this shim is earlier on PATH.
# Remove it with uninstall.sh.
if [[ -x "$saver" ]]; then
  exec "$saver" "\$@"
fi
exec /usr/lib/omarchy-screensaver/omarchy-screensaver "\$@"
EOF
}

lua_block() {
  printf '%s\n' "$start"
  cat <<EOF
-- Runestone screensaver. Omarchy's default/hypr/envs.lua prepends \$OMARCHY_PATH/bin to PATH;
-- this runs after it and puts the install directory first, so \`omarchy-launch-screensaver\`
-- (the command Omarchy's idle service and menus run) resolves to Runestone's shim.
-- uninstall.sh removes these lines.
local runestone_bin = "$bin_dir"
local runestone_path = {}
for entry in (os.getenv("PATH") or "/usr/bin"):gmatch("[^:]+") do
  if entry ~= runestone_bin then
    table.insert(runestone_path, entry)
  end
end
table.insert(runestone_path, 1, runestone_bin)
hl.env("PATH", table.concat(runestone_path, ":"))
EOF
  printf '%s\n' "$end"
}

# The Hyprland user config without our marked block, and without the blank line install.sh
# puts in front of it.
strip_block() {
  awk -v start="$start" -v end="$end" '
    { line[NR] = $0 }
    END {
      first = 0; last = 0
      for (i = 1; i <= NR; i++) if (line[i] == start) { first = i; break }
      if (first) for (i = first; i <= NR; i++) if (line[i] == end) { last = i; break }
      if (!first || !last) { for (i = 1; i <= NR; i++) print line[i]; exit }
      from = first
      if (first > 1 && line[first - 1] == "") from = first - 1
      for (i = 1; i <= NR; i++) if (i < from || i > last) print line[i]
    }' "$1"
}

# 1. The binary.
if [[ -n $prebuilt ]]; then
  source_bin=$prebuilt
elif ((dry_run)); then
  source_bin=""
  note "would build: cargo build --release -p runestone-saver"
else
  command -v cargo >/dev/null || {
    printf 'install.sh: cargo not found; pacman -S --needed rust\n' >&2
    exit 1
  }
  ( cd "$here" && cargo build --release -p runestone-saver )
  source_bin="${CARGO_TARGET_DIR:-$here/target}/release/runestone-saver"
fi
if ((dry_run)); then
  note "would install $source_bin -> $saver"
else
  mkdir -p "$bin_dir"
  install -m 755 "$source_bin" "$saver"
  note "installed $saver"
fi

# 2. The shim Omarchy's command lands on.
if ((dry_run)); then
  note "would write $shim (runs $saver, else Omarchy's own screensaver)"
elif [[ -f $shim ]] && shim_text | cmp -s - "$shim"; then
  note "shim already up to date: $shim"
else
  shim_text > "$shim"
  chmod 755 "$shim"
  note "wrote $shim"
fi

# 3. The PATH hook, so the shim wins over $OMARCHY_PATH/bin.
if [[ ! -f $hyprland ]]; then
  note "note: $hyprland not found (not an Omarchy session?). Add this yourself, after Omarchy's"
  note "      defaults, or re-run this installer once Hyprland's user config exists:"
  lua_block | sed 's/^/      /'
elif ((dry_run)); then
  note "would update $hyprland (PATH hook between '$start' and '$end')"
else
  scratch="$(mktemp)"
  trap 'rm -f "$scratch" "$scratch.next"' EXIT
  strip_block "$hyprland" > "$scratch"
  {
    cat "$scratch"
    printf '\n'
    lua_block
  } > "$scratch.next"
  if cmp -s "$scratch.next" "$hyprland"; then
    note "PATH hook already up to date in $hyprland"
  else
    mv "$scratch.next" "$hyprland"
    note "updated $hyprland (PATH hook)"
  fi
fi

note
note "needs, on Arch: pacman -S --needed rust libxkbcommon wayland vulkan-icd-loader"
note "log out and back in (or restart Hyprland) so the PATH hook reaches the idle service."
