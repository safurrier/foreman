#!/usr/bin/env bash
set -euo pipefail

ROOT="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
SNAPSHOT_DIR="${FOREMAN_OVERLAY_DEMO_SNAPSHOT_DIR:-$ROOT/.ai/validation/macos-overlay/demo-snapshots}"
OUT_DIR="${FOREMAN_OVERLAY_DEMO_DIR:-$ROOT/demos}"
WIDTH="${FOREMAN_OVERLAY_DEMO_WIDTH:-900}"
DURATION="${FOREMAN_OVERLAY_DEMO_FRAME_SECONDS:-1.35}"

if ! command -v ffmpeg >/dev/null 2>&1; then
  echo "ffmpeg is required to build the macOS overlay demo" >&2
  exit 127
fi

mkdir -p "$SNAPSHOT_DIR" "$OUT_DIR"
FOREMAN_OVERLAY_SNAPSHOT_DIR="$SNAPSHOT_DIR" "$ROOT/scripts/render-macos-overlay-snapshots.sh" >/dev/null

states=(
  attention
  pr-active
  compose
  flash
  help
  theme-indigo
)

list_file="$(mktemp)"
palette_file="$(mktemp -t foreman-overlay-demo-palette).png"
cleanup() {
  rm -f "$list_file" "$palette_file"
}
trap cleanup EXIT

for state in "${states[@]}"; do
  image="$SNAPSHOT_DIR/$state.png"
  if [ ! -s "$image" ]; then
    echo "missing snapshot for demo state: $image" >&2
    exit 1
  fi
  printf "file '%s'\n" "$image" >> "$list_file"
  printf "duration %s\n" "$DURATION" >> "$list_file"
done
# concat demuxer needs the final file repeated for the last duration to apply.
printf "file '%s'\n" "$SNAPSHOT_DIR/${states[$((${#states[@]} - 1))]}.png" >> "$list_file"

mp4="$OUT_DIR/macos-overlay-demo.mp4"
gif="$OUT_DIR/macos-overlay-demo.gif"

ffmpeg -hide_banner -loglevel error -y \
  -f concat -safe 0 -i "$list_file" \
  -vf "fps=12,scale=${WIDTH}:-2:flags=lanczos,format=yuv420p" \
  -movflags +faststart \
  "$mp4"

ffmpeg -hide_banner -loglevel error -y \
  -f concat -safe 0 -i "$list_file" \
  -vf "fps=8,scale=${WIDTH}:-2:flags=lanczos,palettegen=max_colors=128" \
  "$palette_file"

ffmpeg -hide_banner -loglevel error -y \
  -f concat -safe 0 -i "$list_file" -i "$palette_file" \
  -lavfi "fps=8,scale=${WIDTH}:-2:flags=lanczos[x];[x][1:v]paletteuse=dither=bayer:bayer_scale=5" \
  "$gif"

cat > "$OUT_DIR/macos-overlay-demo.md" <<EOF
# macOS Overlay Demo

Generated from the real Swift overlay renderer with deterministic fixture data.

- Source snapshots: ${SNAPSHOT_DIR#$ROOT/}
- MP4: ${mp4#$ROOT/}
- GIF: ${gif#$ROOT/}
- States: ${states[*]}
EOF

ls -lh "$mp4" "$gif"
