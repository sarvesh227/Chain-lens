#!/bin/bash
set -e

PORT=${PORT:-3000}

cargo build --release >/dev/null 2>&1

echo "http://127.0.0.1:$PORT"

PORT=$PORT ./target/release/web