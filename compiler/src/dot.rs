//! Utility to generate and render a GraphViz graph.

use std::collections::HashSet;
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
pub trait Graph {

    type Node<'a>: Node where Self: 'a;
    type Edge<'a>: Edge where Self: 'a;

    /// Nodes in this graph.
    fn nodes(&self) -> Vec<Self::Node<'_>>;

    /// Edges in this graph.
    fn edges(&self) -> Vec<Self::Edge<'_>>;

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
        // This will only work on macOS.
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
        let mut visited = HashSet::new();
        let mut text = String::from("digraph {\n");
        for node in self.nodes() {
            node.format(&mut text, &mut visited);
        }
        for edge in self.edges() {
            edge.format(&mut text, &mut visited);
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
    type Edge<'a>: Edge where Self: 'a;

    /// A unique id for this node.
    // I'm unsure whether this should return an owned `String` or a reference `&str`.
    // This way, we don't encounter any problems trying when we derive the id from some other type,
    // for example, from a `u32`.
    fn id(&self) -> String;

    fn label(&self) -> Option<String> {
        None
    }

    /// Edges from this node.
    fn edges(&self) -> Vec<Self::Edge<'_>> {
        vec![]
    }

    /// Formats this node into the `DOT` format. Returns this node's id.
    fn format(&self, text: &mut String, visited: &mut HashSet<String>) {
        let id = escape_string(self.id());
        if visited.contains(&id) {
            // We have already added this node to the graph.
            return;
        }
        visited.insert(id.clone());

        let tags = self.label()
            .map(|label| escape_string(label))
            .map(|label| format!("label=\"{}\"", label))
            .unwrap_or_else(|| String::new());

        writeln!(text, "{}\"{}\"[{}];", " ".repeat(4), id, tags).unwrap();

        for edge in self.edges() {
            edge.format(text, visited);
        }
    }
}

/// An edge between two nodes.
///
/// The edge points from the node from which this edge was returned, to the node specified by this
/// edge.
pub trait Edge {
    type Node<'a>: Node where Self: 'a;

    fn left_id(&self) -> String;

    fn right_id(&self) -> String;

    /// The node on this edge points to.
    ///
    /// An edge isn't required to return anything here if the node is visited
    /// from elsewhere.
    fn right(&self) -> Option<Self::Node<'_>> {
        None
    }

    fn format(&self, text: &mut String, visited: &mut HashSet<String>) {
        let left = escape_string(self.left_id());
        let right = escape_string(self.right_id());
        writeln!(text, "{}\"{}\" -> \"{}\";", " ".repeat(4), left, right).unwrap();
        if let Some(right) = self.right() {
            right.format(text, visited);
        }
    }
}

fn escape_string(text: String) -> String {
    text.replace('"', "\\\"")
        .replace("{", "\\{")
        .replace("}", "\\}")
        .replace(";", "\\;")
        .replace("(", "\\(")
        .replace(")", "\\)")
}