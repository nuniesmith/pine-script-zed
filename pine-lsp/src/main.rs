mod ast;
mod builtins;
mod line_index;
mod linter;
mod parser;
mod server;

use std::process;

use dashmap::DashMap;
use server::PineBackend;
use tower_lsp::{LspService, Server};

/// Run the parser and linter against a single file and print diagnostics to
/// stdout. Returns the number of errors found (exit code).
fn check_file(path: &str) -> i32 {
    let source = match std::fs::read_to_string(path) {
        Ok(s) => s,
        Err(e) => {
            eprintln!("error: cannot read {path}: {e}");
            return 1;
        }
    };

    let line_index = line_index::LineIndex::new(&source);
    let result = parser::parse_script(&source);

    let mut error_count = 0u32;
    let mut warn_count = 0u32;
    let mut hint_count = 0u32;

    // Print parse errors
    for err in &result.errors {
        let pos = line_index.position(err.span.start);
        eprintln!(
            "{}:{}:{}: error: {}",
            path,
            pos.line + 1,
            pos.character + 1,
            err.message
        );
        error_count += 1;
    }

    // Run linter if parsing produced a script
    if let Some(ref script) = result.script {
        let lint_diags = linter::lint(script, &source);
        for ld in &lint_diags {
            let pos = line_index.position(ld.span.start);
            let severity = match ld.severity {
                ast::LintSeverity::Error => {
                    error_count += 1;
                    "error"
                }
                ast::LintSeverity::Warning => {
                    warn_count += 1;
                    "warning"
                }
                ast::LintSeverity::Info => {
                    hint_count += 1;
                    "info"
                }
                ast::LintSeverity::Hint => {
                    hint_count += 1;
                    "hint"
                }
            };
            eprintln!(
                "{}:{}:{}: {}: {}",
                path,
                pos.line + 1,
                pos.character + 1,
                severity,
                ld.message
            );
        }
    }

    // Summary
    let total = error_count + warn_count + hint_count;
    if total == 0 {
        println!("  ✓ {path}: no issues found");
    } else {
        println!("  {path}: {error_count} error(s), {warn_count} warning(s), {hint_count} hint(s)");
    }

    if error_count > 0 {
        1
    } else {
        0
    }
}

fn print_usage() {
    eprintln!("Usage:");
    eprintln!("  pine-lsp              Start the LSP server (stdio transport)");
    eprintln!("  pine-lsp --check <file>  Parse and lint a .pine file");
    eprintln!("  pine-lsp --help        Show this help message");
    eprintln!("  pine-lsp --version     Show version");
}

#[tokio::main]
async fn main() {
    let args: Vec<String> = std::env::args().collect();

    // CLI mode: --check <file>
    if args.len() >= 2 {
        match args[1].as_str() {
            "--check" => {
                if args.len() < 3 {
                    eprintln!("error: --check requires a file path");
                    print_usage();
                    process::exit(2);
                }
                let mut exit_code = 0;
                for path in &args[2..] {
                    let code = check_file(path);
                    if code != 0 {
                        exit_code = code;
                    }
                }
                process::exit(exit_code);
            }
            "--help" | "-h" => {
                print_usage();
                process::exit(0);
            }
            "--version" | "-V" => {
                println!("pine-lsp {}", env!("CARGO_PKG_VERSION"));
                process::exit(0);
            }
            other => {
                eprintln!("error: unknown option: {other}");
                print_usage();
                process::exit(2);
            }
        }
    }

    // Default: run as LSP server over stdin/stdout
    let stdin = tokio::io::stdin();
    let stdout = tokio::io::stdout();

    let (service, socket) = LspService::new(|client| PineBackend {
        client,
        documents: DashMap::new(),
    });

    Server::new(stdin, stdout, socket).serve(service).await;
}
