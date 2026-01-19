use anyhow::Result;
use bumpalo::Bump;
use mago_database::file::FileId;
use mago_syntax::ast::Program;
use mago_syntax::parser::parse_file_content;
use std::path::Path;

/// Parse a PHP file using Mago and return the AST
/// The arena must outlive the returned Program reference
pub fn parse_php_file<'arena>(
    arena: &'arena Bump,
    path: &Path,
    content: &str,
) -> Result<&'arena Program<'arena>> {
    let file_id = FileId::zero();

    let (program, error) = parse_file_content(arena, file_id, content);

    if let Some(err) = error {
        eprintln!("Parse error in {}: {:?}", path.display(), err);
    }

    Ok(program)
}
