use anyhow::{anyhow, Result};
use serde::{Deserialize, Serialize};
use std::collections::BTreeMap;
use std::fs;
use std::io;
use std::path::PathBuf;
use toml;

static AUTOGEN_HEADER: &str = "# WARNING: this file is managed with `elm-forbid-imports`. Manual edits will\n# be overwritten!\n\n";

#[derive(Debug, Deserialize, Serialize)]
pub struct Store {
    forbidden: BTreeMap<String, ForbiddenImport>,
}

#[derive(Debug, Deserialize, Serialize)]
struct ForbiddenImport {
    hint: Option<String>,

    #[serde(default)]
    usages: Vec<PathBuf>,
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
                    usages: Vec::new(),
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
}
