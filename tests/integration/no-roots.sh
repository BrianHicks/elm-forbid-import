#!/usr/bin/env bash

# we need to `cd` into the right place instead of adding it as a root
cd vendor/elm-spa-example || exit 1

ELM_FORBID_IMPORT_CONFIG="../../$ELM_FORBID_IMPORT_CONFIG"
elm-forbid-import forbid Html

if elm-forbid-import check; then
  exit 1 # `check` should fail here
fi
