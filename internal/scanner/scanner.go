package scanner

import (
	"bufio"
	"fmt"
	"io"
	"net/http"
	"os"
	"path/filepath"
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
