# Benchmarks

I'm benchmarking on two repos (see `script/bench.sh`):

| Target                                                                              | Time              |
|-------------------------------------------------------------------------------------|-------------------|
| (elm-spa-example)[https://github.com/rtfeldman/elm-spa-example] (224kb of Elm code) | 10.6 ms ± 0.6 ms  |
| my main work repo (12mb of Elm code)                                                | 196.7 ms ± 5.4 ms |

The standard to meet or exceed here is the tool this is replacing, which runs in `1.210 s ± 0.016 s` on the work repo.
It does a bit more work than this tool, but scanning+matching is the main part of it.

We can probably do better by reducing allocations etc but a 6x speedup is good enough for now.
I doubt there are many repos in the world as big as the one I'm working with, so I think it should be acceptably performant on smaller repos.

## Things I've tried

### November 19: Cutting off matches early

The Elm compiler requires imports to be at the top of the file.
They're usually all in a block, so it's probably fine to stop processing lines once we fail to match after previously matching.

| Target          | Old Time          | New Time          | Speedup                                         |
|-----------------|-------------------|-------------------|-------------------------------------------------|
| elm-spa-example | 11.2 ms ± 0.5 ms  | 10.6 ms ± 0.6 ms  | within the margin of error, probably no speedup |
| work repo       | 235.2 ms ± 6.1 ms | 196.7 ms ± 5.4 ms | ~38ms speedup, 1.19x faster                     |

Seems worth keeping for the larger repo.

### November 19: Switching to regular expressions

Tree-sitter is an amazing tool, but I'm using the simplest possible form of it.
I bet a regex would be faster!

| Target          | Old Time         | New Time          | Speedup                        |
|-----------------|------------------|-------------------|--------------------------------|
| elm-spa-example | 30.9 ms ± 1.3 ms | 11.2 ms ± 0.5 ms  | ~20ms, or ~2.75x faster        |
| work repo       | 2.055 s ± 0.041s | 235.2 ms ± 6.1 ms | ~1.82s, or ~8.75x (!!!) faster |

What a nice win!

### November 19: Link-time Optimization Again

I added `crossbeam` and used a lot more of `ignore`, so maybe LTO has an effect now?

| Target          | Old Time         | New Time          | Speedup                            |
|-----------------|------------------|-------------------|------------------------------------|
| elm-spa-example | 30.9 ms ± 1.3 ms | 29.7 ms ± 1.0 ms  | Something like 1ms                 |
| work repo       | 2.055 s ± 0.041s | 2.129 s ± 0.022 s | actually slowed down by like 0.1s? |

Build times are worse, but in a manageable way: a fresh build of `cargo build --release` now takes 2m19s (up 30s from 1m49s)

In the end, this is still not worth it, and I'm backing the change out.

### November 19: Parallel Walker

`ignore` has a parallel walker implementation.
Here's what happens when we switch to walking in parallel:

| Target          | Old Time        | New Time          | Speedup                |
|-----------------|-----------------|-------------------|------------------------|
| elm-spa-example | 44.9ms ± 2ms    | 30.9 ms ± 1.3 ms  | ~14ms, or 1.45x faster |
| work repo       | 3.920s ± 0.252s | 2.055 s ± 0.041 s | ~1.87s, or 1.9x faster |

### November 18: Link-time Optimization

Enabling link-time optimization with this in `Cargo.toml`:

```
[profile.release]
lto = true
```

... makes compile times go to like 2 minutes on my machine, with only a tiny speedup.
"thin" LTO does not help either (although it's a slightly faster compile.)
