use anyhow::{anyhow, bail, Context, Result};
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

        let mut parser = get_parser().context("could not get the Elm parser")?;

        let query = tree_sitter::Query::new(get_language(), IMPORT_QUERY)
            .map_err(TreeSitterError::QueryError)
            .context("could not instantiate the import query")?;

        let mut out: BTreeMap<String, BTreeSet<FoundImport>> = BTreeMap::new();

        for maybe_dir_entry in builder.build() {
            let dir_entry = maybe_dir_entry.context("could not read an entry from a root")?;

            // skip things that aren't files
            if dir_entry.file_type().map(|ft| ft.is_dir()).unwrap_or(true) {
                continue;
            }

            let source = fs::read(dir_entry.path()).context("could not read an Elm file")?;
            let parsed = match parser.parse(&source, None) {
                Some(p) => p,
                None => return Err(anyhow!("could not parse {:}", dir_entry.path().display())),
            };

            let mut cursor = tree_sitter::QueryCursor::new();
            for match_ in cursor.matches(&query, parsed.root_node(), |_| []) {
                for capture in match_.captures {
                    let import = capture
                        .node
                        .utf8_text(&source)
                        .context("could not convert a match to a source string")?;

                    let found_import = FoundImport {
                        path: dir_entry.path().to_path_buf(),
                        position: capture.node.start_position(),
                    };

                    if let Some(paths) = out.get_mut(import) {
                        paths.insert(found_import);
                    } else {
                        let mut paths = BTreeSet::new();
                        paths.insert(found_import);
                        out.insert(import.to_owned(), paths);
                    }
                }
            }
        }

        Ok(out)
    }
}

#[derive(PartialEq, Eq, PartialOrd, Ord)]
pub struct FoundImport {
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
