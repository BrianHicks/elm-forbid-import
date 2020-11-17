use anyhow::{anyhow, Result};
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
    Check {
        /// Which names, if any, to specifically check up on
        names: Vec<String>,
    },
}

fn main() {
    let opts = Options::parse();

    if let Err(err) = run(opts) {
        eprintln!("There was a problem: {:#?}", err);
        process::exit(1);
    };
}

fn run(opts: Options) -> Result<()> {
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
            store.write(&opts.config_path)
        }

        Mode::Unforbid { name } => {
            store.unforbid(name);
            store.write(&opts.config_path)
        }

        Mode::Update => {
            store.update(opts.root)?;
            store.write(&opts.config_path)
        }

        _ => {
            println!("{:#?}", opts);
            Err(anyhow!("{:?} isn't implemented yet!", opts.mode))
        }
    }
}
