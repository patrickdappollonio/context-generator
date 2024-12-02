package main

import (
	"fmt"
	"os"
)

func main() {
	// Call the run function and handle any errors
	if err := getMainCommand().Execute(); err != nil {
		// Print the error to stderr and exit with code 1
		fmt.Fprintln(os.Stderr, "Error:", err)
		os.Exit(1)
	}
}
