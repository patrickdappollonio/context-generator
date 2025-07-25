//! File system scanning and content generation.
//!
//! This module provides functionality for scanning directory structures,
//! filtering files based on exclusion patterns, and generating formatted
//! output suitable for AI context. It supports both normal scanning mode
//! and dry-run mode for previewing what files would be processed.

use crate::filter::{ExclusionReason, Filter};
use std::fs::File;
use std::io::{BufRead, BufReader, Read, Write};
use std::path::Path;
use walkdir::WalkDir;

/// Separator string used between file sections in the output
const SEPARATOR: &str = "--------------------";

/// File system scanner that generates formatted context output.
///
/// The scanner uses a provided [`Filter`] to determine which files to include
/// or exclude, then processes text files to generate a formatted output
/// suitable for providing context to AI assistants.
///
/// # Examples
///
/// ```rust
/// use context_generator::{filter::Filter, scanner::Scanner};
/// use std::io;
///
/// // Create a filter and scanner
/// let filter = Filter::new_with_defaults(vec![], &[]).unwrap();
/// let scanner = Scanner::new(filter);
///
/// // Scan a directory
/// let mut stdout = io::stdout();
/// scanner.scan("src/", &mut stdout).unwrap();
/// ```
pub struct Scanner {
    filter: Filter,
}

impl Scanner {
    /// Creates a new scanner with the specified filter.
    ///
    /// # Arguments
    ///
    /// * `filter` - The filter to use for determining which files to exclude
    ///
    /// # Examples
    ///
    /// ```rust
    /// use context_generator::{filter::Filter, scanner::Scanner};
    ///
    /// let filter = Filter::new_with_defaults(vec![], &[]).unwrap();
    /// let scanner = Scanner::new(filter);
    /// ```
    pub fn new(filter: Filter) -> Self {
        Scanner { filter }
    }

    /// Scans a directory and generates formatted context output.
    ///
    /// This method walks through the specified directory (or processes a single file),
    /// excludes files based on the filter rules, and generates a formatted output
    /// containing the contents of all text files. The output format includes file
    /// separators and relative paths suitable for AI context.
    ///
    /// # Arguments
    ///
    /// * `directory` - Path to the directory to scan (or single file to process)
    /// * `writer` - Writer to output the formatted content to
    ///
    /// # Returns
    ///
    /// * `Ok(())` - Successfully scanned and generated output
    /// * `Err(anyhow::Error)` - Directory doesn't exist, permission denied, or IO error
    ///
    /// # Examples
    ///
    /// ```rust
    /// use context_generator::{filter::Filter, scanner::Scanner};
    /// use std::io;
    ///
    /// let filter = Filter::new_with_defaults(vec![], &[]).unwrap();
    /// let scanner = Scanner::new(filter);
    ///
    /// // Scan to stdout
    /// scanner.scan("src/", &mut io::stdout()).unwrap();
    ///
    /// // Scan to a buffer
    /// let mut buffer = Vec::new();
    /// scanner.scan("src/", &mut buffer).unwrap();
    /// let output = String::from_utf8(buffer).unwrap();
    /// ```
    ///
    /// # Output Format
    ///
    /// The generated output follows this format:
    /// ```text
    /// --------------------
    /// file: src/main.rs
    /// --------------------
    ///     use std::io;
    ///
    ///     fn main() {
    ///         println!("Hello, world!");
    ///     }
    /// --------------------
    /// file: src/lib.rs
    /// --------------------
    ///     // Library code here...
    /// --------------------
    /// ```
    pub fn scan<P: AsRef<Path>, W: Write>(
        &self,
        directory: P,
        writer: &mut W,
    ) -> anyhow::Result<()> {
        let directory = directory.as_ref();

        if !directory.exists() {
            return Err(anyhow::anyhow!("Directory {:?} does not exist", directory));
        }

        let abs_dir = directory.canonicalize().map_err(|e| {
            anyhow::anyhow!("Error getting absolute path for {:?}: {}", directory, e)
        })?;

        // Handle case where input is a single file
        if abs_dir.is_file() {
            // Use the file itself as the base directory to match Go's filepath.Walk behavior
            self.process_file(&abs_dir, &abs_dir, writer)?;
            writeln!(writer, "{SEPARATOR}")?;
            return Ok(());
        }

        for entry in WalkDir::new(&abs_dir) {
            let entry = entry.map_err(|e| anyhow::anyhow!("Error walking directory: {}", e))?;
            let path = entry.path();

            if self
                .filter
                .should_exclude(path, &abs_dir, entry.file_type().is_dir())
            {
                if entry.file_type().is_dir() {
                    continue;
                }
                continue;
            }

            if entry.file_type().is_file() {
                self.process_file(path, &abs_dir, writer)?;
            }
        }

        writeln!(writer, "{SEPARATOR}")?;
        Ok(())
    }

    /// Performs a dry-run scan showing what files would be processed or excluded.
    ///
    /// This method walks through the directory structure and categorizes files
    /// into those that would be processed and those that would be excluded,
    /// without actually reading or processing any file contents. It provides
    /// a preview of what the regular scan would do.
    ///
    /// # Arguments
    ///
    /// * `directory` - Path to the directory to analyze
    /// * `writer` - Writer to output the dry-run report to
    ///
    /// # Returns
    ///
    /// * `Ok(())` - Successfully generated dry-run report
    /// * `Err(anyhow::Error)` - Directory doesn't exist, permission denied, or IO error
    ///
    /// # Examples
    ///
    /// ```rust
    /// use context_generator::{filter::Filter, scanner::Scanner};
    /// use std::io;
    ///
    /// let filter = Filter::new_with_defaults(vec![], &[]).unwrap();
    /// let scanner = Scanner::new(filter);
    ///
    /// // Preview what would be scanned
    /// scanner.dry_run("src/", &mut io::stdout()).unwrap();
    /// ```
    ///
    /// # Output Format
    ///
    /// ```text
    /// Dry run for directory: src/
    ///
    /// Files that would be processed:
    ///   ├── main.rs
    ///   ├── lib.rs
    ///   └── utils.rs
    ///
    /// Files that would be excluded:
    ///   ├── target/ [Build Artifacts: target]
    ///   ├── *.log [Logs & Temporary: *.log]
    ///   └── .git/ [Version Control: .git]
    /// ```
    pub fn dry_run<P: AsRef<Path>, W: Write>(
        &self,
        directory: P,
        writer: &mut W,
    ) -> anyhow::Result<()> {
        let directory = directory.as_ref();

        if !directory.exists() {
            return Err(anyhow::anyhow!("Directory {:?} does not exist", directory));
        }

        let abs_dir = directory.canonicalize().map_err(|e| {
            anyhow::anyhow!("Error getting absolute path for {:?}: {}", directory, e)
        })?;

        let mut included_files = Vec::new();
        let mut excluded_files = Vec::new();

        for entry in WalkDir::new(&abs_dir) {
            let entry = entry.map_err(|e| anyhow::anyhow!("Error walking directory: {}", e))?;
            let path = entry.path();

            let rel_path = path
                .strip_prefix(&abs_dir)
                .unwrap_or(path)
                .to_string_lossy()
                .to_string();

            if let Some(reason) =
                self.filter
                    .get_exclusion_reason(path, &abs_dir, entry.file_type().is_dir())
            {
                excluded_files.push(FileInfo {
                    rel_path,
                    is_dir: entry.file_type().is_dir(),
                    is_text: false,
                    excluded: true,
                    reason: Some(reason),
                });
                continue;
            }

            let is_text = if entry.file_type().is_file() {
                self.is_text_file(path)?
            } else {
                false
            };

            included_files.push(FileInfo {
                rel_path,
                is_dir: entry.file_type().is_dir(),
                is_text,
                excluded: false,
                reason: None,
            });
        }

        self.print_dry_run_results(&included_files, &excluded_files, directory, writer)?;
        Ok(())
    }

    fn process_file<P: AsRef<Path>, W: Write>(
        &self,
        path: P,
        base_dir: P,
        writer: &mut W,
    ) -> anyhow::Result<()> {
        let path = path.as_ref();
        let base_dir = base_dir.as_ref();

        if !self.is_text_file(path)? {
            return Ok(());
        }

        let rel_path = path
            .strip_prefix(base_dir)
            .unwrap_or(path)
            .to_string_lossy();

        // If rel_path is empty (happens when path == base_dir), use "." like Go does
        let display_path = if rel_path.is_empty() { "." } else { &rel_path };

        writeln!(writer, "{SEPARATOR}")?;
        writeln!(writer, "file: {display_path}")?;
        writeln!(writer, "{SEPARATOR}")?;

        let file = File::open(path)
            .map_err(|e| anyhow::anyhow!("Error opening file {:?}: {}", path, e))?;
        let reader = BufReader::new(file);

        for line in reader.lines() {
            let line = line.map_err(|e| anyhow::anyhow!("Error reading file {:?}: {}", path, e))?;
            writeln!(writer, "    {line}")?;
        }

        Ok(())
    }

    fn is_text_file<P: AsRef<Path>>(&self, path: P) -> anyhow::Result<bool> {
        let path = path.as_ref();
        let mut file = File::open(path)
            .map_err(|e| anyhow::anyhow!("Error opening file {:?}: {}", path, e))?;

        let mut buffer = [0u8; 1024]; // content_inspector recommends at least 1024 bytes
        let n = file
            .read(&mut buffer)
            .map_err(|e| anyhow::anyhow!("Error reading file {:?}: {}", path, e))?;

        if n == 0 {
            return Ok(true); // Empty files are considered text
        }

        // Use content_inspector for robust binary vs text detection
        let content_type = content_inspector::inspect(&buffer[..n]);
        Ok(content_type.is_text())
    }

    fn print_dry_run_results<W: Write>(
        &self,
        included_files: &[FileInfo],
        excluded_files: &[FileInfo],
        directory: &Path,
        writer: &mut W,
    ) -> anyhow::Result<()> {
        let mut included_sorted = included_files.to_vec();
        let mut excluded_sorted = excluded_files.to_vec();

        included_sorted.sort_by(|a, b| a.rel_path.cmp(&b.rel_path));
        excluded_sorted.sort_by(|a, b| a.rel_path.cmp(&b.rel_path));

        writeln!(writer, "Dry run for directory: {}\n", directory.display())?;

        writeln!(writer, "Files that would be processed:")?;
        if included_sorted.is_empty() {
            writeln!(writer, "  (none)")?;
        } else {
            self.print_tree_files(&included_sorted, false, writer)?;
        }

        writeln!(writer, "\nFiles that would be excluded:")?;
        if excluded_sorted.is_empty() {
            writeln!(writer, "  (none)")?;
        } else {
            self.print_tree_files(&excluded_sorted, true, writer)?;
        }

        Ok(())
    }

    fn print_tree_files<W: Write>(
        &self,
        files: &[FileInfo],
        show_reasons: bool,
        writer: &mut W,
    ) -> anyhow::Result<()> {
        let tree = self.build_file_tree(files);
        self.print_file_tree(&tree, "", false, show_reasons, writer)?;
        Ok(())
    }

    fn build_file_tree(&self, files: &[FileInfo]) -> TreeNode {
        let mut root = TreeNode {
            name: String::new(),
            file: None,
            children: Vec::new(),
            is_dir: true,
        };

        for file in files {
            self.insert_into_tree(&mut root, file);
        }

        self.sort_tree_children(&mut root);
        root
    }

    fn insert_into_tree(&self, root: &mut TreeNode, file: &FileInfo) {
        if file.rel_path == "." {
            return;
        }

        let parts: Vec<&str> = file.rel_path.split('/').collect();
        let mut current = root;

        for (i, part) in parts.iter().enumerate() {
            let is_last = i == parts.len() - 1;

            let child_index = current
                .children
                .iter()
                .position(|child| child.name == *part);

            let child_index = if let Some(index) = child_index {
                index
            } else {
                let new_child = TreeNode {
                    name: part.to_string(),
                    file: None,
                    children: Vec::new(),
                    is_dir: !is_last || file.is_dir,
                };
                current.children.push(new_child);
                current.children.len() - 1
            };

            if is_last {
                current.children[child_index].file = Some(file.clone());
                current.children[child_index].is_dir = file.is_dir;
            }

            current = &mut current.children[child_index];
        }
    }

    fn sort_tree_children(&self, node: &mut TreeNode) {
        node.children.sort_by(|a, b| match (a.is_dir, b.is_dir) {
            (true, false) => std::cmp::Ordering::Less,
            (false, true) => std::cmp::Ordering::Greater,
            _ => a.name.cmp(&b.name),
        });

        for child in &mut node.children {
            Self::sort_tree_children_recursive(child);
        }
    }

    fn sort_tree_children_recursive(node: &mut TreeNode) {
        node.children.sort_by(|a, b| match (a.is_dir, b.is_dir) {
            (true, false) => std::cmp::Ordering::Less,
            (false, true) => std::cmp::Ordering::Greater,
            _ => a.name.cmp(&b.name),
        });

        for child in &mut node.children {
            Self::sort_tree_children_recursive(child);
        }
    }

    fn print_file_tree<W: Write>(
        &self,
        node: &TreeNode,
        prefix: &str,
        is_last: bool,
        show_reasons: bool,
        writer: &mut W,
    ) -> anyhow::Result<()> {
        if !node.name.is_empty() {
            self.print_tree_node(node, prefix, is_last, show_reasons, writer)?;
        }

        let child_count = node.children.len();
        for (i, child) in node.children.iter().enumerate() {
            let is_last_child = i == child_count - 1;

            let child_prefix = if node.name.is_empty() {
                "  ".to_string()
            } else if is_last {
                format!("{prefix}    ")
            } else {
                format!("{prefix}│   ")
            };

            self.print_file_tree(child, &child_prefix, is_last_child, show_reasons, writer)?;
        }

        Ok(())
    }

    fn print_tree_node<W: Write>(
        &self,
        node: &TreeNode,
        prefix: &str,
        is_last: bool,
        show_reason: bool,
        writer: &mut W,
    ) -> anyhow::Result<()> {
        let tree_char = if is_last { "└── " } else { "├── " };
        let mut name = node.name.clone();

        if let Some(file) = &node.file {
            if file.is_dir {
                name.push('/');
            } else if !file.is_text && !file.excluded {
                name.push_str(" (binary, will be skipped)");
            }
        } else if node.is_dir {
            name.push('/');
        }

        write!(writer, "{prefix}{tree_char}{name}")?;

        if show_reason {
            if let Some(file) = &node.file {
                if let Some(reason) = &file.reason {
                    write!(writer, " [{}: {}]", reason.category, reason.pattern)?;
                }
            }
        }

        writeln!(writer)?;
        Ok(())
    }
}

#[derive(Debug, Clone)]
struct FileInfo {
    rel_path: String,
    is_dir: bool,
    is_text: bool,
    excluded: bool,
    reason: Option<ExclusionReason>,
}

#[derive(Debug)]
struct TreeNode {
    name: String,
    file: Option<FileInfo>,
    children: Vec<TreeNode>,
    is_dir: bool,
}
