package filter

import (
	"fmt"
	"io"
	"path/filepath"
	"slices"
	"sort"
	"strings"
)

// ExclusionCategory represents a category of exclusions
type ExclusionCategory struct {
	ID          string
	Name        string
	Description string
	Patterns    []string
}

// exclusionCategories defines all default exclusion categories
var exclusionCategories = []ExclusionCategory{
	{
		ID:          "vcs",
		Name:        "Version Control",
		Description: "Version control systems and metadata",
		Patterns:    []string{".git", ".svn", ".hg", ".bzr"},
	},
	{
		ID:          "deps",
		Name:        "Dependencies",
		Description: "Package managers and dependency directories",
		Patterns:    []string{"node_modules", "vendor", "bower_components", ".pnpm-store"},
	},
	{
		ID:          "build",
		Name:        "Build Artifacts",
		Description: "Compiled code and build outputs",
		Patterns:    []string{"target", "build", "dist", "out", "bin", "obj", "*.o", "*.so", "*.dll", "*.exe"},
	},
	{
		ID:          "go",
		Name:        "Go Specific",
		Description: "Go language specific files",
		Patterns:    []string{"go.sum", "*.test", "coverage.out", "*.prof"},
	},
	{
		ID:          "js",
		Name:        "JavaScript/Node.js",
		Description: "JavaScript and Node.js specific files",
		Patterns:    []string{"*.min.js", "*.min.css", ".nyc_output", "coverage"},
	},
	{
		ID:          "python",
		Name:        "Python",
		Description: "Python specific files and directories",
		Patterns:    []string{"__pycache__", "*.pyc", "*.pyo", "*.pyd", ".pytest_cache", ".coverage", ".tox", "*.egg-info", ".venv", "venv"},
	},
	{
		ID:          "python-ds",
		Name:        "Python Data Science",
		Description: "Python data science and machine learning files",
		Patterns:    []string{".ipynb_checkpoints", "*.pkl", "*.pickle", "*.h5", "*.hdf5", "*.joblib", ".mlruns", "mlruns", "wandb", ".neptune", "lightning_logs", "*.model", "*.weights"},
	},
	{
		ID:          "java",
		Name:        "Java",
		Description: "Java specific files and directories",
		Patterns:    []string{"*.class", "*.jar", "*.war", "*.ear", ".gradle", "gradle-wrapper.jar"},
	},
	{
		ID:          "c",
		Name:        "C/C++",
		Description: "C and C++ specific files",
		Patterns:    []string{"*.o", "*.a", "*.lib", "*.obj", "*.pdb", "*.ilk", "*.exp"},
	},
	{
		ID:          "rust",
		Name:        "Rust",
		Description: "Rust specific files and directories",
		Patterns:    []string{"Cargo.lock", "target"},
	},
	{
		ID:          "logs",
		Name:        "Logs & Temporary",
		Description: "Log files and temporary data",
		Patterns:    []string{"*.log", "*.tmp", "*.temp", "*.cache", "*.swp", "*.swo", "*~"},
	},
	{
		ID:          "env",
		Name:        "Environment & Config",
		Description: "Environment variables and sensitive configuration",
		Patterns:    []string{".env*", "*.pem", "*.key", "*.crt", "*.p12", "secrets.json"},
	},
	{
		ID:          "ide",
		Name:        "IDE & Editors",
		Description: "IDE and editor specific files",
		Patterns:    []string{".vscode", ".idea", "*.sublime-*", ".vim", ".emacs.d", ".DS_Store", "Thumbs.db"},
	},
	{
		ID:          "docs",
		Name:        "Documentation",
		Description: "Generated documentation",
		Patterns:    []string{"docs/_build", "site", "_site", ".jekyll-cache"},
	},
	{
		ID:          "typescript",
		Name:        "TypeScript",
		Description: "TypeScript specific files and directories",
		Patterns:    []string{"*.tsbuildinfo", "*.d.ts.map", "tsconfig.tsbuildinfo", ".tscache", "*.js.map", "*.jsx.map", "*.ts.map", "*.tsx.map"},
	},
	{
		ID:          "php",
		Name:        "PHP",
		Description: "PHP specific files and directories",
		Patterns:    []string{"composer.lock", ".phpunit.result.cache", "*.phar", ".php_cs.cache", ".php-cs-fixer.cache", "phpunit.xml", "phpstan.neon", "psalm.xml"},
	},
	{
		ID:          "latex",
		Name:        "LaTeX",
		Description: "LaTeX document preparation system files",
		Patterns:    []string{"*.aux", "*.bbl", "*.blg", "*.fdb_latexmk", "*.fls", "*.log", "*.out", "*.synctex.gz", "*.toc", "*.lof", "*.lot", "*.idx", "*.ind", "*.ilg", "*.nav", "*.snm", "*.vrb"},
	},
	{
		ID:          "ruby",
		Name:        "Ruby",
		Description: "Ruby specific files and directories",
		Patterns:    []string{"Gemfile.lock", ".bundle", ".rspec", "coverage", "spec/reports", ".yardoc", "doc/", "*.gem"},
	},
	{
		ID:          "swift",
		Name:        "Swift",
		Description: "Swift and iOS development files",
		Patterns:    []string{"*.xcworkspace", "*.xcuserdata", "*.xcscheme", "DerivedData", "build", "*.ipa", "*.dSYM", "Pods", "Podfile.lock", "*.swiftpm"},
	},
	{
		ID:          "kotlin",
		Name:        "Kotlin",
		Description: "Kotlin specific files",
		Patterns:    []string{"*.kt~", "*.kts~", ".kotlin"},
	},
}

// DefaultExclusions contains all default exclusion patterns (flattened)
var DefaultExclusions []string

// GetExclusionCategories returns all exclusion categories
func GetExclusionCategories() []ExclusionCategory {
	return exclusionCategories
}

// GetAllPatterns returns all patterns from all categories
func GetAllPatterns() []string {
	return GetFilteredPatterns(nil)
}

// GetFilteredPatterns returns all patterns except those from disabled categories
func GetFilteredPatterns(disabledCategoryIDs []string) []string {
	var patterns []string

	for _, category := range exclusionCategories {
		// Skip if this category is disabled
		if slices.Contains(disabledCategoryIDs, category.ID) {
			continue
		}
		patterns = append(patterns, category.Patterns...)
	}

	return patterns
}

// GetCategoryForPattern returns the category name for a given pattern
func GetCategoryForPattern(pattern string) string {
	for _, category := range exclusionCategories {
		for _, p := range category.Patterns {
			if p == pattern {
				return category.Name
			}
		}
	}
	return "Custom"
}

// ValidateCategoryIDs checks if the provided category IDs are valid
func ValidateCategoryIDs(ids []string) []string {
	var invalid []string
	validIDs := make(map[string]bool)

	for _, category := range exclusionCategories {
		validIDs[category.ID] = true
	}

	for _, id := range ids {
		if !validIDs[id] {
			invalid = append(invalid, id)
		}
	}

	return invalid
}

// PrintExclusions prints all exclusions organized by category with patterns in columns
func PrintExclusions(w io.Writer) {
	fmt.Fprintln(w, "Default Exclusions by Category")
	fmt.Fprintln(w, "==============================")

	for i, category := range exclusionCategories {
		if i > 0 {
			fmt.Fprintln(w) // Add spacing between categories
		}

		// Print category header
		fmt.Fprintf(w, "ID: %s - %s\n", category.ID, category.Name)
		fmt.Fprintf(w, "Description: %s\n", category.Description)
		fmt.Fprintln(w, "Patterns:")

		// Sort patterns for consistent output
		patterns := make([]string, len(category.Patterns))
		copy(patterns, category.Patterns)
		sort.Strings(patterns)

		// Print each pattern on its own line with indentation
		for _, pattern := range patterns {
			fmt.Fprintf(w, "  %s\n", pattern)
		}
	}

	fmt.Fprintf(w, "\nSummary:\n")
	fmt.Fprintf(w, "  Total categories: %d\n", len(exclusionCategories))
	fmt.Fprintf(w, "  Total patterns: %d\n", len(GetAllPatterns()))

	fmt.Fprintln(w, "\nUsage:")
	fmt.Fprintln(w, "  --disable-category <id>     Disable a specific category")
	fmt.Fprintln(w, "  --disable-category go,vcs   Disable multiple categories")

	fmt.Fprintln(w, "\nExamples:")
	fmt.Fprintln(w, "  context-generator --disable-category go     # Include go.sum and Go test files")
	fmt.Fprintln(w, "  context-generator --disable-category vcs    # Include .git directory contents")
	fmt.Fprintln(w, "  context-generator --disable-category logs   # Include log files")
}

// Filter handles path exclusion based on patterns
type Filter struct {
	patterns []string
}

// New creates a new Filter with the given patterns
func New(patterns []string) *Filter {
	return &Filter{
		patterns: patterns,
	}
}

// NewWithDefaults creates a new Filter with default exclusions plus additional patterns
func NewWithDefaults(additionalPatterns, disabledCategoryIDs []string) *Filter {
	defaultPatterns := GetFilteredPatterns(disabledCategoryIDs)
	patterns := make([]string, 0, len(defaultPatterns)+len(additionalPatterns))
	patterns = append(patterns, defaultPatterns...)
	patterns = append(patterns, additionalPatterns...)
	return &Filter{
		patterns: patterns,
	}
}

// ShouldExclude checks if a path should be excluded based on the exclusion patterns.
// It supports folder names, relative paths, and wildcards.
func (f *Filter) ShouldExclude(path, baseDir string, isDir bool) bool {
	reason := f.GetExclusionReason(path, baseDir, isDir)
	return reason != nil
}

// ExclusionReason represents why a file was excluded
type ExclusionReason struct {
	Pattern  string
	Category string
}

// GetExclusionReason returns the reason a path would be excluded, or nil if it wouldn't be excluded.
// It supports folder names, relative paths, and wildcards.
func (f *Filter) GetExclusionReason(path, baseDir string, isDir bool) *ExclusionReason {
	// Get the relative path from the base directory
	relPath, err := filepath.Rel(baseDir, path)
	if err != nil {
		relPath = path
	}

	// Get just the name (last component) of the path
	name := filepath.Base(path)

	for _, pattern := range f.patterns {
		// Try matching against the name
		if matched, _ := filepath.Match(pattern, name); matched {
			return &ExclusionReason{
				Pattern:  pattern,
				Category: GetCategoryForPattern(pattern),
			}
		}

		// Try matching against the relative path
		if matched, _ := filepath.Match(pattern, relPath); matched {
			return &ExclusionReason{
				Pattern:  pattern,
				Category: GetCategoryForPattern(pattern),
			}
		}

		// For directories, also try matching against path components
		if isDir {
			pathComponents := strings.Split(relPath, string(filepath.Separator))
			for _, component := range pathComponents {
				if matched, _ := filepath.Match(pattern, component); matched {
					return &ExclusionReason{
						Pattern:  pattern,
						Category: GetCategoryForPattern(pattern),
					}
				}
			}
		}
	}

	return nil
}

// PrintPatternsOnly prints only the patterns ordered globally (wildcards first, then literals)
func PrintPatternsOnly(w io.Writer) {
	// Collect all patterns from all categories
	var wildcards, literals []string

	for _, category := range exclusionCategories {
		for _, pattern := range category.Patterns {
			if strings.ContainsAny(pattern, "*?[]") {
				wildcards = append(wildcards, pattern)
			} else {
				literals = append(literals, pattern)
			}
		}
	}

	// Sort each group alphabetically
	sort.Strings(wildcards)
	sort.Strings(literals)

	// Print wildcards first, then literals
	for _, pattern := range wildcards {
		fmt.Fprintln(w, pattern)
	}
	for _, pattern := range literals {
		fmt.Fprintln(w, pattern)
	}
}

// PrintCategoryExclusions prints exclusions for a specific category ID
func PrintCategoryExclusions(w io.Writer, categoryID string) {
	for _, category := range exclusionCategories {
		if category.ID != categoryID {
			continue
		}
		// Sort patterns for consistent output
		patterns := make([]string, len(category.Patterns))
		copy(patterns, category.Patterns)
		sort.Strings(patterns)

		// Print each pattern on its own line
		for _, pattern := range patterns {
			fmt.Fprintln(w, pattern)
		}
		return
	}
	// This shouldn't happen due to validation, but just in case
	fmt.Fprintf(w, "Category %s not found\n", categoryID)
}
