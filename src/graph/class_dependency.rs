use crate::analyzer::php_parser::parse_php_file;
use crate::graph::{DependencyGraph, Edge, GraphAnalyzer, Node};
use anyhow::Result;
use bumpalo::Bump;
use indexmap::IndexMap;
use mago_syntax::ast::*;
use std::collections::{HashMap, HashSet};
use std::path::Path;

/// Tracks imported classes via `use` statements
#[derive(Debug, Default, Clone)]
struct ImportContext {
    /// Map of short name -> fully qualified name
    imports: HashMap<String, String>,
}

impl ImportContext {
    fn new() -> Self {
        Self::default()
    }

    fn add_import(&mut self, fully_qualified: String, alias: Option<String>) {
        let short_name = if let Some(alias) = alias {
            alias
        } else {
            // Extract the last part of the FQN as the short name
            fully_qualified
                .split('\\')
                .last()
                .unwrap_or(&fully_qualified)
                .to_string()
        };
        self.imports.insert(short_name, fully_qualified);
    }

    fn resolve(&self, name: &str) -> Option<&String> {
        self.imports.get(name)
    }
}

/// Extracts class dependencies from PHP code
pub struct ClassDependencyAnalyzer {
    /// Map of class name to its file path
    classes: IndexMap<String, String>,
    /// Map of class name to its dependencies
    dependencies: IndexMap<String, HashSet<String>>,
}

impl ClassDependencyAnalyzer {
    pub fn new() -> Self {
        Self {
            classes: IndexMap::new(),
            dependencies: IndexMap::new(),
        }
    }

    /// Visit the AST and extract class information
    fn visit_program(&mut self, program: &Program, file_path: &str, namespace: Option<String>) {
        let mut imports = ImportContext::new();
        for statement in program.statements.iter() {
            self.visit_statement(statement, file_path, namespace.as_deref(), &mut imports);
        }
    }

    fn visit_statement(&mut self, statement: &Statement, file_path: &str, namespace: Option<&str>, imports: &mut ImportContext) {
        match statement {
            Statement::Use(use_stmt) => {
                self.process_use_statement(use_stmt, imports);
            }
            Statement::Namespace(ns) => {
                let ns_name = self.extract_namespace_name(ns);
                // Namespace gets its own import context
                let mut ns_imports = ImportContext::new();

                for stmt in ns.statements().iter() {
                    self.visit_statement(stmt, file_path, Some(&ns_name), &mut ns_imports);
                }
            }
            Statement::Class(class) => {
                self.process_class(class, file_path, namespace, imports);
            }
            Statement::Interface(interface) => {
                self.process_interface(interface, file_path, namespace, imports);
            }
            Statement::Trait(trait_def) => {
                self.process_trait(trait_def, file_path, namespace, imports);
            }
            Statement::Enum(enum_def) => {
                self.process_enum(enum_def, file_path, namespace, imports);
            }
            _ => {}
        }
    }

    fn process_use_statement(&mut self, use_stmt: &Use, imports: &mut ImportContext) {
        // Handle the items in the use statement
        match &use_stmt.items {
            UseItems::Sequence(seq) => {
                for item in seq.items.iter() {
                    let fqn = match &item.name {
                        Identifier::Qualified(q) => q.value.to_string(),
                        Identifier::FullyQualified(f) => f.value.to_string(),
                        Identifier::Local(l) => l.value.to_string(),
                    };

                    let alias = item.alias.as_ref().map(|a| a.identifier.value.to_string());
                    imports.add_import(fqn, alias);
                }
            }
            _ => {} // Handle other use statement types if needed
        }
    }

    fn process_class(&mut self, class: &Class, file_path: &str, namespace: Option<&str>, imports: &ImportContext) {
        let class_name = &class.name.value;
        let fqn = self.get_fqn(class_name, namespace);

        self.classes.insert(fqn.clone(), file_path.to_string());

        // Analyze parent class
        if let Some(ref extends) = class.extends {
            for parent in extends.types.iter() {
                let parent_name = self.extract_identifier_from_name(parent);
                let parent_fqn = self.resolve_class_name(&parent_name, namespace, imports);
                self.add_dependency(&fqn, &parent_fqn);
            }
        }

        // Analyze interfaces
        if let Some(ref implements) = class.implements {
            for interface in implements.types.iter() {
                let interface_name = self.extract_identifier_from_name(interface);
                let interface_fqn = self.resolve_class_name(&interface_name, namespace, imports);
                self.add_dependency(&fqn, &interface_fqn);
            }
        }

        // Visit class members
        for member in class.members.iter() {
            self.visit_class_member(member, &fqn, namespace, imports);
        }
    }

    fn process_interface(&mut self, interface: &Interface, file_path: &str, namespace: Option<&str>, imports: &ImportContext) {
        let interface_name = &interface.name.value;
        let fqn = self.get_fqn(interface_name, namespace);

        self.classes.insert(fqn.clone(), file_path.to_string());

        // Analyze parent interfaces
        if let Some(ref extends) = interface.extends {
            for parent in extends.types.iter() {
                let parent_name = self.extract_identifier_from_name(parent);
                let parent_fqn = self.resolve_class_name(&parent_name, namespace, imports);
                self.add_dependency(&fqn, &parent_fqn);
            }
        }
    }

    fn process_trait(&mut self, trait_def: &Trait, file_path: &str, namespace: Option<&str>, imports: &ImportContext) {
        let trait_name = &trait_def.name.value;
        let fqn = self.get_fqn(trait_name, namespace);

        self.classes.insert(fqn.clone(), file_path.to_string());

        // Visit trait members
        for member in trait_def.members.iter() {
            self.visit_class_member(member, &fqn, namespace, imports);
        }
    }

    fn process_enum(&mut self, enum_def: &Enum, file_path: &str, namespace: Option<&str>, imports: &ImportContext) {
        let enum_name = &enum_def.name.value;
        let fqn = self.get_fqn(enum_name, namespace);

        self.classes.insert(fqn.clone(), file_path.to_string());

        // Analyze backing type hint
        if let Some(ref backing) = enum_def.backing_type_hint {
            self.extract_backing_type_dependencies(backing, &fqn, namespace, imports);
        }

        // Analyze interfaces
        if let Some(ref implements) = enum_def.implements {
            for interface in implements.types.iter() {
                let interface_name = self.extract_identifier_from_name(interface);
                let interface_fqn = self.resolve_class_name(&interface_name, namespace, imports);
                self.add_dependency(&fqn, &interface_fqn);
            }
        }
    }

    fn visit_class_member(&mut self, member: &ClassLikeMember, current_class: &str, namespace: Option<&str>, imports: &ImportContext) {
        match member {
            ClassLikeMember::TraitUse(trait_use) => {
                for trait_name in trait_use.trait_names.iter() {
                    let trait_fqn_str = self.extract_identifier_from_name(trait_name);
                    let trait_fqn = self.resolve_class_name(&trait_fqn_str, namespace, imports);
                    self.add_dependency(current_class, &trait_fqn);
                }
            }
            ClassLikeMember::Property(property) => {
                match property {
                    Property::Plain(plain) => {
                        if let Some(ref hint) = plain.hint {
                            self.extract_hint_dependencies(hint, current_class, namespace, imports);
                        }
                    }
                    Property::Hooked(hooked) => {
                        if let Some(ref hint) = hooked.hint {
                            self.extract_hint_dependencies(hint, current_class, namespace, imports);
                        }
                    }
                }
            }
            ClassLikeMember::Method(method) => {
                // Check return type
                if let Some(ref return_type) = method.return_type_hint {
                    self.extract_return_type_dependencies(return_type, current_class, namespace, imports);
                }

                // Check parameter types
                for param in method.parameter_list.parameters.iter() {
                    if let Some(ref hint) = param.hint {
                        self.extract_hint_dependencies(hint, current_class, namespace, imports);
                    }
                }
            }
            _ => {}
        }
    }

    fn extract_hint_dependencies(&mut self, hint: &Hint, current_class: &str, namespace: Option<&str>, imports: &ImportContext) {
        match hint {
            Hint::Identifier(id) => {
                let type_name = id.value();
                if self.is_class_type(type_name) {
                    let type_fqn = self.resolve_class_name(type_name, namespace, imports);
                    self.add_dependency(current_class, &type_fqn);
                }
            }
            Hint::Parenthesized(p) => {
                self.extract_hint_dependencies(&p.hint, current_class, namespace, imports);
            }
            Hint::Nullable(n) => {
                self.extract_hint_dependencies(&n.hint, current_class, namespace, imports);
            }
            Hint::Union(u) => {
                self.extract_hint_dependencies(&u.left, current_class, namespace, imports);
                self.extract_hint_dependencies(&u.right, current_class, namespace, imports);
            }
            Hint::Intersection(i) => {
                self.extract_hint_dependencies(&i.left, current_class, namespace, imports);
                self.extract_hint_dependencies(&i.right, current_class, namespace, imports);
            }
            _ => {}
        }
    }

    fn extract_return_type_dependencies(&mut self, return_type: &FunctionLikeReturnTypeHint, current_class: &str, namespace: Option<&str>, imports: &ImportContext) {
        self.extract_hint_dependencies(&return_type.hint, current_class, namespace, imports);
    }

    fn extract_backing_type_dependencies(&mut self, backing: &EnumBackingTypeHint, current_class: &str, namespace: Option<&str>, imports: &ImportContext) {
        self.extract_hint_dependencies(&backing.hint, current_class, namespace, imports);
    }

    fn extract_namespace_name(&self, ns: &Namespace) -> String {
        if let Some(name) = &ns.name {
            match name {
                Identifier::Qualified(q) => q.value.to_string(),
                Identifier::Local(l) => l.value.to_string(),
                _ => String::new(),
            }
        } else {
            String::new()
        }
    }

    fn extract_identifier_from_name(&self, name: &Identifier) -> String {
        match name {
            Identifier::Qualified(q) => q.value.to_string(),
            Identifier::Local(l) => l.value.to_string(),
            Identifier::FullyQualified(f) => f.value.to_string(),
        }
    }

    fn get_fqn(&self, name: &str, namespace: Option<&str>) -> String {
        if name.starts_with('\\') {
            name.to_string()
        } else if let Some(ns) = namespace {
            format!("{}\\{}", ns, name)
        } else {
            name.to_string()
        }
    }

    fn resolve_class_name(&self, name: &str, namespace: Option<&str>, imports: &ImportContext) -> String {
        // Check if it's already fully qualified
        if name.starts_with('\\') {
            // Remove leading backslash for consistency
            return name[1..].to_string();
        }

        // Check if there's a use statement import for this name
        if let Some(fqn) = imports.resolve(name) {
            return fqn.clone();
        }

        // Otherwise, resolve relative to current namespace
        if let Some(ns) = namespace {
            format!("{}\\{}", ns, name)
        } else {
            name.to_string()
        }
    }

    fn is_class_type(&self, type_name: &str) -> bool {
        // Filter out built-in types
        !matches!(
            type_name.to_lowercase().as_str(),
            "int" | "float" | "string" | "bool" | "array" | "object"
            | "callable" | "iterable" | "void" | "mixed" | "never"
            | "true" | "false" | "null" | "self" | "parent" | "static"
        )
    }

    fn add_dependency(&mut self, from: &str, to: &str) {
        self.dependencies
            .entry(from.to_string())
            .or_insert_with(HashSet::new)
            .insert(to.to_string());
    }
}

impl GraphAnalyzer for ClassDependencyAnalyzer {
    fn analyze(&mut self, file_path: &str, content: &str) -> Result<()> {
        // Parse the file
        let arena = Bump::new();
        let path = Path::new(file_path);
        let program = parse_php_file(&arena, path, content)?;
        self.visit_program(program, file_path, None);
        Ok(())
    }

    fn build_graph(&self, include_external: bool) -> DependencyGraph {
        let mut graph = DependencyGraph::new();

        // Add all defined classes as nodes (internal dependencies)
        for (class_name, file_path) in &self.classes {
            let node = Node::new(class_name.clone(), class_name.clone())
                .with_metadata("file", file_path.clone())
                .with_metadata("type", "internal");
            graph.add_node(node);
        }

        // Add dependencies as edges
        for (from, deps) in &self.dependencies {
            for to in deps {
                let is_external = !self.classes.contains_key(to);

                if include_external || !is_external {
                    // Add external classes as nodes if including external dependencies
                    if is_external && include_external {
                        let node = Node::new(to.clone(), to.clone())
                            .with_metadata("type", "external");
                        graph.add_node(node);
                    }

                    graph.add_edge(Edge::new(from.clone(), to.clone()));
                }
            }
        }

        graph
    }
}

impl Default for ClassDependencyAnalyzer {
    fn default() -> Self {
        Self::new()
    }
}
