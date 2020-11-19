use anyhow::{bail, Context, Result};
use crossbeam::channel;
use lazy_static::lazy_static;
use regex::Regex;
use serde::Serialize;
use std::collections::{BTreeMap, BTreeSet};
use std::fs;
use std::path::PathBuf;

pub struct ImportFinder {
    roots: BTreeSet<PathBuf>,
}

impl ImportFinder {
    pub fn new(roots: BTreeSet<PathBuf>) -> ImportFinder {
        ImportFinder { roots }
    }

    pub fn find(&self) -> Result<BTreeMap<String, BTreeSet<FoundImport>>> {
        let mut roots = self.roots.iter();

        let first_root = match roots.next() {
            None => bail!("could not find imports, because there were no roots to examine"),
            Some(root) => root,
        };

        let mut builder = ignore::WalkBuilder::new(first_root);
        for root in roots {
            builder.add(root);
        }

        builder.standard_filters(true);

        let types = ignore::types::TypesBuilder::new()
            .add_defaults()
            .select("elm")
            .build()
            .context("could not build extensions to scan for")?;
        builder.types(types);

        let mut out: BTreeMap<String, BTreeSet<FoundImport>> = BTreeMap::new();

        let (parent_results_sender, results_receiver) = channel::unbounded();
        let (parent_error_sender, error_receiver) = channel::unbounded();

        builder.build_parallel().run(|| {
            let results_sender = parent_results_sender.clone();
            let error_sender = parent_error_sender.clone();

            Box::new(move |maybe_dir_entry| {
                let dir_entry = match maybe_dir_entry.context("could not read an entry from a root")
                {
                    Ok(de) => de,
                    Err(err) => {
                        #[allow(unused_must_use)]
                        let _ = error_sender.send(err);
                        return ignore::WalkState::Quit;
                    }
                };

                // skip things that aren't files
                if dir_entry.file_type().map(|ft| ft.is_dir()).unwrap_or(true) {
                    return ignore::WalkState::Continue;
                }

                let source_bytes =
                    match fs::read(dir_entry.path()).context("could not read an Elm file") {
                        Ok(s) => s,
                        Err(err) => {
                            #[allow(unused_must_use)]
                            let _ = error_sender.send(err);
                            return ignore::WalkState::Quit;
                        }
                    };

                let source = match std::str::from_utf8(&source_bytes)
                    .context("could not read the source as utf8")
                {
                    Ok(s) => s,
                    Err(err) => {
                        #[allow(unused_must_use)]
                        let _ = error_sender.send(err);
                        return ignore::WalkState::Quit;
                    }
                };

                lazy_static! {
                    // TODO: maybe faster to use `([^ ]+)` for the match
                    static ref IMPORT_RE: Regex = Regex::new(r"^import +([A-Z][A-Za-z0-9_\.]*)").unwrap();
                }

                // perf idea; keep track of if we've finished the import list and
                // bail on any further lines once we get there. Since imports
                // are forbidden after the block at the top of the module, we
                // shouldn't miss anything by skipping the rest of the lines
                // in each file!

                for (line_number, line) in source.lines().enumerate() {
                    if let Some(import_module) = IMPORT_RE.captures(line).and_then(|m| m.get(1)) {
                        if let Err(err) = results_sender.send(FoundImport {
                            path: dir_entry.path().to_path_buf(),
                            import: import_module.as_str().to_string(),
                            position: Position {
                                row: line_number + 1,
                                column: import_module.start(),
                            },
                        }) {
                            #[allow(unused_must_use)]
                            let _ = error_sender.send(err.into());
                            return ignore::WalkState::Quit;
                        }
                    }
                }

                ignore::WalkState::Continue
            })
        });

        // the sources for the clones in the parallel worker threads have
        // to be dropped or we'll block forever! Oh no!
        drop(parent_results_sender);
        drop(parent_error_sender);

        if let Some(err) = error_receiver.iter().next() {
            return Err(err);
        }

        for result in results_receiver {
            match out.get_mut(&result.import) {
                Some(value) => {
                    value.insert(result);
                }
                None => {
                    let key = result.import.to_string();

                    let mut value = BTreeSet::new();
                    value.insert(result);

                    out.insert(key, value);
                }
            }
        }

        Ok(out)
    }
}

#[derive(Debug, PartialEq, Eq, PartialOrd, Ord)]
pub struct FoundImport {
    pub import: String,
    pub path: PathBuf,
    pub position: Position,
}

#[derive(Debug, Copy, Clone, PartialEq, Eq, PartialOrd, Ord, Serialize)]
pub struct Position {
    pub row: usize,
    pub column: usize,
}
