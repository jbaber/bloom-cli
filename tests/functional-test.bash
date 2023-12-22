#!/usr/bin/env bash

set -e

exe="$PWD"/target/debug/bloom-cli
tmp="$PWD"/target/tmp
mkdir -p "$tmp"
deadbeef="$PWD"/tests/deadbeef
beefs="$PWD"/tests/beefs

[[ -f $deadbeef ]] || exit 1
[[ -f $beefs ]] || exit 1

# Create fresh and insert
rm -f "$tmp"/filter-1
[[ ! -f "$tmp"/filter-1 ]] || exit 1
[[ $("$exe" -v -x "$tmp"/filter-1 -i "$deadbeef" 2>&1) = "Creating a new filter at "* ]] || exit 1
[[ -f "$tmp"/filter-1 ]] || exit 1
[[ -f $deadbeef ]] || exit 1
[[ $("$exe" -x "$tmp"/filter-1 -q "$deadbeef") = "IN" ]] || exit 1
[[ -f $deadbeef ]] || exit 1

# Create blank, then insert
rm -f "$tmp"/filter-2
[[ ! -f "$tmp"/filter-2 ]] || exit 1
[[ $("$exe" -v -x "$tmp"/filter-2 2>&1) = "Creating a new filter at "* ]] || exit 1
[[ -f "$tmp"/filter-2 ]] || exit 1
[[ $("$exe" -x "$tmp"/filter-2 -q "$deadbeef") = "NOT IN" ]] || exit 1
[[ -f $deadbeef ]] || exit 1
[[ $("$exe" -x "$tmp"/filter-2 -i "$deadbeef") = "" ]] || exit 1
[[ -f $deadbeef ]] || exit 1
[[ $("$exe" -x "$tmp"/filter-2 -q "$deadbeef") = "IN" ]] || exit 1
[[ -f $deadbeef ]] || exit 1

# Create blank, add one at a time
rm -f "$tmp"/filter-3
[[ ! -f "$tmp"/filter-3 ]] || exit 1
[[ $("$exe" -v -x "$tmp"/filter-3 2>&1) = "Creating a new filter at "* ]] || exit 1
[[ -f "$tmp"/filter-3 ]] || exit 1
[[ $("$exe" -x "$tmp"/filter-3 -q "$beefs") = "NOT IN" ]] || exit 1
[[ -f $beefs ]] || exit 1
[[ $("$exe" -x "$tmp"/filter-3 -q "$deadbeef") = "NOT IN" ]] || exit 1
[[ -f $deadbeef ]] || exit 1
[[ $("$exe" -x "$tmp"/filter-3 -i "$beefs") = "" ]] || exit 1
[[ -f $beefs ]] || exit 1
[[ $("$exe" -x "$tmp"/filter-3 -q "$beefs") = "IN" ]] || exit 1
[[ -f $beefs ]] || exit 1
[[ $("$exe" -x "$tmp"/filter-3 -q "$deadbeef") = "NOT IN" ]] || exit 1
[[ -f $deadbeef ]] || exit 1
[[ $("$exe" -x "$tmp"/filter-3 -i "$deadbeef") = "" ]] || exit 1
[[ -f $deadbeef ]] || exit 1
[[ $("$exe" -x "$tmp"/filter-3 -q "$deadbeef") = "IN" ]] || exit 1
[[ -f $deadbeef ]] || exit 1
[[ $("$exe" -x "$tmp"/filter-3 -q "$beefs") = "IN" ]] || exit 1
[[ -f $beefs ]] || exit 1

# Errors

# Query and insert simultaneously
rm -f "$tmp"/filter-4
[[ ! -f "$tmp"/filter-4 ]] || exit 1
set +e
"$exe" -x "$tmp"/filter-4 -q $beefs -i $beefs > /dev/null
result=$?
set -e
[[ $result -eq 17 ]] || exit 1
[[ ! -f "$tmp"/filter-4 ]] || exit 1
[[ -f $beefs ]] || exit 1

# Query on newly created filter
rm -f "$tmp"/filter-5
[[ ! -f "$tmp"/filter-5 ]] || exit 1
set +e
"$exe" -x "$tmp"/filter-5 -q $beefs > /dev/null
result=$?
set -e
[[ $result -eq 9 ]] || exit 1
[[ ! -f "$tmp"/filter-5 ]] || exit 1
[[ -f $beefs ]] || exit 1

# Do nothing to an existing filter
rm -f "$tmp"/filter-6
[[ ! -f "$tmp"/filter-6 ]] || exit 1
"$exe" -x "$tmp"/filter-6
[[ -f "$tmp"/filter-6 ]] || exit 1
set +e
"$exe" -x "$tmp"/filter-6 > /dev/null
result=$?
set -e
[[ $result -eq 14 ]] || exit 1
[[ -f "$tmp"/filter-6 ]] || exit 1

# Non regular file as filter
rm -f "$tmp"/x
[[ ! -e "$tmp"/x ]] || exit 1
mkfifo "$tmp"/x
[[ -e "$tmp"/x ]] || exit 1
[[ ! -f "$tmp"/x ]] || exit 1
set +e
"$exe" -x "$tmp"/x
result=$?
set -e
[[ $result -eq 3 ]] || exit 1
[[ -e "$tmp"/x ]] || exit 1
[[ ! -f "$tmp"/x ]] || exit 1
