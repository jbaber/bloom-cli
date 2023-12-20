#!/usr/bin/env bash

set -e

exe="$PWD"/target/debug/bloom-cli
tmp="$PWD"/target/tmp
beef="$PWD"/tests/deadbeef

# Create fresh and insert
"$exe" -x "$tmp"/filter-1 -i "$beef" 2>/dev/null
"$exe" -x "$tmp"/filter-1 -q "$beef" 2>/dev/null
