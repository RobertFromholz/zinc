use crate::cst::parser::Parser;
use crate::cst::tree::Tree;
use crate::query::{Handle, Query};
use std::fs;
use std::path::PathBuf;

pub struct ParseQuery;

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub enum ParseKey {
    File(PathBuf),
    Text(String),
}

impl Query for ParseQuery {
    type Key = ParseKey;
    type Output = Option<Tree>;

    fn compute(handle: Handle<'_>, key: &Self::Key) -> Self::Output {
        let content;
        let text = match key {
            ParseKey::File(path) => {
                // Read the file. This is done as its own query.
                // That way, we clearly register that the parser has a dependency on the file content.
                // Thus, if the file contents haven't changed, we don't need to parse the file again.
                content = handle.compute::<FileQuery>(path.clone());
                content.as_deref()?
            },
            ParseKey::Text(text) => text,
        };
        let mut parser = Parser::new(text);
        parser.file();
        Some(parser.finish())
    }
}

pub struct FileQuery;

impl Query for FileQuery {
    type Key = PathBuf;
    type Output = Option<String>;

    fn compute(_handle: Handle<'_>, key: &Self::Key) -> Self::Output {
        // TODO: Handle a potential error if we can't read the file.
        fs::read_to_string(key).ok()
    }
}