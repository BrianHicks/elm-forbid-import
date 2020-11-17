use tree_sitter;

fn main() {
    println!("Hello, world!");

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
