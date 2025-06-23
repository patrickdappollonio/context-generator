package main

import (
	"fmt"
	"os"

	"github.com/patrickdappollonio/context-generator/cmd"
)

func main() {
	// Call the root command and handle any errors
	if err := cmd.NewRootCommand().Execute(); err != nil {
		// Print the error to stderr and exit with code 1
		fmt.Fprintln(os.Stderr, "Error:", err)
		os.Exit(1)
	}
}
