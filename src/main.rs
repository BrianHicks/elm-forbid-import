use anyhow::Result;
use clap::Clap;
use std::path::PathBuf;
use std::process;
use store::Store;

mod store;

#[derive(Debug, Clap)]
struct Options {
    /// The path to an Elm project (a folder containing elm.json) where
    /// we'll start looking for imports.
    #[clap(short, long, default_value = ".")]
    root: PathBuf,

    /// The file where we'll store configuration about forbidden imports
    /// and todos.
    #[clap(short('c'), long("config"), default_value = "forbidden-imports.toml")]
    config_path: PathBuf,

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

    /// Update the allowed imports list
    Update,

    /// Check what imports still need to be cleaned up
    Check,
}

fn main() {
    let opts = Options::parse();

    match run(opts) {
        Ok(exit_code) => process::exit(exit_code),
        Err(err) => {
            eprintln!("{:#?}", err);
            process::exit(1);
        }
    }
}

fn run(opts: Options) -> Result<i32> {
    let mut store = match Store::from_file_or_empty(&opts.config_path) {
        Ok(s) => s,
        Err(err) => {
            eprintln!("There was a problem loading the config file: {:#?}", err);
            process::exit(1);
        }
    };

    match opts.mode {
        Mode::Forbid { name, hint } => {
            store.forbid(name, hint);
            store.write(&opts.config_path)?;

            Ok(0)
        }

        Mode::Unforbid { name } => {
            store.unforbid(name);
            store.write(&opts.config_path)?;

            Ok(0)
        }

        Mode::Update => {
            store.update(opts.root)?;
            store.write(&opts.config_path)?;

            Ok(0)
        }

        Mode::Check => {
            let results = store.check(opts.root)?;
            let all_in_config = results.iter().all(|item| item.error_is_in_config());

            if !all_in_config {
                println!("I found some forbidden imports!\n")
            }

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
