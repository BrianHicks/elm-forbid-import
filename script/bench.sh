#!/usr/bin/env bash
set -xeuo pipefail

cargo build --release

mkdir -p bench_config
trap 'rm -r bench_config' EXIT

./target/release/elm-forbid-import -c bench_config/config.toml add-root vendor/elm-spa-example
./target/release/elm-forbid-import -c bench_config/config.toml forbid Html --hint "Use Html.Styled"
./target/release/elm-forbid-import -c bench_config/config.toml forbid Html.Events
./target/release/elm-forbid-import -c bench_config/config.toml forbid Html.Attributes
./target/release/elm-forbid-import -c bench_config/config.toml update

hyperfine './target/release/elm-forbid-import -c bench_config/config.toml check'
