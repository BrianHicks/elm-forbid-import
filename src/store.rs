use anyhow::{anyhow, Context, Result};
use serde::{Deserialize, Serialize};
use std::collections::{BTreeMap, BTreeSet};
use std::fmt::{self, Display};
use std::fs;
use std::io;
use std::path::PathBuf;

use crate::importfinder;

static AUTOGEN_HEADER: &str = "# WARNING: this file is managed with `elm-forbid-imports`. Manual edits will\n# be overwritten!\n\n";

#[derive(Debug, Deserialize, Serialize)]
pub struct Store {
    #[serde(skip)]
    config_path: PathBuf,

    #[serde(default, skip_serializing_if = "btreeset_is_empty")]
    roots: BTreeSet<PathBuf>,

    #[serde(default, skip_serializing_if = "forbidden_is_empty")]
    forbidden: BTreeMap<String, ForbiddenImport>,
}

#[derive(Debug, Deserialize, Serialize)]
struct ForbiddenImport {
    hint: Option<String>,

    #[serde(default, skip_serializing_if = "btreeset_is_empty")]
    usages: BTreeSet<PathBuf>,
}

fn btreeset_is_empty(roots: &BTreeSet<PathBuf>) -> bool {
    roots.is_empty()
}

fn forbidden_is_empty(forbidden: &BTreeMap<String, ForbiddenImport>) -> bool {
    forbidden.is_empty()
}

impl Store {
    pub fn from_file_or_empty(path: &PathBuf) -> Result<Store> {
        match fs::read(&path) {
            Ok(source) => {
                let mut out: Store = toml::from_slice(&source)
                    .context("could not read TOML from the config file")?;
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

    fn absolute_from_config_path(&self, path: PathBuf) -> Result<PathBuf> {
        match self.config_path.parent() {
            Some(parent) => {
                let out = parent.join(&path).canonicalize().with_context(|| {
                    format!(
                        "could not make an absolute path with the config file and {}",
                        path.display()
                    )
                })?;

                Ok(out)
            }

            None => Err(anyhow!(
                "root config path ({}) does not have a parent.",
                self.config_path.display(),
            )),
        }
    }

    pub fn add_root(&mut self, path: PathBuf) -> Result<()> {
        self.roots.insert(
            self.relative_to_config_path(path)
                .context("could not find a path from the config file to the new root")?,
        );

        Ok(())
    }

    pub fn remove_root(&mut self, path: PathBuf) -> Result<()> {
        self.roots.remove(
            &self
                .relative_to_config_path(path)
                .context("could not find a path from the config file to the root to remove")?,
        );

        Ok(())
    }

    pub fn write(&self) -> Result<()> {
        let serialized =
            toml::to_string_pretty(self).context("could not serialize the store to TOML")?;

        if serialized.is_empty() && self.config_path.exists() {
            fs::remove_file(&self.config_path)
                .context("could not remove the newly-empty config file")?;
        } else {
            fs::write(
                &self.config_path,
                String::from(AUTOGEN_HEADER) + &serialized,
            )
            .context("could not write the new config file to disk")?;
        }

        Ok(())
    }

    pub fn update(&mut self) -> Result<()> {
        let imports_to_files = self
            .scan()
            .context("could not scan the roots for Elm files")?;

        for (import, existing) in self.forbidden.iter_mut() {
            existing.usages = match imports_to_files.get(import) {
                Some(new_usages) => new_usages.to_owned(),
                None => BTreeSet::new(),
            }
        }

        Ok(())
    }

    pub fn check(&mut self) -> Result<Vec<CheckResult>> {
        let imports_to_files = self
            .scan()
            .context("could not scan the roots for Elm files")?;
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

    pub fn scan(&mut self) -> Result<BTreeMap<String, BTreeSet<PathBuf>>> {
        let mut absolute_roots = BTreeSet::new();

        for root in self.roots.iter() {
            absolute_roots.insert(self.absolute_from_config_path(root.to_owned())?);
        }

        let finder = importfinder::ImportFinder::new(absolute_roots);

        finder.find()
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
