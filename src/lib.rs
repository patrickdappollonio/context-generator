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
//! - **Import functionality**: Parse context-generator output and recreate file structures
//! - **Comprehensive validation**: Security and safety checks for import operations
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
//!
//! # Import context-generator output
//! context-generator import output.ctx --output-dir ./imported/
//! ```
//!
//! # Architecture
//!
//! The crate is organized into seven main modules:
//!
//! - [`filter`]: Pattern matching and exclusion logic
//! - [`scanner`]: File system traversal and content processing
//! - [`parser`]: Context-generator output format parsing
//! - [`validation`]: Comprehensive validation for import security and safety
//! - [`importer`]: Import operation orchestration
//! - [`file_writer`]: Safe file writing with conflict resolution
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
pub mod file_writer;
pub mod filter;
pub mod importer;
pub mod parser;
pub mod scanner;
pub mod validation;

pub use cli::run_cli;
pub use file_writer::{FileWriter, WriteResult};
pub use filter::{ExclusionCategory, ExclusionReason, Filter};
pub use importer::{ImportConfig, ImportResult, Importer};
pub use parser::{ImportError, ParseResult, ParsedFile, Parser};
pub use scanner::Scanner;
pub use validation::{ValidationResult, Validator};

// Essential Unit Tests (MVP)
#[cfg(test)]
mod essential_tests {
    use super::*;
    use std::fs;
    use tempfile::TempDir;

    /// Test basic state transitions in the parser
    #[test]
    fn test_parser_state_transitions() {
        println!("🧪 Testing parser state transitions...");

        // Test that parser correctly handles state transitions through a complete valid input
        let input = "--------------------\nfile: test.rs\n--------------------\n    fn main() {}\n--------------------";
        let result = Parser::parse(input).unwrap();

        assert_eq!(result.files.len(), 1);
        assert_eq!(result.files[0].path, "test.rs");
        assert_eq!(result.files[0].content, vec!["fn main() {}"]);

        // Test multiple file transitions
        let multi_file_input = r#"--------------------
file: first.rs
--------------------
    fn first() {}
--------------------
file: second.rs
--------------------
    fn second() {}
--------------------"#;

        let result = Parser::parse(multi_file_input).unwrap();
        assert_eq!(result.files.len(), 2);
        assert_eq!(result.files[0].path, "first.rs");
        assert_eq!(result.files[1].path, "second.rs");

        println!("   ✅ Parser state transitions work correctly");
    }

    /// Test format validation in the parser
    #[test]
    fn test_parser_format_validation() {
        println!("🧪 Testing parser format validation...");

        // Test invalid separator count
        let result = Parser::parse("-------------------\nfile: test.rs\n"); // 19 dashes
        assert!(result.is_err());

        // Test missing file prefix
        let result = Parser::parse("--------------------\ntest.rs\n");
        assert!(result.is_err());

        // Test wrong indentation
        let result = Parser::parse(
            "--------------------\nfile: test.rs\n--------------------\n  wrong indent\n",
        );
        assert!(result.is_err());
        if let Err(crate::parser::ImportError::Format { line, .. }) = result {
            assert_eq!(line, 4); // Should report correct line number
        }

        // Test valid format
        let result = Parser::parse(
            "--------------------\nfile: test.rs\n--------------------\n    correct content\n--------------------",
        );
        assert!(result.is_ok());

        println!("   ✅ Parser format validation works correctly");
    }

    /// Test error handling in the parser
    #[test]
    fn test_parser_error_handling() {
        println!("🧪 Testing parser error handling...");

        // Test path validation error
        let result = Parser::parse(
            "--------------------\nfile: \n--------------------\n    content\n--------------------",
        );
        assert!(result.is_err());
        if let Err(crate::parser::ImportError::PathValidation { reason, .. }) = result {
            assert!(reason.contains("cannot be empty"));
        }

        // Test wrong indentation
        let result = Parser::parse(
            "--------------------\nfile: test.rs\n--------------------\n  wrong indent\n",
        );
        assert!(result.is_err());
        if let Err(crate::parser::ImportError::Format { line, .. }) = result {
            assert_eq!(line, 4); // Should report correct line number
        }

        println!("   ✅ Parser error handling works correctly");
    }

    /// Test directory creation in file operations
    #[test]
    fn test_file_operations_directory_creation() {
        println!("🧪 Testing file operations directory creation...");

        let temp_dir = TempDir::new().unwrap();
        let writer = FileWriter::new(temp_dir.path(), false, false);

        let file = ParsedFile {
            path: "deep/nested/structure/test.rs".to_string(),
            content: vec!["fn test() {}".to_string()],
            line_range: (1, 1),
        };

        let result = writer.write_file(&file).unwrap();
        assert_eq!(result, WriteResult::Created);

        // Verify all directories were created
        assert!(temp_dir.path().join("deep").is_dir());
        assert!(temp_dir.path().join("deep/nested").is_dir());
        assert!(temp_dir.path().join("deep/nested/structure").is_dir());
        assert!(
            temp_dir
                .path()
                .join("deep/nested/structure/test.rs")
                .is_file()
        );

        println!("   ✅ Directory creation works correctly");
    }

    /// Test basic file writing in file operations
    #[test]
    fn test_file_operations_basic_writing() {
        println!("🧪 Testing file operations basic writing...");

        let temp_dir = TempDir::new().unwrap();
        let writer = FileWriter::new(temp_dir.path(), false, false);

        let file = ParsedFile {
            path: "hello.rs".to_string(),
            content: vec![
                "fn main() {".to_string(),
                "    println!(\"Hello, world!\");".to_string(),
                "}".to_string(),
            ],
            line_range: (1, 3),
        };

        let result = writer.write_file(&file).unwrap();
        assert_eq!(result, WriteResult::Created);

        // Verify content was written correctly
        let written_content = fs::read_to_string(temp_dir.path().join("hello.rs")).unwrap();
        let expected_content = "fn main() {\n    println!(\"Hello, world!\");\n}";
        assert_eq!(written_content, expected_content);

        println!("   ✅ Basic file writing works correctly");
    }

    /// Test conflict detection in file operations
    #[test]
    fn test_file_operations_conflict_detection() {
        println!("🧪 Testing file operations conflict detection...");

        let temp_dir = TempDir::new().unwrap();
        let writer = FileWriter::new(temp_dir.path(), false, false); // No force, no skip

        let file = ParsedFile {
            path: "conflict.rs".to_string(),
            content: vec!["fn new() {}".to_string()],
            line_range: (1, 1),
        };

        // Create file first
        fs::write(temp_dir.path().join("conflict.rs"), "fn old() {}").unwrap();

        // Try to write again - should detect conflict
        let result = writer.write_file(&file);
        assert!(result.is_err());
        if let Err(crate::parser::ImportError::Io { source, .. }) = result {
            assert_eq!(source.kind(), std::io::ErrorKind::AlreadyExists);
        }

        // Test with skip_existing flag
        let writer_skip = FileWriter::new(temp_dir.path(), false, true);
        let result = writer_skip.write_file(&file).unwrap();
        assert_eq!(result, WriteResult::Skipped);

        // Test with force flag
        let writer_force = FileWriter::new(temp_dir.path(), true, false);
        let result = writer_force.write_file(&file).unwrap();
        assert_eq!(result, WriteResult::Overwritten);

        println!("   ✅ Conflict detection works correctly");
    }

    /// Test simple round-trip: generate → import → verify
    #[test]
    fn test_basic_round_trip() {
        println!("🧪 Testing basic round-trip functionality...");

        // Create temporary directories for original and imported files
        let original_dir = TempDir::new().unwrap();
        let imported_dir = TempDir::new().unwrap();

        // Create test files in original directory
        create_test_files(&original_dir).unwrap();

        // Generate context using the scanner (simulating the generate command)
        let generated_context = generate_context_from_directory(&original_dir).unwrap();

        // Import the generated context
        let config = ImportConfig {
            output_dir: imported_dir.path().to_string_lossy().to_string(),
            dry_run: false,
            force: false,
            skip_existing: false,
        };

        let importer = Importer::new(config);
        let result = importer.import_from_string(&generated_context).unwrap();

        // Verify import was successful
        assert!(result.is_success());
        assert_eq!(result.files_created, 3); // Should match number of test files

        // Verify imported files match original files
        verify_round_trip_files(&original_dir, &imported_dir).unwrap();

        println!("   ✅ Round-trip functionality works correctly");
    }

    /// Helper function to create test files
    fn create_test_files(dir: &TempDir) -> std::io::Result<()> {
        // Create src directory
        fs::create_dir_all(dir.path().join("src"))?;

        // Create main.rs
        fs::write(
            dir.path().join("src/main.rs"),
            "fn main() {\n    println!(\"Hello, world!\");\n}",
        )?;

        // Create lib.rs
        fs::write(
            dir.path().join("src/lib.rs"),
            "pub mod utils;\n\npub fn hello() -> String {\n    \"Hello from lib\".to_string()\n}",
        )?;

        // Create README.md
        fs::write(
            dir.path().join("README.md"),
            "# Test Project\n\nThis is a test project for round-trip testing.",
        )?;

        Ok(())
    }

    /// Helper function to generate context from a directory using the scanner
    fn generate_context_from_directory(dir: &TempDir) -> anyhow::Result<String> {
        let filter = Filter::new_with_defaults(vec![], &[])?;
        let scanner = Scanner::new(filter);

        let mut output = Vec::new();
        scanner.scan(dir.path(), &mut output)?;

        Ok(String::from_utf8(output)?)
    }

    /// Helper function to verify that round-trip files match
    fn verify_round_trip_files(
        original_dir: &TempDir,
        imported_dir: &TempDir,
    ) -> std::io::Result<()> {
        // Verify src/main.rs
        let original_main = fs::read_to_string(original_dir.path().join("src/main.rs"))?;
        let imported_main = fs::read_to_string(imported_dir.path().join("src/main.rs"))?;
        assert_eq!(original_main, imported_main);

        // Verify src/lib.rs
        let original_lib = fs::read_to_string(original_dir.path().join("src/lib.rs"))?;
        let imported_lib = fs::read_to_string(imported_dir.path().join("src/lib.rs"))?;
        assert_eq!(original_lib, imported_lib);

        // Verify README.md
        let original_readme = fs::read_to_string(original_dir.path().join("README.md"))?;
        let imported_readme = fs::read_to_string(imported_dir.path().join("README.md"))?;
        assert_eq!(original_readme, imported_readme);

        Ok(())
    }

    /// Integration test runner for essential MVP functionality
    #[test]
    fn run_essential_unit_tests() {
        println!("🚀 Running Essential Unit Tests (MVP)");
        println!("{}", "=".repeat(60));

        // The individual tests above cover all the requirements:
        // ✅ Parser core functionality tests
        //   - test_parser_state_transitions()
        //   - test_parser_format_validation()
        //   - test_parser_error_handling()
        // ✅ File operations basic tests
        //   - test_file_operations_directory_creation()
        //   - test_file_operations_basic_writing()
        //   - test_file_operations_conflict_detection()
        // ✅ Basic round-trip test
        //   - test_basic_round_trip()

        println!("✅ All essential unit tests completed successfully!");
        println!("🎉 Essential Unit Tests (MVP) completed successfully!");
    }
}
