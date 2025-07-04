//! Command-line interface implementation using clap.
//!
//! This module defines the CLI structure, command parsing, and command execution
//! for the context-generator tool. It handles both the main scanning command
//! and the list-exclusions subcommand.

use crate::filter::{
    print_category_exclusions, print_exclusions, print_patterns_only, validate_category_ids, Filter,
};
use crate::scanner::Scanner;
use clap::{Parser, Subcommand};
use std::io;

/// Main CLI structure defining all command-line arguments and options.
///
/// This struct uses clap's derive API to automatically generate argument parsing
/// and help text. It supports both the main scanning operation and subcommands
/// for listing exclusion patterns.
///
/// # Examples
///
/// ```rust
/// use context_generator::cli::Cli;
/// use clap::Parser;
///
/// // Parse from command line args
/// let cli = Cli::parse();
///
/// // Parse from custom args
/// let cli = Cli::try_parse_from(&["program", "--dry-run", "src/"]).unwrap();
/// ```
#[derive(Parser)]
#[command(
    name = "context-generator",
    version,
    author = "Patrick D'appollonio <hey@patrickdap.com>",
    about = "Generate copy-pastable context from your source code for AI assistants"
)]
pub struct Cli {
    /// Directory to scan (defaults to current directory)
    pub directory: Option<String>,

    /// Exclude files/folders matching these patterns (supports wildcards)
    #[arg(long, value_name = "PATTERN")]
    pub exclude: Vec<String>,

    /// Disable default exclusion categories by ID (use list-exclusions to see IDs)
    #[arg(long = "disable-category", value_name = "ID")]
    pub disable_category: Vec<String>,

    /// Disable default exclusions
    #[arg(long)]
    pub no_defaults: bool,

    /// Show files that would be processed and excluded without generating output
    #[arg(long)]
    pub dry_run: bool,

    #[command(subcommand)]
    pub command: Option<Commands>,
}

/// Available subcommands for the CLI.
///
/// Currently supports the `list-exclusions` subcommand for exploring
/// available exclusion categories and patterns.
#[derive(Subcommand)]
pub enum Commands {
    /// List all default exclusions organized by category
    ListExclusions {
        /// Show patterns for a specific category ID only
        #[arg(long, value_name = "ID")]
        category: Option<String>,

        /// Show only patterns ordered by category (wildcards first, then literals)
        #[arg(long)]
        patterns_only: bool,
    },
}

/// Main entry point for CLI execution.
///
/// This function parses command-line arguments and dispatches to the appropriate
/// handler based on whether a subcommand was provided or not.
///
/// # Returns
///
/// * `Ok(())` - Command executed successfully
/// * `Err(anyhow::Error)` - Error during execution (invalid args, IO errors, etc.)
///
/// # Examples
///
/// ```rust
/// use context_generator::cli::run_cli;
///
/// // This would typically be called from main()
/// if let Err(e) = run_cli() {
///     eprintln!("Error: {}", e);
///     std::process::exit(1);
/// }
/// ```
pub fn run_cli() -> anyhow::Result<()> {
    let cli = Cli::parse();

    match &cli.command {
        Some(Commands::ListExclusions {
            category,
            patterns_only,
        }) => {
            handle_list_exclusions(category.as_deref(), *patterns_only)?;
        }
        None => {
            handle_main_command(&cli)?;
        }
    }

    Ok(())
}

/// Handles the `list-exclusions` subcommand.
///
/// This function processes the list-exclusions subcommand with its various options:
/// - No options: Shows all categories with descriptions and patterns
/// - `--category <id>`: Shows patterns for a specific category
/// - `--patterns-only`: Shows all patterns without category information
///
/// # Arguments
///
/// * `category` - Optional category ID to show patterns for
/// * `patterns_only` - Whether to show only patterns without category info
///
/// # Returns
///
/// * `Ok(())` - Successfully printed exclusion information
/// * `Err(anyhow::Error)` - Invalid category ID or IO error
///
/// # Examples
///
/// This function is called internally by the CLI parser and is not part of the public API.
/// Use the CLI commands instead:
///
/// ```bash
/// # Show all categories
/// context-generator list-exclusions
///
/// # Show specific category
/// context-generator list-exclusions --category python
///
/// # Show patterns only
/// context-generator list-exclusions --patterns-only
/// ```
fn handle_list_exclusions(category: Option<&str>, patterns_only: bool) -> anyhow::Result<()> {
    let mut stdout = io::stdout();

    if let Some(category_id) = category {
        let invalid = validate_category_ids(&[category_id.to_string()]);
        if !invalid.is_empty() {
            return Err(anyhow::anyhow!(
                "Invalid category ID: {}. Use \"list-exclusions\" to see valid IDs",
                invalid.join(", ")
            ));
        }
        print_category_exclusions(&mut stdout, category_id)?;
    } else if patterns_only {
        print_patterns_only(&mut stdout)?;
    } else {
        print_exclusions(&mut stdout)?;
    }

    Ok(())
}

/// Handles the main scanning command.
///
/// This function processes the main context generation command. It:
/// 1. Validates any disabled category IDs
/// 2. Creates an appropriate filter based on options
/// 3. Creates a scanner and executes either scanning or dry-run mode
///
/// # Arguments
///
/// * `cli` - Parsed CLI arguments containing all options
///
/// # Returns
///
/// * `Ok(())` - Successfully completed scanning or dry-run
/// * `Err(anyhow::Error)` - Invalid arguments, filter creation failure, or scanner error
///
/// # Examples
///
/// This function is called internally by the CLI parser and is not part of the public API.
/// Use the main CLI interface instead:
///
/// ```bash
/// # Basic usage
/// context-generator src/
///
/// # With options
/// context-generator --dry-run --exclude "*.backup" src/
/// ```
fn handle_main_command(cli: &Cli) -> anyhow::Result<()> {
    let directory = cli.directory.as_deref().unwrap_or(".");
    let exclude_patterns = cli.exclude.clone();
    let disable_categories: Vec<String> = cli
        .disable_category
        .iter()
        .flat_map(|s| s.split(',').map(|s| s.trim().to_string()))
        .collect();

    // Validate disabled category IDs
    if !disable_categories.is_empty() {
        let invalid = validate_category_ids(&disable_categories);
        if !invalid.is_empty() {
            return Err(anyhow::anyhow!(
                "Invalid category IDs: {}. Use 'list-exclusions' to see valid IDs",
                invalid.join(", ")
            ));
        }
    }

    // Create the appropriate filter
    let filter = if cli.no_defaults {
        Filter::new(exclude_patterns)?
    } else {
        Filter::new_with_defaults(exclude_patterns, &disable_categories)?
    };

    // Create scanner and run
    let scanner = Scanner::new(filter);
    let mut stdout = io::stdout();

    if cli.dry_run {
        scanner.dry_run(directory, &mut stdout)?;
    } else {
        scanner.scan(directory, &mut stdout)?;
    }

    Ok(())
}
