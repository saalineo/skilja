#!/bin/bash
set -e

PROJECT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
cd "$PROJECT_DIR"

echo "Building audio library..."
./scripts/build_audio.sh

echo "Building skilja..."
odin build . -out:skilja

echo "Done. Run with ./skilja"
