#!/usr/bin/env bash
# Regenerate every Runestone asset from the scenes.
#
#   scripts/render.sh              # everything: stills, loops, previews, colours
#   scripts/render.sh --check      # the same, then prove each loop closes on itself
#   scripts/render.sh --stills     # only the stills and previews
#
# Stills are 3840x2160 at scale 1 and land as WebP q90; loops are 2560x1440 at 30 fps for one
# PERIOD (16 s), H.264 yuv420p, and have to stay under 25 MB each. Set CARGO_TARGET_DIR to
# share a build directory, or SKIP_BUILD=1 to reuse an already-built binary.
set -euo pipefail
cd "$(dirname "$0")/.."

TARGET="${CARGO_TARGET_DIR:-target}"
BIN="$TARGET/release/runestone-render"

STILL_SIZE=3840x2160
LOOP_SIZE=2560x1440
LOOP_FPS=30
PERIOD=16
LOOP_FRAMES=$((LOOP_FPS * PERIOD))

CHECK=0
STILLS_ONLY=0
for arg in "$@"; do
  case "$arg" in
    --check) CHECK=1 ;;
    --stills) STILLS_ONLY=1 ;;
    *) echo "unknown argument: $arg" >&2; exit 2 ;;
  esac
done

say() { printf '\n\033[1m%s\033[0m\n' "$*"; }

say "build"
[ "${SKIP_BUILD:-0}" = 1 ] || cargo build --release -p runestone-render
[ -x "$BIN" ] || { echo "no binary at $BIN" >&2; exit 1; }

say "colors.toml"
"$BIN" --colors-toml > colors.toml
echo "  $(wc -l < colors.toml) lines"

# The picker takes every image and video directly in backgrounds/ (bin/omarchy-theme-bg-next),
# so only the loops live there: a stale still beside a loop would make six backgrounds, not three.
rm -f backgrounds/*.webp
mkdir -p backgrounds backgrounds/stills .frames
for scene in monolith ring script; do
  say "still: $scene"
  "$BIN" --scene "$scene" --snap ".frames/$scene.png" --at 2.6 \
    --size "$STILL_SIZE" --scale 1
  # Lossless: a 4K near-black gradient is the worst case for a lossy encoder, and lossless
  # keeps every level of the aurora and the grain exactly as rendered.
  ffmpeg -v error -y -i ".frames/$scene.png" \
    -c:v libwebp -lossless 1 -compression_level 6 -preset picture backgrounds/stills/"$scene".webp
  echo "  backgrounds/stills/$scene.webp $(du -h backgrounds/stills/"$scene".webp | cut -f1)"
done

if [ "$STILLS_ONLY" = 0 ]; then
  for scene in monolith ring script; do
    say "loop: $scene ($LOOP_SIZE, $LOOP_FPS fps, ${PERIOD}s)"
    rm -rf ".frames/$scene"
    # One extra frame: frame 0 of a run is drawn before the renderer has settled, so the loop
    # starts at frame 1 and ends on the frame a full PERIOD later.
    "$BIN" --scene "$scene" --frames $((LOOP_FRAMES + 1)) --fps "$LOOP_FPS" \
      --size "$LOOP_SIZE" --scale 1 --out ".frames/$scene"
    ffmpeg -v error -y -framerate "$LOOP_FPS" -start_number 1 \
      -i ".frames/$scene/frame_%04d.png" -frames:v "$LOOP_FRAMES" \
      -c:v libx264 -preset slow -crf 23 -pix_fmt yuv420p -profile:v high -level 4.2 \
      -x264-params aq-mode=3 -movflags +faststart backgrounds/"$scene".mp4
    size=$(du -b backgrounds/"$scene".mp4 | cut -f1)
    echo "  backgrounds/$scene.mp4 $((size / 1024 / 1024)) MB"
    [ "$size" -le $((25 * 1024 * 1024)) ] || { echo "  over the 25 MB budget" >&2; exit 1; }
    rm -rf ".frames/$scene"
  done
fi

say "previews"
ffmpeg -v error -y -i .frames/monolith.png -vf scale=2560:1440:flags=lanczos \
  -pix_fmt rgb24 preview.png
"$BIN" --scene unlock --snap .frames/unlock.png --at 0 --size 800x188 --scale 1
cp .frames/unlock.png unlock.png
ffmpeg -v error -y -f lavfi -i color=c=0x05060a:s=1920x1080 -i unlock.png \
  -filter_complex "[1]scale=1200:-1[mark];[0][mark]overlay=(W-w)/2:(H-h)/2" \
  -frames:v 1 -pix_fmt rgb24 preview-unlock.png
ls -la preview.png preview-unlock.png unlock.png

if [ "$CHECK" = 1 ]; then
  say "seamless: does the loop wrap like any other frame?"
  # A frame's time is its index over the fps, and only a power-of-two fps lands exactly on
  # 16.000 s: at 32 fps frame 512 is t=16.0 (phase 0) and frame 513 is t=16.03125, the phase of
  # frame 1. So the wrap (frame 512 -> frame 1) is exactly the step every other frame makes
  # (frame 1 -> frame 2), and both can be measured against the noise floor: the same phase
  # rendered twice (frame 1 -> frame 513), which Rune draws a little differently because a
  # frame's pixels depend on where in a run it is drawn.
  for scene in monolith ring script; do
    mkdir -p ".frames/seam/$scene"
    "$BIN" --scene "$scene" --frames $((32 * PERIOD + 2)) --fps 32 \
      --size 960x540 --scale 1 --out ".frames/seam/$scene" >/dev/null
    dir=".frames/seam/$scene"
    psnr() {
      ffmpeg -hide_banner -nostats -i "$1" -i "$2" -lavfi psnr -f null - 2>&1 |
        sed -n 's/.*average:\([^ ]*\).*/\1/p' | tail -1
    }
    wrap=$(psnr "$dir/frame_0512.png" "$dir/frame_0001.png")
    step=$(psnr "$dir/frame_0001.png" "$dir/frame_0002.png")
    floor=$(psnr "$dir/frame_0001.png" "$dir/frame_0513.png")
    echo "  $scene: wrap ${wrap} dB vs ordinary step ${step} dB (same phase twice: ${floor} dB)"
  done
fi

say "done"
du -sh backgrounds
