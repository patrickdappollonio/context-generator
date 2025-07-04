//! Context Generator - Generate copy-pastable context from source code for AI assistants.
//!
//! This crate provides a command-line tool and library for scanning source code directories
//! and generating formatted output suitable for providing context to AI assistants like
//! ChatGPT, Claude, or Copilot.
//!
//! # Features
//!
//! - **Smart filtering**: Automatically excludes build artifacts, dependencies, and temporary files
//! - **20+ exclusion categories**: Built-in patterns for popular languages and frameworks
//! - **Configurable**: Disable categories or add custom exclusion patterns
//! - **Fast scanning**: Efficient directory traversal and pattern matching
//! - **AI-optimized output**: Clear, structured format that AI tools understand
//! - **Dry-run mode**: Preview what files would be processed before scanning
//! - **Zero dependencies at runtime**: All exclusion data embedded at compile time
//!
//! # Quick Start
//!
//! ## As a Library
//!
//! ```rust
//! use context_generator::{Filter, Scanner};
//! use std::io;
//!
//! // Create a filter with default exclusions
//! let filter = Filter::new_with_defaults(vec![], &[]).unwrap();
//!
//! // Create scanner and process directory
//! let scanner = Scanner::new(filter);
//! scanner.scan("src/", &mut io::stdout()).unwrap();
//! ```
//!
//! ## As a CLI Tool
//!
//! ```bash
//! # Scan current directory
//! context-generator
//!
//! # Scan specific directory
//! context-generator src/
//!
//! # Preview what would be scanned
//! context-generator --dry-run
//!
//! # Exclude additional patterns
//! context-generator --exclude "*.backup" --exclude "temp/*"
//!
//! # Disable specific exclusion categories
//! context-generator --disable-category logs,vcs
//!
//! # List all available exclusion categories
//! context-generator list-exclusions
//! ```
//!
//! # Architecture
//!
//! The crate is organized into three main modules:
//!
//! - [`filter`]: Pattern matching and exclusion logic
//! - [`scanner`]: File system traversal and content processing
//! - [`cli`]: Command-line interface implementation
//!
//! # Exclusion Categories
//!
//! The tool includes 20+ built-in exclusion categories covering:
//!
//! - **Languages**: Go, Python, JavaScript, TypeScript, Java, C/C++, Rust, Ruby, Swift, etc.
//! - **Build systems**: Maven, Gradle, CMake, Make, etc.
//! - **Package managers**: npm, pip, cargo, composer, etc.
//! - **Development tools**: IDEs, version control, linters, etc.
//! - **Generated files**: Build artifacts, logs, caches, temporary files, etc.
//!
//! Use `context-generator list-exclusions` to see all available categories.

pub mod cli;
pub mod filter;
pub mod scanner;

pub use cli::run_cli;
pub use filter::{ExclusionCategory, ExclusionReason, Filter};
pub use scanner::Scanner;
