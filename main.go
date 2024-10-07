package main

import (
	"bufio"
	"fmt"
	"io"
	"os"
	"path/filepath"
)

const separator = "--------------------"

func run(w io.Writer) error {
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

		// Skip directories; process only files
		if info.IsDir() {
			return nil
		}

		// Write the first line of dashes
		fmt.Fprintln(w, separator)
		// Write the relative file path
		fmt.Fprintln(w, "file:", path)
		// Write the second line of dashes
		fmt.Fprintln(w, separator)

		// Open the file for reading
		file, err := os.Open(path)
		if err != nil {
			// Return the error to be handled by the caller
			return err
		}
		defer file.Close()

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

func main() {
	// Call the run function and handle any errors
	if err := run(os.Stdout); err != nil {
		// Print the error to stderr and exit with code 1
		fmt.Fprintln(os.Stderr, "Error:", err)
		os.Exit(1)
	}
}
