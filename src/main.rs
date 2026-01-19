mod analyzer;
mod graph;

use analyzer::{read_file, PhpFileDiscovery};
use clap::Parser;
use graph::{
    class_dependency::ClassDependencyAnalyzer,
    namespace_dependency::NamespaceDependencyAnalyzer,
    dot_writer::DotWriter,
    module_recommender::ModuleRecommender,
    GraphAnalyzer,
};
use std::fs::File;
use std::io::BufWriter;
use std::path::PathBuf;

#[derive(Parser, Debug)]
#[command(author, version, about, long_about = None)]
#[command(name = "php-modfather")]
#[command(about = "Analyze PHP monoliths and generate dependency graphs")]
struct Cli {
    /// Directory or directories containing PHP files to analyze
    #[arg(required = true)]
    paths: Vec<PathBuf>,

    /// Output file for the DOT graph (default: stdout)
    #[arg(short, long)]
    output: Option<PathBuf>,

    /// Name of the graph
    #[arg(short = 'n', long, default_value = "php_dependencies")]
    graph_name: String,

    /// Type of analysis to perform
    #[arg(short = 't', long, default_value = "class", value_parser = ["class", "namespace", "recommend"])]
    analysis_type: String,

    /// Include external dependencies (classes referenced but not defined in analyzed code)
    #[arg(long)]
    include_external: bool,

    /// Verbose output
    #[arg(short, long)]
    verbose: bool,
}

fn main() -> anyhow::Result<()> {
    let cli = Cli::parse();

    // Discover PHP files
    let mut discovery = PhpFileDiscovery::new();
    for path in &cli.paths {
        if !path.exists() {
            eprintln!("Error: Path does not exist: {}", path.display());
            std::process::exit(1);
        }

        if path.is_dir() {
            discovery.scan_directory(path)?;
        } else if path.is_file() {
            discovery.paths.push(path.clone());
        }
    }

    let files = discovery.get_files();
    if cli.verbose {
        println!("Found {} PHP files", files.len());
    }

    // Handle "recommend" mode differently - it generates a text report, not a DOT graph
    if cli.analysis_type == "recommend" {
        // For recommendations, we need namespace-level analysis
        let mut analyzer = NamespaceDependencyAnalyzer::new();

        // Analyze each file
        for (i, file_path) in files.iter().enumerate() {
            if cli.verbose {
                println!("[{}/{}] Analyzing: {}", i + 1, files.len(), file_path.display());
            }

            match read_file(file_path) {
                Ok(content) => {
                    if let Err(e) = analyzer.analyze(&file_path.display().to_string(), &content) {
                        eprintln!("Warning: Failed to analyze {}: {}", file_path.display(), e);
                    }
                }
                Err(e) => {
                    eprintln!("Warning: Failed to read {}: {}", file_path.display(), e);
                }
            }
        }

        // Build namespace dependency graph (without external dependencies for cleaner analysis)
        let graph = analyzer.build_graph(false);

        if cli.verbose {
            println!("\nAnalyzing modularization opportunities...\n");
        }

        // Generate recommendations
        let recommender = ModuleRecommender::new(&graph);
        let report = recommender.generate_report();

        // Output report
        let report_text = report.format_text();

        if let Some(output_path) = cli.output {
            std::fs::write(&output_path, report_text)?;
            if cli.verbose {
                println!("Report written to: {}", output_path.display());
            }
        } else {
            println!("{}", report_text);
        }
    } else {
        // Standard graph generation mode
        let mut analyzer: Box<dyn GraphAnalyzer> = match cli.analysis_type.as_str() {
            "class" => Box::new(ClassDependencyAnalyzer::new()),
            "namespace" => Box::new(NamespaceDependencyAnalyzer::new()),
            _ => {
                eprintln!("Unknown analysis type: {}", cli.analysis_type);
                std::process::exit(1);
            }
        };

        // Analyze each file
        for (i, file_path) in files.iter().enumerate() {
            if cli.verbose {
                println!("[{}/{}] Analyzing: {}", i + 1, files.len(), file_path.display());
            }

            match read_file(file_path) {
                Ok(content) => {
                    if let Err(e) = analyzer.analyze(&file_path.display().to_string(), &content) {
                        eprintln!("Warning: Failed to analyze {}: {}", file_path.display(), e);
                    }
                }
                Err(e) => {
                    eprintln!("Warning: Failed to read {}: {}", file_path.display(), e);
                }
            }
        }

        // Build the dependency graph
        let graph = analyzer.build_graph(cli.include_external);

        if cli.verbose {
            println!("\nGraph statistics:");
            println!("  Nodes: {}", graph.nodes.len());
            println!("  Edges: {}", graph.edges.len());
        }

        // Write the graph in DOT format
        let writer = DotWriter::new(&cli.graph_name);

        if let Some(output_path) = cli.output {
            let file = File::create(&output_path)?;
            let mut buf_writer = BufWriter::new(file);
            writer.write(&graph, &mut buf_writer)?;
            if cli.verbose {
                println!("\nGraph written to: {}", output_path.display());
            }
        } else {
            let stdout = std::io::stdout();
            let mut handle = stdout.lock();
            writer.write(&graph, &mut handle)?;
        }
    }

    Ok(())
}
