# elm-forbid-import

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

## Q&A

### How can I get these results in my editor?

Most editors can parse something that looks like `filename:row:column:message`.
That's this tool's default output, but if your editor doesn't like the additional message for the human at the bottom, use `--format editor` to get *only* that output.

If your editor requires another form of output, use `--format json` to get structured output which can be reformatted however you like.
If that's not enough, please [let me know](mailto:brian@brianthicks.com).

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
That version of the tool turned out to be pretty slow, so I dropped `tree-sitter` in favor of regular expressions.
Now it's pretty quick!

### How quick is it?

It runs on elm-spa-example (224kb of Elm code) in about 10ms, and my main work repo (12mb of Elm code) in about 200ms.
Most projects fall somewhere in between there, so I'm confident in saying that you probably won't get bored while waiting for this to run, and it won't add an unmanageable amount of overhead to your CI runs.

That said, wec could probably go faster; in particular, the tool does a lot more allocations than it strictly needs to.
But... 10—200ms is well within the acceptable times for a development tool, so I think it's about fast enough.
See [BENCHMARKING.md](BENCHMARKING.md) for more and how it got to this speed.

If you're an experienced Rust programmer and know about easy further wins here, please [get in touch](mailto:brian@brianthicks.com).

## Contributing

This source code is hosted on git.bytes.zone, my personal git host.
I don't plan on opening up registrations (mostly because I don't want to deal with spam or email configuration.)
To issue bugs, please [email me](mailto:brian@brianthicks.com).

If you have an idea for how to improve elm-forbid-import, please email me and we'll figure it out.
This tool is intentionally limited in scope, but you may have ideas for things that would fit!

If you're a more experience Rust user, I would also appreciate pointers of how to make this code more idiomatic or faster.

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
