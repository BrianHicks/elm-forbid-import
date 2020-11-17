use clap::Clap;
use std::path::PathBuf;
use tree_sitter;

#[derive(Debug, Clap)]
struct Options {
    /// Imports to forbid
    forbid: Vec<String>,

    /// The path to an Elm project (a folder containing elm.json) where
    /// we'll start looking for imports.
    #[clap(short, long, default_value = ".")]
    root: PathBuf,
}

fn main() {
    let opts = Options::parse();
    println!("{:#?}", opts);

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
