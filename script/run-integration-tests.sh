#!/usr/bin/env bash
set -euo pipefail
IFS=$'\n\t'

run_test() {
  TEST_FILE="${1:-}"
  NAME="$(basename "$TEST_FILE")"

  GOLDEN="tests/golden-results/$NAME.txt"
  CURRENT="$GOLDEN.current"

  echo "===== $NAME"
  env PATH="$(pwd)/target/debug:$PATH" bash -xeou pipefail "$TEST_FILE" > "$CURRENT"

  diff -U 0 "$GOLDEN" "$CURRENT"
}

cargo build

EXIT=0

for TEST_FILE in $(find tests/integration -type f -name '*.sh'); do
  set +e
  run_test "$TEST_FILE"
  if test "$?" != "0"; then EXIT=1; fi
  set -e
  echo
done

exit "$EXIT"
