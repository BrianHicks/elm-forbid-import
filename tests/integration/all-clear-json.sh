#!/usr/bin/env bash

elm-forbid-import add-root vendor/elm-spa-example
elm-forbid-import update

elm-forbid-import --format json check | jq .
