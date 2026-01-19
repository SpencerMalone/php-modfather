use super::{DependencyGraph, Edge, Node};
use std::io::Write;

/// Writes a dependency graph in Graphviz DOT format
pub struct DotWriter {
    pub graph_name: String,
    pub graph_attributes: Vec<(String, String)>,
    pub node_attributes: Vec<(String, String)>,
    pub edge_attributes: Vec<(String, String)>,
}

impl DotWriter {
    pub fn new(graph_name: impl Into<String>) -> Self {
        Self {
            graph_name: graph_name.into(),
            graph_attributes: vec![
                ("rankdir".to_string(), "LR".to_string()),
                ("splines".to_string(), "ortho".to_string()),
            ],
            node_attributes: vec![
                ("shape".to_string(), "box".to_string()),
                ("style".to_string(), "rounded,filled".to_string()),
                ("fillcolor".to_string(), "lightblue".to_string()),
            ],
            edge_attributes: vec![
                ("color".to_string(), "gray".to_string()),
            ],
        }
    }

    pub fn write<W: Write>(&self, graph: &DependencyGraph, writer: &mut W) -> anyhow::Result<()> {
        writeln!(writer, "digraph {} {{", self.escape_id(&self.graph_name))?;

        // Write graph attributes
        for (key, value) in &self.graph_attributes {
            writeln!(writer, "  {}=\"{}\";", key, self.escape_string(value))?;
        }
        writeln!(writer)?;

        // Write default node attributes
        write!(writer, "  node [")?;
        for (i, (key, value)) in self.node_attributes.iter().enumerate() {
            if i > 0 {
                write!(writer, ", ")?;
            }
            write!(writer, "{}=\"{}\"", key, self.escape_string(value))?;
        }
        writeln!(writer, "];")?;

        // Write default edge attributes
        write!(writer, "  edge [")?;
        for (i, (key, value)) in self.edge_attributes.iter().enumerate() {
            if i > 0 {
                write!(writer, ", ")?;
            }
            write!(writer, "{}=\"{}\"", key, self.escape_string(value))?;
        }
        writeln!(writer, "];")?;
        writeln!(writer)?;

        // Write nodes
        let mut sorted_nodes: Vec<_> = graph.nodes.values().collect();
        sorted_nodes.sort_by(|a, b| a.id.cmp(&b.id));

        for node in sorted_nodes {
            self.write_node(writer, node)?;
        }
        writeln!(writer)?;

        // Write edges
        let mut sorted_edges: Vec<_> = graph.edges.iter().collect();
        sorted_edges.sort_by(|a, b| {
            a.from.cmp(&b.from).then_with(|| a.to.cmp(&b.to))
        });

        for edge in sorted_edges {
            self.write_edge(writer, edge)?;
        }

        writeln!(writer, "}}")?;
        Ok(())
    }

    fn write_node<W: Write>(&self, writer: &mut W, node: &Node) -> anyhow::Result<()> {
        write!(writer, "  {} [label=\"{}\"",
               self.escape_id(&node.id),
               self.escape_string(&node.label))?;

        for (key, value) in &node.metadata {
            write!(writer, ", {}=\"{}\"", key, self.escape_string(value))?;
        }

        writeln!(writer, "];")?;
        Ok(())
    }

    fn write_edge<W: Write>(&self, writer: &mut W, edge: &Edge) -> anyhow::Result<()> {
        write!(writer, "  {} -> {}",
               self.escape_id(&edge.from),
               self.escape_id(&edge.to))?;

        if edge.label.is_some() || !edge.metadata.is_empty() {
            write!(writer, " [")?;
            let mut first = true;

            if let Some(label) = &edge.label {
                write!(writer, "label=\"{}\"", self.escape_string(label))?;
                first = false;
            }

            for (key, value) in &edge.metadata {
                if !first {
                    write!(writer, ", ")?;
                }
                write!(writer, "{}=\"{}\"", key, self.escape_string(value))?;
                first = false;
            }

            write!(writer, "]")?;
        }

        writeln!(writer, ";")?;
        Ok(())
    }

    fn escape_id(&self, s: &str) -> String {
        if s.chars().all(|c| c.is_alphanumeric() || c == '_') && !s.is_empty() {
            s.to_string()
        } else {
            format!("\"{}\"", self.escape_string(s))
        }
    }

    fn escape_string(&self, s: &str) -> String {
        s.replace('\\', "\\\\")
            .replace('"', "\\\"")
            .replace('\n', "\\n")
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::graph::Node;

    #[test]
    fn test_simple_graph() {
        let mut graph = DependencyGraph::new();
        graph.add_node(Node::new("A", "Class A"));
        graph.add_node(Node::new("B", "Class B"));
        graph.add_edge(Edge::new("A", "B"));

        let writer = DotWriter::new("test");
        let mut output = Vec::new();
        writer.write(&graph, &mut output).unwrap();

        let result = String::from_utf8(output).unwrap();
        assert!(result.contains("digraph test"));
        assert!(result.contains("A [label=\"Class A\"]"));
        assert!(result.contains("B [label=\"Class B\"]"));
        assert!(result.contains("A -> B"));
    }
}
