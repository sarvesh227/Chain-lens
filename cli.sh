#!/bin/bash
set -e

PORT=${PORT:-3000}

# Build release quietly
cargo build --release >/dev/null 2>&1

BIN="./target/release/cli"

# BLOCK MODE
if [ "$1" == "--block" ]; then
  if [ $# -ne 4 ]; then
    echo '{"ok":false,"error":{"code":"USAGE_ERROR","message":"Usage: cli.sh --block <blk.dat> <rev.dat> <xor.dat>"}}'
    exit 1
  fi

  $BIN --block "$2" "$3" "$4"
  exit $?
fi

# SINGLE TX MODE
if [ $# -ne 1 ]; then
  echo '{"ok":false,"error":{"code":"USAGE_ERROR","message":"Usage: cli.sh <fixture.json>"}}'
  exit 1
fi

$BIN "$1"
exit $?