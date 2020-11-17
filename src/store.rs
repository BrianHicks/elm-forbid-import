use anyhow::{anyhow, Result};
use serde::{Deserialize, Serialize};
use std::collections::{BTreeMap, BTreeSet};
use std::fmt::{self, Display};
use std::fs;
use std::io;
use std::path::PathBuf;
use thiserror::Error;
use toml;

static AUTOGEN_HEADER: &str = "# WARNING: this file is managed with `elm-forbid-imports`. Manual edits will\n# be overwritten!\n\n";

static IMPORT_QUERY: &str = "(import_clause (import) (upper_case_qid)@import)";

#[derive(Debug, Deserialize, Serialize)]
pub struct Store {
    forbidden: BTreeMap<String, ForbiddenImport>,
}

#[derive(Debug, Deserialize, Serialize)]
struct ForbiddenImport {
    hint: Option<String>,

    #[serde(default)]
    usages: BTreeSet<PathBuf>,
}

impl Store {
    pub fn from_file_or_empty(path: &PathBuf) -> Result<Store> {
        match fs::read(path) {
            Ok(source) => {
                let out: Store = toml::from_slice(&source)?;
                Ok(out)
            }

            Err(err) => match err.kind() {
                io::ErrorKind::NotFound => Ok(Store {
                    forbidden: BTreeMap::new(),
                }),
                _ => Err(anyhow!(err)),
            },
        }
    }

    pub fn forbid(&mut self, name: String, hint: Option<String>) {
        if let Some(value) = self.forbidden.get_mut(&name) {
            value.hint = hint
        } else {
            self.forbidden.insert(
                name,
                ForbiddenImport {
                    hint,
                    usages: BTreeSet::new(),
                },
            );
        };
    }

    pub fn unforbid(&mut self, name: String) {
        self.forbidden.remove(&name);
    }

    pub fn write(&self, path: &PathBuf) -> Result<()> {
        let serialized = toml::to_string(self)?;
        fs::write(path, String::from(AUTOGEN_HEADER) + &serialized)?;

        Ok(())
    }

    pub fn update(&mut self, root: PathBuf) -> Result<()> {
        let imports_to_files = self.scan(root)?;

        for (import, existing) in self.forbidden.iter_mut() {
            if let Some(new_usages) = imports_to_files.get(import) {
                existing.usages = new_usages.to_owned();
            }
        }

        Ok(())
    }

    pub fn check(&mut self, root: PathBuf) -> Result<Vec<CheckResult>> {
        let imports_to_files = self.scan(root)?;
        let mut out = Vec::new();

        for (import, existing) in self.forbidden.iter() {
            if let Some(new_usages) = imports_to_files.get(import) {
                for file in new_usages.difference(&existing.usages) {
                    out.push(CheckResult {
                        file: file.to_path_buf(),
                        import: import.to_string(),
                        error_location: ErrorLocation::InElmSource,
                    });
                }

                for file in existing.usages.difference(new_usages) {
                    out.push(CheckResult {
                        file: file.to_path_buf(),
                        import: import.to_string(),
                        error_location: ErrorLocation::InForbiddenImportsConfig,
                    })
                }
            }
        }

        Ok(out)
    }

    pub fn scan(&mut self, root: PathBuf) -> Result<BTreeMap<String, BTreeSet<PathBuf>>> {
        let types = ignore::types::TypesBuilder::new()
            .add_defaults()
            .select("elm")
            .build()?;

        let mut parser = get_parser()?;

        let query = tree_sitter::Query::new(get_language(), IMPORT_QUERY)
            .map_err(TreeSitterError::QueryError)?;

        let mut out: BTreeMap<String, BTreeSet<PathBuf>> = BTreeMap::new();

        let walker = ignore::WalkBuilder::new(root)
            .types(types)
            .standard_filters(true)
            .build();

        for maybe_dir_entry in walker {
            let dir_entry = maybe_dir_entry?;

            // skip things that aren't files
            if dir_entry.file_type().map(|ft| ft.is_dir()).unwrap_or(true) {
                continue;
            }

            let source = fs::read(dir_entry.path())?;
            let parsed = match parser.parse(&source, None) {
                Some(p) => p,
                None => return Err(anyhow!("could not parse {:}", dir_entry.path().display())),
            };

            let mut cursor = tree_sitter::QueryCursor::new();
            for match_ in cursor.matches(&query, parsed.root_node(), |_| []) {
                for capture in match_.captures {
                    let import = capture.node.utf8_text(&source)?;

                    if let Some(paths) = out.get_mut(import) {
                        paths.insert(dir_entry.path().to_path_buf());
                    } else {
                        let mut paths = BTreeSet::new();
                        paths.insert(dir_entry.path().to_path_buf());
                        out.insert((&import).to_string(), paths);
                    }
                }
            }
        }

        Ok(out)
    }
}

#[derive(Debug)]
pub struct CheckResult {
    file: PathBuf,
    import: String,
    error_location: ErrorLocation,
}

#[derive(Debug)]
enum ErrorLocation {
    InElmSource,
    InForbiddenImportsConfig,
}

impl Display for CheckResult {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self.error_location {
            ErrorLocation::InElmSource => write!(
                f,
                "{} imports {}",
                self.file.to_str().unwrap_or("<unprintable file path>"),
                self.import,
            ),
            ErrorLocation::InForbiddenImportsConfig => write!(
                f,
                "{} used to import {}, but no longer!",
                self.file.to_str().unwrap_or("<unprintable file path>"),
                self.import,
            ),
        }
    }
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
