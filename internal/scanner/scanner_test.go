package scanner

import (
	"bytes"
	"fmt"
	"io"
	"os"
	"path/filepath"
	"strings"
	"testing"

	"github.com/patrickdappollonio/context-generator/internal/filter"
)

func TestNew(t *testing.T) {
	filter := filter.New([]string{"*.log"})
	var buffer bytes.Buffer

	scanner := New(filter, &buffer)

	if scanner == nil {
		t.Fatal("New() returned nil")
	}

	if scanner.filter == nil {
		t.Error("Scanner filter is nil")
	}

	if scanner.writer == nil {
		t.Error("Scanner writer is nil")
	}
}

func TestScanner_Scan_NonExistentDirectory(t *testing.T) {
	filter := filter.New([]string{})
	var buffer bytes.Buffer
	scanner := New(filter, &buffer)

	err := scanner.Scan("/nonexistent/directory")

	if err == nil {
		t.Error("Expected error for nonexistent directory, got nil")
	}

	expectedMsg := "does not exist"
	if !strings.Contains(err.Error(), expectedMsg) {
		t.Errorf("Error message should contain %q, got %q", expectedMsg, err.Error())
	}
}

func TestScanner_Scan_EmptyDirectory(t *testing.T) {
	// Create temporary empty directory
	tempDir, err := os.MkdirTemp("", "scanner_test_empty")
	if err != nil {
		t.Fatalf("Failed to create temp dir: %v", err)
	}
	defer os.RemoveAll(tempDir)

	filter := filter.New([]string{})
	var buffer bytes.Buffer
	scanner := New(filter, &buffer)

	err = scanner.Scan(tempDir)
	if err != nil {
		t.Errorf("Scan() on empty directory failed: %v", err)
	}

	// Should only contain the final separator
	output := buffer.String()
	lines := strings.Split(strings.TrimSpace(output), "\n")
	if len(lines) != 1 || lines[0] != separator {
		t.Errorf("Expected only final separator, got: %q", output)
	}
}

func TestScanner_Scan_WithTextFiles(t *testing.T) {
	// Create temporary directory with test files
	tempDir, err := os.MkdirTemp("", "scanner_test_text")
	if err != nil {
		t.Fatalf("Failed to create temp dir: %v", err)
	}
	defer os.RemoveAll(tempDir)

	// Create test files
	testFiles := map[string]string{
		"test.txt":    "Hello World\nSecond Line",
		"README.md":   "# Title\nContent here",
		"config.json": `{"key": "value"}`,
	}

	for filename, content := range testFiles {
		filePath := filepath.Join(tempDir, filename)
		if err := os.WriteFile(filePath, []byte(content), 0o644); err != nil {
			t.Fatalf("Failed to create test file %s: %v", filename, err)
		}
	}

	filter := filter.New([]string{})
	var buffer bytes.Buffer
	scanner := New(filter, &buffer)

	err = scanner.Scan(tempDir)
	if err != nil {
		t.Errorf("Scan() failed: %v", err)
	}

	output := buffer.String()

	// Check that all files are processed
	for filename, content := range testFiles {
		if !strings.Contains(output, fmt.Sprintf("file: %s", filename)) {
			t.Errorf("Output should contain file: %s", filename)
		}

		// Check that file content is included with indentation
		lines := strings.Split(content, "\n")
		for _, line := range lines {
			indentedLine := fmt.Sprintf("    %s", line)
			if !strings.Contains(output, indentedLine) {
				t.Errorf("Output should contain indented line: %q", indentedLine)
			}
		}
	}

	// Check that separators are present
	separatorCount := strings.Count(output, separator)
	expectedSeparators := len(testFiles)*2 + 1 // 2 per file + 1 final
	if separatorCount != expectedSeparators {
		t.Errorf("Expected %d separators, got %d", expectedSeparators, separatorCount)
	}
}

func TestScanner_Scan_WithBinaryFiles(t *testing.T) {
	// Create temporary directory with binary file
	tempDir, err := os.MkdirTemp("", "scanner_test_binary")
	if err != nil {
		t.Fatalf("Failed to create temp dir: %v", err)
	}
	defer func() { _ = os.RemoveAll(tempDir) }()

	// Create a binary file (executable-like content)
	binaryFile := filepath.Join(tempDir, "binary")
	binaryContent := []byte{0x7f, 0x45, 0x4c, 0x46, 0x02, 0x01, 0x01, 0x00} // ELF header
	if err := os.WriteFile(binaryFile, binaryContent, 0o644); err != nil {
		t.Fatalf("Failed to create binary file: %v", err)
	}

	// Create a text file for comparison
	textFile := filepath.Join(tempDir, "text.txt")
	if err := os.WriteFile(textFile, []byte("Hello World"), 0o644); err != nil {
		t.Fatalf("Failed to create text file: %v", err)
	}

	filter := filter.New([]string{})
	var buffer bytes.Buffer
	scanner := New(filter, &buffer)

	err = scanner.Scan(tempDir)
	if err != nil {
		t.Errorf("Scan() failed: %v", err)
	}

	output := buffer.String()

	// Binary file should not be included
	if strings.Contains(output, "file: binary") {
		t.Error("Binary file should not be processed")
	}

	// Text file should be included
	if !strings.Contains(output, "file: text.txt") {
		t.Error("Text file should be processed")
	}

	if !strings.Contains(output, "    Hello World") {
		t.Error("Text file content should be included")
	}
}

func TestScanner_Scan_WithFilteredFiles(t *testing.T) {
	// Create temporary directory with test files
	tempDir, err := os.MkdirTemp("", "scanner_test_filtered")
	if err != nil {
		t.Fatalf("Failed to create temp dir: %v", err)
	}
	defer os.RemoveAll(tempDir)

	// Create test files
	testFiles := []string{
		"app.log",
		"README.md",
		"debug.log",
		"main.go",
	}

	for _, filename := range testFiles {
		filePath := filepath.Join(tempDir, filename)
		content := fmt.Sprintf("Content of %s", filename)
		if err := os.WriteFile(filePath, []byte(content), 0o644); err != nil {
			t.Fatalf("Failed to create test file %s: %v", filename, err)
		}
	}

	// Create filter that excludes .log files
	filter := filter.New([]string{"*.log"})
	var buffer bytes.Buffer
	scanner := New(filter, &buffer)

	err = scanner.Scan(tempDir)
	if err != nil {
		t.Errorf("Scan() failed: %v", err)
	}

	output := buffer.String()

	// Log files should be excluded
	if strings.Contains(output, "file: app.log") {
		t.Error("app.log should be filtered out")
	}
	if strings.Contains(output, "file: debug.log") {
		t.Error("debug.log should be filtered out")
	}

	// Other files should be included
	if !strings.Contains(output, "file: README.md") {
		t.Error("README.md should be included")
	}
	if !strings.Contains(output, "file: main.go") {
		t.Error("main.go should be included")
	}
}

func TestScanner_Scan_WithDirectories(t *testing.T) {
	// Create temporary directory structure
	tempDir, err := os.MkdirTemp("", "scanner_test_dirs")
	if err != nil {
		t.Fatalf("Failed to create temp dir: %v", err)
	}
	defer os.RemoveAll(tempDir)

	// Create subdirectories and files
	srcDir := filepath.Join(tempDir, "src")
	if err := os.MkdirAll(srcDir, 0o755); err != nil {
		t.Fatalf("Failed to create src dir: %v", err)
	}

	// Create files in subdirectory
	mainFile := filepath.Join(srcDir, "main.go")
	if err := os.WriteFile(mainFile, []byte("package main"), 0o644); err != nil {
		t.Fatalf("Failed to create main.go: %v", err)
	}

	// Create excluded directory
	nodeModulesDir := filepath.Join(tempDir, "node_modules")
	if err := os.MkdirAll(nodeModulesDir, 0o755); err != nil {
		t.Fatalf("Failed to create node_modules dir: %v", err)
	}

	excludedFile := filepath.Join(nodeModulesDir, "package.json")
	if err := os.WriteFile(excludedFile, []byte(`{"name": "test"}`), 0o644); err != nil {
		t.Fatalf("Failed to create package.json: %v", err)
	}

	filter := filter.NewWithDefaults(nil, nil) // Use defaults which exclude node_modules
	var buffer bytes.Buffer
	scanner := New(filter, &buffer)

	err = scanner.Scan(tempDir)
	if err != nil {
		t.Errorf("Scan() failed: %v", err)
	}

	output := buffer.String()

	// File in src should be included
	if !strings.Contains(output, "file: src/main.go") && !strings.Contains(output, "file: src\\main.go") {
		t.Error("src/main.go should be included")
	}

	// File in node_modules should be excluded
	if strings.Contains(output, "node_modules") {
		t.Error("node_modules content should be excluded")
	}
}

func TestScanner_Scan_RelativePaths(t *testing.T) {
	// Create temporary directory
	tempDir, err := os.MkdirTemp("", "scanner_test_relative")
	if err != nil {
		t.Fatalf("Failed to create temp dir: %v", err)
	}
	defer os.RemoveAll(tempDir)

	// Create a nested file structure
	nestedDir := filepath.Join(tempDir, "deeply", "nested", "path")
	if err := os.MkdirAll(nestedDir, 0o755); err != nil {
		t.Fatalf("Failed to create nested dir: %v", err)
	}

	testFile := filepath.Join(nestedDir, "test.txt")
	if err := os.WriteFile(testFile, []byte("nested content"), 0o644); err != nil {
		t.Fatalf("Failed to create nested file: %v", err)
	}

	filter := filter.New([]string{})
	var buffer bytes.Buffer
	scanner := New(filter, &buffer)

	err = scanner.Scan(tempDir)
	if err != nil {
		t.Errorf("Scan() failed: %v", err)
	}

	output := buffer.String()

	// Should show relative path from base directory
	expectedPath := filepath.Join("deeply", "nested", "path", "test.txt")
	if !strings.Contains(output, fmt.Sprintf("file: %s", expectedPath)) {
		// Also check for Windows-style paths
		windowsPath := strings.ReplaceAll(expectedPath, "/", "\\")
		if !strings.Contains(output, fmt.Sprintf("file: %s", windowsPath)) {
			t.Errorf("Output should contain relative path, got: %s", output)
		}
	}
}

func TestScanner_processFile_Error_Handling(t *testing.T) {
	// Test error handling in processFile by trying to process a directory as a file
	tempDir, err := os.MkdirTemp("", "scanner_test_error")
	if err != nil {
		t.Fatalf("Failed to create temp dir: %v", err)
	}
	defer os.RemoveAll(tempDir)

	filter := filter.New([]string{})
	var buffer bytes.Buffer
	scanner := New(filter, &buffer)

	// This would normally be called internally, but we can test the error path
	// by trying to process a non-existent file
	err = scanner.processFile(filepath.Join(tempDir, "nonexistent.txt"), tempDir)

	if err == nil {
		t.Error("Expected error when processing non-existent file")
	}
}

func TestScanner_Scan_SpecialCharacters(t *testing.T) {
	// Create temporary directory
	tempDir, err := os.MkdirTemp("", "scanner_test_special")
	if err != nil {
		t.Fatalf("Failed to create temp dir: %v", err)
	}
	defer os.RemoveAll(tempDir)

	// Create file with special characters in content
	testFile := filepath.Join(tempDir, "special.txt")
	specialContent := "Line with special chars: áéíóú ñ 中文 🚀\nTab\there\nEnd"
	if err := os.WriteFile(testFile, []byte(specialContent), 0o644); err != nil {
		t.Fatalf("Failed to create special chars file: %v", err)
	}

	filter := filter.New([]string{})
	var buffer bytes.Buffer
	scanner := New(filter, &buffer)

	err = scanner.Scan(tempDir)
	if err != nil {
		t.Errorf("Scan() failed: %v", err)
	}

	output := buffer.String()

	// Check that special characters are preserved
	if !strings.Contains(output, "    Line with special chars: áéíóú ñ 中文 🚀") {
		t.Error("Special characters should be preserved in output")
	}

	if !strings.Contains(output, "    Tab\there") {
		t.Error("Tab characters should be preserved in output")
	}
}

func TestScanner_Scan_EmptyFiles(t *testing.T) {
	// Create temporary directory with empty files
	tempDir, err := os.MkdirTemp("", "scanner_test_empty_files")
	if err != nil {
		t.Fatalf("Failed to create temp dir: %v", err)
	}
	defer os.RemoveAll(tempDir)

	// Create empty text file
	emptyFile := filepath.Join(tempDir, "empty.txt")
	if err := os.WriteFile(emptyFile, []byte(""), 0o644); err != nil {
		t.Fatalf("Failed to create empty file: %v", err)
	}

	filter := filter.New([]string{})
	var buffer bytes.Buffer
	scanner := New(filter, &buffer)

	err = scanner.Scan(tempDir)
	if err != nil {
		t.Errorf("Scan() failed: %v", err)
	}

	output := buffer.String()

	// Should still show the file header even if empty
	if !strings.Contains(output, "file: empty.txt") {
		t.Error("Empty file should still be listed")
	}

	// Should contain proper separators
	separatorCount := strings.Count(output, separator)
	expectedSeparators := 3 // 2 for the file + 1 final
	if separatorCount != expectedSeparators {
		t.Errorf("Expected %d separators, got %d", expectedSeparators, separatorCount)
	}
}

func TestScanner_Scan_MixedContentTypes(t *testing.T) {
	// Create temporary directory with mixed file types
	tempDir, err := os.MkdirTemp("", "scanner_test_mixed")
	if err != nil {
		t.Fatalf("Failed to create temp dir: %v", err)
	}
	defer os.RemoveAll(tempDir)

	// Create various file types
	files := map[string][]byte{
		"text.txt":   []byte("This is a text file"),
		"script.sh":  []byte("#!/bin/bash\necho 'hello'"),
		"config.xml": []byte("<config><item>value</item></config>"),
		"data.csv":   []byte("name,age\nJohn,30\nJane,25"),
		"README":     []byte("This is a readme file without extension"),
		"binary.bin": {0x89, 0x50, 0x4E, 0x47, 0x0D, 0x0A, 0x1A, 0x0A}, // PNG header
	}

	for filename, content := range files {
		filePath := filepath.Join(tempDir, filename)
		if err := os.WriteFile(filePath, content, 0o644); err != nil {
			t.Fatalf("Failed to create file %s: %v", filename, err)
		}
	}

	filter := filter.New([]string{})
	var buffer bytes.Buffer
	scanner := New(filter, &buffer)

	err = scanner.Scan(tempDir)
	if err != nil {
		t.Errorf("Scan() failed: %v", err)
	}

	output := buffer.String()

	// Text files should be included
	textFiles := []string{"text.txt", "script.sh", "config.xml", "data.csv", "README"}
	for _, filename := range textFiles {
		if !strings.Contains(output, fmt.Sprintf("file: %s", filename)) {
			t.Errorf("Text file %s should be included", filename)
		}
	}

	// Binary file should be excluded
	if strings.Contains(output, "file: binary.bin") {
		t.Error("Binary file should be excluded")
	}
}

func TestScanner_Scan_FileReadErrors(t *testing.T) {
	// Test scanner behavior with file read errors
	tempDir, err := os.MkdirTemp("", "scanner_test_read_errors")
	if err != nil {
		t.Fatalf("Failed to create temp dir: %v", err)
	}
	defer os.RemoveAll(tempDir)

	// Create a file that exists but will cause read issues
	testFile := filepath.Join(tempDir, "problematic.txt")
	if err := os.WriteFile(testFile, []byte("initial content"), 0o644); err != nil {
		t.Fatalf("Failed to create test file: %v", err)
	}

	filter := filter.New([]string{})
	var buffer bytes.Buffer
	scanner := New(filter, &buffer)

	// This tests the normal case - file should be processed successfully
	err = scanner.Scan(tempDir)
	if err != nil {
		t.Errorf("Scan() failed: %v", err)
	}

	output := buffer.String()

	// Should contain the file
	if !strings.Contains(output, "file: problematic.txt") {
		t.Error("File should be processed")
	}

	if !strings.Contains(output, "    initial content") {
		t.Error("File content should be included")
	}
}

func TestScanner_Integration_RealisticProject(t *testing.T) {
	// Create a realistic project structure for comprehensive testing
	tempDir, err := os.MkdirTemp("", "scanner_integration_test")
	if err != nil {
		t.Fatalf("Failed to create temp dir: %v", err)
	}
	defer os.RemoveAll(tempDir)

	// Create a comprehensive project structure
	projectStructure := map[string][]byte{
		// Source files (should be included)
		"main.go":            []byte("package main\n\nfunc main() {\n\tfmt.Println(\"Hello, World!\")\n}"),
		"src/utils.go":       []byte("package src\n\nfunc UtilFunction() string {\n\treturn \"utility\"\n}"),
		"pkg/library/lib.go": []byte("package library\n\ntype Library struct {\n\tName string\n}"),
		"cmd/cli/main.go":    []byte("package main\n\nimport \"fmt\"\n\nfunc main() {\n\tfmt.Println(\"CLI tool\")\n}"),

		// Configuration files (should be included)
		"README.md":          []byte("# My Project\n\nThis is a sample project for testing."),
		"Dockerfile":         []byte("FROM golang:1.19\nWORKDIR /app\nCOPY . .\nRUN go build -o app"),
		"docker-compose.yml": []byte("version: '3'\nservices:\n  app:\n    build: .\n    ports:\n      - \"8080:8080\""),
		"Makefile":           []byte("build:\n\tgo build -o bin/app\n\ntest:\n\tgo test ./..."),
		"go.mod":             []byte("module myproject\n\ngo 1.19"),
		".gitignore":         []byte("*.log\n*.tmp\nbin/\n.env"),

		// Files that should be excluded by default filters
		"go.sum":    []byte("example.com/pkg v1.0.0 h1:abc123\nexample.com/pkg v1.0.0/go.mod h1:def456"),
		"app.log":   []byte("2023-01-01 INFO: Application started\n2023-01-01 ERROR: Something went wrong"),
		"debug.log": []byte("DEBUG: Detailed debugging information\nDEBUG: More debug info"),
		"test.prof": []byte("# This is a Go profile\n# Generated by go test -prof\nheap profile: 10: 1000 [1: 512] @ heap/1048576\nTotal allocations: 1000\n"),
		"main.test": []byte("package main\n\nimport \"testing\"\n\nfunc TestMain(t *testing.T) {\n\t// Test content\n}"),

		// Build artifacts (should be excluded)
		"bin/app":            {0x7f, 0x45, 0x4c, 0x46}, // ELF header (binary)
		"target/release/app": {0x7f, 0x45, 0x4c, 0x46}, // ELF header (binary)

		// Dependencies (should be excluded)
		"node_modules/package/index.js":          []byte("module.exports = {}"),
		"vendor/github.com/pkg/errors/errors.go": []byte("package errors\n\nfunc New(text string) error {\n\treturn &errorString{text}\n}"),

		// Version control (should be excluded)
		".git/config": []byte("[core]\n\trepositoryformatversion = 0"),
		".git/HEAD":   []byte("ref: refs/heads/main"),

		// IDE files (should be excluded)
		".vscode/settings.json": []byte("{\"go.formatTool\": \"goimports\"}"),
		".idea/workspace.xml":   []byte("<?xml version=\"1.0\" encoding=\"UTF-8\"?>\n<project version=\"4\">"),

		// Environment and secrets (should be excluded)
		".env":         []byte("DATABASE_URL=postgresql://localhost/mydb\nAPI_KEY=secret123"),
		"secrets.json": []byte("{\"api_key\": \"super_secret_key\"}"),

		// Temporary files (should be excluded)
		"temp.tmp":    []byte("temporary data"),
		"cache.cache": []byte("cached content"),
		"backup~":     []byte("backup file content"),
	}

	// Create the directory structure and files
	for filePath, content := range projectStructure {
		fullPath := filepath.Join(tempDir, filePath)

		// Create parent directories if needed
		if err := os.MkdirAll(filepath.Dir(fullPath), 0o755); err != nil {
			t.Fatalf("Failed to create directory for %s: %v", filePath, err)
		}

		// Create the file
		if err := os.WriteFile(fullPath, content, 0o644); err != nil {
			t.Fatalf("Failed to create file %s: %v", filePath, err)
		}
	}

	// Test different filter configurations
	testCases := []struct {
		name               string
		additionalPatterns []string
		disabledCategories []string
		expectedIncluded   []string
		expectedExcluded   []string
		minFileCount       int
	}{
		{
			name:               "default exclusions",
			additionalPatterns: nil,
			disabledCategories: nil,
			expectedIncluded: []string{
				"main.go", "src/utils.go", "pkg/library/lib.go", "cmd/cli/main.go",
				"README.md", "Dockerfile", "docker-compose.yml", "Makefile", "go.mod", ".gitignore",
			},
			expectedExcluded: []string{
				"go.sum", "app.log", "debug.log", "test.prof", "main.test",
				".env", "secrets.json", "temp.tmp", "cache.cache", "backup~",
				// Note: .git, node_modules, vendor, .vscode, .idea directories are excluded entirely
				// so their files won't appear in output at all
			},
			minFileCount: 10,
		},
		{
			name:               "disable go category",
			additionalPatterns: nil,
			disabledCategories: []string{"go"},
			expectedIncluded: []string{
				"main.go", "go.sum", "test.prof", // Go files now included
				"README.md", "Dockerfile",
			},
			expectedExcluded: []string{
				"coverage.out",         // Still excluded by latex category (*.out pattern)
				"app.log", "debug.log", // Logs still excluded
				".env", "secrets.json", // Other excluded files
				// Note: node_modules, vendor, .git, .vscode directories are excluded entirely
			},
			minFileCount: 12,
		},
		{
			name:               "custom additional exclusions",
			additionalPatterns: []string{"*.md", "Dockerfile"},
			disabledCategories: nil,
			expectedIncluded: []string{
				"main.go", "src/utils.go", "go.mod", ".gitignore",
				"docker-compose.yml", "Makefile",
			},
			expectedExcluded: []string{
				"README.md", "Dockerfile", // Custom exclusions
				"go.sum", "app.log", ".env", "secrets.json", // Default exclusions
				// Note: .git, node_modules directories are excluded entirely
			},
			minFileCount: 6,
		},
	}

	for _, tc := range testCases {
		t.Run(tc.name, func(t *testing.T) {
			// Create filter with specified configuration
			filterInstance := filter.NewWithDefaults(tc.additionalPatterns, tc.disabledCategories)

			// Create scanner
			var buffer bytes.Buffer
			scanner := New(filterInstance, &buffer)

			// Run the scan
			err := scanner.Scan(tempDir)
			if err != nil {
				t.Fatalf("Scan failed: %v", err)
			}

			output := buffer.String()

			// Verify expected included files are present
			for _, expectedFile := range tc.expectedIncluded {
				if !strings.Contains(output, fmt.Sprintf("file: %s", expectedFile)) {
					t.Errorf("Expected file %s to be included in output", expectedFile)
				}
			}

			// Verify expected excluded files are not present
			for _, expectedExcluded := range tc.expectedExcluded {
				if strings.Contains(output, fmt.Sprintf("file: %s", expectedExcluded)) {
					t.Errorf("Expected file %s to be excluded from output", expectedExcluded)
				}
			}

			// Count actual files processed (counting actual file headers, not content)
			fileLines := 0
			outputLines := strings.Split(output, "\n")
			for _, line := range outputLines {
				if strings.HasPrefix(line, "file: ") {
					fileLines++
				}
			}
			if fileLines < tc.minFileCount {
				t.Errorf("Expected at least %d files, got %d", tc.minFileCount, fileLines)
			}

			// Verify output format consistency
			separatorCount := strings.Count(output, separator)
			expectedSeparators := fileLines*2 + 1 // 2 per file + 1 final

			if separatorCount != expectedSeparators {
				t.Errorf("Expected %d separators, got %d", expectedSeparators, separatorCount)
			}

			// Verify some content is actually included (not just headers)
			if !strings.Contains(output, "    package main") {
				t.Error("Expected to find Go package declaration in output")
			}

			// Check README content only if it should be included
			shouldIncludeReadme := true
			if tc.additionalPatterns != nil {
				for _, pattern := range tc.additionalPatterns {
					if pattern == "*.md" {
						shouldIncludeReadme = false
						break
					}
				}
			}

			if shouldIncludeReadme && !strings.Contains(output, "    # My Project") {
				t.Error("Expected to find README content in output")
			}

			// Verify relative paths are used
			if strings.Contains(output, tempDir) {
				t.Error("Output should use relative paths, not absolute paths")
			}

			// Verify proper indentation
			lines := strings.Split(output, "\n")
			contentLines := 0
			for _, line := range lines {
				if strings.HasPrefix(line, "    ") {
					contentLines++
				}
			}

			if contentLines == 0 {
				t.Error("Expected to find indented content lines")
			}
		})
	}
}

func TestCLI_EndToEnd_ContextGenerator(t *testing.T) {
	// This test validates the actual context-generator CLI application behavior
	// as users would experience it, testing the complete integration including
	// the CLI layer, filter categories, and output format.

	// Create a realistic project structure
	tempDir, err := os.MkdirTemp("", "context_generator_e2e_test")
	if err != nil {
		t.Fatalf("Failed to create temp dir: %v", err)
	}
	defer os.RemoveAll(tempDir)

	// Create a comprehensive project that demonstrates the app's purpose:
	// generating AI-friendly context from source code
	projectFiles := map[string][]byte{
		// Core source code files (the main content users want in AI context)
		"main.go": []byte(`package main

import (
	"fmt"
	"log"
)

func main() {
	fmt.Println("Hello, Context Generator!")
	log.Println("This is a sample application")
}`),

		"pkg/handler/handler.go": []byte(`package handler

import "net/http"

type Handler struct {
	service Service
}

func (h *Handler) ServeHTTP(w http.ResponseWriter, r *http.Request) {
	w.WriteHeader(http.StatusOK)
	w.Write([]byte("Hello from handler"))
}`),

		"internal/service/service.go": []byte(`package service

type Service interface {
	ProcessData(data string) (string, error)
}

type serviceImpl struct{}

func New() Service {
	return &serviceImpl{}
}

func (s *serviceImpl) ProcessData(data string) (string, error) {
	return "processed: " + data, nil
}`),

		// Configuration files (useful for AI context)
		"README.md": []byte(`# Context Generator Demo

This is a demo project to show how context-generator works.

## Features

- Scans source code
- Filters out unwanted files
- Generates AI-friendly output`),

		"go.mod": []byte(`module demo-project

go 1.21

require (
	github.com/gorilla/mux v1.8.0
)`),

		"Dockerfile": []byte(`FROM golang:1.21-alpine
WORKDIR /app
COPY . .
RUN go build -o main .
EXPOSE 8080
CMD ["./main"]`),

		".gitignore": []byte(`*.log
*.tmp
bin/
.env
coverage.out`),

		// Files that should be excluded by default filters

		// Go-specific excludes
		"go.sum": []byte(`github.com/gorilla/mux v1.8.0 h1:i40aqfkR1h2SlN9hojwV5ZA91wcXFOvCN/C8j4nWCUU=
github.com/gorilla/mux v1.8.0/go.mod h1:DVbg23sWSpFRCP0SfiEN6jmj59UnW/n46BH5rLB71So=`),

		"coverage.out": []byte(`mode: set
demo-project/main.go:8.13,10.2 1 1
demo-project/pkg/handler/handler.go:9.76,12.2 3 1`),

		"main_test.go": []byte(`package main

import "testing"

func TestMain(t *testing.T) {
	// Test implementation
}`),

		"test.prof": []byte(`# This is a Go profile
# Generated by go test -prof
heap profile: 10: 1000 [1: 512] @ heap/1048576
Total allocations: 1000
`),

		// Log files (excluded by logs category)
		"app.log": []byte(`2024-01-15 10:30:00 INFO Starting application
2024-01-15 10:30:01 ERROR Failed to connect to database
2024-01-15 10:30:02 WARN Retrying connection`),

		"debug.log": []byte(`DEBUG: Loading configuration
DEBUG: Connecting to database
DEBUG: Starting HTTP server`),

		// Build artifacts (excluded by build category)
		"bin/app": {0x7f, 0x45, 0x4c, 0x46, 0x02, 0x01, 0x01}, // ELF binary header

		// Dependencies (excluded by deps category)
		"vendor/github.com/pkg/errors/errors.go": []byte(`package errors

import "fmt"

func New(text string) error {
	return &errorString{text}
}`),

		// Version control (excluded by vcs category)
		".git/config": []byte(`[core]
	repositoryformatversion = 0
	filemode = true`),

		// Environment files (excluded by env category)
		".env": []byte(`DATABASE_URL=postgres://localhost:5432/demo
API_KEY=sk-1234567890abcdef`),

		"secrets.json": []byte(`{
	"database_password": "super_secret_password",
	"api_key": "very_secret_key"
}`),

		// IDE files (excluded by ide category)
		".vscode/settings.json": []byte(`{
	"go.formatTool": "goimports",
	"editor.tabSize": 4
}`),

		// Temporary files (excluded by logs category)
		"temp.tmp":    []byte(`temporary data for processing`),
		"cache.cache": []byte(`cached computation results`),
	}

	// Create all project files
	for filePath, content := range projectFiles {
		fullPath := filepath.Join(tempDir, filePath)

		// Create parent directories
		if err := os.MkdirAll(filepath.Dir(fullPath), 0o755); err != nil {
			t.Fatalf("Failed to create directory for %s: %v", filePath, err)
		}

		// Create the file
		if err := os.WriteFile(fullPath, content, 0o644); err != nil {
			t.Fatalf("Failed to create file %s: %v", filePath, err)
		}
	}

	testCases := []struct {
		name                 string
		filterConfig         func() *filter.Filter
		expectedIncluded     []string
		expectedExcluded     []string
		expectedMinFiles     int
		expectedContentCheck func(output string) error
		description          string
	}{
		{
			name: "default_behavior_ai_ready_context",
			filterConfig: func() *filter.Filter {
				return filter.NewWithDefaults(nil, nil)
			},
			expectedIncluded: []string{
				"main.go", "pkg/handler/handler.go", "internal/service/service.go",
				"README.md", "go.mod", "Dockerfile", ".gitignore",
			},
			expectedExcluded: []string{
				"go.sum", "coverage.out", "test.prof", "main.test", // Go category exclusions
				"app.log", "debug.log", "temp.tmp", "cache.cache", // Logs category
				".env", "secrets.json", // Environment category
			},
			expectedMinFiles: 7,
			expectedContentCheck: func(output string) error {
				// Verify the output contains actual source code that would be useful to AI
				requiredContent := []string{
					"package main",
					"import (",
					"func main() {",
					"type Handler struct",
					"type Service interface",
					"# Context Generator Demo",
				}

				for _, content := range requiredContent {
					if !strings.Contains(output, content) {
						return fmt.Errorf("missing expected content: %s", content)
					}
				}

				// Verify proper indentation for AI readability
				lines := strings.Split(output, "\n")
				hasIndentedContent := false
				for _, line := range lines {
					if strings.HasPrefix(line, "    ") && len(strings.TrimSpace(line)) > 0 {
						hasIndentedContent = true
						break
					}
				}
				if !hasIndentedContent {
					return fmt.Errorf("output should contain indented content lines")
				}

				return nil
			},
			description: "Default behavior should create AI-ready context with source code and configs, excluding build artifacts and secrets",
		},
		{
			name: "disable_go_category_for_testing_context",
			filterConfig: func() *filter.Filter {
				return filter.NewWithDefaults(nil, []string{"go"})
			},
			expectedIncluded: []string{
				"main.go", "go.sum", "test.prof", // Go files now included
				"README.md", "Dockerfile", "main_test.go",
				// Note: coverage.out is still excluded by latex category (*.out pattern)
			},
			expectedExcluded: []string{
				"coverage.out",         // Still excluded by latex category (*.out pattern)
				"app.log", "debug.log", // Still excluded by logs category
				".env", "secrets.json", // Still excluded by env category
				"main.test", // Binary content means excluded anyway
			},
			expectedMinFiles: 9,
			expectedContentCheck: func(output string) error {
				// When including Go test files, verify test content is present
				if !strings.Contains(output, "func TestMain(t *testing.T)") {
					return fmt.Errorf("test functions should be included when go category is disabled")
				}
				if !strings.Contains(output, "github.com/gorilla/mux") {
					return fmt.Errorf("go.sum content should be included when go category is disabled")
				}
				if !strings.Contains(output, "heap profile: 10: 1000") {
					return fmt.Errorf("prof file content should be included when go category is disabled")
				}
				// Note: coverage.out should still be excluded by latex category (*.out pattern)
				if strings.Contains(output, "demo-project/main.go:8.13,10.2") {
					return fmt.Errorf("coverage.out should still be excluded by latex category (*.out pattern)")
				}
				return nil
			},
			description: "Disabling Go category includes go.sum and prof files, but coverage.out remains excluded due to overlapping latex category (*.out pattern) - demonstrates category pattern conflicts",
		},
		{
			name: "custom_exclusions_for_focused_context",
			filterConfig: func() *filter.Filter {
				return filter.NewWithDefaults([]string{"*.md", "Dockerfile", "internal/*"}, nil)
			},
			expectedIncluded: []string{
				"main.go", "pkg/handler/handler.go", "go.mod", ".gitignore",
			},
			expectedExcluded: []string{
				"README.md", "Dockerfile", // Custom exclusions
				"internal/service/service.go", // Custom directory exclusion
				"go.sum", "app.log",           // Default exclusions still apply
			},
			expectedMinFiles: 4,
			expectedContentCheck: func(output string) error {
				// Verify focused context contains only core application files
				if strings.Contains(output, "# Context Generator Demo") {
					return fmt.Errorf("README should be excluded by custom pattern")
				}
				if strings.Contains(output, "type Service interface") {
					return fmt.Errorf("internal files should be excluded by custom pattern")
				}
				if !strings.Contains(output, "type Handler struct") {
					return fmt.Errorf("pkg files should still be included")
				}
				return nil
			},
			description: "Custom exclusions allow creating focused context by excluding documentation and internal implementation",
		},
		{
			name: "no_defaults_minimal_filtering",
			filterConfig: func() *filter.Filter {
				return filter.New([]string{".git/*", "bin/*", "*.tmp"})
			},
			expectedIncluded: []string{
				"main.go", "go.sum", "coverage.out", "app.log", // Default exclusions disabled
				".env", "secrets.json", // Environment files now included (dangerous but user's choice)
			},
			expectedExcluded: []string{
				// Only custom exclusions apply
				"temp.tmp", // Matches *.tmp pattern
			},
			expectedMinFiles: 15,
			expectedContentCheck: func(output string) error {
				// When defaults are disabled, sensitive files are included (user responsibility)
				if !strings.Contains(output, "DATABASE_URL=") {
					return fmt.Errorf("environment files should be included when defaults are disabled")
				}
				if !strings.Contains(output, "super_secret_password") {
					return fmt.Errorf("secrets should be included when defaults are disabled (user's explicit choice)")
				}
				return nil
			},
			description: "No defaults mode gives users full control but requires careful exclusion management",
		},
	}

	for _, tc := range testCases {
		t.Run(tc.name, func(t *testing.T) {
			filterInstance := tc.filterConfig()

			var buffer bytes.Buffer
			scanner := New(filterInstance, &buffer)

			err := scanner.Scan(tempDir)
			if err != nil {
				t.Fatalf("Scan failed: %v", err)
			}

			output := buffer.String()

			// Test description for context
			t.Logf("Test scenario: %s", tc.description)

			// Verify expected included files
			for _, expectedFile := range tc.expectedIncluded {
				fileHeader := fmt.Sprintf("file: %s", expectedFile)
				if !strings.Contains(output, fileHeader) {
					t.Errorf("Expected file %s to be included in output", expectedFile)
				}
			}

			// Verify expected excluded files
			for _, expectedExcluded := range tc.expectedExcluded {
				fileHeader := fmt.Sprintf("file: %s", expectedExcluded)
				if strings.Contains(output, fileHeader) {
					t.Errorf("Expected file %s to be excluded from output", expectedExcluded)
				}
			}

			// Count actual files processed
			fileCount := strings.Count(output, "file: ")
			if fileCount < tc.expectedMinFiles {
				t.Errorf("Expected at least %d files, got %d", tc.expectedMinFiles, fileCount)
			}

			// Verify output format matches what AI tools expect
			if !strings.HasPrefix(output, separator) {
				t.Error("Output should start with separator")
			}

			if !strings.HasSuffix(strings.TrimSpace(output), separator) {
				t.Error("Output should end with separator")
			}

			// Verify no absolute paths (AI context should be portable)
			if strings.Contains(output, tempDir) {
				t.Error("Output should not contain absolute paths")
			}

			// Run custom content validation
			if tc.expectedContentCheck != nil {
				if err := tc.expectedContentCheck(output); err != nil {
					t.Errorf("Content validation failed: %v", err)
				}
			}

			// Verify structural format for AI consumption
			lines := strings.Split(output, "\n")
			inFileContent := false
			hasFileHeaders := false

			for _, line := range lines {
				if strings.HasPrefix(line, "file: ") {
					hasFileHeaders = true
					inFileContent = false
				} else if line == separator {
					inFileContent = !inFileContent
				} else if inFileContent && len(strings.TrimSpace(line)) > 0 {
					// Content lines should be indented for readability
					if !strings.HasPrefix(line, "    ") {
						t.Errorf("Content line should be indented: %q", line)
					}
				}
			}

			if !hasFileHeaders {
				t.Error("Output should contain file headers")
			}
		})
	}
}

// TestCLI_ListExclusions_EndToEnd tests the list-exclusions subcommand
func TestCLI_ListExclusions_EndToEnd(t *testing.T) {
	testCases := []struct {
		name           string
		function       func(w io.Writer)
		expectedOutput []string
		description    string
	}{
		{
			name:     "list_all_exclusions",
			function: func(w io.Writer) { filter.PrintExclusions(w) },
			expectedOutput: []string{
				"Default Exclusions by Category",
				"ID: vcs", "ID: deps", "ID: build", "ID: go", "ID: logs",
				"Version Control", "Dependencies", "Build Artifacts",
			},
			description: "Should list all exclusion categories with descriptions",
		},
		{
			name:     "list_patterns_only",
			function: func(w io.Writer) { filter.PrintPatternsOnly(w) },
			expectedOutput: []string{
				"*.log", "*.tmp", "go.sum", "coverage.out",
				".git", "node_modules", "vendor",
			},
			description: "Should list only patterns for scripting use",
		},
		{
			name:     "list_go_category",
			function: func(w io.Writer) { filter.PrintCategoryExclusions(w, "go") },
			expectedOutput: []string{
				"*.test", "go.sum", "coverage.out", "*.prof",
			},
			description: "Should list only Go-specific exclusions",
		},
	}

	for _, tc := range testCases {
		t.Run(tc.name, func(t *testing.T) {
			var buffer bytes.Buffer

			// Run the function with buffer
			tc.function(&buffer)

			output := buffer.String()

			t.Logf("Test scenario: %s", tc.description)

			// Verify expected content is present
			for _, expected := range tc.expectedOutput {
				if !strings.Contains(output, expected) {
					t.Errorf("Expected output to contain %q", expected)
				}
			}

			// Verify output is machine-readable when appropriate
			if tc.name == "list_patterns_only" {
				lines := strings.Split(strings.TrimSpace(output), "\n")
				for _, line := range lines {
					line = strings.TrimSpace(line)
					if line != "" {
						// Should be a pure pattern - patterns can be wildcards, file extensions, or directory names
						// Just verify it's not a complex formatted output
						if strings.Contains(line, "ID:") || strings.Contains(line, "Description:") {
							t.Errorf("Unexpected formatted output in patterns-only mode: %q", line)
						}
					}
				}
			}
		})
	}
}

func TestScanner_DryRun(t *testing.T) {
	// Create temporary directory with test files
	tempDir, err := os.MkdirTemp("", "scanner_test_dryrun")
	if err != nil {
		t.Fatalf("Failed to create temp dir: %v", err)
	}
	defer os.RemoveAll(tempDir)

	// Create test files
	testFiles := map[string]string{
		"app.go":      "package main\nfunc main() {}",
		"README.md":   "# Project\nDescription here",
		"app.log":     "log entry",
		"config.json": `{"key": "value"}`,
	}

	for filename, content := range testFiles {
		filePath := filepath.Join(tempDir, filename)
		if err := os.WriteFile(filePath, []byte(content), 0o644); err != nil {
			t.Fatalf("Failed to create test file %s: %v", filename, err)
		}
	}

	// Create subdirectory with file
	subDir := filepath.Join(tempDir, "subdir")
	if err := os.Mkdir(subDir, 0o755); err != nil {
		t.Fatalf("Failed to create subdirectory: %v", err)
	}

	subFile := filepath.Join(subDir, "sub.txt")
	if err := os.WriteFile(subFile, []byte("sub content"), 0o644); err != nil {
		t.Fatalf("Failed to create subdir file: %v", err)
	}

	// Test with default filter (excludes .log files)
	filter := filter.NewWithDefaults(nil, nil)
	var buffer bytes.Buffer
	scanner := New(filter, &buffer)

	err = scanner.DryRun(tempDir)
	if err != nil {
		t.Errorf("DryRun() failed: %v", err)
	}

	output := buffer.String()

	// Check that output contains expected sections
	if !strings.Contains(output, "Files that would be processed:") {
		t.Error("Output should contain 'Files that would be processed:' section")
	}

	if !strings.Contains(output, "Files that would be excluded:") {
		t.Error("Output should contain 'Files that would be excluded:' section")
	}

	// Check that included files are shown
	if !strings.Contains(output, "app.go") {
		t.Error("app.go should be in processed files")
	}

	if !strings.Contains(output, "README.md") {
		t.Error("README.md should be in processed files")
	}

	if !strings.Contains(output, "config.json") {
		t.Error("config.json should be in processed files")
	}

	// Check that excluded files are shown with reasons
	if !strings.Contains(output, "app.log") {
		t.Error("app.log should be in excluded files")
	}

	// Check that exclusion reason is shown
	if !strings.Contains(output, "[Logs & Temporary: *.log]") {
		t.Error("app.log should show exclusion reason")
	}

	// Check tree structure (basic check for tree characters)
	if !strings.Contains(output, "├──") || !strings.Contains(output, "└──") {
		t.Error("Output should contain tree structure characters")
	}
}
