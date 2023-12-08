#!/usr/bin/env bash

set -e

exe="$PWD"/target/debug/bloom-cli
filter="$PWD"/target/tmp/filter-file
beef="$PWD"/tests/deadbeef

"$exe" -x "$filter" -i "$beef" 2>/dev/null
