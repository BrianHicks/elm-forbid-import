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
                "config path ({}) does not have a parent.",
                self.config_path.display()
            )),
        }
    }

    fn absolute_config_parent_path(&self) -> Result<PathBuf> {
        match self.config_path.parent() {
            Some(parent) => {
                if parent.as_os_str().is_empty() {
                    std::env::current_dir().context("could not get the current working directory")
                } else {
                    parent
                        .canonicalize()
                        .context("could not make an absolute path to the config's parent directory")
                }
            }

            None => Err(anyhow!(
                "config path ({}) does not have a parent.",
                self.config_path.display(),
            )),
        }
    }

    fn absolute_from_config_path(&self, path: PathBuf) -> Result<PathBuf> {
        self.absolute_config_parent_path()?
            .join(&path)
            .canonicalize()
            .with_context(|| {
                format!(
                    "could not make an absolute path with the config file and {}",
                    path.display()
                )
            })
    }

    pub fn add_root(&mut self, path: PathBuf) -> Result<()> {
        self.roots.insert(
            self.relative_to_config_path(path)
                .context("could not find a path from the config file to the new project root")?,
        );

        Ok(())
    }

    pub fn remove_root(&mut self, path: PathBuf) -> Result<()> {
        self.roots.remove(
            &self.relative_to_config_path(path).context(
                "could not find a path from the config file to the project root to remove",
            )?,
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
            .context("could not scan the project roots for Elm files")?;

        let parent_path = self
            .absolute_config_parent_path()
            .context("could not get parent path to write new usages")?;

        for (import, value) in self.forbidden.iter_mut() {
            if let Some(new_usages) = imports_to_files.get(import) {
                value.usages = new_usages
                    .iter()
                    .flat_map(|usage| pathdiff::diff_paths(&usage.path, &parent_path))
                    .collect()
            };
        }

        Ok(())
    }

    pub fn check(&mut self) -> Result<Vec<CheckResult>> {
        let imports_to_files = self
            .scan()
            .context("could not scan the project roots for Elm files")?;
        let mut out = Vec::new();

        for (import, existing) in self.forbidden.iter() {
            if let Some(found_imports) = imports_to_files.get(import) {
                let new_usages = found_imports
                    .iter()
                    .map(|found| found.path.to_owned())
                    .collect::<BTreeSet<PathBuf>>();

                let mut to_positions: BTreeMap<&PathBuf, importfinder::Position> = BTreeMap::new();

                for import in found_imports.iter() {
                    to_positions.insert(&import.path, import.position);
                }

                for file in new_usages.difference(&existing.usages) {
                    out.push(CheckResult {
                        path: file.to_path_buf(),
                        position: to_positions.get(file).copied(),
                        import: import.to_string(),
                        error_location: ErrorLocation::InElmSource {
                            hint: existing.hint.as_ref(),
                        },
                    });
                }

                for file in existing.usages.difference(&new_usages) {
                    out.push(CheckResult {
                        path: file.to_path_buf(),
                        position: None,
                        import: import.to_string(),
                        error_location: ErrorLocation::InConfig,
                    })
                }
            }
        }

        Ok(out)
    }

    pub fn scan(&self) -> Result<BTreeMap<String, BTreeSet<importfinder::FoundImport>>> {
        let mut absolute_roots = BTreeSet::new();

        for root in self.roots.iter() {
            absolute_roots.insert(self.absolute_from_config_path(root.to_owned())?);
        }

        if absolute_roots.is_empty() {
            absolute_roots
                .insert(std::env::current_dir().context("could not get current directory")?);
        }

        let finder = importfinder::ImportFinder::new(absolute_roots);

        finder.find()
    }
}

#[derive(Debug, Serialize)]
pub struct CheckResult<'a> {
    path: PathBuf,
    position: Option<importfinder::Position>,
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

impl CheckResult<'_> {
    pub fn relative_path(&self) -> PathBuf {
        std::env::current_dir()
            .ok()
            .and_then(|cwd| pathdiff::diff_paths(&self.path, &cwd))
            .unwrap_or_else(|| self.path.to_owned())
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

                let position_string = match &self.position {
                    Some(position) => format!(":{}:{}", position.row, position.column),
                    None => String::new(),
                };

                write!(
                    f,
                    "{}{}:forbidden import {}{}",
                    self.relative_path().display(),
                    position_string,
                    self.import,
                    hint_string,
                )
            }
            ErrorLocation::InConfig => write!(
                f,
                "{}: removed forbidden import {}! (Run me with `update` to fix this.)",
                self.relative_path().display(),
                self.import,
            ),
        }
    }
}
