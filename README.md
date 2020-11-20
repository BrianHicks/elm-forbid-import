# elm-forbid-imports

This tool is based on the realization that we often intend to deprecate some import (say, to upgrade from `Regex` to `Parser` or from `Html` to `Html.Styled`) but we can't do it all *right now*—these can be big projects!

Fortunately, we can work on these kinds of projects a little bit at a time!
But on the other hand… it's too easy to `import Html` in the meantime without remembering, especially if you're working on a big team.

What to do?
Enter `elm-forbid-imports`, which lets you mark current imports of a module as allowed, but will forbid any future imports.
This lets you whittle away at a deprecated import until you can finally remove it from your `elm.json`.

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
...
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
However, if you add (or remove) more you'll be prompted to either remove the imports or accept them with `update`.

All this will create a `forbidden-imports.toml` file in the current directory (you can control this name and location with `--config` or by setting `ELM_FORBID_IMPORT_CONFIG`.)
You should check this file in!
Doing so means that you can run `elm-forbid-import check` in your CI setup so that you and your team are reminded to keep the list up-to-date.

## Install

This isn't packaged in a way you can just download a binary yet.
But that's fine, you can use [`nix`](https://nixos.org/download.html) to install:

```
nix-env -if https://git.bytes.zone/brian/elm-forbid-import/archive/main.tar.gz
```

If you don't want to install globally, check out this repo and run `nix-build` inside it.
If you've got a rust toolchain set up, `cargo build` in the root directory should also work.

## Roadmap

- [x] Persist other parts of the config, like the roots.
- [x] Parallel crawling in large projects
- [x] Look in all the `source-directories` in any specified `elm.json`s
- [x] Add golden tests to cover usage patterns
- [x] Make sure the README looks really nice
- [ ] Figure out licensing
- [ ] Release 1.0! (or 0.1, whatever)

## Frequently Asked Questions

### How can I check multiple source roots with this tool?

Use the `add-root` command.

### How is this better than `grep`?

If you're OK with removing a dependency thoroughly and never allowing any wiggle room for it to come back, a simple shell script might be more appropriate for your use. Here you go:

```
if grep -re 'import Regex' -e 'import OtherThing' src; then
  echo 'Forbidden imports found oh noooooo'
  exit 1
fi
```

But if you're working on a large project where you're managing or trying to reduce dependencies, this tool will allow you to do so without making other folks on your team pull their hair out when CI fails for the bajillionth time.

### How common is forbidding an import, really?

We do it pretty frequently at NoRedInk.

First of all, we use `Html.Styled` everywhere we can, so `Html` is forbidden.
There are a couple little modules like that, for example all the stuff under `Html` like `Html.Events`, etc.

Second, we've moved from an internal styleguide to an [external one](https://github.com/NoRedInk/noredink-ui) over time, which means that we forbid imports of all the previous style guide stuff we haven't made the time to migrate yet.
This keeps us moving in the right direction towards consolidating into a single style guide.

Finally, in our external style guide we version modules like `Nri.Ui.Doodad.V1`—we bump that to `V2` or whatever to avoid releasing major versions all the time, since doing so would be pretty painful with the size of our codebase.

### Why is this written in Rust instead of `X`?

I wanted to learn Rust, and I started off by using `tree-sitter`, which has excellent Rust bindings.
That version of the tool turned out to be pretty slow, so I dropped it in favor of regular expressions.
Now it's pretty quick!

### Is this fast?

It's pretty fast.
It could probably go faster, but I think we've reached the point of diminishing returns here.
See [BENCHMARKING.md](BENCHMARKING.md).

## Contributing

This source code is hosted on git.bytes.zone, my personal git host.
I don't plan on opening up registrations (mostly because I don't want to deal with spam or email configuration.)
To issue bugs, please [email me](mailto:brian@brianthicks.com).

If you have an idea for how to improve elm-forbid-import, please email me and we'll figure it out.
This tool is intentionally limited in scope, but you may have ideas for things that would fit!

If you're a more experience Rust user, I would also appreciate pointers of how to make this code more idiomatic or faster.

## Climate Action

I want my open-source activities to support projects addressing the climate crisis (for example, projects in clean energy, public transit, reforestation, or sustainable agriculture.)
If you are working on such a project, and find a bug or missing feature in any of my libraries, **please let me know and I will treat your issue as high priority.**
I'd also be happy to support such projects in other ways.
In particular, I've worked with Elm for a long time and would be happy to advise on your implementation.

## License

TODO. Probably BSD 3-Clause to match the rest of the Elm community stuff, but I have to make sure that's compatible with the dependencies I'm using.
