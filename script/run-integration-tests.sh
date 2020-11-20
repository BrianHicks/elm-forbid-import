#!/usr/bin/env bash
set -euo pipefail
IFS=$'\n\t'

run_test() {
  TEST_FILE="${1:-}"
  NAME="$(basename "$TEST_FILE")"

  echo "===== $NAME"
  env PATH="$(pwd)/target/debug:$PATH" bash -xeou pipefail "$TEST_FILE" > "tests/golden-results/$NAME.txt"
}

cargo build
find tests/integration -type f -name '*.sh' | while read -r TEST_FILE; do
  run_test "$TEST_FILE"
  echo
done
