#!/bin/bash
set -eu
exec cargo run --example validate_json -- "$@"
