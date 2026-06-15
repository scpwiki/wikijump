#!/bin/bash
set -eu
exec cargo run --example dump_json -- "$@"
