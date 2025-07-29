//! File writing functionality with safety checks and conflict resolution.
//!
//! This module provides the FileWriter struct that safely creates files from
//! parsed context-generator output, with built-in protections against common
//! security issues and file system conflicts.

use crate::parser::{ImportError, ParsedFile};
use std::fs;
use std::path::{Path, PathBuf};

/// Safe file writer with conflict resolution and path validation
#[derive(Debug)]
pub struct FileWriter {
    /// Base output directory where files will be created
    output_dir: PathBuf,
    /// Whether to overwrite existing files
    force: bool,
    /// Whether to skip existing files instead of failing
    skip_existing: bool,
}

/// Result of a file write operation
#[derive(Debug, PartialEq)]
pub enum WriteResult {
    /// File was successfully created
    Created,
    /// File already existed and was skipped
    Skipped,
    /// File already existed and was overwritten
    Overwritten,
}

impl FileWriter {
    /// Create a new FileWriter with the specified configuration
    ///
    /// # Arguments
    ///
    /// * `output_dir` - Base directory where files will be created
    /// * `force` - Whether to overwrite existing files
    /// * `skip_existing` - Whether to skip existing files instead of failing
    ///
    /// # Examples
    ///
    /// ```rust
    /// use context_generator::FileWriter;
    /// use tempfile::TempDir;
    ///
    /// # fn main() -> Result<(), Box<dyn std::error::Error>> {
    /// let temp_dir = TempDir::new()?;
    /// let writer = FileWriter::new(temp_dir.path(), false, false);
    /// # Ok(())
    /// # }
    /// ```
    pub fn new<P: AsRef<Path>>(output_dir: P, force: bool, skip_existing: bool) -> Self {
        Self {
            output_dir: output_dir.as_ref().to_path_buf(),
            force,
            skip_existing,
        }
    }

    /// Write a single file to disk with safety checks
    ///
    /// This method performs comprehensive validation and safety checks:
    /// - Validates the file path for security issues
    /// - Creates parent directories as needed
    /// - Handles file conflicts according to configuration
    /// - Writes the file content safely
    ///
    /// # Arguments
    ///
    /// * `file` - The parsed file to write
    ///
    /// # Returns
    ///
    /// * `Ok(WriteResult)` - File was processed successfully
    /// * `Err(ImportError)` - Validation failed or I/O error occurred
    ///
    /// # Examples
    ///
    /// ```rust
    /// use context_generator::{FileWriter, ParsedFile};
    /// use tempfile::TempDir;
    ///
    /// # fn main() -> Result<(), Box<dyn std::error::Error>> {
    /// let temp_dir = TempDir::new()?;
    /// let writer = FileWriter::new(temp_dir.path(), false, false);
    /// let file = ParsedFile {
    ///     path: "src/main.rs".to_string(),
    ///     content: vec!["fn main() {}".to_string()],
    ///     line_range: (1, 1),
    /// };
    ///
    /// let result = writer.write_file(&file)?;
    /// # Ok(())
    /// # }
    /// ```
    pub fn write_file(&self, file: &ParsedFile) -> Result<WriteResult, ImportError> {
        // Validate the file path for security issues
        self.validate_path(&file.path)?;

        // Construct the full output path
        let full_path = self.output_dir.join(&file.path);

        // Check for file conflicts
        if full_path.exists() {
            return self.handle_existing_file(&full_path, file);
        }

        // Create parent directories if they don't exist
        if let Some(parent) = full_path.parent() {
            fs::create_dir_all(parent).map_err(|e| ImportError::Io {
                path: parent.display().to_string(),
                source: e,
            })?;
        }

        // Write the file content
        self.write_content(&full_path, file)?;

        Ok(WriteResult::Created)
    }

    /// Validate a file path for security and correctness
    ///
    /// This performs essential security checks to prevent:
    /// - Path traversal attacks (../ sequences)
    /// - Absolute path injection
    /// - Empty or invalid paths
    ///
    /// # Arguments
    ///
    /// * `path` - The file path to validate
    ///
    /// # Returns
    ///
    /// * `Ok(())` - Path is valid and safe
    /// * `Err(ImportError)` - Path failed validation
    fn validate_path(&self, path: &str) -> Result<(), ImportError> {
        // Basic validation
        if path.is_empty() {
            return Err(ImportError::PathValidation {
                path: path.to_string(),
                reason: "path cannot be empty".to_string(),
            });
        }

        // Check for absolute paths
        if path.starts_with('/') || path.starts_with('\\') {
            return Err(ImportError::PathValidation {
                path: path.to_string(),
                reason: "absolute paths are not allowed for security reasons".to_string(),
            });
        }

        // Check for path traversal
        if path.contains("..") {
            return Err(ImportError::PathValidation {
                path: path.to_string(),
                reason: "path traversal sequences (..) are not allowed for security reasons"
                    .to_string(),
            });
        }

        // Additional security check: ensure the resolved path stays within output directory
        let full_path = self.output_dir.join(path);
        let canonical_output = self
            .output_dir
            .canonicalize()
            .map_err(|e| ImportError::Io {
                path: self.output_dir.display().to_string(),
                source: e,
            })?;

        // Try to canonicalize the parent directory of the target file
        if let Some(parent) = full_path.parent() {
            if parent.exists() {
                let canonical_parent = parent.canonicalize().map_err(|e| ImportError::Io {
                    path: parent.display().to_string(),
                    source: e,
                })?;

                if !canonical_parent.starts_with(&canonical_output) {
                    return Err(ImportError::PathValidation {
                        path: path.to_string(),
                        reason: "resolved path would escape output directory".to_string(),
                    });
                }
            }
        }

        Ok(())
    }

    /// Handle existing file conflicts according to configuration
    ///
    /// # Arguments
    ///
    /// * `full_path` - Full path to the existing file
    /// * `file` - The parsed file to write
    ///
    /// # Returns
    ///
    /// * `Ok(WriteResult)` - File was handled according to policy
    /// * `Err(ImportError)` - Conflict could not be resolved
    fn handle_existing_file(
        &self,
        full_path: &Path,
        file: &ParsedFile,
    ) -> Result<WriteResult, ImportError> {
        if self.skip_existing {
            Ok(WriteResult::Skipped)
        } else if self.force {
            self.write_content(full_path, file)?;
            Ok(WriteResult::Overwritten)
        } else {
            // Default behavior: fail if file exists (safe default)
            Err(ImportError::Io {
                path: file.path.clone(),
                source: std::io::Error::new(
                    std::io::ErrorKind::AlreadyExists,
                    "file already exists (use --force to overwrite or --skip-existing to skip)",
                ),
            })
        }
    }

    /// Write file content to disk
    ///
    /// # Arguments
    ///
    /// * `full_path` - Full path where to write the file
    /// * `file` - The parsed file containing content to write
    ///
    /// # Returns
    ///
    /// * `Ok(())` - File was written successfully
    /// * `Err(ImportError)` - I/O error occurred during writing
    fn write_content(&self, full_path: &Path, file: &ParsedFile) -> Result<(), ImportError> {
        let content = file.content.join("\n");

        fs::write(full_path, content).map_err(|e| ImportError::Io {
            path: file.path.clone(),
            source: e,
        })?;

        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs;
    use tempfile::TempDir;

    fn create_test_file() -> ParsedFile {
        ParsedFile {
            path: "test.rs".to_string(),
            content: vec![
                "fn main() {".to_string(),
                "    println!(\"Hello\");".to_string(),
                "}".to_string(),
            ],
            line_range: (1, 3),
        }
    }

    #[test]
    fn test_file_writer_creation() {
        let writer = FileWriter::new("/tmp", false, false);
        assert_eq!(writer.output_dir, PathBuf::from("/tmp"));
        assert!(!writer.force);
        assert!(!writer.skip_existing);
    }

    #[test]
    fn test_path_validation_empty_path() {
        let temp_dir = TempDir::new().unwrap();
        let writer = FileWriter::new(temp_dir.path(), false, false);

        let result = writer.validate_path("");
        assert!(matches!(result, Err(ImportError::PathValidation { .. })));
    }

    #[test]
    fn test_path_validation_absolute_path() {
        let temp_dir = TempDir::new().unwrap();
        let writer = FileWriter::new(temp_dir.path(), false, false);

        let result = writer.validate_path("/etc/passwd");
        assert!(matches!(result, Err(ImportError::PathValidation { .. })));

        if let Err(ImportError::PathValidation { reason, .. }) = result {
            assert!(reason.contains("absolute paths are not allowed"));
        }
    }

    #[test]
    fn test_path_validation_traversal_attack() {
        let temp_dir = TempDir::new().unwrap();
        let writer = FileWriter::new(temp_dir.path(), false, false);

        let result = writer.validate_path("../../../etc/passwd");
        assert!(matches!(result, Err(ImportError::PathValidation { .. })));

        if let Err(ImportError::PathValidation { reason, .. }) = result {
            assert!(reason.contains("path traversal sequences"));
        }
    }

    #[test]
    fn test_path_validation_valid_relative_path() {
        let temp_dir = TempDir::new().unwrap();
        let writer = FileWriter::new(temp_dir.path(), false, false);

        let result = writer.validate_path("src/main.rs");
        assert!(result.is_ok());
    }

    #[test]
    fn test_write_file_success() {
        let temp_dir = TempDir::new().unwrap();
        let writer = FileWriter::new(temp_dir.path(), false, false);
        let file = create_test_file();

        let result = writer.write_file(&file).unwrap();
        assert_eq!(result, WriteResult::Created);

        // Verify file was created with correct content
        let written_content = fs::read_to_string(temp_dir.path().join("test.rs")).unwrap();
        assert_eq!(written_content, "fn main() {\n    println!(\"Hello\");\n}");
    }

    #[test]
    fn test_write_file_with_subdirectory() {
        let temp_dir = TempDir::new().unwrap();
        let writer = FileWriter::new(temp_dir.path(), false, false);
        let mut file = create_test_file();
        file.path = "src/main.rs".to_string();

        let result = writer.write_file(&file).unwrap();
        assert_eq!(result, WriteResult::Created);

        // Verify directory was created
        assert!(temp_dir.path().join("src").is_dir());

        // Verify file was created
        let written_content = fs::read_to_string(temp_dir.path().join("src/main.rs")).unwrap();
        assert_eq!(written_content, "fn main() {\n    println!(\"Hello\");\n}");
    }

    #[test]
    fn test_file_conflict_default_behavior() {
        let temp_dir = TempDir::new().unwrap();
        let writer = FileWriter::new(temp_dir.path(), false, false);
        let file = create_test_file();

        // Create file first time
        writer.write_file(&file).unwrap();

        // Try to create again - should fail by default
        let result = writer.write_file(&file);
        assert!(matches!(result, Err(ImportError::Io { .. })));
    }

    #[test]
    fn test_file_conflict_force_overwrite() {
        let temp_dir = TempDir::new().unwrap();
        let writer = FileWriter::new(temp_dir.path(), true, false); // force = true
        let file = create_test_file();

        // Create file first time
        writer.write_file(&file).unwrap();

        // Try to create again with force - should overwrite
        let result = writer.write_file(&file).unwrap();
        assert_eq!(result, WriteResult::Overwritten);
    }

    #[test]
    fn test_file_conflict_skip_existing() {
        let temp_dir = TempDir::new().unwrap();
        let writer = FileWriter::new(temp_dir.path(), false, true); // skip_existing = true
        let file = create_test_file();

        // Create file first time
        writer.write_file(&file).unwrap();

        // Try to create again with skip_existing - should skip
        let result = writer.write_file(&file).unwrap();
        assert_eq!(result, WriteResult::Skipped);
    }
}
