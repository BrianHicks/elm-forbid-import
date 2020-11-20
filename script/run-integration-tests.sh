#!/usr/bin/env bash
set -euo pipefail
IFS=$'\n\t'

if test -d target/debug; then
  cargo build
  BIN_PATH="$(pwd)/target/debug"
elif test -d target/release; then
  BIN_PATH="$(pwd)/target/release"
fi

mkdir tmp
trap 'rm -rf tmp' EXIT

run_test() {
  if test -f tmp/forbidden-imports.toml; then
    rm tmp/forbidden-imports.toml
  fi

  TEST_FILE="${1:-}"
  NAME="$(basename "$TEST_FILE")"

  GOLDEN="tests/golden-results/$NAME.txt"
  CURRENT="$GOLDEN.current"

  echo "===== $NAME"
  env PATH="$BIN_PATH:$PATH" ELM_FORBID_IMPORT_CONFIG="tmp/forbidden-imports.toml" bash -xeou pipefail "$TEST_FILE" > "$CURRENT"

  if ! test -e "$GOLDEN"; then
    cp "$CURRENT" "$GOLDEN"
  fi

  diff -U 0 "$GOLDEN" "$CURRENT"
}

EXIT=0

for TEST_FILE in $(find tests/integration -type f -name '*.sh'); do
  set +e
  run_test "$TEST_FILE"
  if test "$?" != "0"; then EXIT=1; fi
  set -e
  echo
done

exit "$EXIT"
