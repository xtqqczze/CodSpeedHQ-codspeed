#!/bin/sh
# Toggle "samply dev mode" for the runner.
#
# Dev mode redirects the runner's `samply`, `framehop`, and
# `linux-perf-event-reader` dependencies to local sibling checkouts by
# appending a `[patch]` block to the relevant `Cargo.toml` files. This lets you
# iterate on all three crates in place and have the runner pick the changes up
# immediately.
#
# Nothing is hidden from git: the appended blocks and the resulting `Cargo.lock`
# edits show up in `git status` like any other change. It is on you not to commit
# them — run `off` to remove the blocks when you're done.
#
# The block is delimited by sentinel comments so `off` can strip it cleanly:
#   - <runner>/Cargo.toml          patches samply + framehop + reader -> local
#   - <samply>/Cargo.toml          patches framehop + reader -> local (so samply
#                                  standalone builds also use them)
#
# Usage:
#   scripts/samply-dev.sh on    enable dev mode (append patch blocks)
#   scripts/samply-dev.sh off   disable dev mode (remove patch blocks)
set -eu

# Resolve repo roots relative to this script, not the cwd.
RUNNER_ROOT=$(CDPATH= cd -- "$(dirname -- "$0")/.." && pwd)
SAMPLY_ROOT=$(CDPATH= cd -- "$RUNNER_ROOT/../samply-codspeed" 2>/dev/null && pwd || true)

SAMPLY_URL="https://github.com/CodSpeedHQ/samply-codspeed"
FRAMEHOP_URL="https://github.com/CodSpeedHQ/framehop"

RUNNER_MANIFEST="$RUNNER_ROOT/Cargo.toml"
SAMPLY_MANIFEST="$SAMPLY_ROOT/Cargo.toml"

BEGIN="# >>> samply-dev (do not commit) >>>"
END="# <<< samply-dev <<<"

usage() {
  echo "Usage: $0 {on|off}" >&2
  echo "  on   enable dev mode (append patch blocks)" >&2
  echo "  off  disable dev mode (remove patch blocks)" >&2
  exit 2
}

# Remove the sentinel-delimited block from a manifest, if present.
strip_block() {
  manifest=$1
  [ -f "$manifest" ] || return 0
  sed "/^$BEGIN\$/,/^$END\$/d" "$manifest" > "$manifest.tmp" && mv "$manifest.tmp" "$manifest"
}

# Append a sentinel-delimited block to a manifest, replacing any existing one.
append_block() {
  manifest=$1
  body=$2
  strip_block "$manifest"
  printf '%s\n%s%s\n' "$BEGIN" "$body" "$END" >> "$manifest"
}

enable() {
  if [ -z "$SAMPLY_ROOT" ]; then
    echo "error: ../samply-codspeed not found next to the runner repo" >&2
    exit 1
  fi

  append_block "$RUNNER_MANIFEST" "\
[patch.\"$SAMPLY_URL\"]
samply = { path = \"../samply-codspeed/samply\" }

[patch.\"$FRAMEHOP_URL\"]
framehop = { path = \"../framehop\" }

[patch.crates-io]
linux-perf-event-reader = { path = \"../linux-perf-event-reader\" }
"

  append_block "$SAMPLY_MANIFEST" "\
[patch.\"$FRAMEHOP_URL\"]
framehop = { path = \"../framehop\" }

[patch.crates-io]
linux-perf-event-reader = { path = \"../linux-perf-event-reader\" }
"

  echo "samply dev mode: ON"
  echo "  patched $RUNNER_MANIFEST"
  echo "  patched $SAMPLY_MANIFEST"
}

disable() {
  strip_block "$RUNNER_MANIFEST"
  echo "  cleaned $RUNNER_MANIFEST"
  if [ -n "$SAMPLY_ROOT" ]; then
    strip_block "$SAMPLY_MANIFEST"
    echo "  cleaned $SAMPLY_MANIFEST"
  fi
  echo "samply dev mode: OFF"
}

[ $# -eq 1 ] || usage

case "$1" in
  on) enable ;;
  off) disable ;;
  *) usage ;;
esac
