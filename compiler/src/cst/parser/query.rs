use std::fs;
use std::path::PathBuf;
use crate::cst::parser::Parser;
use crate::cst::tree::Tree;
use crate::query::{Handle, Query};

pub struct ParserQuery;

impl Query for ParserQuery {
    type Key = PathBuf;
    type Output = Tree;

    // TODO: We technically want to have the file content as the key.
    //  That way, we can also compile smaller snippets and cache their result.
    //  However, doing that requires us to store the entire file content in query::Context.
    //  I guess we could add a trait similar to Key<T>. If a type implements Key<T> it can be used
    //  as a key in place of T. Key<T>::into_key(query::Handle) computes the key and can declare
    //  it's own dependencies.
    fn compute(handle: Handle<'_>, key: &Self::Key) -> Self::Output {
        // Read the file. This is done as its own query.
        // That way, we clearly register that the parser has a dependency on the file content.
        // Thus, if the file contents haven't changed, we don't need to parse the file again.
        let content = handle.compute::<FileQuery>(key.clone());
        let mut parser = Parser::new(&content);
        parser.file();
        parser.finish()
    }
}

pub struct FileQuery;

impl Query for FileQuery {
    type Key = PathBuf;
    type Output = String;

    fn compute(handle: Handle<'_>, key: &Self::Key) -> Self::Output {
        // TODO: Handle a potential error if we can't read the file.
        fs::read_to_string(key).unwrap()
    }
}