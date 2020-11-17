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

    /// Check what imports still need to be cleaned up
    Todo {
        /// Which names, if any, to specifically check up on
        names: Vec<String>,
    },

    /// Check that no forbidden+unknown imports are present (use me in CI!)
    Check,

    /// Update the allowed imports list
    Update,
}

fn main() {
    let opts = Options::parse();

    let mut store = match Store::from_file_or_empty(&opts.config_path) {
        Ok(s) => s,
        Err(err) => {
            eprintln!("There was a problem loading the config file: {:#?}", err);
            process::exit(1);
        }
    };

    let result = match opts.mode {
        Mode::Forbid { name, hint } => {
            store.forbid(name, hint);
            store.write(&opts.config_path)
        }

        Mode::Unforbid { name } => {
            store.unforbid(name);
            store.write(&opts.config_path)
        }

        Mode::Update => store.update(opts.root),

        _ => {
            println!("{:#?}", opts);
            process::exit(1);
        }
    };

    if let Err(err) = result {
        eprintln!("There was a problem: {:#?}", err);
        process::exit(1);
    };
}
