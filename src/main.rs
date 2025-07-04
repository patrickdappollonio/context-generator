//! Context Generator - CLI entry point.
//!
//! This is the main executable for the context-generator tool. It provides a simple
//! entry point that delegates to the CLI module for argument parsing and execution.

mod cli;
mod filter;
mod scanner;

use std::process;

/// Main entry point for the context-generator CLI tool.
///
/// Parses command-line arguments and executes the appropriate command,
/// handling any errors by printing them to stderr and exiting with status code 1.
fn main() {
    if let Err(e) = cli::run_cli() {
        eprintln!("Error: {e}");
        process::exit(1);
    }
}
