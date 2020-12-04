use anyhow::{anyhow, Context, Result};
use clap::Clap;
use std::path::PathBuf;
use std::process;
use store::Store;

mod importfinder;
mod store;

#[derive(Debug, Clap)]
struct Options {
    /// The file where we'll store configuration about forbidden imports
    /// and todos.
    #[clap(
        short('c'),
        long("config"),
        env("ELM_FORBID_IMPORT_CONFIG"),
        default_value = "forbidden-imports.toml"
    )]
    config_path: PathBuf,

    /// How do you want the results presented? Only really useful if you're a
    /// computer calling this script. If you're an editor, try the `editor` for
    /// line info without the human-readable action message at the bottom. If
    /// you're not, try the `json` output. Delicious!
    #[clap(long, env("ELM_FORBID_IMPORT_FORMAT"), default_value = "human")]
    format: Format,

    #[clap(subcommand)]
    mode: Mode,
}

#[derive(Debug, Clap)]
enum Mode {
    /// Forbid a new import
    Forbid {
        /// The fully-qualified name to forbid (e.g. `Html.Events`)
        name: String,

        /// An additional string to print when showing an error for this
        /// import. (idea: what should we do instead? Import something else? Use
        /// another approach? Give up and buy a farm?)
        #[clap(short, long)]
        hint: Option<String>,
    },

    /// Forbid a list of imports held in a CSV. The file should be a 2-column
    /// CSV with "module" and "hint" fields and no headers.
    ForbidFromCsv {
        /// What file has the forbidden import list?
        path: PathBuf,
    },

    /// Stop forbidding the use of a specific import.
    Unforbid {
        /// The fully-qualified name to forbid (e.g. `Html.Events`)
        name: String,
    },

    /// Add a project root (a directory containing `elm.json`) to check for imports.
    AddRoot {
        // The path to the project, as relative to the working directory.
        path: PathBuf,
    },

    /// Remove a project root from checking.
    RemoveRoot {
        // The path to the project, as relative to the working directory.
        path: PathBuf,
    },

    /// Update the allowed imports list
    Update,

    /// Check what imports still need to be cleaned up
    Check,
}

#[derive(Debug, PartialEq)]
enum Format {
    Human,
    Editor,
    JSON,
}

impl std::str::FromStr for Format {
    type Err = BadFormat;

    fn from_str(input: &str) -> Result<Format, BadFormat> {
        match input {
            "human" => Ok(Format::Human),
            "editor" => Ok(Format::Editor),
            "json" => Ok(Format::JSON),
            _ => Err(BadFormat {}),
        }
    }
}

#[derive(Debug)]
struct BadFormat {}

impl ToString for BadFormat {
    fn to_string(&self) -> String {
        String::from("bad format")
    }
}

fn main() {
    let opts = Options::parse();

    match run(opts) {
        Ok(exit_code) => process::exit(exit_code),
        Err(err) => {
            eprintln!("{:?}", err);
            process::exit(1);
        }
    }
}

fn run(opts: Options) -> Result<i32> {
    let mut store = Store::from_file_or_empty(&opts.config_path).with_context(|| {
        format!(
            "could not load the config at {}",
            &opts.config_path.display()
        )
    })?;

    match opts.mode {
        Mode::Forbid { name, hint } => {
            store.forbid(name, hint);
            store.write().context("could not update the config file")?;

            Ok(0)
        }

        Mode::ForbidFromCsv { path } => {
            let mut reader = csv::ReaderBuilder::new()
                .flexible(true)
                .has_headers(false)
                .from_path(path)
                .context("could not read the CSV of forbidden imports")?;

            for record in reader.records() {
                let mut record = record.context("could not read record")?;
                record.trim();

                let module = record
                    .get(0)
                    .and_then(|name| if !name.is_empty() { Some(name) } else { None })
                    .map(|name| name.to_string())
                    .ok_or(anyhow!(
                        "I need a module name in the first column of the CSV at "
                    ))?;
                let hint = record.get(1).map(|name| name.to_string());

                store.forbid(module, hint);
            }

            store.write().context("could not update the config file")?;

            Ok(0)
        }

        Mode::Unforbid { name } => {
            store.unforbid(name);
            store.write().context("could not update the config file")?;

            Ok(0)
        }

        Mode::AddRoot { path } => {
            store
                .add_root(path)
                .context("could not add the new project root")?;
            store.write().context("could not update the config file")?;

            Ok(0)
        }

        Mode::RemoveRoot { path } => {
            store
                .remove_root(path)
                .context("could not remove the project root")?;
            store.write().context("could not update the config file")?;

            Ok(0)
        }

        Mode::Update => {
            store
                .update()
                .context("could not update usage information")?;
            store.write().context("could not update the config file")?;

            Ok(0)
        }

        Mode::Check => {
            let results = store
                .check()
                .context("could not check for forbidden imports")?;

            match opts.format {
                Format::JSON => {
                    println!(
                        "{}",
                        serde_json::to_string(&results)
                            .context("when formatting results as JSON")?
                    );
                    if results.is_empty() {
                        Ok(0)
                    } else {
                        Ok(1)
                    }
                }
                _ => {
                    let all_in_config =
                        !results.is_empty() && results.iter().all(|item| item.error_is_in_config());

                    for result in &results {
                        println!("{}", result);
                    }

                    if opts.format == Format::Human {
                        if all_in_config {
                            println!( "\nIt looks like you removed some forbidden imports. Good job! To update the config\nand remove this error, just run me with the `update` command!" );
                        } else if !results.is_empty() {
                            println!( "\nIf these are too much to handle right now (or you intended to import a forbidden\nmodule), please run me with the `update` command!" );
                        }
                    }

                    if !results.is_empty() {
                        Ok(1)
                    } else {
                        Ok(0)
                    }
                }
            }
        }
    }
}
