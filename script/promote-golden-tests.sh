#!/usr/bin/env bash
set -eou pipefail

for $TO_PROMOTE in $(find tests/golden-results -type f -name '*.current'); do
  mv "$TO_PROMOTE" "$(echo "$TO_PROMOTE" | sed -E 's/.current$//')"
done
