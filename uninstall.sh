#!/usr/bin/env bash
# Remove the Runestone screensaver and put Omarchy's own screensaver back.
#
#   ./uninstall.sh               remove the binary, the shim and the PATH hook
#   ./uninstall.sh --dry-run     print every change instead of making it
#   ./uninstall.sh --prefix DIR  uninstall from somewhere other than $HOME/.local/bin
#
# Only what install.sh created is removed: the binary, the shim that carries its marker, and
# the marked block in ~/.config/hypr/hyprland.lua. Nothing else in $HOME is touched.
set -euo pipefail

dry_run=0
prefix=""

usage() {
  sed -n '2,9p' "${BASH_SOURCE[0]}" | sed 's/^# \{0,1\}//'
}

while (($#)); do
  case "$1" in
    --dry-run) dry_run=1; shift ;;
    --prefix)
      [[ -n ${2:-} ]] || { printf 'uninstall.sh: --prefix needs a value\n' >&2; exit 2; }
      prefix=$2
      shift 2 ;;
    -h|--help) usage; exit 0 ;;
    *) printf 'uninstall.sh: unknown argument %s\n' "$1" >&2; exit 2 ;;
  esac
done

note() { printf '%s\n' "$*"; }

home="${HOME:?HOME is not set}"
bin_dir="${prefix:-$home/.local/bin}"
saver="$bin_dir/runestone-saver"
shim="$bin_dir/omarchy-launch-screensaver"
hyprland="$home/.config/hypr/hyprland.lua"
start='-- >>> runestone-saver >>>'
end='-- <<< runestone-saver <<<'

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

has_block() {
  [[ -f $1 ]] && grep -qxF -- "$start" "$1"
}

# 1. The shim, only when it is ours.
if [[ -f $shim ]] && grep -q "installed by omarchy-runestone-theme" "$shim"; then
  if ((dry_run)); then
    note "would remove $shim"
  else
    rm -f "$shim"
    note "removed $shim"
  fi
fi

# 2. The binary.
if [[ -f $saver ]]; then
  if ((dry_run)); then
    note "would remove $saver"
  else
    rm -f "$saver"
    note "removed $saver"
  fi
fi

# 3. The PATH hook.
if has_block "$hyprland"; then
  if ((dry_run)); then
    note "would update $hyprland (drop the PATH hook between '$start' and '$end')"
  else
    scratch="$(mktemp)"
    trap 'rm -f "$scratch"' EXIT
    strip_block "$hyprland" > "$scratch"
    mv "$scratch" "$hyprland"
    note "updated $hyprland (PATH hook removed)"
  fi
fi

# 4. Directories that are now empty, as install.sh created them.
if ((!dry_run)); then
  for dir in "$bin_dir" "$(dirname -- "$bin_dir")"; do
    [[ -d $dir && $dir != "$home" ]] || continue
    rmdir --ignore-fail-on-non-empty -- "$dir" 2>/dev/null || true
  done
fi

note "Omarchy's own screensaver is back: omarchy-launch-screensaver resolves to"
note "/usr/lib/omarchy-screensaver/omarchy-screensaver again after a relog."
