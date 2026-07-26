#!/bin/sh
set -eu

ROOT=$(CDPATH='' cd -- "$(dirname -- "$0")/.." && pwd)
VENV="$ROOT/.venv"

python3 -m venv --clear "$VENV"
"$VENV/bin/python" -m pip install --requirement "$ROOT/requirements.lock"
