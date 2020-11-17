use anyhow::Result;
use serde::Serialize;
use std::collections::HashMap;
use std::fs;
use std::path::PathBuf;
use toml;

static AUTOGEN_HEADER: &str = "# WARNING: this file is managed with `elm-forbid-imports`. Manual edits will\n# be overwritten!\n\n";

#[derive(Debug, Serialize)]
pub struct Store {
    forbidden: HashMap<String, ForbiddenImport>,
}

#[derive(Debug, Serialize)]
struct ForbiddenImport {
    hint: Option<String>,
}

impl Store {
    pub fn from_file_or_empty(path: &PathBuf) -> Store {
        Store {
            forbidden: HashMap::new(),
        }
    }

    pub fn forbid(&mut self, name: String, hint: Option<String>) {
        self.forbidden.insert(name, ForbiddenImport { hint });
    }

    pub fn write(&self, path: &PathBuf) -> Result<()> {
        let serialized = toml::to_string(self)?;
        fs::write(path, String::from(AUTOGEN_HEADER) + &serialized)?;

        Ok(())
    }
}
