# Benchmarks

I'm benchmarking on two repos (see `script/bench.sh`):

| Target                                                                              | Time              |
|-------------------------------------------------------------------------------------|-------------------|
| (elm-spa-example)[https://github.com/rtfeldman/elm-spa-example] (224kb of Elm code) | 30.9 ms ± 1.3 ms  |
| my main work repo (12mb of Elm code)                                                | 2.055 s ± 0.041 s |

I want this to be much faster.
The old tool that this is replacing for me does more and runs in `1.210 s ± 0.016 s` on the work repo... and it's written in Python (not that Python is necessarily *slow* but it's probably possible to get a faster result!)

It would probably also be OK to drop tree-sitter.
It's not really super essential for this task, since the regex I actually want is tiny: `^import ([A-Z][\w\d\.]+)`

We can probably do better by reducing allocations etc but there are lower-hanging fruit.

## Things I've tried

### November 19: Link-time Optimization Again

I added `crossbeam` and used a lot more of `ignore`, so maybe LTO has an effect now?

| Target          | Old Time         | New Time           | Speedup                            |
|-----------------|------------------|--------------------|------------------------------------|
| elm-spa-example | 30.9 ms ± 1.3 ms | 29.7 ms ± 1.0 ms   | Something like 1ms                 |
| work repo       | 2.055 s ± 0.041s | 2.129 s ± 0.022 s  | actually slowed down by like 0.1s? |

Build times are worse, but in a manageable way: a fresh build of `cargo build --release` now takes 2m19s (up 30s from 1m49s)

In the end, this is still not worth it, and I'm backing the change out.

### November 19: Parallel Walker

`ignore` has a parallel walker implementation.
Here's what happens when we switch to walking in parallel:

| Target          | Old Time        | New Time          | Speedup                     |
|-----------------|-----------------|-------------------|-----------------------------|
| elm-spa-example | 44.9ms ± 2ms    | 30.9 ms ± 1.3 ms  | ~14ms, or ~69% of the time  |
| work repo       | 3.920s ± 0.252s | 2.055 s ± 0.041 s | ~1.87s, or ~52% of the time |

### November 18: Link-time Optimization

Enabling link-time optimization with this in `Cargo.toml`:

```
[profile.release]
lto = true
```

... makes compile times go to like 2 minutes on my machine, with only a tiny speedup.
"thin" LTO does not help either (although it's a slightly faster compile.)
