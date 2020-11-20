#!/usr/bin/env bash

elm-forbid-import add-root vendor/elm-spa-example
elm-forbid-import forbid Html

if elm-forbid-import check; then
  exit 1 # check should fail here!
fi
