# PHP Modfather

A Rust-based tool for analyzing PHP monolithic applications and generating dependency graphs. This tool helps you understand class dependencies in your PHP codebase by parsing PHP files using the [Mago](https://github.com/carthage-software/mago) toolchain and outputting directed graphs in Graphviz DOT format.

This is mostly AI slop, sorry y'all just being upfront and goofing off.

## Features

- **Class Dependency Analysis**: Analyzes PHP classes, interfaces, traits, and enums
- **Namespace Dependency Analysis**: Visualizes dependencies between namespaces
- **Namespace Support**: Properly handles namespaced classes and fully qualified names
- **Use Statement Support**: Fully resolves PHP `use` statements and aliases for accurate dependency tracking
- **Type Hint Analysis**: Extracts dependencies from:
  - Class inheritance (`extends`)
  - Interface implementations (`implements`)
  - Trait usage
  - Property type hints
  - Method parameter type hints
  - Method return type hints
- **DOT Format Output**: Generates Graphviz-compatible directed graphs
- **Extensible Architecture**: Modular design makes it easy to add new analysis types

## Installation

### Prerequisites

- Rust 1.70 or higher (edition 2021)
- Cargo

### Build from source

```bash
git clone <repository-url>
cd php-modfather
cargo build --release
```

The binary will be available at `target/release/php-modfather`.

## Usage

### Basic Usage

Analyze a directory of PHP files:

```bash
php-modfather /path/to/php/code
```

### Save output to file

```bash
php-modfather /path/to/php/code --output dependencies.dot
```

### Analyze multiple directories

```bash
php-modfather /path/to/src /path/to/lib
```

### Namespace dependency analysis

Generate a graph showing dependencies between namespaces instead of individual classes:

```bash
php-modfather /path/to/php/code -t namespace --output namespace-deps.dot
```

This is particularly useful for:
- Understanding high-level architecture
- Identifying module boundaries
- Planning modularization strategies
- Detecting circular dependencies between modules

### Verbose mode

```bash
php-modfather /path/to/php/code --verbose
```

### Options

- `paths`: One or more directories or files to analyze (required)
- `-o, --output <FILE>`: Output file for the DOT graph (default: stdout)
- `-n, --graph-name <NAME>`: Name of the graph (default: "php_dependencies")
- `-t, --analysis-type <TYPE>`: Type of analysis to perform
  - `class` (default): Individual class dependencies
  - `namespace`: Namespace-level dependencies
- `-v, --verbose`: Enable verbose output showing progress

## Visualizing the Graph

After generating a DOT file, you can visualize it using Graphviz:

```bash
# Generate PNG image
dot -Tpng dependencies.dot -o dependencies.png

# Generate SVG image
dot -Tsvg dependencies.dot -o dependencies.svg

# Generate PDF
dot -Tpdf dependencies.dot -o dependencies.pdf

# Interactive visualization
dot -Tx11 dependencies.dot
```

## Example Output

Given PHP files with the following structure:

```php
<?php
namespace App\Controllers;

use App\Models\User;
use App\Services\UserService;

class UserController {
    private UserService $service;

    public function getUser(int $id): User {
        return $this->service->findUser($id);
    }
}
```

The tool will generate a DOT graph showing:
- `UserController` depends on `User` (via return type hint, resolved from `use` statement)
- `UserController` depends on `UserService` (via property type hint, resolved from `use` statement)

The analyzer fully supports:
- PHP `use` statements with aliases (e.g., `use App\Services\UserService as Service`)
- Class inheritance, interface implementations, and trait usage
- Type hints in properties, parameters, and return types

## Architecture

The project is structured for extensibility:

```
src/
├── analyzer/           # PHP file discovery and parsing
│   ├── mod.rs
│   └── php_parser.rs   # Mago-based PHP parser
├── graph/              # Graph generation
│   ├── mod.rs               # Core graph structures
│   ├── class_dependency.rs  # Class dependency analyzer
│   ├── namespace_dependency.rs  # Namespace dependency analyzer
│   └── dot_writer.rs        # DOT format output
└── main.rs             # CLI application
```

### Adding New Graph Types

The architecture supports adding new analyzers:

1. Implement the `GraphAnalyzer` trait
2. Add your analyzer to `src/graph/`
3. Update the CLI options in `main.rs`
4. Add a match arm to instantiate your analyzer

```rust
pub trait GraphAnalyzer {
    fn analyze(&mut self, file_path: &str, content: &str) -> Result<()>;
    fn build_graph(&self) -> DependencyGraph;
}
```

Example: The `NamespaceDependencyAnalyzer` aggregates class-level dependencies to create namespace-level dependency graphs.

## Dependencies

- **mago-syntax**: PHP parser from the Mago toolchain
- **bumpalo**: Arena allocator for AST parsing
- **clap**: Command-line argument parsing
- **walkdir**: Recursive directory traversal
- **anyhow**: Error handling
- **indexmap**: Ordered hash maps for deterministic output

## Limitations

- Only analyzes classes defined within the scanned directories
- External dependencies (e.g., vendor libraries) are not included in the graph
- Does not analyze dynamic class references (e.g., via strings or variables)
- Property promotion in constructors is supported via Mago's AST

## Future Features

- Module/package level dependency analysis
- Detection and highlighting of circular dependencies
- Export to additional formats (JSON, GraphML, Mermaid)
- Integration with PHP autoloading standards (PSR-4)
- Filtering options (exclude vendors, test files, etc.)
- Metrics (coupling, cohesion scores)

## Contributing

Contributions are welcome! Areas for improvement:

- Additional graph types
- Performance optimizations for large codebases
- Better handling of complex PHP features
- Export format options

## License

[Your chosen license]

## Acknowledgments

- Built with the [Mago PHP toolchain](https://github.com/carthage-software/mago)
- Inspired by tools like php-parser and PHPStan
