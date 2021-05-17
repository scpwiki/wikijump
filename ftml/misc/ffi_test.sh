#!/bin/sh

# Example compilation for ffi_test.c
# Adjust for your particular build and system.

target='debug'
output=/tmp/ftml_ffi_test

cc \
	-I "target/$target" \
	-L "target/$target" \
	-l ftml \
	-o "$output"

LD_LIBRARY_PATH="$LD_LIBRARY_PATH:./target/$target" "$output"
