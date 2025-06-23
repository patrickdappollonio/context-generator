package scanner

import (
	"bufio"
	"fmt"
	"io"
	"net/http"
	"os"
	"path/filepath"
	"sort"
	"strings"

	"github.com/patrickdappollonio/context-generator/internal/filter"
)

const separator = "--------------------"

// Scanner handles file system scanning and content processing
type Scanner struct {
	filter *filter.Filter
	writer io.Writer
}

// New creates a new Scanner with the given filter and writer
func New(f *filter.Filter, w io.Writer) *Scanner {
	return &Scanner{
		filter: f,
		writer: w,
	}
}

// Scan walks through the directory and processes text files
func (s *Scanner) Scan(directory string) error {
	// Check if the directory provided exists
	if _, err := os.Stat(directory); err != nil {
		if os.IsNotExist(err) {
			return fmt.Errorf("directory %q does not exist", directory)
		}
		return fmt.Errorf("error checking directory %q: %w", directory, err)
	}

	// Get absolute path for consistent relative path calculations
	absDir, err := filepath.Abs(directory)
	if err != nil {
		return fmt.Errorf("error getting absolute path for %q: %w", directory, err)
	}

	// Walk through all files and directories starting from the specified directory
	err = filepath.Walk(absDir, func(path string, info os.FileInfo, err error) error {
		if err != nil {
			return err
		}

		// Check if this path should be excluded
		if s.filter.ShouldExclude(path, absDir, info.IsDir()) {
			if info.IsDir() {
				return filepath.SkipDir
			}
			return nil
		}

		// Skip directories; process only files
		if info.IsDir() {
			return nil
		}

		return s.processFile(path, absDir)
	})

	// Write the final line of dashes
	fmt.Fprintln(s.writer, separator)

	return err
}

// processFile reads and outputs a single file if it's a text file
func (s *Scanner) processFile(path, baseDir string) error {
	// Open the file for reading
	file, err := os.Open(path)
	if err != nil {
		return err
	}
	defer file.Close()

	// Read the first 512 bytes to detect content type
	buffer := make([]byte, 512)
	n, err := file.Read(buffer)
	if err != nil && err != io.EOF {
		return err
	}

	// Reset the file pointer to the beginning
	if _, err := file.Seek(0, io.SeekStart); err != nil {
		return err
	}

	// Detect content type
	contentType := http.DetectContentType(buffer[:n])

	// Check if the content type indicates a text file
	if !strings.HasPrefix(contentType, "text/") {
		return nil
	}

	// Get relative path for display
	relPath, err := filepath.Rel(baseDir, path)
	if err != nil {
		relPath = path
	}

	// Write the first line of dashes
	fmt.Fprintln(s.writer, separator)
	// Write the relative file path
	fmt.Fprintln(s.writer, "file:", relPath)
	// Write the second line of dashes
	fmt.Fprintln(s.writer, separator)

	// Create a scanner to read the file line by line
	scanner := bufio.NewScanner(file)
	for scanner.Scan() {
		// Write each line with 4 spaces indentation
		fmt.Fprintf(s.writer, "    %s\n", scanner.Text())
	}

	// Check for errors during scanning
	if err := scanner.Err(); err != nil {
		return fmt.Errorf("error reading file %q: %w", path, err)
	}

	return nil
}

// FileInfo represents information about a file for dry-run output
type FileInfo struct {
	Path     string
	RelPath  string
	IsDir    bool
	IsText   bool
	Excluded bool
	Reason   *filter.ExclusionReason
}

// DryRun shows what files would be processed and which would be excluded
func (s *Scanner) DryRun(directory string) error {
	// Check if the directory provided exists
	if _, err := os.Stat(directory); err != nil {
		if os.IsNotExist(err) {
			return fmt.Errorf("directory %q does not exist", directory)
		}
		return fmt.Errorf("error checking directory %q: %w", directory, err)
	}

	// Get absolute path for consistent relative path calculations
	absDir, err := filepath.Abs(directory)
	if err != nil {
		return fmt.Errorf("error getting absolute path for %q: %w", directory, err)
	}

	var includedFiles []FileInfo
	var excludedFiles []FileInfo

	// Walk through all files and directories starting from the specified directory
	err = filepath.Walk(absDir, func(path string, info os.FileInfo, err error) error {
		if err != nil {
			return err
		}

		// Get relative path for display
		relPath, err := filepath.Rel(absDir, path)
		if err != nil {
			relPath = path
		}

		// Check if this path should be excluded
		reason := s.filter.GetExclusionReason(path, absDir, info.IsDir())
		if reason != nil {
			excludedFiles = append(excludedFiles, FileInfo{
				Path:     path,
				RelPath:  relPath,
				IsDir:    info.IsDir(),
				Excluded: true,
				Reason:   reason,
			})
			if info.IsDir() {
				return filepath.SkipDir
			}
			return nil
		}

		// For included files, check if they're text files (only for regular files)
		isText := false
		if !info.IsDir() {
			isText = s.isTextFile(path)
		}

		includedFiles = append(includedFiles, FileInfo{
			Path:     path,
			RelPath:  relPath,
			IsDir:    info.IsDir(),
			IsText:   isText,
			Excluded: false,
		})

		return nil
	})
	if err != nil {
		return err
	}

	// Print the results
	s.printDryRunResults(includedFiles, excludedFiles, directory)
	return nil
}

// isTextFile checks if a file is a text file
func (s *Scanner) isTextFile(path string) bool {
	file, err := os.Open(path)
	if err != nil {
		return false
	}
	defer file.Close()

	// Read the first 512 bytes to detect content type
	buffer := make([]byte, 512)
	n, err := file.Read(buffer)
	if err != nil && err != io.EOF {
		return false
	}

	// Detect content type
	contentType := http.DetectContentType(buffer[:n])
	return strings.HasPrefix(contentType, "text/")
}

// printDryRunResults prints the tree-style output for dry run
func (s *Scanner) printDryRunResults(includedFiles, excludedFiles []FileInfo, directory string) {
	// Sort files for consistent output
	sort.Slice(includedFiles, func(i, j int) bool {
		return includedFiles[i].RelPath < includedFiles[j].RelPath
	})
	sort.Slice(excludedFiles, func(i, j int) bool {
		return excludedFiles[i].RelPath < excludedFiles[j].RelPath
	})

	fmt.Fprintf(s.writer, "Dry run for directory: %s\n\n", directory)

	// Show files that would be processed
	fmt.Fprintf(s.writer, "Files that would be processed:\n")
	if len(includedFiles) == 0 {
		fmt.Fprintf(s.writer, "  (none)\n")
	} else {
		s.printTreeFiles(includedFiles, false)
	}

	fmt.Fprintf(s.writer, "\nFiles that would be excluded:\n")
	if len(excludedFiles) == 0 {
		fmt.Fprintf(s.writer, "  (none)\n")
	} else {
		s.printTreeFiles(excludedFiles, true)
	}
}

// printTreeFiles prints files in a tree-like structure
func (s *Scanner) printTreeFiles(files []FileInfo, showReasons bool) {
	// Build tree structure
	tree := s.buildFileTree(files)
	s.printFileTree(tree, "", false, showReasons)
}

// TreeNode represents a node in the file tree
type TreeNode struct {
	Name     string
	File     *FileInfo
	Children []*TreeNode
	IsDir    bool
}

// buildFileTree builds a hierarchical tree structure from flat file list
func (s *Scanner) buildFileTree(files []FileInfo) *TreeNode {
	root := &TreeNode{Name: "", IsDir: true}

	for _, file := range files {
		s.insertIntoTree(root, file)
	}

	// Sort children at each level
	s.sortTreeChildren(root)
	return root
}

// insertIntoTree inserts a file into the tree structure
func (s *Scanner) insertIntoTree(root *TreeNode, file FileInfo) {
	if file.RelPath == "." {
		return // Skip root directory entry
	}

	parts := strings.Split(file.RelPath, string(filepath.Separator))
	current := root

	// Navigate/create path to the file
	for i, part := range parts {
		isLast := i == len(parts)-1

		// Find existing child or create new one
		var child *TreeNode
		for _, c := range current.Children {
			if c.Name == part {
				child = c
				break
			}
		}

		if child == nil {
			child = &TreeNode{
				Name:  part,
				IsDir: !isLast || file.IsDir,
			}
			current.Children = append(current.Children, child)
		}

		// If this is the final part, store the file info
		if isLast {
			child.File = &file
			child.IsDir = file.IsDir
		}

		current = child
	}
}

// sortTreeChildren recursively sorts children in each tree node
func (s *Scanner) sortTreeChildren(node *TreeNode) {
	// Sort current level: directories first, then files, alphabetically within each group
	sort.Slice(node.Children, func(i, j int) bool {
		a, b := node.Children[i], node.Children[j]
		if a.IsDir != b.IsDir {
			return a.IsDir // Directories first
		}
		return a.Name < b.Name
	})

	// Recursively sort children
	for _, child := range node.Children {
		s.sortTreeChildren(child)
	}
}

// printFileTree recursively prints the tree structure
func (s *Scanner) printFileTree(node *TreeNode, prefix string, isLast bool, showReasons bool) {
	if node.Name != "" { // Don't print the root node
		s.printTreeNode(node, prefix, isLast, showReasons)
	}

	childCount := len(node.Children)
	for i, child := range node.Children {
		isLastChild := i == childCount-1

		var childPrefix string
		if node.Name == "" { // Root node
			childPrefix = "  "
		} else {
			if isLast {
				childPrefix = prefix + "    "
			} else {
				childPrefix = prefix + "│   "
			}
		}

		s.printFileTree(child, childPrefix, isLastChild, showReasons)
	}
}

// printTreeNode prints a single tree node with appropriate formatting
func (s *Scanner) printTreeNode(node *TreeNode, prefix string, isLast bool, showReason bool) {
	var treeChar string
	if isLast {
		treeChar = "└── "
	} else {
		treeChar = "├── "
	}

	name := node.Name

	// Add type indicator
	if node.File != nil {
		if node.File.IsDir {
			name += "/"
		} else if !node.File.IsText && !node.File.Excluded {
			name += " (binary, will be skipped)"
		}
	} else if node.IsDir {
		name += "/"
	}

	fmt.Fprintf(s.writer, "%s%s%s", prefix, treeChar, name)

	// Add exclusion reason if this is an excluded file
	if showReason && node.File != nil && node.File.Reason != nil {
		fmt.Fprintf(s.writer, " [%s: %s]", node.File.Reason.Category, node.File.Reason.Pattern)
	}

	fmt.Fprintf(s.writer, "\n")
}
