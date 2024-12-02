package main

import (
	"bufio"
	"fmt"
	"io"
	"net/http"
	"os"
	"path/filepath"
	"strings"

	"github.com/spf13/cobra"
)

const separator = "--------------------"

func contains[T comparable](slice []T, value T) bool {
	for _, item := range slice {
		if item == value {
			return true
		}
	}

	return false
}

func run(excludedFolderNames, excludedFileNames []string, w io.Writer) error {
	currentDirectory := "."

	// Fetch a second parameter from the command line
	if len(os.Args) == 2 {
		currentDirectory = os.Args[1]
	}

	// Check if the directory provided exists
	if _, err := os.Stat(currentDirectory); err != nil {
		if os.IsNotExist(err) {
			return fmt.Errorf("directory %q does not exist", currentDirectory)
		}

		return fmt.Errorf("error checking directory %q: %w", currentDirectory, err)
	}

	// Walk through all files and directories starting from the current directory
	err := filepath.Walk(currentDirectory, func(path string, info os.FileInfo, err error) error {
		if err != nil {
			// Return the error to be handled by the caller
			return err
		}

		// Check if the directory should be excluded
		if info.IsDir() && contains(excludedFolderNames, info.Name()) {
			// Skip the directory and its contents
			return filepath.SkipDir
		}

		// Skip files that are in the excludedFileNames list
		if !info.IsDir() && contains(excludedFileNames, info.Name()) {
			return nil
		}

		// Skip directories; process only files
		if info.IsDir() {
			return nil
		}

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
			// Skip binary files
			return nil
		}

		// Write the first line of dashes
		fmt.Fprintln(w, separator)
		// Write the relative file path
		fmt.Fprintln(w, "file:", path)
		// Write the second line of dashes
		fmt.Fprintln(w, separator)

		// Create a scanner to read the file line by line
		scanner := bufio.NewScanner(file)
		for scanner.Scan() {
			// Write each line with 4 spaces indentation
			fmt.Fprintf(w, "    %s\n", scanner.Text())
		}

		// Check for errors during scanning
		if err := scanner.Err(); err != nil {
			return fmt.Errorf("error reading file %q: %w", path, err)
		}

		return nil
	})

	// Write the third line of dashes
	fmt.Fprintln(w, separator)

	// Return any error encountered during the file walk
	return err
}

func getAppName() string {
	n := filepath.Base(os.Args[0])
	return strings.TrimFunc(n, func(r rune) bool { return r == '/' || r == '.' })
}

func getMainCommand() *cobra.Command {
	var excludedFolderNames []string
	var excludedFileNames []string

	cmd := &cobra.Command{
		Use:           getAppName(),
		Short:         fmt.Sprintf("%s allows you to quickly create contexts to be given to GPT-like apps from your source code", getAppName()),
		SilenceErrors: true,
		SilenceUsage:  true,
		RunE: func(cmd *cobra.Command, args []string) error {
			return run(excludedFolderNames, excludedFileNames, os.Stdout)
		},
	}

	cmd.Flags().StringSliceVar(&excludedFolderNames, "exclude-folder", []string{".git", "node_modules"}, "exclude folders with these names")
	cmd.Flags().StringSliceVar(&excludedFileNames, "exclude-file", []string{".DS_Store"}, "exclude files with these names")

	return cmd
}
