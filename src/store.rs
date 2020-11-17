use std::collections::HashMap;
use std::path::PathBuf;

#[derive(Debug)]
pub struct Store {
    path: PathBuf,
    forbidden_imports: HashMap<String, Option<String>>,
}

impl Store {
    pub fn from_file_or_empty(path: PathBuf) -> Store {
        Store {
            path: path,
            forbidden_imports: HashMap::new(),
        }
    }

    pub fn forbid(&mut self, name: String, hint: Option<String>) {
        self.forbidden_imports.insert(name, hint);
    }
}
