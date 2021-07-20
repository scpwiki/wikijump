#!/bin/sh
set -eu

# Example compilation for ffi_test.c
# Adjust for your particular build and system.

target='debug'
input="${0%/*}/ffi_test.c"
output=/tmp/ftml_ffi_test

cc \
	-I "target/$target" \
	-L "target/$target" \
	-l ftml \
	"$input" \
	-o "$output"

LD_LIBRARY_PATH="$LD_LIBRARY_PATH:./target/$target" "$output"
