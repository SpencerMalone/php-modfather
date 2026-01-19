use crate::graph::DependencyGraph;
use indexmap::{IndexMap, IndexSet};
use petgraph::algo::tarjan_scc;
use petgraph::graph::{DiGraph, NodeIndex};
use std::collections::{HashMap, HashSet};

/// Represents a suggested module grouping
#[derive(Debug, Clone)]
pub struct ModuleSuggestion {
    pub name: String,
    pub namespaces: Vec<String>,
    pub class_count: usize,
    pub internal_dependencies: usize,
    pub external_dependencies: usize,
    pub cohesion_score: f64,
}

/// Represents a cycle between namespaces
#[derive(Debug, Clone)]
pub struct CycleDetection {
    pub namespaces: Vec<String>,
    pub cycle_type: CycleType,
    pub severity: CycleSeverity,
}

#[derive(Debug, Clone, PartialEq)]
pub enum CycleType {
    SelfCycle,        // A -> A
    Simple,           // A -> B -> A
    Complex,          // A -> B -> C -> A (3+ nodes)
}

#[derive(Debug, Clone, PartialEq)]
pub enum CycleSeverity {
    Low,     // Few edges in cycle
    Medium,  // Multiple edges
    High,    // Many edges, tightly coupled
}

/// Recommendation for breaking a cycle
#[derive(Debug, Clone)]
pub struct CycleBreakingRecommendation {
    pub cycle: CycleDetection,
    pub suggestions: Vec<String>,
    pub impact: String,
}

/// Analyzes a dependency graph and recommends module structure
pub struct ModuleRecommender {
    namespace_graph: DiGraph<String, ()>,
    namespace_to_index: HashMap<String, NodeIndex>,
    namespace_metrics: HashMap<String, NamespaceMetrics>,
}

#[derive(Debug, Clone, Default)]
struct NamespaceMetrics {
    class_count: usize,
    incoming_edges: usize,
    outgoing_edges: usize,
    classes: HashSet<String>,
}

impl ModuleRecommender {
    /// Create a new recommender from a namespace-level dependency graph
    pub fn new(graph: &DependencyGraph) -> Self {
        let mut namespace_graph = DiGraph::new();
        let mut namespace_to_index = HashMap::new();
        let mut namespace_metrics: HashMap<String, NamespaceMetrics> = HashMap::new();

        // Create nodes for each namespace
        for (ns_id, node) in &graph.nodes {
            let idx = namespace_graph.add_node(ns_id.clone());
            namespace_to_index.insert(ns_id.clone(), idx);

            // Extract metrics from node metadata
            let class_count = node.metadata.get("files")
                .and_then(|s| s.parse().ok())
                .unwrap_or(0);

            namespace_metrics.insert(ns_id.clone(), NamespaceMetrics {
                class_count,
                classes: HashSet::new(),
                incoming_edges: 0,
                outgoing_edges: 0,
            });
        }

        // Add edges
        for edge in &graph.edges {
            if let (Some(&from_idx), Some(&to_idx)) = (
                namespace_to_index.get(&edge.from),
                namespace_to_index.get(&edge.to),
            ) {
                namespace_graph.add_edge(from_idx, to_idx, ());

                // Update metrics
                if let Some(metrics) = namespace_metrics.get_mut(&edge.from) {
                    metrics.outgoing_edges += 1;
                }
                if let Some(metrics) = namespace_metrics.get_mut(&edge.to) {
                    metrics.incoming_edges += 1;
                }
            }
        }

        Self {
            namespace_graph,
            namespace_to_index,
            namespace_metrics,
        }
    }

    /// Detect all cycles in the namespace graph
    pub fn detect_cycles(&self) -> Vec<CycleDetection> {
        let sccs = tarjan_scc(&self.namespace_graph);
        let mut cycles = Vec::new();

        for scc in sccs {
            // Only interested in SCCs with more than 1 node or self-loops
            if scc.len() > 1 {
                let mut namespaces: Vec<String> = scc
                    .iter()
                    .filter_map(|&idx| self.namespace_graph.node_weight(idx))
                    .cloned()
                    .collect();
                namespaces.sort();

                let cycle_type = match namespaces.len() {
                    2 => CycleType::Simple,
                    _ => CycleType::Complex,
                };

                // Calculate severity based on number of edges in cycle
                let edge_count = self.count_edges_in_cycle(&scc);
                let severity = match edge_count {
                    0..=2 => CycleSeverity::Low,
                    3..=5 => CycleSeverity::Medium,
                    _ => CycleSeverity::High,
                };

                cycles.push(CycleDetection {
                    namespaces,
                    cycle_type,
                    severity,
                });
            } else if scc.len() == 1 {
                // Check for self-loops
                let idx = scc[0];
                if self.namespace_graph.neighbors(idx).any(|n| n == idx) {
                    if let Some(ns) = self.namespace_graph.node_weight(idx) {
                        cycles.push(CycleDetection {
                            namespaces: vec![ns.clone()],
                            cycle_type: CycleType::SelfCycle,
                            severity: CycleSeverity::Medium,
                        });
                    }
                }
            }
        }

        cycles
    }

    /// Count edges within a strongly connected component
    fn count_edges_in_cycle(&self, scc: &[NodeIndex]) -> usize {
        let scc_set: HashSet<_> = scc.iter().copied().collect();
        let mut count = 0;

        for &node in scc {
            for neighbor in self.namespace_graph.neighbors(node) {
                if scc_set.contains(&neighbor) {
                    count += 1;
                }
            }
        }

        count
    }

    /// Generate recommendations for breaking cycles
    pub fn recommend_cycle_breaking(&self, cycles: &[CycleDetection]) -> Vec<CycleBreakingRecommendation> {
        cycles
            .iter()
            .map(|cycle| {
                let suggestions = self.generate_cycle_breaking_suggestions(cycle);
                let impact = self.assess_cycle_impact(cycle);

                CycleBreakingRecommendation {
                    cycle: cycle.clone(),
                    suggestions,
                    impact,
                }
            })
            .collect()
    }

    /// Generate specific suggestions for breaking a cycle
    fn generate_cycle_breaking_suggestions(&self, cycle: &CycleDetection) -> Vec<String> {
        let mut suggestions = Vec::new();

        match cycle.cycle_type {
            CycleType::SelfCycle => {
                suggestions.push(format!(
                    "Namespace '{}' has internal circular dependencies",
                    cycle.namespaces[0]
                ));
                suggestions.push("Consider splitting into separate sub-namespaces".to_string());
                suggestions.push("Extract interfaces to break direct class dependencies".to_string());
            }
            CycleType::Simple => {
                suggestions.push(format!(
                    "Cycle between: {} ↔ {}",
                    cycle.namespaces[0], cycle.namespaces[1]
                ));
                suggestions.push("Option 1: Extract shared interfaces into a common namespace".to_string());
                suggestions.push("Option 2: Move coupled classes into one namespace".to_string());
                suggestions.push(format!(
                    "Option 3: Introduce dependency inversion - make {} depend on abstractions from {}",
                    cycle.namespaces[1], cycle.namespaces[0]
                ));
            }
            CycleType::Complex => {
                let cycle_str = cycle.namespaces.join(" → ");
                suggestions.push(format!("Complex cycle detected: {} → [back to start]", cycle_str));
                suggestions.push("Option 1: Extract a shared 'Core' or 'Common' namespace for shared types".to_string());
                suggestions.push("Option 2: Consider if these namespaces should be merged into a single module".to_string());
                suggestions.push("Option 3: Apply dependency inversion principle with interfaces".to_string());
                suggestions.push("Option 4: Identify and remove unnecessary dependencies".to_string());
            }
        }

        suggestions
    }

    /// Assess the impact of a cycle on modularization
    fn assess_cycle_impact(&self, cycle: &CycleDetection) -> String {
        match cycle.severity {
            CycleSeverity::Low => {
                "Low impact: Few dependencies involved, should be straightforward to resolve".to_string()
            }
            CycleSeverity::Medium => {
                "Medium impact: Moderate coupling, may require interface extraction or class movement".to_string()
            }
            CycleSeverity::High => {
                "High impact: Tight coupling detected, likely requires significant refactoring or module merging".to_string()
            }
        }
    }

    /// Suggest module groupings based on namespaces, prioritizing acyclic structure
    pub fn suggest_modules(&self) -> Vec<ModuleSuggestion> {
        let cycles = self.detect_cycles();
        let cycle_namespaces: HashSet<String> = cycles
            .iter()
            .flat_map(|c| c.namespaces.iter().cloned())
            .collect();

        let mut suggestions = Vec::new();

        // Group namespaces by their top-level prefix
        let mut namespace_groups: IndexMap<String, Vec<String>> = IndexMap::new();

        for (namespace, metrics) in &self.namespace_metrics {
            // Skip the global namespace
            if namespace == "\\" {
                continue;
            }

            // Extract top-level namespace (e.g., "App" from "App\Models")
            let top_level = namespace
                .split('\\')
                .next()
                .unwrap_or(namespace)
                .to_string();

            namespace_groups
                .entry(top_level)
                .or_insert_with(Vec::new)
                .push(namespace.clone());
        }

        // Create suggestions for each group
        for (top_level, namespaces) in namespace_groups {
            let has_cycles = namespaces.iter().any(|ns| cycle_namespaces.contains(ns));

            let class_count: usize = namespaces
                .iter()
                .filter_map(|ns| self.namespace_metrics.get(ns))
                .map(|m| m.class_count)
                .sum();

            let (internal_deps, external_deps) = self.calculate_module_dependencies(&namespaces);

            let cohesion_score = if internal_deps + external_deps > 0 {
                internal_deps as f64 / (internal_deps + external_deps) as f64
            } else {
                1.0
            };

            let module_name = if has_cycles {
                format!("{} ⚠️ (contains cycles)", top_level)
            } else {
                top_level.clone()
            };

            suggestions.push(ModuleSuggestion {
                name: module_name,
                namespaces,
                class_count,
                internal_dependencies: internal_deps,
                external_dependencies: external_deps,
                cohesion_score,
            });
        }

        // Sort by cohesion score (descending) - higher is better
        suggestions.sort_by(|a, b| {
            b.cohesion_score
                .partial_cmp(&a.cohesion_score)
                .unwrap_or(std::cmp::Ordering::Equal)
        });

        suggestions
    }

    /// Calculate internal vs external dependencies for a group of namespaces
    fn calculate_module_dependencies(&self, namespaces: &[String]) -> (usize, usize) {
        let namespace_set: HashSet<_> = namespaces.iter().collect();
        let mut internal = 0;
        let mut external = 0;

        for namespace in namespaces {
            if let Some(&idx) = self.namespace_to_index.get(namespace) {
                for neighbor in self.namespace_graph.neighbors(idx) {
                    if let Some(neighbor_ns) = self.namespace_graph.node_weight(neighbor) {
                        if namespace_set.contains(neighbor_ns) {
                            internal += 1;
                        } else {
                            external += 1;
                        }
                    }
                }
            }
        }

        (internal, external)
    }

    /// Generate a summary report
    pub fn generate_report(&self) -> ModularizationReport {
        let cycles = self.detect_cycles();
        let recommendations = self.recommend_cycle_breaking(&cycles);
        let module_suggestions = self.suggest_modules();

        let total_namespaces = self.namespace_metrics.len();
        let namespaces_in_cycles = cycles
            .iter()
            .flat_map(|c| c.namespaces.iter())
            .collect::<HashSet<_>>()
            .len();

        ModularizationReport {
            total_namespaces,
            namespaces_in_cycles,
            cycles,
            cycle_breaking_recommendations: recommendations,
            module_suggestions,
        }
    }
}

/// Complete report of modularization analysis
#[derive(Debug)]
pub struct ModularizationReport {
    pub total_namespaces: usize,
    pub namespaces_in_cycles: usize,
    pub cycles: Vec<CycleDetection>,
    pub cycle_breaking_recommendations: Vec<CycleBreakingRecommendation>,
    pub module_suggestions: Vec<ModuleSuggestion>,
}

impl ModularizationReport {
    /// Format the report as human-readable text
    pub fn format_text(&self) -> String {
        let mut output = String::new();

        output.push_str("# PHP Modularization Analysis Report\n\n");

        // Overview
        output.push_str("## Overview\n\n");
        output.push_str(&format!("- Total namespaces analyzed: {}\n", self.total_namespaces));
        output.push_str(&format!("- Namespaces involved in cycles: {}\n", self.namespaces_in_cycles));
        output.push_str(&format!("- Cycles detected: {}\n\n", self.cycles.len()));

        // Cycle Detection
        if !self.cycles.is_empty() {
            output.push_str("## ⚠️  Circular Dependencies Detected\n\n");
            output.push_str("Circular dependencies prevent clean module boundaries and should be resolved.\n\n");

            for (i, cycle) in self.cycles.iter().enumerate() {
                output.push_str(&format!("### Cycle #{}\n\n", i + 1));

                let severity_icon = match cycle.severity {
                    CycleSeverity::Low => "🟢",
                    CycleSeverity::Medium => "🟡",
                    CycleSeverity::High => "🔴",
                };

                output.push_str(&format!("**Severity**: {} {:?}\n\n", severity_icon, cycle.severity));
                output.push_str(&format!("**Type**: {:?}\n\n", cycle.cycle_type));
                output.push_str("**Namespaces involved**:\n");

                for ns in &cycle.namespaces {
                    output.push_str(&format!("- `{}`\n", ns));
                }
                output.push_str("\n");
            }

            // Recommendations
            output.push_str("## 💡 Recommendations to Break Cycles\n\n");

            for (i, rec) in self.cycle_breaking_recommendations.iter().enumerate() {
                output.push_str(&format!("### Cycle #{}\n\n", i + 1));
                output.push_str(&format!("**Impact**: {}\n\n", rec.impact));
                output.push_str("**Suggestions**:\n\n");

                for suggestion in &rec.suggestions {
                    output.push_str(&format!("- {}\n", suggestion));
                }
                output.push_str("\n");
            }
        } else {
            output.push_str("## ✅ No Circular Dependencies\n\n");
            output.push_str("Great! Your namespace structure is acyclic, which supports clean modularization.\n\n");
        }

        // Module Suggestions
        output.push_str("## 📦 Suggested Module Groupings\n\n");
        output.push_str("Modules are suggested based on top-level namespaces. Higher cohesion scores indicate better module candidates.\n\n");

        for (i, module) in self.module_suggestions.iter().enumerate() {
            output.push_str(&format!("### {}. {}\n\n", i + 1, module.name));
            output.push_str(&format!("- **Classes**: {}\n", module.class_count));
            output.push_str(&format!("- **Cohesion Score**: {:.2} (higher is better)\n", module.cohesion_score));
            output.push_str(&format!("- **Internal Dependencies**: {}\n", module.internal_dependencies));
            output.push_str(&format!("- **External Dependencies**: {}\n", module.external_dependencies));
            output.push_str("\n**Namespaces**:\n");

            for ns in &module.namespaces {
                output.push_str(&format!("- `{}`\n", ns));
            }
            output.push_str("\n");
        }

        output
    }
}
