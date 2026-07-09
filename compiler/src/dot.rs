//! Utility to generate and render a GraphViz graph.

use std::env::temp_dir;
use std::fmt::Write as _;
use std::fs::File;
use std::io;
use std::io::Write as _;
use std::path::Path;
use std::process::{Command, Stdio};

/// A graph.
///
/// For simplicity, the graph is always a directed graph. That is, edges always point from the
/// current node to the target node.
pub trait Graph<N: Node, E: Edge> {
    /// All nodes in this graph.
    fn nodes(&self) -> Vec<N>;

    /// All edges in this graph.
    fn edges(&self) -> Vec<E>;

    /// Draws this graph into an SVG file.
    ///
    /// Requires `dot` be installed and in the path.
    ///
    /// The file is written to the provided path.
    fn draw_graph(&self, path: &Path) -> Result<File, io::Error> {
        let text = self.format();
        let mut command = Command::new("dot")
            .arg("-Tsvg")
            .arg("-o")
            .arg(path)
            // The DOT file is written directly to stdin.
            .stdin(Stdio::piped())
            // We don't care about the program's output.
            .stdout(Stdio::null())
            // We want to be able to read any error messages we encounter.
            .stderr(Stdio::piped())
            .spawn()?;
        {
            let Some(mut stdin) = command.stdin.take() else {
                return Err(io::Error::new(io::ErrorKind::Other, "could not capture stdin"));
            };
            write!(stdin, "{}", text)?;
        } // stdin is dropped (and as a result closed). This causes 'dot' to begin processing the file.
        let output = command.wait_with_output()?;
        if output.status.success() {
            Ok(File::open(path)?)
        } else {
            // We use from_utf8_lossy to avoid dealing with error handling if the output isn't valid.
            // We expect that the output will be valid UTF-8.
            let message = String::from_utf8_lossy(&output.stderr).into_owned();
            Err(io::Error::new(io::ErrorKind::Other, message))
        }
    }

    /// Draws this graph into an SVG file and opens it in the default application.
    ///
    /// Requires `dot` be installed and in the path.
    ///
    /// The file is written to the temporary directory.
    fn draw_and_open_graph(&self) -> Result<(), io::Error> {
        let temp_dir = temp_dir();
        let path = temp_dir.join("graph.svg");
        self.draw_graph(&path)?;
        // This will most only work on macOS.
        // However, for the time being this is only being developed on and for macOS.
        let output = Command::new("open")
            .arg(&path)
            .output()?;
        if output.status.success() {
            Ok(())
        } else {
            let message = String::from_utf8_lossy(&output.stderr).into_owned();
            Err(io::Error::new(io::ErrorKind::Other, message))
        }
    }

    /// Converts this graph into a `DOT` graph.
    fn format(&self) -> String {
        let mut text = String::from("digraph {\n");
        for node in self.nodes() {
            node.format(&mut text);
        }
        for edge in self.edges() {
            edge.format(&mut text);
        }
        text += "}";
        text
    }
}

/// A node in a graph.
///
/// Every node in a given graph has a unique id. If multiple nodes share the same id, edges to one
/// node might point to any other node with the same id.
pub trait Node {
    /// A unique id for this node.
    // I'm unsure whether this should return an owned `String` or a reference `&str`.
    // This way, we don't encounter any problems trying when we derive the id from some other type,
    // for example, from a `u32`.
    fn id(&self) -> String;

    /// Formats this node into the `DOT` format. Returns this node's id.
    fn format(&self, text: &mut String) {
        let id = escape_string(self.id());
        writeln!(text, "{}\"{}\";", " ".repeat(4), id).unwrap();
    }
}

/// An edge between two nodes.
///
/// The edge points from the node from which this edge was returned, to the node specified by this
/// edge.
pub trait Edge {
    fn left(&self) -> String;

    fn right(&self) -> String;

    fn format(&self, text: &mut String) {
        let left = escape_string(self.left());
        let right = escape_string(self.right());
        writeln!(text, "{}\"{}\" -> \"{}\";", " ".repeat(4), left, right).unwrap();
    }
}

fn escape_string(text: String) -> String {
    text.replace('"', "\\\"")
}