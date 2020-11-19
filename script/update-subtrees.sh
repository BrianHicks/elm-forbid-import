#!/usr/bin/env bash
set -euo pipefail

git subtree pull --squash --prefix vendor/elm-spa-example https://github.com/rtfeldman/elm-spa-example master
