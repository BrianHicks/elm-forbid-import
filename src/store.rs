use anyhow::{anyhow, Result};
use serde::{Deserialize, Serialize};
use std::collections::{BTreeMap, BTreeSet};
use std::fmt::{self, Display};
use std::fs;
use std::io;
use std::path::PathBuf;
use thiserror::Error;

static AUTOGEN_HEADER: &str = "# WARNING: this file is managed with `elm-forbid-imports`. Manual edits will\n# be overwritten!\n\n";

static IMPORT_QUERY: &str = "(import_clause (import) (upper_case_qid)@import)";

#[derive(Debug, Deserialize, Serialize)]
pub struct Store {
    #[serde(skip)]
    config_path: PathBuf,

    #[serde(default)]
    roots: BTreeSet<PathBuf>,

    #[serde(default)]
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
        match fs::read(&path) {
            Ok(source) => {
                let mut out: Store = toml::from_slice(&source)?;
                out.config_path = path.to_owned();
                Ok(out)
            }

            Err(err) => match err.kind() {
                io::ErrorKind::NotFound => Ok(Store {
                    config_path: path.to_owned(),
                    roots: BTreeSet::new(),
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

    fn relative_to_config_path(&self, path: PathBuf) -> Result<PathBuf> {
        match self.config_path.parent() {
            Some(parent) => match pathdiff::diff_paths(&path.to_owned(), parent) {
                Some(relative) => Ok(relative),
                None => Err(anyhow!(
                    "could not compute a relative path between {} and {}",
                    self.config_path.display(),
                    path.display()
                )),
            },
            None => Err(anyhow!(
                "root config path ({}) does not have a parent.",
                self.config_path.display()
            )),
        }
    }
    pub fn add_root(&mut self, path: PathBuf) -> Result<()> {
        self.roots.insert(self.relative_to_config_path(path)?);

        Ok(())
    }

    pub fn remove_root(&mut self, path: PathBuf) -> Result<()> {
        self.roots.remove(&self.relative_to_config_path(path)?);

        Ok(())
    }

    pub fn write(&self) -> Result<()> {
        let serialized = toml::to_string_pretty(self)?;
        fs::write(
            &self.config_path,
            String::from(AUTOGEN_HEADER) + &serialized,
        )?;

        Ok(())
    }

    pub fn update(&mut self, root: PathBuf) -> Result<()> {
        let imports_to_files = self.scan(root)?;

        for (import, existing) in self.forbidden.iter_mut() {
            existing.usages = match imports_to_files.get(import) {
                Some(new_usages) => new_usages.to_owned(),
                None => BTreeSet::new(),
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
                        error_location: ErrorLocation::InElmSource {
                            hint: existing.hint.as_ref(),
                        },
                    });
                }

                for file in existing.usages.difference(new_usages) {
                    out.push(CheckResult {
                        file: file.to_path_buf(),
                        import: import.to_string(),
                        error_location: ErrorLocation::InConfig,
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
                        out.insert(import.to_owned(), paths);
                    }
                }
            }
        }

        Ok(out)
    }
}

#[derive(Debug, Serialize)]
pub struct CheckResult<'a> {
    file: PathBuf,
    import: String,
    error_location: ErrorLocation<'a>,
}

#[derive(Debug, PartialEq, Serialize)]
#[serde(rename_all = "snake_case")]
enum ErrorLocation<'a> {
    InElmSource { hint: Option<&'a String> },
    InConfig,
}

impl CheckResult<'_> {
    pub fn error_is_in_config(&self) -> bool {
        self.error_location == ErrorLocation::InConfig
    }
}

impl Display for CheckResult<'_> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self.error_location {
            ErrorLocation::InElmSource { hint } => {
                let hint_string = match hint {
                    Some(an_actual_hint) => format!(" ({})", an_actual_hint),
                    None => String::new(),
                };
                write!(
                    f,
                    "{} imports {}{}",
                    self.file.to_str().unwrap_or("<unprintable file path>"),
                    self.import,
                    hint_string,
                )
            }
            ErrorLocation::InConfig => write!(
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
