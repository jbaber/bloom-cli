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

# Can't add non-regular file to a fresh filter
rm -f "$tmp"/x
[[ ! -e "$tmp"/x ]] || exit 1
mkfifo "$tmp"/x
[[ -e "$tmp"/x ]] || exit 1
[[ ! -f "$tmp"/x ]] || exit 1
set +e
"$exe" -x "$tmp"/filter-7 -i "$tmp"/x >/dev/null
result=$?
set -e
[[ $result -eq 3 ]] || exit 1
[[ -e "$tmp"/x ]] || exit 1
[[ ! -f "$tmp"/x ]] || exit 1
[[ ! -e "$tmp"/filter-7 ]] || exit 1

# Can't add or query non-regular file to existing filter
rm -f "$tmp"/x
[[ ! -e "$tmp"/x ]] || exit 1
mkfifo "$tmp"/x
[[ -e "$tmp"/x ]] || exit 1
[[ ! -f "$tmp"/x ]] || exit 1
rm -f "$tmp"/filter-8
[[ ! -e "$tmp"/filter-8 ]] || exit 1
"$exe" -x "$tmp"/filter-8
[[ -f "$tmp"/filter-8 ]] || exit 1

# insert
set +e
"$exe" -x "$tmp"/filter-8 -i "$tmp"/x > /dev/null
result=$?
set -e
[[ $result -eq 3 ]] || exit 1
[[ -e "$tmp"/x ]] || exit 1
[[ ! -f "$tmp"/x ]] || exit 1
[[ -f "$tmp"/filter-8 ]] || exit 1

# query
set +e
"$exe" -x "$tmp"/filter-8 -q "$tmp"/x >/dev/null
result=$?
set -e
[[ $result -eq 3 ]] || exit 1
[[ -e "$tmp"/x ]] || exit 1
[[ ! -f "$tmp"/x ]] || exit 1
[[ -f "$tmp"/filter-8 ]] || exit 1

# Can't add/query non-existent files from existing filter
rm -f "$tmp"/filter-9
[[ ! -f "$tmp"/filter-9 ]] || exit 1
"$exe" -x "$tmp"/filter-9
[[ -f "$tmp"/filter-9 ]] || exit 1
set +e
"$exe" -x "$tmp"/filter-9 -i UNLIKELY_TO_EXIST >/dev/null
result=$?
set -e
[[ $result -eq 2 ]] || exit 1
[[ -f "$tmp"/filter-9 ]] || exit 1
set +e
"$exe" -x "$tmp"/filter-9 -q UNLIKELY_TO_EXIST >/dev/null
result=$?
set -e
[[ $result -eq 2 ]] || exit 1
[[ -f "$tmp"/filter-9 ]] || exit 1

# Can't add non-existent file to fresh filter
rm -f "$tmp"/filter-10
[[ ! -e "$tmp"/filter-10 ]] || exit 1
set +e
"$exe" -x "$tmp"/filter-10 -i UNLIKELY_TO_EXIST >/dev/null
result=$?
set -e
[[ $result -eq 2 ]] || exit 1
[[ ! -e "$tmp"/filter-10 ]] || exit 1

# Can't add/query filter to itself
rm -f "$tmp"/filter-11
[[ ! -e "$tmp"/filter-11 ]] || exit 1
"$exe" -x "$tmp"/filter-11
[[ -f "$tmp"/filter-11 ]] || exit 1
set +e
"$exe" -x "$tmp"/filter-11 -i "$tmp"/filter-11 >/dev/null
result=$?
set -e
[[ $result -eq 5 ]] || exit 1
[[ -f "$tmp"/filter-11 ]] || exit 1
set +e
"$exe" -x "$tmp"/filter-11 -q "$tmp"/filter-11 >/dev/null
result=$?
set -e
[[ $result -eq 5 ]] || exit 1
[[ -f "$tmp"/filter-11 ]] || exit 1

# Don't clobber existing filter
rm -f "$tmp"/filter-12
[[ ! -e "$tmp"/filter-12 ]] || exit 1
"$exe" -x "$tmp"/filter-12
[[ -f "$tmp"/filter-12 ]] || exit 1
set +e
"$exe" -x "$tmp"/filter-12
result=$?
[[ $result -eq 14 ]] || exit
[[ -f "$tmp"/filter-12 ]] || exit 1
