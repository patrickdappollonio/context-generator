//! Import orchestration module.
//!
//! This module provides the high-level import functionality that coordinates
//! parsing, validation, and file writing operations to recreate file structures
//! from context-generator output.

use crate::file_writer::{FileWriter, WriteResult};
use crate::parser::{ImportError, Parser};
use crate::validation::Validator;
use anyhow::Result;
use std::io::Read;

/// Main importer structure that orchestrates the import process
#[derive(Debug)]
pub struct Importer {
    /// Configuration for the import operation
    config: ImportConfig,
    /// Validator for comprehensive validation checks
    validator: Validator,
}

/// Configuration options for import operations
#[derive(Debug, Clone)]
pub struct ImportConfig {
    /// Output directory where files should be created
    pub output_dir: String,
    /// Whether this is a dry run (preview only)
    pub dry_run: bool,
    /// Whether to overwrite existing files
    pub force: bool,
    /// Whether to skip existing files instead of failing
    pub skip_existing: bool,
}

impl Default for ImportConfig {
    fn default() -> Self {
        Self {
            output_dir: ".".to_string(),
            dry_run: false,
            force: false,
            skip_existing: false,
        }
    }
}

impl Importer {
    /// Create a new importer with the specified configuration
    pub fn new(config: ImportConfig) -> Self {
        Self {
            config,
            validator: Validator::new(),
        }
    }

    /// Import from a file path (or "-" for stdin)
    pub fn import_from_path(&self, input_path: &str) -> Result<ImportResult> {
        let content = if input_path == "-" {
            self.read_from_stdin()?
        } else {
            std::fs::read_to_string(input_path).map_err(|e| ImportError::Io {
                path: input_path.to_string(),
                source: e,
            })?
        };

        self.import_from_string(&content)
    }

    /// Import from a string containing the context-generator output
    pub fn import_from_string(&self, content: &str) -> Result<ImportResult> {
        // Parse the input content
        let parse_result = Parser::parse(content)?;

        // Validate the parsed content
        let validation_result = self
            .validator
            .validate_import(&parse_result, &self.config.output_dir);

        // If validation failed, return the errors
        if !validation_result.is_valid {
            return Ok(ImportResult {
                files_processed: parse_result.files.len(),
                files_created: 0,
                files_skipped: 0,
                warnings: self.merge_warnings(&parse_result.warnings, &validation_result.warnings),
                errors: self.merge_errors(&parse_result.errors, &validation_result.errors),
            });
        }

        // If this is a dry run, just return the summary without writing files
        if self.config.dry_run {
            return Ok(ImportResult {
                files_processed: parse_result.files.len(),
                files_created: 0,
                files_skipped: 0,
                warnings: self.merge_warnings(&parse_result.warnings, &validation_result.warnings),
                errors: parse_result.errors,
            });
        }

        // Create the file writer with the current configuration
        let file_writer = FileWriter::new(
            &self.config.output_dir,
            self.config.force,
            self.config.skip_existing,
        );

        // Write all files and collect results
        let mut files_created = 0;
        let mut files_skipped = 0;
        let mut errors = parse_result.errors;
        let warnings = self.merge_warnings(&parse_result.warnings, &validation_result.warnings);

        for file in &parse_result.files {
            match file_writer.write_file(file) {
                Ok(WriteResult::Created) => files_created += 1,
                Ok(WriteResult::Overwritten) => files_created += 1,
                Ok(WriteResult::Skipped) => files_skipped += 1,
                Err(e) => {
                    errors.push(format!("failed to write {}: {}", file.path, e));
                }
            }
        }

        Ok(ImportResult {
            files_processed: parse_result.files.len(),
            files_created,
            files_skipped,
            warnings,
            errors,
        })
    }

    /// Read input from stdin
    fn read_from_stdin(&self) -> Result<String> {
        let mut buffer = String::new();
        std::io::stdin()
            .read_to_string(&mut buffer)
            .map_err(|e| ImportError::Io {
                path: "<stdin>".to_string(),
                source: e,
            })?;
        Ok(buffer)
    }

    /// Merge warnings from different sources
    fn merge_warnings(
        &self,
        parse_warnings: &[String],
        validation_warnings: &[String],
    ) -> Vec<String> {
        let mut warnings = parse_warnings.to_vec();
        warnings.extend_from_slice(validation_warnings);
        warnings
    }

    /// Merge errors from different sources
    fn merge_errors(&self, parse_errors: &[String], validation_errors: &[String]) -> Vec<String> {
        let mut errors = parse_errors.to_vec();
        errors.extend_from_slice(validation_errors);
        errors
    }
}

/// Result of an import operation
#[derive(Debug)]
pub struct ImportResult {
    /// Total number of files processed from input
    pub files_processed: usize,
    /// Number of files actually created on disk
    pub files_created: usize,
    /// Number of files skipped (already existed)
    pub files_skipped: usize,
    /// Warnings encountered during import
    pub warnings: Vec<String>,
    /// Errors encountered during import
    pub errors: Vec<String>,
}

impl ImportResult {
    /// Check if the import operation was successful
    pub fn is_success(&self) -> bool {
        self.errors.is_empty()
    }

    /// Get a summary message for the import operation
    pub fn summary(&self) -> String {
        if self.errors.is_empty() {
            format!(
                "Import completed: {} files processed, {} created, {} skipped",
                self.files_processed, self.files_created, self.files_skipped
            )
        } else {
            format!(
                "Import failed: {} errors, {} warnings",
                self.errors.len(),
                self.warnings.len()
            )
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::TempDir;

    #[test]
    fn test_import_config_default() {
        let config = ImportConfig::default();
        assert_eq!(config.output_dir, ".");
        assert!(!config.dry_run);
        assert!(!config.force);
        assert!(!config.skip_existing);
    }

    #[test]
    fn test_importer_creation() {
        let config = ImportConfig::default();
        let importer = Importer::new(config);
        assert_eq!(importer.config.output_dir, ".");
    }

    #[test]
    fn test_import_result_success() {
        let result = ImportResult {
            files_processed: 3,
            files_created: 3,
            files_skipped: 0,
            warnings: vec![],
            errors: vec![],
        };

        assert!(result.is_success());
        assert!(result.summary().contains("3 files processed"));
    }

    #[test]
    fn test_import_result_with_errors() {
        let result = ImportResult {
            files_processed: 3,
            files_created: 1,
            files_skipped: 0,
            warnings: vec!["test warning".to_string()],
            errors: vec!["test error".to_string()],
        };

        assert!(!result.is_success());
        assert!(result.summary().contains("Import failed"));
        assert!(result.summary().contains("1 errors"));
        assert!(result.summary().contains("1 warnings"));
    }

    #[test]
    fn test_import_from_string_basic() {
        let temp_dir = TempDir::new().unwrap();
        let mut config = ImportConfig::default();
        config.output_dir = temp_dir.path().to_string_lossy().to_string();
        let importer = Importer::new(config);

        let input = "--------------------\nfile: test.rs\n--------------------\n    fn main() {}\n--------------------";
        let result = importer.import_from_string(input).unwrap();

        assert_eq!(result.files_processed, 1);
        assert!(result.is_success());
    }

    #[test]
    fn test_import_from_string_invalid_format() {
        let config = ImportConfig::default();
        let importer = Importer::new(config);

        let input = "invalid format";
        let result = importer.import_from_string(input);

        assert!(result.is_err());
    }

    #[test]
    fn test_import_validation_failure() {
        let config = ImportConfig::default();
        let importer = Importer::new(config);

        // Use absolute path which should fail validation
        let input = "--------------------\nfile: /etc/passwd\n--------------------\n    content\n--------------------";
        let result = importer.import_from_string(input).unwrap();

        assert!(!result.is_success());
        assert!(!result.errors.is_empty());
        assert!(result.errors[0].contains("absolute path not allowed"));
    }

    #[test]
    fn test_import_dry_run() {
        let mut config = ImportConfig::default();
        config.dry_run = true;
        let importer = Importer::new(config);

        let input = "--------------------\nfile: test.rs\n--------------------\n    fn main() {}\n--------------------";
        let result = importer.import_from_string(input).unwrap();

        assert_eq!(result.files_processed, 1);
        assert_eq!(result.files_created, 0); // No files should be created in dry run
        assert!(result.is_success());
    }

    #[test]
    fn test_import_with_file_writing() {
        let temp_dir = TempDir::new().unwrap();
        let mut config = ImportConfig::default();
        config.output_dir = temp_dir.path().to_string_lossy().to_string();
        let importer = Importer::new(config);

        let input = "--------------------\nfile: test.rs\n--------------------\n    fn main() {}\n--------------------";
        let result = importer.import_from_string(input).unwrap();

        assert_eq!(result.files_processed, 1);
        assert_eq!(result.files_created, 1);
        assert_eq!(result.files_skipped, 0);
        assert!(result.is_success());

        // Verify file was actually created
        let file_path = temp_dir.path().join("test.rs");
        assert!(file_path.exists());
        let content = std::fs::read_to_string(file_path).unwrap();
        assert_eq!(content, "fn main() {}");
    }

    #[test]
    fn test_import_with_subdirectory() {
        let temp_dir = TempDir::new().unwrap();
        let mut config = ImportConfig::default();
        config.output_dir = temp_dir.path().to_string_lossy().to_string();
        let importer = Importer::new(config);

        let input = "--------------------\nfile: src/main.rs\n--------------------\n    fn main() {}\n--------------------";
        let result = importer.import_from_string(input).unwrap();

        assert_eq!(result.files_processed, 1);
        assert_eq!(result.files_created, 1);
        assert!(result.is_success());

        // Verify subdirectory and file were created
        let src_dir = temp_dir.path().join("src");
        assert!(src_dir.is_dir());
        let file_path = src_dir.join("main.rs");
        assert!(file_path.exists());
    }

    #[test]
    fn test_import_conflict_resolution() {
        let temp_dir = TempDir::new().unwrap();
        let mut config = ImportConfig::default();
        config.output_dir = temp_dir.path().to_string_lossy().to_string();
        config.skip_existing = true;
        let importer = Importer::new(config);

        // Create the file first
        std::fs::write(temp_dir.path().join("test.rs"), "existing content").unwrap();

        let input = "--------------------\nfile: test.rs\n--------------------\n    fn main() {}\n--------------------";
        let result = importer.import_from_string(input).unwrap();

        assert_eq!(result.files_processed, 1);
        assert_eq!(result.files_created, 0);
        assert_eq!(result.files_skipped, 1);
        assert!(result.is_success());

        // Verify file content wasn't changed
        let content = std::fs::read_to_string(temp_dir.path().join("test.rs")).unwrap();
        assert_eq!(content, "existing content");
    }

    #[test]
    fn test_import_with_warnings() {
        let temp_dir = TempDir::new().unwrap();
        let mut config = ImportConfig::default();
        config.output_dir = temp_dir.path().to_string_lossy().to_string();
        let importer = Importer::new(config);

        // Use CON.txt which should generate a warning but still succeed
        let input = "--------------------\nfile: CON.txt\n--------------------\n    content\n--------------------";
        let result = importer.import_from_string(input).unwrap();

        assert!(result.is_success());
        assert!(!result.warnings.is_empty());
        assert!(result.warnings[0].contains("potentially problematic filename"));
    }
}
