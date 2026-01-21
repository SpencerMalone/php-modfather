pub mod class_dependency;
pub mod namespace_dependency;
pub mod dot_writer;
pub mod csv_writer;
pub mod module_recommender;

use std::collections::{HashMap, HashSet};

/// Represents a node in the dependency graph
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Node {
    pub id: String,
    pub label: String,
    pub metadata: HashMap<String, String>,
}

impl std::hash::Hash for Node {
    fn hash<H: std::hash::Hasher>(&self, state: &mut H) {
        self.id.hash(state);
    }
}

impl Node {
    pub fn new(id: impl Into<String>, label: impl Into<String>) -> Self {
        Self {
            id: id.into(),
            label: label.into(),
            metadata: HashMap::new(),
        }
    }

    pub fn with_metadata(mut self, key: impl Into<String>, value: impl Into<String>) -> Self {
        self.metadata.insert(key.into(), value.into());
        self
    }
}

/// Represents an edge (dependency) in the graph
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Edge {
    pub from: String,
    pub to: String,
    pub label: Option<String>,
    pub metadata: HashMap<String, String>,
}

impl std::hash::Hash for Edge {
    fn hash<H: std::hash::Hasher>(&self, state: &mut H) {
        self.from.hash(state);
        self.to.hash(state);
    }
}

impl Edge {
    pub fn new(from: impl Into<String>, to: impl Into<String>) -> Self {
        Self {
            from: from.into(),
            to: to.into(),
            label: None,
            metadata: HashMap::new(),
        }
    }

    pub fn with_label(mut self, label: impl Into<String>) -> Self {
        self.label = Some(label.into());
        self
    }

    pub fn with_metadata(mut self, key: impl Into<String>, value: impl Into<String>) -> Self {
        self.metadata.insert(key.into(), value.into());
        self
    }
}

/// A dependency graph that can be exported to various formats
#[derive(Debug, Default)]
pub struct DependencyGraph {
    pub nodes: HashMap<String, Node>,
    pub edges: HashSet<Edge>,
}

impl DependencyGraph {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn add_node(&mut self, node: Node) {
        self.nodes.insert(node.id.clone(), node);
    }

    pub fn add_edge(&mut self, edge: Edge) {
        // Ensure both nodes exist
        if !self.nodes.contains_key(&edge.from) {
            self.add_node(Node::new(edge.from.clone(), edge.from.clone()));
        }
        if !self.nodes.contains_key(&edge.to) {
            self.add_node(Node::new(edge.to.clone(), edge.to.clone()));
        }
        self.edges.insert(edge);
    }

    pub fn get_dependencies(&self, node_id: &str) -> Vec<&Node> {
        self.edges
            .iter()
            .filter(|e| e.from == node_id)
            .filter_map(|e| self.nodes.get(&e.to))
            .collect()
    }

    pub fn get_dependents(&self, node_id: &str) -> Vec<&Node> {
        self.edges
            .iter()
            .filter(|e| e.to == node_id)
            .filter_map(|e| self.nodes.get(&e.from))
            .collect()
    }
}

/// Trait for graph analyzers that extract dependencies from PHP code
pub trait GraphAnalyzer {
    fn analyze(&mut self, file_path: &str, content: &str) -> anyhow::Result<()>;
    fn build_graph(&self, include_external: bool) -> DependencyGraph;
}
