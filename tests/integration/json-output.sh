#!/usr/bin/env bash

elm-forbid-import add-root vendor/elm-spa-example
elm-forbid-import forbid Html --hint "use Html.Styled"

if elm-forbid-import --format json check | jq . | sed "s|$PWD/||g"; then
  exit 1 # elm-forbid-import should exit with 1 here
fi
