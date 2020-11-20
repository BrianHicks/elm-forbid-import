#!/usr/bin/env bash
set -euo pipefail
IFS=$'\n\t'

if ! command -v elm-forbid-import; then
  cargo build
  PATH="$(pwd)/target/debug:$PATH"
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

  GOLDEN_CONFIG="tests/golden-results/$NAME.config.toml"
  CURRENT_CONFIG="$GOLDEN_CONFIG.current"

  echo "===== $NAME"
  env ELM_FORBID_IMPORT_CONFIG="tmp/forbidden-imports.toml" bash -xeou pipefail "$TEST_FILE" > "$CURRENT"
  mv tmp/forbidden-imports.toml "$CURRENT_CONFIG"

  if ! test -e "$GOLDEN"; then
    cp "$CURRENT" "$GOLDEN"
  fi

  if ! test -e "$GOLDEN_CONFIG"; then
    cp "$CURRENT_CONFIG" "$GOLDEN_CONFIG"
  fi

  echo "----- diffing output"
  diff -U 0 "$GOLDEN" "$CURRENT"

  echo '----- diffing config'
  diff -U 0 "$GOLDEN_CONFIG" "$CURRENT_CONFIG"
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
