//! Validation module for import operations.
//!
//! This module provides comprehensive validation for context-generator import operations,
//! ensuring safety, security, and data integrity during the import process.

use crate::parser::{ParseResult, ParsedFile};
use std::path::Path;

/// Comprehensive validator for import operations
#[derive(Debug)]
pub struct Validator;

/// Result of validation operations
#[derive(Debug)]
pub struct ValidationResult {
    /// Whether validation passed
    pub is_valid: bool,
    /// Validation errors encountered
    pub errors: Vec<String>,
    /// Validation warnings
    pub warnings: Vec<String>,
}

impl ValidationResult {
    /// Create a successful validation result
    pub fn success() -> Self {
        Self {
            is_valid: true,
            errors: Vec::new(),
            warnings: Vec::new(),
        }
    }

    /// Create a failed validation result with an error
    #[allow(dead_code)] // Used in tests
    pub fn error(message: String) -> Self {
        Self {
            is_valid: false,
            errors: vec![message],
            warnings: Vec::new(),
        }
    }

    /// Add an error to the validation result
    pub fn add_error(&mut self, message: String) {
        self.errors.push(message);
        self.is_valid = false;
    }

    /// Add a warning to the validation result
    pub fn add_warning(&mut self, message: String) {
        self.warnings.push(message);
    }

    /// Merge another validation result into this one
    pub fn merge(&mut self, other: ValidationResult) {
        self.errors.extend(other.errors);
        self.warnings.extend(other.warnings);
        self.is_valid = self.is_valid && other.is_valid;
    }
}

impl Validator {
    /// Create a new validator instance
    pub fn new() -> Self {
        Self
    }

    /// Perform comprehensive validation on parsed content
    ///
    /// This method orchestrates all validation checks:
    /// - Format validation
    /// - Path validation
    /// - Content validation
    ///
    /// # Arguments
    ///
    /// * `parse_result` - The result from parsing the input
    /// * `output_dir` - Target output directory for additional path checks
    ///
    /// # Returns
    ///
    /// A ValidationResult indicating success or failure with details
    pub fn validate_import(
        &self,
        parse_result: &ParseResult,
        output_dir: &str,
    ) -> ValidationResult {
        let mut result = ValidationResult::success();

        // Validate overall format structure
        let format_result = self.validate_format(&parse_result.files);
        result.merge(format_result);

        // Validate all file paths
        for file in &parse_result.files {
            let path_result = self.validate_file_path(&file.path, output_dir);
            result.merge(path_result);

            let content_result = self.validate_content(file);
            result.merge(content_result);
        }

        result
    }

    /// Validate the overall format structure of parsed files
    ///
    /// Checks for format consistency and structural issues beyond basic parsing
    ///
    /// # Arguments
    ///
    /// * `files` - Collection of parsed files to validate
    ///
    /// # Returns
    ///
    /// ValidationResult with format-specific issues
    pub fn validate_format(&self, files: &[ParsedFile]) -> ValidationResult {
        let mut result = ValidationResult::success();

        // Check if we have any files at all
        if files.is_empty() {
            result
                .add_error("no files found in input - input may be malformed or empty".to_string());
            return result;
        }

        // Check for duplicate file paths
        let mut seen_paths = std::collections::HashSet::new();
        for file in files {
            if !seen_paths.insert(&file.path) {
                result.add_error(format!("duplicate file path found: {}", file.path));
            }
        }

        // Validate line ranges make sense
        for file in files {
            if file.line_range.0 > file.line_range.1 {
                result.add_warning(format!(
                    "invalid line range for file {}: start {} > end {}",
                    file.path, file.line_range.0, file.line_range.1
                ));
            }
        }

        result
    }

    /// Validate a single file path for security and correctness
    ///
    /// Performs comprehensive path validation including:
    /// - Path traversal prevention
    /// - Absolute path rejection
    /// - Character validation
    /// - Length limits
    ///
    /// # Arguments
    ///
    /// * `path` - File path to validate
    /// * `output_dir` - Base output directory for additional checks
    ///
    /// # Returns
    ///
    /// ValidationResult specific to this path
    pub fn validate_file_path(&self, path: &str, output_dir: &str) -> ValidationResult {
        let mut result = ValidationResult::success();

        // Basic path validation
        if path.is_empty() {
            result.add_error("file path cannot be empty".to_string());
            return result;
        }

        // Reject absolute paths
        if Path::new(path).is_absolute() {
            result.add_error(format!(
                "absolute path not allowed: {path} - all paths must be relative for security"
            ));
        }

        // Prevent path traversal attacks
        if path.contains("..") {
            result.add_error(format!(
                "path traversal detected: {path} - paths containing '..' are not allowed"
            ));
        }

        // Check for null bytes (potential security issue)
        if path.contains('\0') {
            result.add_error(format!(
                "null byte in path: {path} - invalid character detected"
            ));
        }

        // Check path length (reasonable limit)
        if path.len() > 4096 {
            result.add_error(format!(
                "path too long: {path} - maximum 4096 characters allowed"
            ));
        }

        // Validate individual path components
        for component in Path::new(path).components() {
            if let std::path::Component::Normal(name) = component {
                if let Some(name_str) = name.to_str() {
                    // Check for reserved Windows filenames (even on Unix for cross-platform safety)
                    if self.is_reserved_windows_name(name_str) {
                        result.add_warning(format!(
                            "potentially problematic filename: {name_str} - this may cause issues on Windows systems"
                        ));
                    }

                    // Check for problematic characters
                    if name_str.contains(['<', '>', ':', '"', '|', '?', '*']) {
                        result.add_warning(format!(
                            "filename contains potentially problematic characters: {name_str}"
                        ));
                    }

                    // Check for names starting/ending with whitespace or dots
                    if name_str.starts_with(' ')
                        || name_str.ends_with(' ')
                        || name_str.starts_with('.') && name_str.len() == 1
                        || name_str == ".."
                    {
                        result.add_warning(format!(
                            "potentially problematic filename format: {name_str}"
                        ));
                    }
                }
            }
        }

        // Additional check: ensure resolved path would stay within output directory
        let output_path = Path::new(output_dir);
        let full_path = output_path.join(path);

        // Basic check to ensure the path doesn't try to escape (redundant but safe)
        if let Ok(_canonical_output) = output_path.canonicalize() {
            if let Some(parent) = full_path.parent() {
                // Only check if parent would exist - this is a best-effort security check
                if parent.starts_with("..") {
                    result.add_error(format!("path would escape output directory: {path}"));
                }
            }
        }

        result
    }

    /// Validate file content for obvious issues
    ///
    /// Checks for:
    /// - Binary content patterns
    /// - Extremely long lines
    /// - Unusual character sequences
    ///
    /// # Arguments
    ///
    /// * `file` - Parsed file to validate content
    ///
    /// # Returns
    ///
    /// ValidationResult with content-specific issues
    pub fn validate_content(&self, file: &ParsedFile) -> ValidationResult {
        let mut result = ValidationResult::success();

        // Check for binary content indicators
        for (line_idx, line) in file.content.iter().enumerate() {
            // Check for null bytes (strong indicator of binary content)
            if line.contains('\0') {
                result.add_error(format!(
                    "binary content detected in file {} at line {} - null bytes found",
                    file.path,
                    line_idx + 1
                ));
            }

            // Check for unusual control characters (except common ones like \t, \r, \n)
            let unusual_chars: Vec<char> = line
                .chars()
                .filter(|&c| c.is_control() && c != '\t' && c != '\r' && c != '\n')
                .collect();

            if !unusual_chars.is_empty() {
                result.add_warning(format!(
                    "unusual control characters in file {} at line {} - may indicate binary content",
                    file.path, line_idx + 1
                ));
            }

            // Check for extremely long lines (potential issue)
            if line.len() > 32768 {
                result.add_warning(format!(
                    "extremely long line in file {} at line {} ({} characters) - may indicate binary content",
                    file.path, line_idx + 1, line.len()
                ));
            }

            // Check for high percentage of non-printable characters
            let non_printable_count = line
                .chars()
                .filter(|&c| !c.is_ascii_graphic() && !c.is_ascii_whitespace())
                .count();

            if line.len() > 100 && non_printable_count as f64 / line.len() as f64 > 0.3 {
                result.add_warning(format!(
                    "high ratio of non-printable characters in file {} at line {} - may indicate binary content",
                    file.path, line_idx + 1
                ));
            }
        }

        // Check for suspiciously large files (content-wise)
        if file.content.len() > 100000 {
            result.add_warning(format!(
                "file {} has {} lines - unusually large file, verify this is intended",
                file.path,
                file.content.len()
            ));
        }

        // Check for empty files (not necessarily an error, but worth noting)
        if file.content.is_empty() {
            result.add_warning(format!(
                "file {} is empty - this may be intentional",
                file.path
            ));
        }

        result
    }

    /// Check if a filename is a reserved Windows name
    fn is_reserved_windows_name(&self, name: &str) -> bool {
        let reserved_names = [
            "CON", "PRN", "AUX", "NUL", "COM1", "COM2", "COM3", "COM4", "COM5", "COM6", "COM7",
            "COM8", "COM9", "LPT1", "LPT2", "LPT3", "LPT4", "LPT5", "LPT6", "LPT7", "LPT8", "LPT9",
        ];

        let name_upper = name.to_uppercase();
        reserved_names.contains(&name_upper.as_str())
            || reserved_names
                .iter()
                .any(|&reserved| name_upper.starts_with(&format!("{reserved}.")))
    }
}

impl Default for Validator {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn create_test_file(path: &str, content: Vec<&str>) -> ParsedFile {
        ParsedFile {
            path: path.to_string(),
            content: content.iter().map(|s| s.to_string()).collect(),
            line_range: (1, content.len()),
        }
    }

    #[test]
    fn test_validator_creation() {
        let validator = Validator::new();
        // Just ensure it can be created
        let _ = validator;
    }

    #[test]
    fn test_validation_result_success() {
        let result = ValidationResult::success();
        assert!(result.is_valid);
        assert!(result.errors.is_empty());
        assert!(result.warnings.is_empty());
    }

    #[test]
    fn test_validation_result_error() {
        let result = ValidationResult::error("test error".to_string());
        assert!(!result.is_valid);
        assert_eq!(result.errors.len(), 1);
        assert_eq!(result.errors[0], "test error");
    }

    #[test]
    fn test_validation_result_merge() {
        let mut result1 = ValidationResult::success();
        result1.add_warning("warning 1".to_string());

        let mut result2 = ValidationResult::error("error 1".to_string());
        result2.add_warning("warning 2".to_string());

        result1.merge(result2);

        assert!(!result1.is_valid);
        assert_eq!(result1.errors.len(), 1);
        assert_eq!(result1.warnings.len(), 2);
    }

    #[test]
    fn test_format_validation_empty_files() {
        let validator = Validator::new();
        let files = vec![];
        let result = validator.validate_format(&files);

        assert!(!result.is_valid);
        assert!(result.errors[0].contains("no files found"));
    }

    #[test]
    fn test_format_validation_duplicate_paths() {
        let validator = Validator::new();
        let files = vec![
            create_test_file("test.rs", vec!["fn main() {}"]),
            create_test_file("test.rs", vec!["fn other() {}"]),
        ];
        let result = validator.validate_format(&files);

        assert!(!result.is_valid);
        assert!(result.errors[0].contains("duplicate file path"));
    }

    #[test]
    fn test_path_validation_empty_path() {
        let validator = Validator::new();
        let result = validator.validate_file_path("", "/tmp");

        assert!(!result.is_valid);
        assert!(result.errors[0].contains("cannot be empty"));
    }

    #[test]
    fn test_path_validation_absolute_path() {
        let validator = Validator::new();
        let result = validator.validate_file_path("/etc/passwd", "/tmp");

        assert!(!result.is_valid);
        assert!(result.errors[0].contains("absolute path not allowed"));
    }

    #[test]
    fn test_path_validation_traversal_attack() {
        let validator = Validator::new();
        let result = validator.validate_file_path("../../../etc/passwd", "/tmp");

        assert!(!result.is_valid);
        assert!(result.errors[0].contains("path traversal detected"));
    }

    #[test]
    fn test_path_validation_null_byte() {
        let validator = Validator::new();
        let result = validator.validate_file_path("test\0.rs", "/tmp");

        assert!(!result.is_valid);
        assert!(result.errors[0].contains("null byte in path"));
    }

    #[test]
    fn test_path_validation_reserved_windows_names() {
        let validator = Validator::new();
        let result = validator.validate_file_path("CON.txt", "/tmp");

        // Should pass validation but with a warning
        assert!(result.is_valid);
        assert!(!result.warnings.is_empty());
        assert!(result.warnings[0].contains("potentially problematic filename"));
    }

    #[test]
    fn test_path_validation_valid_path() {
        let validator = Validator::new();
        let result = validator.validate_file_path("src/main.rs", "/tmp");

        assert!(result.is_valid);
        assert!(result.errors.is_empty());
    }

    #[test]
    fn test_content_validation_null_bytes() {
        let validator = Validator::new();
        let file = create_test_file("binary.dat", vec!["text line", "binary\0content"]);
        let result = validator.validate_content(&file);

        assert!(!result.is_valid);
        assert!(result.errors[0].contains("binary content detected"));
    }

    #[test]
    fn test_content_validation_extremely_long_line() {
        let validator = Validator::new();
        let long_line = "x".repeat(40000);
        let file = create_test_file("long.txt", vec![&long_line]);
        let result = validator.validate_content(&file);

        assert!(result.is_valid); // Should pass but with warning
        assert!(!result.warnings.is_empty());
        assert!(result.warnings[0].contains("extremely long line"));
    }

    #[test]
    fn test_content_validation_empty_file() {
        let validator = Validator::new();
        let file = create_test_file("empty.txt", vec![]);
        let result = validator.validate_content(&file);

        assert!(result.is_valid); // Should pass but with warning
        assert!(!result.warnings.is_empty());
        assert!(result.warnings[0].contains("is empty"));
    }

    #[test]
    fn test_content_validation_normal_content() {
        let validator = Validator::new();
        let file = create_test_file(
            "normal.rs",
            vec!["fn main() {", "    println!(\"Hello, world!\");", "}"],
        );
        let result = validator.validate_content(&file);

        assert!(result.is_valid);
        assert!(result.errors.is_empty());
        assert!(result.warnings.is_empty());
    }

    #[test]
    fn test_reserved_windows_name_detection() {
        let validator = Validator::new();

        assert!(validator.is_reserved_windows_name("CON"));
        assert!(validator.is_reserved_windows_name("con"));
        assert!(validator.is_reserved_windows_name("PRN.txt"));
        assert!(validator.is_reserved_windows_name("COM1"));
        assert!(validator.is_reserved_windows_name("LPT9.dat"));

        assert!(!validator.is_reserved_windows_name("CONFIG"));
        assert!(!validator.is_reserved_windows_name("normal.txt"));
        assert!(!validator.is_reserved_windows_name("CONSOLE"));
    }

    #[test]
    fn test_comprehensive_validation() {
        let validator = Validator::new();
        let parse_result = ParseResult {
            files: vec![
                create_test_file("src/main.rs", vec!["fn main() {}", "}"]),
                create_test_file("README.md", vec!["# Project", "Description"]),
            ],
            warnings: vec![],
            errors: vec![],
        };

        let result = validator.validate_import(&parse_result, "/tmp");
        assert!(result.is_valid);
        assert!(result.errors.is_empty());
    }
}
