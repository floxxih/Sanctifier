#![recursion_limit = "512"]

use clap::{Parser, Subcommand};
use colored::*;
use sanctifier_core::{Analyzer, ArithmeticIssue, PanicIssue, SizeWarning, UnsafePattern};
use std::fs;
use std::path::{Path, PathBuf};
use sanctifier_core::{callgraph_to_dot, Analyzer, SanctifyConfig};
use std::fs;
use std::path::{Path, PathBuf};
use tracing::error;

mod commands;
mod logging;
pub mod vulndb;

#[derive(Parser)]
#[command(name = "sanctifier")]
#[command(about = "Stellar Soroban Security & Formal Verification Suite", long_about = None)]
struct Cli {
    #[command(subcommand)]
    command: Commands,
}

#[derive(Subcommand)]
pub enum Commands {
    /// Analyze a Soroban contract for vulnerabilities
    Analyze(commands::analyze::AnalyzeArgs),
    /// Compare current scan results against a baseline to find only NEW vulnerabilities
    Diff(commands::diff::DiffArgs),
    /// Generate a dynamic Sanctifier status badge
    Badge(commands::badge::BadgeArgs),
    /// Generate a Markdown or HTML security report
    Report(commands::report::ReportArgs),
    /// Detect potential storage key collisions in Soroban contracts
    Storage(commands::storage::StorageArgs),
    /// Initialize Sanctifier in a new project
    Init(commands::init::InitArgs),
    /// Show per-contract complexity metrics (cyclomatic complexity, nesting, LOC)
    Complexity(commands::complexity::ComplexityArgs),
    /// Generate a Graphviz DOT call graph of cross-contract calls (env.invoke_contract)
    Callgraph {
        /// Path to a contract directory, workspace directory, or a single .rs file
        #[arg(default_value = ".")]
        path: PathBuf,
        
        /// Output format: text | json | junit
        #[arg(short, long, default_value = "text")]
        format: String,

        /// Output DOT file path
        #[arg(short, long, default_value = "callgraph.dot")]
        output: PathBuf,
    },
    /// Check for and download the latest Sanctifier binary
    Update,
    /// Detect reentrancy vulnerabilities (state mutation before external call)
    Reentrancy(commands::reentrancy::ReentrancyArgs),
    /// Verify local source against on-chain bytecode
    Verify(commands::verify::VerifyArgs),
    /// Analyze an entire Cargo workspace (multiple contracts/libs)
    Workspace(commands::workspace::WorkspaceArgs),
}

fn main() {
    if let Err(err) = run() {
        eprintln!("Error: {}", err);
        std::process::exit(2);
    }
}

    match &cli.command {
        Commands::Analyze { path, format, limit } => {
            let is_json = format == "json";
            let is_junit = format == "junit";
            let is_machine = is_json || is_junit;
fn run() -> anyhow::Result<()> {
    let cli = Cli::parse();
    let log_output = match &cli.command {
        Commands::Analyze(args) if args.format == "json" => logging::LogOutput::Json,
        Commands::Diff(args) if args.format == "json" => logging::LogOutput::Json,
        Commands::Storage(args) if args.format == commands::storage::OutputFormat::Json => {
            logging::LogOutput::Json
        }
        _ => logging::LogOutput::Text,
    };
    logging::init(log_output)?;

    match cli.command {
        Commands::Analyze(args) => commands::analyze::exec(args)?,
        Commands::Diff(args) => commands::diff::exec(args)?,
        Commands::Badge(args) => {
            commands::badge::exec(args)?;
        }
        Commands::Complexity(args) => {
            commands::complexity::exec(args)?;
        }
        Commands::Report(args) => {
            commands::report::exec(args)?;
        }
        Commands::Storage(args) => {
            commands::storage::exec(args)?;
        }
        Commands::Init(args) => {
            let path = Some(args.path.clone());
            commands::init::exec(args, path)?;
        }
        Commands::Callgraph { path, output } => {
            let config = load_config(&path);
            let analyzer = Analyzer::new(config.clone());

            // In machine-output modes send informational lines to stderr so stdout is clean.
            if is_machine {
                eprintln!("{} Sanctifier: Valid Soroban project found at {:?}", "✨".green(), path);
                eprintln!("{} Analyzing contract at {:?}...", "🔍".blue(), path);
            let mut rs_files: Vec<PathBuf> = Vec::new();
            if path.is_dir() {
                collect_rs_files(&path, &config, &mut rs_files);
            } else {
                rs_files.push(path.clone());
            }

            let mut edges = Vec::new();
            for f in rs_files {
                if f.extension().and_then(|s| s.to_str()) != Some("rs") {
                    continue;
                }

                let content = match fs::read_to_string(&f) {
                    Ok(c) => c,
                    Err(_) => continue,
                };

                let caller = infer_contract_name(&content).unwrap_or_else(|| {
                    f.file_stem()
                        .and_then(|s| s.to_str())
                        .unwrap_or("<unknown>")
                        .to_string()
                });

                let file_label = f.display().to_string();
                edges.extend(analyzer.scan_invoke_contract_calls(&content, &caller, &file_label));
            }

            if is_machine {
                eprintln!("{} Static analysis complete.", "✅".green());
            } else {
                println!("{} Static analysis complete.", "✅".green());
            }

            if is_json {
                let output = serde_json::json!({
                    "size_warnings": all_size_warnings,
                    "unsafe_patterns": all_unsafe_patterns,
                    "auth_gaps": all_auth_gaps,
                    "panic_issues": all_panic_issues,
                    "arithmetic_issues": all_arithmetic_issues,
                });
                println!("{}", serde_json::to_string_pretty(&output).unwrap_or_else(|_| "{}".to_string()));
            } else if is_junit {
                print_junit_report(
                    &all_size_warnings,
                    &all_unsafe_patterns,
                    &all_auth_gaps,
                    &all_panic_issues,
                    &all_arithmetic_issues,
                );
            } else {
                if all_size_warnings.is_empty() {
                    println!("No ledger size issues found.");
                } else {
                    for warning in &all_size_warnings {
                        println!(
                            "{} Warning: Struct {} is approaching ledger entry size limit!",
                            "⚠️".yellow(),
                            warning.struct_name.bold()
                        );
                        println!(
                            "   Estimated size: {} bytes (Limit: {} bytes)",
                            warning.estimated_size.to_string().red(),
                            warning.limit
                        );
                    }
                }

                if !all_auth_gaps.is_empty() {
                    println!("\n{} Found potential Authentication Gaps!", "🛑".red());
                    for gap in &all_auth_gaps {
                        println!("   {} Function {} is modifying state without require_auth()", "->".red(), gap.bold());
                    }
                } else {
                    println!("\nNo authentication gaps found.");
                }

                if !all_panic_issues.is_empty() {
                    println!("\n{} Found explicit Panics/Unwraps!", "🛑".red());
                    for issue in &all_panic_issues {
                        println!(
                            "   {} Function {}: Using {} (Location: {})",
                            "->".red(),
                            issue.function_name.bold(),
                            issue.issue_type.yellow().bold(),
                            issue.location
                        );
                    }
                    println!("   {} Tip: Prefer returning Result or Error types for better contract safety.", "💡".blue());
                } else {
                    println!("\nNo panic/unwrap issues found.");
                }

                if !all_arithmetic_issues.is_empty() {
                    println!("\n{} Found unchecked Arithmetic Operations!", "🔢".yellow());
                    for issue in &all_arithmetic_issues {
                        println!(
                            "   {} Function {}: Unchecked `{}` ({})",
                            "->".red(),
                            issue.function_name.bold(),
                            issue.operation.yellow().bold(),
                            issue.location
                        );
                        println!("      {} {}", "💡".blue(), issue.suggestion);
                    }
                } else {
                    println!("\nNo arithmetic overflow risks found.");
                }
            }
        },
        Commands::Report { output } => {
            println!("{} Generating report...", "📄".yellow());
            if let Some(p) = output {
                println!("Report saved to {:?}", p);
            } else {
                println!("Report printed to stdout.");
            let dot = callgraph_to_dot(&edges);
            if let Err(e) = fs::write(&output, dot) {
                error!(
                    target: "sanctifier",
                    output = %output.display(),
                    error = %e,
                    "Failed to write DOT file"
                );
                std::process::exit(1);
            }
            println!(
                "{} Wrote call graph to {:?} ({} edges)",
                "✅".green(),
                output,
                edges.len()
            );
        }
        Commands::Update => {
            commands::update::exec()?;
        }
        Commands::Reentrancy(args) => {
            commands::reentrancy::exec(args)?;
        }
        Commands::Verify(args) => {
            commands::verify::exec(args)?;
        }
        Commands::Workspace(args) => {
            commands::workspace::exec(args)?;
        }
    }

    Ok(())
}

fn collect_rs_files(dir: &Path, config: &SanctifyConfig, out: &mut Vec<PathBuf>) {
    let entries = match fs::read_dir(dir) {
        Ok(e) => e,
        Err(_) => return,
    };

    for entry in entries.flatten() {
        let path = entry.path();
        let name = path.file_name().and_then(|n| n.to_str()).unwrap_or("");
        if path.is_dir() {
            if config.ignore_paths.iter().any(|p| name.contains(p)) {
                continue;
            }
            collect_rs_files(&path, config, out);
        } else if path.extension().and_then(|s| s.to_str()) == Some("rs") {
            out.push(path);
        }
    }
}

fn infer_contract_name(source: &str) -> Option<String> {
    let mut saw_contract_attr = false;
    for line in source.lines() {
        let l = line.trim();
        if l.starts_with("#[contract]") {
            saw_contract_attr = true;
            continue;
        }
        if saw_contract_attr {
            if let Some(rest) = l.strip_prefix("pub struct ") {
                return Some(
                    rest.trim_end_matches(';')
                        .split_whitespace()
                        .next()?
                        .to_string(),
                );
            }
            if let Some(rest) = l.strip_prefix("struct ") {
                return Some(
                    rest.trim_end_matches(';')
                        .split_whitespace()
                        .next()?
                        .to_string(),
                );
            }
        }
    }
    None
}

// ── JUnit XML output ──────────────────────────────────────────────────────────

fn xml_escape(s: &str) -> String {
    s.replace('&', "&amp;")
        .replace('<', "&lt;")
        .replace('>', "&gt;")
        .replace('"', "&quot;")
        .replace('\'', "&apos;")
}

fn junit_suite(name: &str, cases: &[String]) -> String {
    let failures = cases.iter().filter(|c| c.contains("<failure")).count();
    let mut out = format!(
        "  <testsuite name=\"{name}\" tests=\"{tests}\" failures=\"{failures}\" errors=\"0\" time=\"0\">\n",
        name = name,
        tests = cases.len(),
        failures = failures,
    );
    for c in cases {
        out.push_str(c);
        out.push('\n');
    }
    out.push_str("  </testsuite>");
    out
}

fn print_junit_report(
    size_warnings: &[SizeWarning],
    unsafe_patterns: &[UnsafePattern],
    auth_gaps: &[String],
    panic_issues: &[PanicIssue],
    arithmetic_issues: &[ArithmeticIssue],
) {
    let mut total_tests = 0usize;
    let mut total_failures = 0usize;
    let mut suites: Vec<String> = Vec::new();

    // auth_gaps
    {
        let cases: Vec<String> = if auth_gaps.is_empty() {
            total_tests += 1;
            vec!["    <testcase name=\"no_auth_gaps\" classname=\"sanctifier.auth_gaps\" time=\"0\"/>".into()]
        } else {
            total_tests += auth_gaps.len();
            total_failures += auth_gaps.len();
            auth_gaps.iter().enumerate().map(|(i, g)| {
                format!("    <testcase name=\"auth_gap_{i}\" classname=\"sanctifier.auth_gaps\" time=\"0\"><failure message=\"Authentication gap detected\" type=\"AuthGap\">{}</failure></testcase>", xml_escape(g))
            }).collect()
        };
        suites.push(junit_suite("auth_gaps", &cases));
    }

    // panic_issues
    {
        let cases: Vec<String> = if panic_issues.is_empty() {
            total_tests += 1;
            vec!["    <testcase name=\"no_panic_issues\" classname=\"sanctifier.panic_issues\" time=\"0\"/>".into()]
        } else {
            total_tests += panic_issues.len();
            total_failures += panic_issues.len();
            panic_issues.iter().enumerate().map(|(i, p)| {
                format!("    <testcase name=\"panic_issue_{i}\" classname=\"sanctifier.panic_issues\" time=\"0\"><failure message=\"{} detected\" type=\"PanicIssue\">{}</failure></testcase>",
                    xml_escape(&p.issue_type), xml_escape(&p.location))
            }).collect()
        };
        suites.push(junit_suite("panic_issues", &cases));
    }

    // arithmetic_issues
    {
        let cases: Vec<String> = if arithmetic_issues.is_empty() {
            total_tests += 1;
            vec!["    <testcase name=\"no_arithmetic_issues\" classname=\"sanctifier.arithmetic_issues\" time=\"0\"/>".into()]
        } else {
            total_tests += arithmetic_issues.len();
            total_failures += arithmetic_issues.len();
            arithmetic_issues.iter().enumerate().map(|(i, a)| {
                format!("    <testcase name=\"arithmetic_issue_{i}\" classname=\"sanctifier.arithmetic_issues\" time=\"0\"><failure message=\"{} overflow risk\" type=\"ArithmeticIssue\">{}: {}</failure></testcase>",
                    xml_escape(&a.operation), xml_escape(&a.location), xml_escape(&a.suggestion))
            }).collect()
        };
        suites.push(junit_suite("arithmetic_issues", &cases));
    }

    // size_warnings
    {
        let cases: Vec<String> = if size_warnings.is_empty() {
            total_tests += 1;
            vec!["    <testcase name=\"no_size_warnings\" classname=\"sanctifier.size_warnings\" time=\"0\"/>".into()]
        } else {
            total_tests += size_warnings.len();
            total_failures += size_warnings.len();
            size_warnings.iter().enumerate().map(|(i, w)| {
                format!("    <testcase name=\"size_warning_{i}\" classname=\"sanctifier.size_warnings\" time=\"0\"><failure message=\"{name} exceeds ledger size limit\" type=\"SizeWarning\">{name}: {size} bytes (limit: {limit})</failure></testcase>",
                    name = xml_escape(&w.struct_name), size = w.estimated_size, limit = w.limit)
            }).collect()
        };
        suites.push(junit_suite("size_warnings", &cases));
    }

    // unsafe_patterns
    {
        let cases: Vec<String> = if unsafe_patterns.is_empty() {
            total_tests += 1;
            vec!["    <testcase name=\"no_unsafe_patterns\" classname=\"sanctifier.unsafe_patterns\" time=\"0\"/>".into()]
        } else {
            total_tests += unsafe_patterns.len();
            total_failures += unsafe_patterns.len();
            unsafe_patterns.iter().enumerate().map(|(i, p)| {
                format!("    <testcase name=\"unsafe_pattern_{i}\" classname=\"sanctifier.unsafe_patterns\" time=\"0\"><failure message=\"Unsafe pattern detected\" type=\"UnsafePattern\">{}</failure></testcase>",
                    xml_escape(&p.snippet))
            }).collect()
        };
        suites.push(junit_suite("unsafe_patterns", &cases));
    }

    println!("<?xml version=\"1.0\" encoding=\"UTF-8\"?>");
    println!(
        "<testsuites name=\"sanctifier-analysis\" tests=\"{total_tests}\" failures=\"{total_failures}\" errors=\"0\" time=\"0\">",
        total_tests = total_tests,
        total_failures = total_failures,
    );
    for suite in &suites {
        println!("{}", suite);
    }
    println!("</testsuites>");
}

// ── Directory walker ──────────────────────────────────────────────────────────

#[allow(clippy::too_many_arguments)]
fn analyze_directory(
    dir: &Path,
    analyzer: &Analyzer,
    all_size_warnings: &mut Vec<SizeWarning>,
    all_unsafe_patterns: &mut Vec<UnsafePattern>,
    all_auth_gaps: &mut Vec<String>,
    all_panic_issues: &mut Vec<PanicIssue>,
    all_arithmetic_issues: &mut Vec<ArithmeticIssue>,
) {
    if let Ok(entries) = fs::read_dir(dir) {
        for entry in entries.flatten() {
            let path = entry.path();
            if path.is_dir() {
                analyze_directory(&path, analyzer, all_size_warnings, all_unsafe_patterns, all_auth_gaps, all_panic_issues, all_arithmetic_issues);
            } else if path.extension().and_then(|s| s.to_str()) == Some("rs") {
                if let Ok(content) = fs::read_to_string(&path) {
                    let warnings = analyzer.analyze_ledger_size(&content);
                    for mut w in warnings {
                        w.struct_name = format!("{}: {}", path.display(), w.struct_name);
                        all_size_warnings.push(w);
                    }

                    let gaps = analyzer.scan_auth_gaps(&content);
                    for g in gaps {
                        all_auth_gaps.push(format!("{}: {}", path.display(), g));
                    }

                    let panics = analyzer.scan_panics(&content);
                    for p in panics {
                        let mut p_mod = p.clone();
                        p_mod.location = format!("{}: {}", path.display(), p.location);
                        all_panic_issues.push(p_mod);
                    }
fn load_config(path: &Path) -> SanctifyConfig {
    let mut current = if path.is_file() {
        path.parent()
            .map(|p| p.to_path_buf())
            .unwrap_or_else(|| PathBuf::from("."))
    } else {
        path.to_path_buf()
    };

    loop {
        let config_path = current.join(".sanctify.toml");
        if config_path.exists() {
            if let Ok(content) = fs::read_to_string(&config_path) {
                match toml::from_str(&content) {
                    Ok(config) => return config,
                    Err(e) => {
                        eprintln!(
                            "Error: Found .sanctify.toml at {} but it could not be parsed:\n  {}\n\
                             \n\
                             Run 'sanctifier init' to regenerate a valid config, or check the schema at:\n\
                             https://github.com/HyperSafeD/Sanctifier/blob/main/schemas/sanctify-config.schema.json",
                            config_path.display(),
                            e
                        );
                        std::process::exit(1);
                    }
                }
            }
        }
        if !current.pop() {
            break;
        }
    }
    SanctifyConfig::default()
}
