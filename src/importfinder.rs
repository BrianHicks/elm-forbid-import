use anyhow::{anyhow, bail, Context, Result};
use crossbeam::channel;
use std::collections::{BTreeMap, BTreeSet};
use std::fs;
use std::path::PathBuf;
use thiserror::Error;

static IMPORT_QUERY: &str = "(import_clause (import) (upper_case_qid)@import)";

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

        let query = tree_sitter::Query::new(get_language(), IMPORT_QUERY)
            .map_err(TreeSitterError::QueryError)
            .context("could not instantiate the import query")?;

        let mut out: BTreeMap<String, BTreeSet<FoundImport>> = BTreeMap::new();

        let (parent_results_sender, results_receiver) = channel::unbounded();
        let (parent_error_sender, error_receiver) = channel::unbounded();

        builder.build_parallel().run(|| {
            let results_sender = parent_results_sender.clone();
            let error_sender = parent_error_sender.clone();

            let mut parser = get_parser().unwrap();

            let query = &query;

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

                let source = match fs::read(dir_entry.path()).context("could not read an Elm file")
                {
                    Ok(s) => s,
                    Err(err) => {
                        #[allow(unused_must_use)]
                        let _ = error_sender.send(err);
                        return ignore::WalkState::Quit;
                    }
                };

                let parsed = match parser.parse(&source, None) {
                    Some(p) => p,
                    None => {
                        #[allow(unused_must_use)]
                        let _ = error_sender
                            .send(anyhow!("could not parse {:}", dir_entry.path().display()));
                        return ignore::WalkState::Quit;
                    }
                };

                let mut cursor = tree_sitter::QueryCursor::new();

                for match_ in cursor.matches(&query, parsed.root_node(), |_| []) {
                    for capture in match_.captures {
                        let import = match capture
                            .node
                            .utf8_text(&source)
                            .context("could not convert a match to a source string")
                        {
                            Ok(i) => i,
                            #[allow(unused_must_use)]
                            Err(err) => {
                                error_sender.send(err);
                                return ignore::WalkState::Quit;
                            }
                        };

                        if let Err(err) = results_sender.send(FoundImport {
                            import: import.to_string(),
                            path: dir_entry.path().to_path_buf(),
                            position: capture.node.start_position(),
                        }) {
                            #[allow(unused_must_use)]
                            let _ = error_sender.send(err.into());
                            return ignore::WalkState::Quit;
                        };
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
    pub position: tree_sitter::Point,
}

// tree sitter

extern "C" {
    fn tree_sitter_elm() -> tree_sitter::Language;
}

fn get_language() -> tree_sitter::Language {
    unsafe { tree_sitter_elm() }
}

fn get_parser() -> Result<tree_sitter::Parser, TreeSitterError> {
    let mut parser = tree_sitter::Parser::new();

    parser
        .set_language(get_language())
        .map_err(TreeSitterError::LanguageError)?;

    Ok(parser)
}

#[derive(Debug, Error)]
enum TreeSitterError {
    #[error("language error: {0}")]
    LanguageError(tree_sitter::LanguageError),

    #[error("query error: {0:?}")]
    QueryError(tree_sitter::QueryError),
}
