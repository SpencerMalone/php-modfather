use crate::graph::DependencyGraph;
use anyhow::Result;
use std::io::Write;

/// Writes dependency graphs in CSV format (edge list)
pub struct CsvWriter {
    include_header: bool,
}

impl CsvWriter {
    pub fn new() -> Self {
        Self {
            include_header: true,
        }
    }

    pub fn without_header(mut self) -> Self {
        self.include_header = false;
        self
    }

    /// Write the graph as CSV (edge list)
    pub fn write<W: Write>(&self, graph: &DependencyGraph, writer: &mut W) -> Result<()> {
        // Write header if requested
        if self.include_header {
            writeln!(writer, "from,to")?;
        }

        // Write each edge as a row
        for edge in &graph.edges {
            writeln!(writer, "{},{}", edge.from, edge.to)?;
        }

        Ok(())
    }
}

impl Default for CsvWriter {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::graph::{Edge, Node};

    #[test]
    fn test_csv_writer_with_header() {
        let mut graph = DependencyGraph::new();
        graph.add_node(Node::new("ClassA", "ClassA"));
        graph.add_node(Node::new("ClassB", "ClassB"));
        graph.add_edge(Edge::new("ClassA", "ClassB"));

        let writer = CsvWriter::new();
        let mut output = Vec::new();
        writer.write(&graph, &mut output).unwrap();

        let result = String::from_utf8(output).unwrap();
        assert_eq!(result, "from,to\nClassA,ClassB\n");
    }

    #[test]
    fn test_csv_writer_without_header() {
        let mut graph = DependencyGraph::new();
        graph.add_node(Node::new("ClassA", "ClassA"));
        graph.add_node(Node::new("ClassB", "ClassB"));
        graph.add_edge(Edge::new("ClassA", "ClassB"));

        let writer = CsvWriter::new().without_header();
        let mut output = Vec::new();
        writer.write(&graph, &mut output).unwrap();

        let result = String::from_utf8(output).unwrap();
        assert_eq!(result, "ClassA,ClassB\n");
    }

    #[test]
    fn test_csv_writer_multiple_edges() {
        let mut graph = DependencyGraph::new();
        graph.add_node(Node::new("A", "A"));
        graph.add_node(Node::new("B", "B"));
        graph.add_node(Node::new("C", "C"));
        graph.add_edge(Edge::new("A", "B"));
        graph.add_edge(Edge::new("A", "C"));
        graph.add_edge(Edge::new("B", "C"));

        let writer = CsvWriter::new();
        let mut output = Vec::new();
        writer.write(&graph, &mut output).unwrap();

        let result = String::from_utf8(output).unwrap();
        assert_eq!(result, "from,to\nA,B\nA,C\nB,C\n");
    }
}
