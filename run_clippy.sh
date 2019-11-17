#!/usr/bin/env bash

set -euo pipefail

# Lints we disagree with and choose to keep in our code with no warning
mapfile -t clippy_lints_to_allow < "clippy_lints_to_allow.txt"

# Known failing lints we want to receive warnings for, but not fail the build
mapfile -t clippy_lints_to_warn < "clippy_lints_to_warn.txt"

# Lints we don't expect to have in our code at all and want to avoid adding
# even at the cost of failing the build
mapfile -t clippy_lints_to_deny < "clippy_lints_to_deny.txt"

clippy_args="cargo clippy --all-targets -- "

add_lints_to_clippy_args() {
  flag=$1
  shift
  for lint
  do
	clippy_arg=" $flag $lint "
	clippy_args="$clippy_args $clippy_arg"
  done
}
set +u # See https://stackoverflow.com/questions/7577052/bash-empty-Wrray-expansion-with-set-u/39687362#39687362
add_lints_to_clippy_args -A "${clippy_lints_to_allow[@]}"
add_lints_to_clippy_args -W "${clippy_lints_to_warn[@]}"
add_lints_to_clippy_args -D "${clippy_lints_to_deny[@]}"
set -u

echo "--- Running clippy!"
cargo version
cargo clippy --version

echo "------"
echo "Clippy rules: $clippy_args"
echo "------"
cargo clean
eval "$clippy_args"
