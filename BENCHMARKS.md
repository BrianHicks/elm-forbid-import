# Benchmarks

I'm benchmarking on two repos (see `script/bench.sh`):

| Target                                                                              | Time             |
|-------------------------------------------------------------------------------------|------------------|
| (elm-spa-example)[https://github.com/rtfeldman/elm-spa-example] (224kb of Elm code) | 30.9 ms ± 1.3 ms |
| my main work repo (12mb of Elm code)                                                | 2.055 s ± 0.041s |

We're doing pretty well on these.
We can probably do better by reducing allocations etc but it's not slow enough to justify the effort right now, IMO.

## Things I've tried

### November 19: Parallel Walker

`ignore` has a parallel walker implementation.
Here's what happens when we switch to walking in parallel:

| Target          | Old Time        | New Time         | Speedup                     |
|-----------------|-----------------|------------------|-----------------------------|
| elm-spa-example | 44.9ms ± 2ms    | 30.9 ms ± 1.3 ms | ~14ms, or ~69% of the time  |
| work repo       | 3.920s ± 0.252s | 2.055 s ± 0.041s | ~1.87s, or ~52% of the time |

### November 18: Link-time Optimization

Enabling link-time optimization with this in `Cargo.toml`:

```
[profile.release]
lto = true
```

... makes compile times go to like 2 minutes on my machine, with only a tiny speedup.
"thin" LTO does not help either (although it's a slightly faster compile.)
