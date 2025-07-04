//! File filtering and exclusion pattern management.
//!
//! This module provides functionality for filtering files based on exclusion patterns
//! organized into categories. The exclusion patterns are defined in `exclusions.yaml`
//! and embedded into the binary at compile time for zero runtime overhead.
//!
//! # Examples
//!
//! ```rust
//! use context_generator::filter::{Filter, get_exclusion_categories};
//!
//! // Create a filter with default exclusions
//! let filter = Filter::new_with_defaults(vec![], &[]).unwrap();
//!
//! // Check if a file should be excluded
//! let should_exclude = filter.should_exclude("/path/to/file.log", "/base/dir", false);
//!
//! // Get all available categories
//! let categories = get_exclusion_categories();
//! ```

use glob::Pattern;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::path::Path;

/// Embedded YAML file containing all exclusion categories and patterns.
/// This is loaded at compile time using `include_str!` for zero runtime cost.
const EXCLUSIONS_YAML: &str = include_str!("../exclusions.yaml");

/// Represents a category of exclusion patterns.
///
/// Each category groups related patterns together (e.g., all Python-related files,
/// all build artifacts, etc.) to make it easier to selectively enable/disable
/// groups of exclusions.
///
/// # Fields
///
/// * `id` - Unique identifier for the category (e.g., "python", "logs")
/// * `name` - Human-readable name for the category (e.g., "Python", "Logs & Temporary")
/// * `description` - Brief description of what files this category excludes
/// * `patterns` - List of glob patterns that define which files to exclude
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ExclusionCategory {
    pub id: String,
    pub name: String,
    pub description: String,
    pub patterns: Vec<String>,
}

/// Internal structure for parsing the YAML data.
///
/// This represents the root structure of the `exclusions.yaml` file,
/// which contains a single `categories` field with an array of exclusion categories.
#[derive(Debug, Clone, Serialize, Deserialize)]
struct ExclusionData {
    categories: Vec<ExclusionCategory>,
}

/// Information about why a specific file was excluded.
///
/// This provides context when a file is filtered out, showing both the specific
/// pattern that matched and the category it belongs to.
///
/// # Fields
///
/// * `pattern` - The specific glob pattern that matched (e.g., "*.log")
/// * `category` - The name of the category this pattern belongs to (e.g., "Logs & Temporary")
#[derive(Debug, Clone)]
pub struct ExclusionReason {
    pub pattern: String,
    pub category: String,
}

/// File filter that determines which files should be excluded from processing.
///
/// The filter compiles glob patterns into efficient matchers and provides
/// methods to check if files should be excluded. It supports both default
/// exclusion categories and custom patterns.
///
/// # Examples
///
/// ```rust
/// use context_generator::filter::Filter;
///
/// // Create filter with only custom patterns
/// let filter = Filter::new(vec!["*.tmp".to_string(), "build/*".to_string()]).unwrap();
///
/// // Create filter with defaults plus custom patterns
/// let filter = Filter::new_with_defaults(
///     vec!["*.custom".to_string()],
///     &["logs".to_string()] // disable logs category
/// ).unwrap();
/// ```
pub struct Filter {
    /// Compiled glob patterns for efficient matching
    patterns: Vec<Pattern>,
    /// Maps pattern strings to their category names for reporting
    pattern_to_category: HashMap<String, String>,
}

impl Filter {
    /// Creates a new filter with only the specified custom patterns.
    ///
    /// This creates a minimal filter that excludes only files matching the provided
    /// glob patterns. No default exclusions are applied.
    ///
    /// # Arguments
    ///
    /// * `patterns` - Vector of glob pattern strings (e.g., `["*.tmp", "build/*"]`)
    ///
    /// # Returns
    ///
    /// * `Ok(Filter)` - Successfully created filter
    /// * `Err(glob::PatternError)` - Invalid glob pattern provided
    ///
    /// # Examples
    ///
    /// ```rust
    /// use context_generator::filter::Filter;
    ///
    /// let filter = Filter::new(vec![
    ///     "*.log".to_string(),
    ///     "temp/*".to_string(),
    ///     ".DS_Store".to_string(),
    /// ]).unwrap();
    /// ```
    pub fn new(patterns: Vec<String>) -> Result<Self, glob::PatternError> {
        let mut compiled_patterns = Vec::new();
        let mut pattern_to_category = HashMap::new();

        for pattern_str in patterns {
            let pattern = Pattern::new(&pattern_str)?;
            compiled_patterns.push(pattern);
            pattern_to_category.insert(pattern_str, "Custom".to_string());
        }

        Ok(Filter {
            patterns: compiled_patterns,
            pattern_to_category,
        })
    }

    /// Creates a new filter with default exclusions plus additional custom patterns.
    ///
    /// This is the most commonly used constructor. It loads all default exclusion
    /// categories from the embedded YAML data, optionally disables specific categories,
    /// and adds any additional custom patterns.
    ///
    /// # Arguments
    ///
    /// * `additional_patterns` - Extra custom patterns to add beyond defaults
    /// * `disabled_category_ids` - Category IDs to disable (e.g., `["logs", "vcs"]`)
    ///
    /// # Returns
    ///
    /// * `Ok(Filter)` - Successfully created filter
    /// * `Err(glob::PatternError)` - Invalid glob pattern in defaults or additional patterns
    ///
    /// # Examples
    ///
    /// ```rust
    /// use context_generator::filter::Filter;
    ///
    /// // Use all defaults plus custom patterns
    /// let filter = Filter::new_with_defaults(
    ///     vec!["*.backup".to_string()],
    ///     &[]
    /// ).unwrap();
    ///
    /// // Disable logs category but include custom pattern
    /// let filter = Filter::new_with_defaults(
    ///     vec!["*.secret".to_string()],
    ///     &["logs".to_string()]
    /// ).unwrap();
    /// ```
    pub fn new_with_defaults(
        additional_patterns: Vec<String>,
        disabled_category_ids: &[String],
    ) -> Result<Self, glob::PatternError> {
        let default_patterns = get_filtered_patterns(disabled_category_ids);
        let mut all_patterns = default_patterns;
        all_patterns.extend(additional_patterns);

        let mut compiled_patterns = Vec::new();
        let mut pattern_to_category = HashMap::new();

        for pattern_str in all_patterns {
            let pattern = Pattern::new(&pattern_str)?;
            compiled_patterns.push(pattern);

            let category = get_category_for_pattern(&pattern_str);
            pattern_to_category.insert(pattern_str, category);
        }

        Ok(Filter {
            patterns: compiled_patterns,
            pattern_to_category,
        })
    }

    /// Determines if a file should be excluded based on the configured patterns.
    ///
    /// This is a convenience method that returns a simple boolean. For more detailed
    /// information about why a file was excluded, use [`get_exclusion_reason`].
    ///
    /// # Arguments
    ///
    /// * `path` - Path to the file to check
    /// * `base_dir` - Base directory for calculating relative paths
    /// * `is_dir` - Whether the path represents a directory
    ///
    /// # Returns
    ///
    /// * `true` - File should be excluded
    /// * `false` - File should be included
    ///
    /// # Examples
    ///
    /// ```rust
    /// use context_generator::filter::Filter;
    /// use std::path::Path;
    ///
    /// let filter = Filter::new(vec!["*.log".to_string()]).unwrap();
    /// let base_dir = Path::new("/project");
    /// let log_file = Path::new("/project/app.log");
    /// let rs_file = Path::new("/project/main.rs");
    ///
    /// assert!(filter.should_exclude(log_file, base_dir, false));
    /// assert!(!filter.should_exclude(rs_file, base_dir, false));
    /// ```
    ///
    /// [`get_exclusion_reason`]: Filter::get_exclusion_reason
    pub fn should_exclude<P: AsRef<Path>>(&self, path: P, base_dir: P, is_dir: bool) -> bool {
        self.get_exclusion_reason(path, base_dir, is_dir).is_some()
    }

    /// Gets detailed information about why a file was excluded.
    ///
    /// This method provides the specific pattern that matched and the category
    /// it belongs to. Returns `None` if the file should not be excluded.
    ///
    /// # Arguments
    ///
    /// * `path` - Path to the file to check
    /// * `base_dir` - Base directory for calculating relative paths
    /// * `_is_dir` - Whether the path represents a directory (currently unused)
    ///
    /// # Returns
    ///
    /// * `Some(ExclusionReason)` - File should be excluded, with details
    /// * `None` - File should be included
    ///
    /// # Examples
    ///
    /// ```rust
    /// use context_generator::filter::Filter;
    /// use std::path::Path;
    ///
    /// let filter = Filter::new_with_defaults(vec![], &[]).unwrap();
    /// let base_dir = Path::new("/project");
    /// let log_file = Path::new("/project/app.log");
    ///
    /// if let Some(reason) = filter.get_exclusion_reason(log_file, base_dir, false) {
    ///     println!("Excluded by pattern '{}' in category '{}'", reason.pattern, reason.category);
    /// }
    /// ```
    pub fn get_exclusion_reason<P: AsRef<Path>>(
        &self,
        path: P,
        base_dir: P,
        _is_dir: bool,
    ) -> Option<ExclusionReason> {
        let path = path.as_ref();
        let base_dir = base_dir.as_ref();

        let rel_path = path.strip_prefix(base_dir).unwrap_or(path);
        let name = path.file_name()?.to_str()?;

        for pattern in &self.patterns {
            if pattern.matches(name) || pattern.matches(&rel_path.to_string_lossy()) {
                let pattern_str = pattern.as_str();
                let category = self
                    .pattern_to_category
                    .get(pattern_str)
                    .cloned()
                    .unwrap_or_else(|| "Custom".to_string());

                return Some(ExclusionReason {
                    pattern: pattern_str.to_string(),
                    category,
                });
            }
        }

        None
    }
}

/// Loads and parses the embedded YAML exclusion data.
///
/// This function is called internally to parse the embedded `exclusions.yaml` content.
/// The parsing happens at runtime but the YAML content is embedded at compile time.
///
/// # Returns
///
/// * `Ok(ExclusionData)` - Successfully parsed exclusion data
/// * `Err(serde_yaml::Error)` - YAML parsing error
fn load_exclusion_data() -> Result<ExclusionData, serde_yaml::Error> {
    serde_yaml::from_str(EXCLUSIONS_YAML)
}

/// Gets all available exclusion categories from the embedded YAML data.
///
/// This function loads and returns all exclusion categories defined in `exclusions.yaml`.
/// Each category contains an ID, name, description, and list of glob patterns.
///
/// # Returns
///
/// Vector of all available exclusion categories. Returns empty vector if YAML parsing fails.
///
/// # Examples
///
/// ```rust
/// use context_generator::filter::get_exclusion_categories;
///
/// let categories = get_exclusion_categories();
/// for category in categories {
///     println!("Category {}: {} patterns", category.name, category.patterns.len());
/// }
/// ```
pub fn get_exclusion_categories() -> Vec<ExclusionCategory> {
    match load_exclusion_data() {
        Ok(data) => data.categories,
        Err(e) => {
            eprintln!("Warning: Failed to load exclusion categories: {e}");
            Vec::new()
        }
    }
}

/// Gets all exclusion patterns from all categories.
///
/// This is a convenience function that returns all patterns from all categories
/// without any filtering. Equivalent to calling `get_filtered_patterns(&[])`.
///
/// # Returns
///
/// Vector of all exclusion patterns from all categories.
///
/// # Examples
///
/// ```rust
/// use context_generator::filter::get_all_patterns;
///
/// let all_patterns = get_all_patterns();
/// println!("Total exclusion patterns: {}", all_patterns.len());
/// ```
pub fn get_all_patterns() -> Vec<String> {
    get_filtered_patterns(&[])
}

/// Gets exclusion patterns with specific categories disabled.
///
/// This function returns all patterns from all categories except those whose
/// IDs are listed in the `disabled_category_ids` parameter. This is used to
/// implement the `--disable-category` CLI flag.
///
/// # Arguments
///
/// * `disabled_category_ids` - List of category IDs to exclude (e.g., `["logs", "vcs"]`)
///
/// # Returns
///
/// Vector of patterns from all enabled categories.
///
/// # Examples
///
/// ```rust
/// use context_generator::filter::get_filtered_patterns;
///
/// // Get all patterns except logs and version control
/// let patterns = get_filtered_patterns(&["logs".to_string(), "vcs".to_string()]);
///
/// // Get all patterns (no filtering)
/// let all_patterns = get_filtered_patterns(&[]);
/// ```
pub fn get_filtered_patterns(disabled_category_ids: &[String]) -> Vec<String> {
    let categories = get_exclusion_categories();
    let mut patterns = Vec::new();

    for category in categories {
        if !disabled_category_ids.contains(&category.id) {
            patterns.extend(category.patterns);
        }
    }

    patterns
}

/// Finds which category a specific pattern belongs to.
///
/// Given a pattern string, this function searches through all categories to find
/// which one contains that pattern. Returns "Custom" if the pattern is not found
/// in any default category.
///
/// # Arguments
///
/// * `pattern` - The pattern string to search for (e.g., "*.log", "node_modules")
///
/// # Returns
///
/// The name of the category containing this pattern, or "Custom" if not found.
///
/// # Examples
///
/// ```rust
/// use context_generator::filter::get_category_for_pattern;
///
/// assert_eq!(get_category_for_pattern("*.log"), "LaTeX"); // or "Logs & Temporary"
/// assert_eq!(get_category_for_pattern("node_modules"), "Dependencies");
/// assert_eq!(get_category_for_pattern("my_custom_pattern"), "Custom");
/// ```
///
/// # Note
///
/// If a pattern exists in multiple categories, this function returns the name of
/// the first category that contains it (based on the order in `exclusions.yaml`).
pub fn get_category_for_pattern(pattern: &str) -> String {
    let categories = get_exclusion_categories();
    for category in categories {
        if category.patterns.contains(&pattern.to_string()) {
            return category.name;
        }
    }
    "Custom".to_string()
}

/// Validates a list of category IDs and returns any invalid ones.
///
/// This function checks if the provided category IDs exist in the exclusion
/// categories. It's used to validate user input for the `--disable-category` flag.
///
/// # Arguments
///
/// * `ids` - List of category IDs to validate (e.g., `["go", "python", "invalid"]`)
///
/// # Returns
///
/// Vector containing only the invalid category IDs. Empty vector if all IDs are valid.
///
/// # Examples
///
/// ```rust
/// use context_generator::filter::validate_category_ids;
///
/// let invalid = validate_category_ids(&["go".to_string(), "invalid".to_string()]);
/// assert_eq!(invalid, vec!["invalid".to_string()]);
///
/// let valid = validate_category_ids(&["go".to_string(), "python".to_string()]);
/// assert!(valid.is_empty());
/// ```
pub fn validate_category_ids(ids: &[String]) -> Vec<String> {
    let categories = get_exclusion_categories();
    let valid_ids: Vec<String> = categories.iter().map(|c| c.id.clone()).collect();

    ids.iter()
        .filter(|id| !valid_ids.contains(id))
        .cloned()
        .collect()
}

/// Prints all exclusion categories with their patterns in a formatted display.
///
/// This function outputs a comprehensive list of all exclusion categories, their
/// descriptions, and patterns. It's used to implement the `list-exclusions` command.
/// The output includes category information, usage examples, and summary statistics.
///
/// # Arguments
///
/// * `writer` - The writer to output to (e.g., `stdout`, `stderr`, or a file)
///
/// # Returns
///
/// * `Ok(())` - Successfully printed exclusions
/// * `Err(std::io::Error)` - IO error during writing
///
/// # Examples
///
/// ```rust
/// use context_generator::filter::print_exclusions;
/// use std::io;
///
/// // Print to stdout
/// print_exclusions(&mut io::stdout()).unwrap();
///
/// // Print to a string buffer
/// let mut buffer = Vec::new();
/// print_exclusions(&mut buffer).unwrap();
/// let output = String::from_utf8(buffer).unwrap();
/// ```
pub fn print_exclusions<W: std::io::Write>(writer: &mut W) -> std::io::Result<()> {
    let categories = get_exclusion_categories();

    writeln!(writer, "Default Exclusions by Category")?;
    writeln!(writer, "==============================")?;

    for (i, category) in categories.iter().enumerate() {
        if i > 0 {
            writeln!(writer)?;
        }

        writeln!(writer, "ID: {} - {}", category.id, category.name)?;
        writeln!(writer, "Description: {}", category.description)?;
        writeln!(writer, "Patterns:")?;

        let mut patterns = category.patterns.clone();
        patterns.sort();

        for pattern in patterns {
            writeln!(writer, "  {pattern}")?;
        }
    }

    writeln!(writer, "\nSummary:")?;
    writeln!(writer, "  Total categories: {}", categories.len())?;
    writeln!(writer, "  Total patterns: {}", get_all_patterns().len())?;

    writeln!(writer, "\nUsage:")?;
    writeln!(
        writer,
        "  --disable-category <id>     Disable a specific category"
    )?;
    writeln!(
        writer,
        "  --disable-category go,vcs   Disable multiple categories"
    )?;

    writeln!(writer, "\nExamples:")?;
    writeln!(
        writer,
        "  context-generator --disable-category go     # Include go.sum and Go test files"
    )?;
    writeln!(
        writer,
        "  context-generator --disable-category vcs    # Include .git directory contents"
    )?;
    writeln!(
        writer,
        "  context-generator --disable-category logs   # Include log files"
    )?;

    Ok(())
}

/// Prints only the exclusion patterns without category information.
///
/// This function outputs a clean list of all exclusion patterns, with wildcards
/// (patterns containing `*`, `?`, or `[`) listed first followed by literal patterns.
/// This format is useful for scripting and automated processing.
///
/// # Arguments
///
/// * `writer` - The writer to output to (e.g., `stdout`, `stderr`, or a file)
///
/// # Returns
///
/// * `Ok(())` - Successfully printed patterns
/// * `Err(std::io::Error)` - IO error during writing
///
/// # Examples
///
/// ```rust
/// use context_generator::filter::print_patterns_only;
/// use std::io;
///
/// // Print to stdout
/// print_patterns_only(&mut io::stdout()).unwrap();
///
/// // Count total patterns
/// let mut buffer = Vec::new();
/// print_patterns_only(&mut buffer).unwrap();
/// let pattern_count = String::from_utf8(buffer).unwrap().lines().count();
/// ```
pub fn print_patterns_only<W: std::io::Write>(writer: &mut W) -> std::io::Result<()> {
    let categories = get_exclusion_categories();
    let mut wildcards = Vec::new();
    let mut literals = Vec::new();

    for category in categories {
        for pattern in category.patterns {
            if pattern.contains('*') || pattern.contains('?') || pattern.contains('[') {
                wildcards.push(pattern);
            } else {
                literals.push(pattern);
            }
        }
    }

    wildcards.sort();
    literals.sort();

    for pattern in wildcards {
        writeln!(writer, "{pattern}")?;
    }
    for pattern in literals {
        writeln!(writer, "{pattern}")?;
    }

    Ok(())
}

/// Prints exclusion patterns for a specific category.
///
/// This function outputs only the patterns belonging to the specified category ID.
/// Patterns are sorted alphabetically for consistent output. If the category ID
/// is not found, an error message is printed.
///
/// # Arguments
///
/// * `writer` - The writer to output to (e.g., `stdout`, `stderr`, or a file)
/// * `category_id` - The ID of the category to print (e.g., "go", "python", "logs")
///
/// # Returns
///
/// * `Ok(())` - Successfully printed category patterns or error message
/// * `Err(std::io::Error)` - IO error during writing
///
/// # Examples
///
/// ```rust
/// use context_generator::filter::print_category_exclusions;
/// use std::io;
///
/// // Print Go-specific patterns
/// print_category_exclusions(&mut io::stdout(), "go").unwrap();
///
/// // Print Python patterns to a buffer
/// let mut buffer = Vec::new();
/// print_category_exclusions(&mut buffer, "python").unwrap();
/// let patterns = String::from_utf8(buffer).unwrap();
/// ```
pub fn print_category_exclusions<W: std::io::Write>(
    writer: &mut W,
    category_id: &str,
) -> std::io::Result<()> {
    let categories = get_exclusion_categories();

    for category in categories {
        if category.id == category_id {
            let mut patterns = category.patterns.clone();
            patterns.sort();

            for pattern in patterns {
                writeln!(writer, "{pattern}")?;
            }
            return Ok(());
        }
    }

    writeln!(writer, "Category {category_id} not found")?;
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::path::PathBuf;

    #[test]
    fn test_filter_creation() {
        let filter = Filter::new(vec!["*.txt".to_string()]).unwrap();
        assert_eq!(filter.patterns.len(), 1);
    }

    #[test]
    fn test_filter_with_defaults() {
        let filter = Filter::new_with_defaults(vec!["*.custom".to_string()], &[]).unwrap();
        assert!(filter.patterns.len() > 1);
    }

    #[test]
    fn test_should_exclude() {
        let filter = Filter::new(vec!["*.log".to_string()]).unwrap();
        let base_dir = PathBuf::from("/tmp");
        let log_file = PathBuf::from("/tmp/test.log");
        let txt_file = PathBuf::from("/tmp/test.txt");

        assert!(filter.should_exclude(&log_file, &base_dir, false));
        assert!(!filter.should_exclude(&txt_file, &base_dir, false));
    }

    #[test]
    fn test_validate_category_ids() {
        let invalid = validate_category_ids(&["nonexistent".to_string()]);
        assert_eq!(invalid, vec!["nonexistent".to_string()]);

        let invalid = validate_category_ids(&["go".to_string(), "python".to_string()]);
        assert!(invalid.is_empty());
    }

    #[test]
    fn test_get_category_for_pattern() {
        // Note: *.log appears in both LaTeX and Logs categories, function returns first match
        assert_eq!(get_category_for_pattern("go.sum"), "Go Specific");
        assert_eq!(get_category_for_pattern("*.pyc"), "Python");
        assert_eq!(get_category_for_pattern("*.tmp"), "Logs & Temporary");
        assert_eq!(get_category_for_pattern("nonexistent"), "Custom");
    }

    #[test]
    fn test_load_exclusion_data() {
        let data = load_exclusion_data().unwrap();
        assert!(!data.categories.is_empty());

        // Check that we have all expected categories
        let ids: Vec<String> = data.categories.iter().map(|c| c.id.clone()).collect();
        assert!(ids.contains(&"go".to_string()));
        assert!(ids.contains(&"python".to_string()));
        assert!(ids.contains(&"logs".to_string()));
        assert!(ids.contains(&"vcs".to_string()));
    }

    #[test]
    fn test_yaml_parsing() {
        let categories = get_exclusion_categories();
        assert!(!categories.is_empty());

        // Find the Go category and verify its patterns
        let go_category = categories.iter().find(|c| c.id == "go");
        assert!(go_category.is_some());

        let go_category = go_category.unwrap();
        assert!(go_category.patterns.contains(&"go.sum".to_string()));
        assert!(go_category.patterns.contains(&"*.test".to_string()));
    }
}
