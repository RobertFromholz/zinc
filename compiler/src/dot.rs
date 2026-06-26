//! Utility to generate and render a GraphViz graph.

use std::fmt::Write as _;
use std::io::Write as _;
use std::collections::HashSet;
use std::env::temp_dir;
use std::fs::File;
use std::io;
use std::path::Path;
use std::process::{Command, Stdio};

/// A graph.
///
/// For simplicity, the graph is always a directed graph. That is, edges always point from the
/// current node to the target node.
///
/// The graph is computed by iterating over all nodes, starting with the root node.
/// Cycles are supported. If a cycle is detected, the node is not traversed again.
pub trait Graph {
    /// The root node of the graph.
    fn root(&self) -> &dyn Node;

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
        let mut nodes = HashSet::new();
        let mut text = String::from("digraph {\n");
        self.root().format(&mut nodes, &mut text);
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

    /// A list of edges pointing from this node to other nodes.
    fn edges(&self) -> Vec<&dyn Edge>;

    /// Formats this node into the `DOT` format. Returns this node's id.
    fn format(&self, nodes: &mut HashSet<String>, text: &mut String) -> String {
        let id = self.id();
        if nodes.contains(&id) {
            return id;
        }
        nodes.insert(id.clone());
        writeln!(text, "{}\"{}\";", " ".repeat(4), id).unwrap();
        for edge in self.edges() {
            edge.format(nodes, text, &id);
        }
        id
    }
}

/// An edge between two nodes.
///
/// The edge points from the node from which this edge was returned, to the node specified by this
/// edge.
pub trait Edge {
    /// The target node for this edge.
    fn node(&self) -> &dyn Node;

    fn format(&self, nodes: &mut HashSet<String>, text: &mut String, node_id: &str) {
        let node = self.node();
        let other_id = node.format(nodes, text);
        writeln!(text, "{}\"{}\" -> \"{}\";", " ".repeat(4), node_id, other_id).unwrap();
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    // An extremely simple graph implementation.
    struct SimpleGraph(SimpleNode);
    struct SimpleNode(u32, Vec<SimpleEdge>);
    struct SimpleEdge(SimpleNode);

    impl Graph for SimpleGraph {
        fn root(&self) -> &dyn Node {
            &self.0
        }
    }

    impl Node for SimpleNode {
        fn id(&self) -> String {
            self.0.to_string()
        }

        fn edges(&self) -> Vec<&dyn Edge> {
            self.1.iter()
                .map(|edge| edge as &dyn Edge)
                .collect::<Vec<_>>()
        }
    }

    impl Edge for SimpleEdge {
        fn node(&self) -> &dyn Node {
            &self.0
        }
    }

    #[test]
    fn simple_dot_graph() {
        let graph = SimpleGraph(
            SimpleNode(0, vec![
                SimpleEdge(SimpleNode(1, vec![])),
                SimpleEdge(SimpleNode(2, vec![])),
            ])
        );
        let text = graph.format();
        assert_eq!(text, r#"digraph {
    "0";
    "1";
    "0" -> "1";
    "2";
    "0" -> "2";
}"#)
    }
}
