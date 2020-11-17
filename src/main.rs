use clap::Clap;
use std::path::PathBuf;
use std::process;
use tree_sitter;
mod store;
use store::Store;

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

    /// Check what imports still need to be cleaned up
    Todo {
        /// Which names, if any, to specifically check up on
        names: Vec<String>,
    },

    /// Check that no uncontrolled imports are present (use me in CI!)
    Check,

    /// Update the allowed imports list
    Update,
}

fn main() {
    let opts = Options::parse();
    println!("{:#?}", opts);

    let mut store = Store::from_file_or_empty(&opts.config_path);
    println!("{:#?}", store);

    let result = match opts.mode {
        Mode::Forbid { name, hint } => {
            store.forbid(name, hint);
            store.write(&opts.config_path)
        }

        _ => {
            process::exit(1);
        }
    };

    if let Err(err) = result {
        eprintln!("There was a problem: {:#?}", err);
        process::exit(1);
    };

    let _parser = get_parser();
}

// tree sitter

extern "C" {
    fn tree_sitter_elm() -> tree_sitter::Language;
}

fn get_parser() -> Result<tree_sitter::Parser, tree_sitter::LanguageError> {
    let mut parser = tree_sitter::Parser::new();

    let elm = unsafe { tree_sitter_elm() };
    parser.set_language(elm)?;

    Ok(parser)
}
