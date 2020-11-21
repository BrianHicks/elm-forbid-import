#!/usr/bin/env bash

elm-forbid-import add-root vendor/elm-spa-example
elm-forbid-import forbid Html

set +e
MATCH="$(elm-forbid-import --format json check | jq '.[0]')"
set -e

elm-forbid-import update

FILE="$(jq -r '.path' <<< "$MATCH")"
cp "$FILE" "$FILE.bak"
sed -i "$(jq '.position.row' <<< "$MATCH")d" "$FILE"

if elm-forbid-import check; then
  exit 1 # this check should exit 1
fi

mv "$FILE.bak" "$FILE"
