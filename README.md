# elm-forbid-import

This tool is based on the realization that when we intend to deprecate some import (say, to upgrade from `Regex` to `Parser` or from `Html` to `Html.Styled`) we often can't do it all immediately—these can be big projects!
Fortunately, we can work on these kinds of projects a little bit at a time!
But in this situation—especially as part of a team—you want to avoid adding new imports of the thing you're trying to remove.

Enter `elm-forbid-imports`, which lets you mark current imports of a module as allowed, but will forbid any future imports.
This lets you whittle away at a deprecated import until you can finally remove it from your `elm.json` or be fine with the few remaining imports.

## Usage

To forbid an import (say, `Html`), run:

```sh
$ elm-forbid-import forbid Html --hint 'use Html.Styled'
```

Now `Html` is forbidden in your project!
The hint you (optionally) pass in will be used in the error messages to remind you what to do instead.

Let's see what needs work:

```sh
$ elm-forbid-import check
src/Article/Body.elm:3:7:forbidden import Html (use Html.Styled)
src/Article/Feed.elm:9:7:forbidden import Html (use Html.Styled)
...
```

Now get to work on removing those imports!
When you're done, or can't remove any more, allow the remaining usages:

```sh
$ elm-forbid-import update
```

Now `check` will not report any further errors on files in that list.
However, if you add or remove more you'll be prompted to either remove the imports or accept them with `update`.

All this will create a `forbidden-imports.toml` file in the current directory (you can control this name and location with `--config` or by setting `ELM_FORBID_IMPORT_CONFIG`.)
**You should check this file in!**
Doing so means that you can run `elm-forbid-import check` in your CI setup so that you cannot enforce which modules are forbidden.

## Install

This tool isn't packaged in a way you can just download a binary yet.
In the meantime (or even after), you can use [`nix`](https://nixos.org/download.html) to install:

```
nix-env -if https://git.bytes.zone/brian/elm-forbid-import/archive/main.tar.gz
```

If you don't want to install globally, check out this repo and run `nix-build` inside it; the binary will end up in `result/bin/`.
If you've got a rust toolchain set up, `cargo build --release` in the root directory should also work; the binary will end up in `release/release`.

## Q&A

### How common is forbidding an import, really?

We do it pretty frequently at NoRedInk.

First of all, we use `Html.Styled` everywhere we can, so `Html` is forbidden (as well as `Html.Events`, `Html.Attributes`, etc.)

Second, we've moved from an internal styleguide to an [external one](https://github.com/NoRedInk/noredink-ui) over time, which means that we forbid new imports of all the previous style guide stuff we haven't made the time to migrate yet.
This keeps us moving in the right direction.

Finally, in our external style guide we version modules like `Nri.Ui.Doodad.V1`—we bump that version avoid releasing major versions all the time, since it's often infeasible to upgrade all our code at once.
This can leave our codebase somewhat fragmented, and forbidding imports of old modules helps us fix that over time.

### How can I get results in my editor?

Most editors can parse something that looks like `filename:row:column:message`.
That's this tool's default output, but if your editor doesn't like the additional message for the human at the bottom, use `--format editor` to remove it.

If your editor requires another form of output, use `--format json` to get structured output which can be reformatted however you like (with, say, [jq](https://stedolan.github.io/jq/).)
If that's not enough, please [let me know](mailto:brian@brianthicks.com).

### Can I check multiple project roots with this tool?

Yep!
Use the `add-root` command.

### Why is this written in Rust instead of `X`?

Well, I wanted to learn Rust.
I thought it'd be a good idea to use `tree-sitter` for parsing here, which has excellent Rust bindings.
That version of the tool turned out to be pretty slow, so I dropped `tree-sitter` in favor of regular expressions (we really just need to look for lines starting with `import`).
Now it's pretty quick!

### How quick is it?

It runs on elm-spa-example (224kb of Elm code) in about 10ms, and my main work repo (12mb of Elm code) in about 200ms.
Most projects fall somewhere in between there, so I'm confident in saying that you probably won't get bored while waiting for this to run, and it won't add an unmanageable amount of overhead to your CI runs.

That said, we could probably go faster.
In particular, the tool does a lot more allocations than it strictly needs to.
But... 10–200ms is well within the acceptable times for a development tool, so I think it's about fast enough.

See [BENCHMARKING.md](BENCHMARKING.md) for more.

If you're an experienced Rust programmer and know about easy further wins here, please [get in touch](mailto:brian@brianthicks.com).

## Contributing

This source code is hosted on git.bytes.zone, my personal git host.
I don't plan on opening up registrations (mostly because I don't want to deal with email configuration or spam.)
To issue bugs, please [email me](mailto:brian@brianthicks.com).

If you have an idea for how to improve elm-forbid-import, please email me and we'll figure it out.
This tool is intentionally limited in scope, but you may have ideas for things that would fit!

If you're a more experienced Rust user, I would also appreciate advice on how to make this code more idiomatic or faster.

### Development Setup

If you're going to work on this codebase, you'll need to set up `nix` and `direnv`.
Then run `direnv allow` in this directory to get all the tools.
If you run into missing dependencies, please let me know and I'll add them to the configuration!

## Climate Action

I want my open-source activities to support projects addressing the climate crisis (for example, projects in clean energy, public transit, reforestation, or sustainable agriculture.)
If you are working on such a project, and find a bug or missing feature in any of my libraries, **please let me know and I will treat your issue as high priority.**
I'd also be happy to support such projects in other ways.
In particular, I've worked with Elm for a long time and would be happy to advise on your implementation.

## License

BSD 3-Clause.
See [LICENSE](LICENSE).
