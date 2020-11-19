# elm-forbid-imports

Forbid imports across an entire Elm project.

## Usage

This tool is based on the realization that we often *intend* to deprecate some import (say, to upgrade `Regex` to `Parser` or `Html` to `Html.Styled`) but you can't do it *now*, or even *completely*.
To help, we keep a list of modules you're trying to remove from your project and a list of places where they're still used.

Then you can stick a line that says `elm-forbid-import check` line into your CI config to get reminders to not add any new instances, and to regenerate the list when you remove old ones.

So, let's get started!
To forbid an import (say, `Regex`), run:

```sh
$ elm-forbid-import forbid Regex
```

Now `Regex` is forbidden in your project!

Let's see what needs work:

```sh
$ elm-forbid-import check
```

... and get to work!

When you're done, or can't remove any more, update the list of usages:

```sh
$ elm-forbid-import update
```

You can also run `update` if you really need to add another instance of your forbidden import (for example, if you're short on time and just need CI to pass.)

## Install

This isn't packaged in a way you can just download a binary yet.
But that's fine, you can use [`nix`](https://nixos.org/download.html) to install.

```
nix-env -if https://git.bytes.zone/brian/elm-forbid-import/archive/main.tar.gz
```

If you don't want to install globally, check out this repo and run `nix-build` inside it.
If you've got a rust toolchain set up, `cargo build` in the root directory should also work.

## Roadmap

- [x] Persist other parts of the config, like the roots.
- [x] Parallel crawling in large projects
- [ ] Look in all the `source-directories` in any specified `elm.json`s
- [ ] Add golden tests to cover usage patterns
- [ ] Make sure the README looks really nice
- [ ] Figure out licensing
- [ ] Release 1.0! (or 0.1, whatever)

## Frequently Asked Questions

### How is this better than `grep`?

If you're OK with removing a dependency thoroughly and never allowing any wiggle room for it to come back, a simple shell script will probably be more appropriate for your use. Here you go:

```
if grep -re 'import Regex' -e 'import OtherThing' src; then
  echo 'Forbidden imports found oh noooooo'
  exit 1
fi
```

But if you're working on a large project where you're managing or trying to reduce dependencies, this tool will allow you to do so without making other folks on your team pull their hair out when CI fails for the bajillionth time.

### How common is forbidding an import, really?

We do it pretty frequently at NoRedInk.

First of all, we have a package called `noredink-ui` which is designed with forward-compatibility in mind.
All the modules are named like `Nri.Ui.Doodad.V1`, so when we need to make an API-incompatible change, we add a new module `V2` under that hierarchy and make a minor release.
Next, we forbid new imports of `V1` and work to upgrade existing ones.
When that's done, we'll make the occasional major release to clean up unused doodads.
It works pretty well!
Since we have such a huge codebase and a big team, we use tools like this to communicate intent.

Aside from our unusual naming convention, we've also found it helpful to deprecate modules which we have available but don't want to use.
For example, we forbid `Html` except in `Main` files where we need it in type signatures (we use `Html.Styled` from elm-css instead.)

## License

TODO. Probably BSD 3-Clause to match the rest of the Elm community stuff, but I have to make sure that's compatible with the dependencies I'm using.
