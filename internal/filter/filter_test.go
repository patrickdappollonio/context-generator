package filter

import (
	"bytes"
	"os"
	"path/filepath"
	"reflect"
	"slices"
	"strings"
	"testing"
)

func TestGetExclusionCategories(t *testing.T) {
	categories := GetExclusionCategories()

	if len(categories) == 0 {
		t.Error("GetExclusionCategories() returned empty slice")
	}

	// Check that we have expected categories
	categoryIDs := make(map[string]bool)
	for _, cat := range categories {
		categoryIDs[cat.ID] = true

		// Validate category structure
		if cat.ID == "" {
			t.Error("Found category with empty ID")
		}
		if cat.Name == "" {
			t.Error("Found category with empty Name")
		}
		if len(cat.Patterns) == 0 {
			t.Errorf("Category %s has no patterns", cat.ID)
		}
	}

	// Check for some expected categories
	expectedCats := []string{"vcs", "deps", "build", "go", "js", "python"}
	for _, expected := range expectedCats {
		if !categoryIDs[expected] {
			t.Errorf("Expected category %s not found", expected)
		}
	}
}

func TestGetAllPatterns(t *testing.T) {
	patterns := GetAllPatterns()

	if len(patterns) == 0 {
		t.Error("GetAllPatterns() returned empty slice")
	}

	// Should contain patterns from all categories
	categories := GetExclusionCategories()
	expectedCount := 0
	for _, cat := range categories {
		expectedCount += len(cat.Patterns)
	}

	if len(patterns) != expectedCount {
		t.Errorf("GetAllPatterns() returned %d patterns, expected %d", len(patterns), expectedCount)
	}
}

func TestGetFilteredPatterns(t *testing.T) {
	tests := []struct {
		name               string
		disabledCategories []string
		shouldContain      []string
		shouldNotContain   []string
	}{
		{
			name:               "no disabled categories",
			disabledCategories: nil,
			shouldContain:      []string{".git", "node_modules", "*.o"},
			shouldNotContain:   nil,
		},
		{
			name:               "disable vcs category",
			disabledCategories: []string{"vcs"},
			shouldContain:      []string{"node_modules", "*.o"},
			shouldNotContain:   []string{".git", ".svn"},
		},
		{
			name:               "disable multiple categories",
			disabledCategories: []string{"vcs", "deps"},
			shouldContain:      []string{"*.o", "*.log"},
			shouldNotContain:   []string{".git", "node_modules"},
		},
		{
			name:               "disable nonexistent category",
			disabledCategories: []string{"nonexistent"},
			shouldContain:      []string{".git", "node_modules"},
			shouldNotContain:   nil,
		},
	}

	for _, tt := range tests {
		t.Run(tt.name, func(t *testing.T) {
			patterns := GetFilteredPatterns(tt.disabledCategories)

			for _, pattern := range tt.shouldContain {
				if !slices.Contains(patterns, pattern) {
					t.Errorf("Expected pattern %s not found in filtered patterns", pattern)
				}
			}

			for _, pattern := range tt.shouldNotContain {
				if slices.Contains(patterns, pattern) {
					t.Errorf("Pattern %s should not be in filtered patterns but was found", pattern)
				}
			}
		})
	}
}

func TestGetCategoryForPattern(t *testing.T) {
	tests := []struct {
		name     string
		pattern  string
		expected string
	}{
		{
			name:     "git pattern",
			pattern:  ".git",
			expected: "Version Control",
		},
		{
			name:     "node_modules pattern",
			pattern:  "node_modules",
			expected: "Dependencies",
		},
		{
			name:     "unknown pattern",
			pattern:  "unknown-pattern",
			expected: "Custom",
		},
		{
			name:     "empty pattern",
			pattern:  "",
			expected: "Custom",
		},
	}

	for _, tt := range tests {
		t.Run(tt.name, func(t *testing.T) {
			result := GetCategoryForPattern(tt.pattern)
			if result != tt.expected {
				t.Errorf("GetCategoryForPattern(%q) = %q, want %q", tt.pattern, result, tt.expected)
			}
		})
	}
}

func TestValidateCategoryIDs(t *testing.T) {
	tests := []struct {
		name     string
		ids      []string
		expected []string
	}{
		{
			name:     "all valid IDs",
			ids:      []string{"vcs", "deps", "build"},
			expected: nil,
		},
		{
			name:     "some invalid IDs",
			ids:      []string{"vcs", "invalid1", "deps", "invalid2"},
			expected: []string{"invalid1", "invalid2"},
		},
		{
			name:     "all invalid IDs",
			ids:      []string{"invalid1", "invalid2"},
			expected: []string{"invalid1", "invalid2"},
		},
		{
			name:     "empty slice",
			ids:      []string{},
			expected: nil,
		},
		{
			name:     "nil slice",
			ids:      nil,
			expected: nil,
		},
	}

	for _, tt := range tests {
		t.Run(tt.name, func(t *testing.T) {
			result := ValidateCategoryIDs(tt.ids)
			if !reflect.DeepEqual(result, tt.expected) {
				t.Errorf("ValidateCategoryIDs(%v) = %v, want %v", tt.ids, result, tt.expected)
			}
		})
	}
}

func TestNew(t *testing.T) {
	patterns := []string{"*.txt", "*.log"}
	filter := New(patterns)

	if filter == nil {
		t.Error("New() returned nil")
	}

	// Test that the filter was created with the patterns
	// We can't directly access the patterns field since it's private,
	// but we can test the behavior indirectly
}

func TestNewWithDefaults(t *testing.T) {
	tests := []struct {
		name                string
		additionalPatterns  []string
		disabledCategoryIDs []string
		shouldExclude       []string
		shouldNotExclude    []string
	}{
		{
			name:                "with additional patterns",
			additionalPatterns:  []string{"*.custom"},
			disabledCategoryIDs: nil,
		},
		{
			name:                "with disabled categories",
			additionalPatterns:  nil,
			disabledCategoryIDs: []string{"vcs"},
		},
		{
			name:                "with both additional and disabled",
			additionalPatterns:  []string{"*.custom"},
			disabledCategoryIDs: []string{"vcs", "deps"},
		},
	}

	for _, tt := range tests {
		t.Run(tt.name, func(t *testing.T) {
			filter := NewWithDefaults(tt.additionalPatterns, tt.disabledCategoryIDs)
			if filter == nil {
				t.Error("NewWithDefaults() returned nil")
			}
		})
	}
}

func TestFilter_ShouldExclude(t *testing.T) {
	// Create a temporary directory structure for testing
	tempDir, err := os.MkdirTemp("", "filter_test")
	if err != nil {
		t.Fatalf("Failed to create temp dir: %v", err)
	}
	defer os.RemoveAll(tempDir)

	// Create test files and directories
	testPaths := []string{
		".git",
		"node_modules",
		"src/main.go",
		"README.md",
		"target/release",
		"app.log",
		"debug.log",
	}

	for _, path := range testPaths {
		fullPath := filepath.Join(tempDir, path)
		if strings.Contains(path, "/") {
			if err := os.MkdirAll(filepath.Dir(fullPath), 0o755); err != nil {
				t.Fatalf("Failed to create dir %s: %v", filepath.Dir(fullPath), err)
			}
		}
		if !strings.HasSuffix(path, "/") && path != ".git" && path != "node_modules" && path != "target/release" {
			if err := os.WriteFile(fullPath, []byte("test"), 0o644); err != nil {
				t.Fatalf("Failed to create file %s: %v", fullPath, err)
			}
		} else if strings.HasSuffix(path, ".log") {
			if err := os.WriteFile(fullPath, []byte("log content"), 0o644); err != nil {
				t.Fatalf("Failed to create file %s: %v", fullPath, err)
			}
		} else {
			if err := os.MkdirAll(fullPath, 0o755); err != nil {
				t.Fatalf("Failed to create dir %s: %v", fullPath, err)
			}
		}
	}

	filter := NewWithDefaults(nil, nil)

	tests := []struct {
		name          string
		path          string
		isDir         bool
		shouldExclude bool
	}{
		{
			name:          "git directory should be excluded",
			path:          filepath.Join(tempDir, ".git"),
			isDir:         true,
			shouldExclude: true,
		},
		{
			name:          "node_modules should be excluded",
			path:          filepath.Join(tempDir, "node_modules"),
			isDir:         true,
			shouldExclude: true,
		},
		{
			name:          "source file should not be excluded",
			path:          filepath.Join(tempDir, "src", "main.go"),
			isDir:         false,
			shouldExclude: false,
		},
		{
			name:          "readme should not be excluded",
			path:          filepath.Join(tempDir, "README.md"),
			isDir:         false,
			shouldExclude: false,
		},
		{
			name:          "log file should be excluded",
			path:          filepath.Join(tempDir, "app.log"),
			isDir:         false,
			shouldExclude: true,
		},
		{
			name:          "target directory should be excluded",
			path:          filepath.Join(tempDir, "target/release"),
			isDir:         true,
			shouldExclude: true,
		},
	}

	for _, tt := range tests {
		t.Run(tt.name, func(t *testing.T) {
			result := filter.ShouldExclude(tt.path, tempDir, tt.isDir)
			if result != tt.shouldExclude {
				t.Errorf("ShouldExclude(%q, %q, %v) = %v, want %v",
					tt.path, tempDir, tt.isDir, result, tt.shouldExclude)
			}
		})
	}
}

func TestFilter_ShouldExclude_Patterns(t *testing.T) {
	tempDir, err := os.MkdirTemp("", "filter_pattern_test")
	if err != nil {
		t.Fatalf("Failed to create temp dir: %v", err)
	}
	defer os.RemoveAll(tempDir)

	tests := []struct {
		name          string
		patterns      []string
		path          string
		isDir         bool
		shouldExclude bool
	}{
		{
			name:          "exact match",
			patterns:      []string{"test.txt"},
			path:          "test.txt",
			isDir:         false,
			shouldExclude: true,
		},
		{
			name:          "wildcard match",
			patterns:      []string{"*.log"},
			path:          "app.log",
			isDir:         false,
			shouldExclude: true,
		},
		{
			name:          "directory match",
			patterns:      []string{"temp"},
			path:          "temp",
			isDir:         true,
			shouldExclude: true,
		},
		{
			name:          "no match",
			patterns:      []string{"*.txt"},
			path:          "app.go",
			isDir:         false,
			shouldExclude: false,
		},
	}

	for _, tt := range tests {
		t.Run(tt.name, func(t *testing.T) {
			filter := New(tt.patterns)
			fullPath := filepath.Join(tempDir, tt.path)
			result := filter.ShouldExclude(fullPath, tempDir, tt.isDir)
			if result != tt.shouldExclude {
				t.Errorf("ShouldExclude with pattern %v for path %q = %v, want %v",
					tt.patterns, tt.path, result, tt.shouldExclude)
			}
		})
	}
}

func TestFilter_ShouldExclude_EdgeCases(t *testing.T) {
	tempDir, err := os.MkdirTemp("", "filter_edge_test")
	if err != nil {
		t.Fatalf("Failed to create temp dir: %v", err)
	}
	defer os.RemoveAll(tempDir)

	// Create nested directory structure for testing directory component matching
	nestedPath := filepath.Join(tempDir, "project", "node_modules", "package")
	if err := os.MkdirAll(nestedPath, 0o755); err != nil {
		t.Fatalf("Failed to create nested dir: %v", err)
	}

	filter := NewWithDefaults(nil, nil)

	tests := []struct {
		name          string
		path          string
		baseDir       string
		isDir         bool
		shouldExclude bool
	}{
		{
			name:          "nested excluded directory component",
			path:          nestedPath,
			baseDir:       tempDir,
			isDir:         true,
			shouldExclude: true,
		},
		{
			name:          "filepath.Rel error handling - invalid base dir",
			path:          filepath.Join(tempDir, "test.txt"),
			baseDir:       "/nonexistent/path",
			isDir:         false,
			shouldExclude: false, // Should handle error gracefully
		},
		{
			name:          "empty pattern list should not exclude anything",
			path:          filepath.Join(tempDir, "test.txt"),
			baseDir:       tempDir,
			isDir:         false,
			shouldExclude: false,
		},
	}

	for _, tt := range tests {
		t.Run(tt.name, func(t *testing.T) {
			// For the empty pattern test, use a filter with no patterns
			testFilter := filter
			if tt.name == "empty pattern list should not exclude anything" {
				testFilter = New([]string{})
			}

			result := testFilter.ShouldExclude(tt.path, tt.baseDir, tt.isDir)
			if result != tt.shouldExclude {
				t.Errorf("ShouldExclude(%q, %q, %v) = %v, want %v",
					tt.path, tt.baseDir, tt.isDir, result, tt.shouldExclude)
			}
		})
	}
}

func TestPrintExclusions(t *testing.T) {
	var buffer bytes.Buffer

	// Call the function with buffer
	PrintExclusions(&buffer)

	output := buffer.String()

	// Check that output contains expected content
	if !strings.Contains(output, "Default Exclusions by Category") {
		t.Error("Output should contain title")
	}

	if !strings.Contains(output, "Version Control") {
		t.Error("Output should contain category names")
	}

	if !strings.Contains(output, ".git") {
		t.Error("Output should contain patterns")
	}

	if !strings.Contains(output, "Total categories:") {
		t.Error("Output should contain summary")
	}

	if !strings.Contains(output, "Usage:") {
		t.Error("Output should contain usage examples")
	}
}

func TestPrintPatternsOnly(t *testing.T) {
	var buffer bytes.Buffer

	// Call the function with buffer
	PrintPatternsOnly(&buffer)

	output := buffer.String()

	// Should contain patterns from all categories
	expectedPatterns := []string{".git", "node_modules", "*.o", "*.log"}
	for _, pattern := range expectedPatterns {
		if !strings.Contains(output, pattern) {
			t.Errorf("Output should contain pattern %s", pattern)
		}
	}

	// Check that we have multiple lines (one per pattern)
	lines := strings.Split(strings.TrimSpace(output), "\n")
	if len(lines) < 10 { // Should have many patterns
		t.Errorf("Expected many patterns, got %d lines", len(lines))
	}
}

func TestPrintCategoryExclusions(t *testing.T) {
	tests := []struct {
		name          string
		categoryID    string
		shouldContain []string
		shouldError   bool
	}{
		{
			name:          "valid category - vcs",
			categoryID:    "vcs",
			shouldContain: []string{".git", ".svn"},
			shouldError:   false,
		},
		{
			name:          "valid category - deps",
			categoryID:    "deps",
			shouldContain: []string{"node_modules", "vendor"},
			shouldError:   false,
		},
		{
			name:          "invalid category",
			categoryID:    "nonexistent",
			shouldContain: nil,
			shouldError:   true,
		},
	}

	for _, tt := range tests {
		t.Run(tt.name, func(t *testing.T) {
			var buffer bytes.Buffer

			// Call the function with buffer
			PrintCategoryExclusions(&buffer, tt.categoryID)

			output := buffer.String()

			if tt.shouldError {
				if !strings.Contains(output, "not found") {
					t.Error("Expected error message for invalid category")
				}
				if !strings.Contains(output, tt.categoryID) {
					t.Errorf("Error output should mention category %s", tt.categoryID)
				}
			} else {
				// Check that expected patterns are in output
				for _, pattern := range tt.shouldContain {
					if !strings.Contains(output, pattern) {
						t.Errorf("Output should contain pattern %s", pattern)
					}
				}
			}
		})
	}
}
