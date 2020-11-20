#!/usr/bin/env bash
set -eou pipefail

find tests/golden-results -type f -name '*.current' | while read -r TO_PROMOTE; do
  mv "$TO_PROMOTE" "$(echo "$TO_PROMOTE" | sed -E 's/.current$//')"
done
