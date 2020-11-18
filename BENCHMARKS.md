# Benchmarks

The current version of elm-deprecated-import can scan for imports in [elm-spa-example](https://github.com/rtfeldman/elm-spa-example) (224kb of Elm code) in 44.9ms, ± 2ms (see script/bench.sh in the source for the setup here.)
On my main work repo (12mb of Elm code) it runs in 3.920s ± 0.252s.

We can probably get faster than that!

## Things I've tried

### Link-time Optimization

Enabling link-time optimization with this in `Cargo.toml`:

```
[profile.release]
lto = true
```

... makes compile times go to like 2 minutes on my machine, with only a tiny speedup.
"thin" LTO does not help either (although it's a slightly faster compile.)
