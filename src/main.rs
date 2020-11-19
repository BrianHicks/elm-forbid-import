use anyhow::{Context, Result};
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
    #[clap(short('c'), long("config"), default_value = "forbidden-imports.toml")]
    config_path: PathBuf,

    /// How do you want the results presented? Only really useful if you
    /// want JSON output to reformat for an external system.
    #[clap(long, default_value = "human")]
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

    /// Stop forbidding the use of a specific import.
    Unforbid {
        /// The fully-qualified name to forbid (e.g. `Html.Events`)
        name: String,
    },

    /// Add a project root (a directory containing `elm.json`) to check for imports.
    AddRoot {
        // The path of the directory, as relative to the working directory.
        path: PathBuf,
    },

    /// Remove a root from checking.
    RemoveRoot {
        // The path of the root, as relative to the working directory.
        path: PathBuf,
    },

    /// Update the allowed imports list
    Update,

    /// Check what imports still need to be cleaned up
    Check,
}

#[derive(Debug)]
enum Format {
    Human,
    JSON,
}

impl std::str::FromStr for Format {
    type Err = BadFormat;

    fn from_str(input: &str) -> Result<Format, BadFormat> {
        match input {
            "human" => Ok(Format::Human),
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

        Mode::Unforbid { name } => {
            store.unforbid(name);
            store.write().context("could not update the config file")?;

            Ok(0)
        }

        Mode::AddRoot { path } => {
            store.add_root(path).context("could not add the new root")?;
            store.write().context("could not update the config file")?;

            Ok(0)
        }

        Mode::RemoveRoot { path } => {
            store
                .remove_root(path)
                .context("could not remove the root")?;
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
                Format::Human => {
                    let all_in_config =
                        !results.is_empty() && results.iter().all(|item| item.error_is_in_config());

                    for result in &results {
                        println!("{}", result);
                    }

                    if all_in_config {
                        println!(
                    "\nIt looks like you removed some forbidden imports. Good job! To update the config\nand remove this error, just run me with the `update` command!"
                );
                        Ok(1)
                    } else if !results.is_empty() {
                        println!(
                    "\nIf these are too much to handle right now (or you intended to import a forbidden\nmodule), please run me with the `update` command!"
                );
                        Ok(1)
                    } else {
                        Ok(0)
                    }
                }
            }
        }
    }
}
