#!/usr/bin/env bash

set -e

exe="$PWD"/target/debug/bloom-cli
tmp="$PWD"/target/tmp
mkdir -p "$tmp"
deadbeef="$PWD"/tests/deadbeef
beefs="$PWD"/tests/beefs

# Create fresh and insert
rm -f "$tmp"/filter-1
[[ $("$exe" -v -x "$tmp"/filter-1 -i "$deadbeef" 2>&1) = "Creating a new filter at "* ]] || exit 1
[[ $("$exe" -x "$tmp"/filter-1 -q "$deadbeef") = "IN" ]] || exit 1

# Create blank, then insert
rm -f "$tmp"/filter-2
[[ $("$exe" -v -x "$tmp"/filter-2 2>&1) = "Creating a new filter at "* ]] || exit 1
[[ $("$exe" -x "$tmp"/filter-2 -q "$deadbeef") = "NOT IN" ]] || exit 1
[[ $("$exe" -x "$tmp"/filter-2 -i "$deadbeef") = "" ]] || exit 1
[[ $("$exe" -x "$tmp"/filter-2 -q "$deadbeef") = "IN" ]] || exit 1

# Create blank, add one at a time
rm -f "$tmp"/filter-3
[[ $("$exe" -v -x "$tmp"/filter-3 2>&1) = "Creating a new filter at "* ]] || exit 1
[[ $("$exe" -x "$tmp"/filter-3 -q "$beefs") = "NOT IN" ]] || exit 1
[[ $("$exe" -x "$tmp"/filter-3 -q "$deadbeef") = "NOT IN" ]] || exit 1
[[ $("$exe" -x "$tmp"/filter-3 -i "$beefs") = "" ]] || exit 1
[[ $("$exe" -x "$tmp"/filter-3 -q "$beefs") = "IN" ]] || exit 1
[[ $("$exe" -x "$tmp"/filter-3 -q "$deadbeef") = "NOT IN" ]] || exit 1
[[ $("$exe" -x "$tmp"/filter-3 -i "$deadbeef") = "" ]] || exit 1
[[ $("$exe" -x "$tmp"/filter-3 -q "$deadbeef") = "IN" ]] || exit 1
[[ $("$exe" -x "$tmp"/filter-3 -q "$beefs") = "IN" ]] || exit 1
