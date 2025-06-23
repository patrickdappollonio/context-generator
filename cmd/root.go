package cmd

import (
	"fmt"
	"os"
	"path/filepath"
	"strings"

	"github.com/patrickdappollonio/context-generator/internal/filter"
	"github.com/patrickdappollonio/context-generator/internal/scanner"
	"github.com/spf13/cobra"
)

// getAppName returns the application name from the executable
func getAppName() string {
	n := filepath.Base(os.Args[0])
	return strings.TrimFunc(n, func(r rune) bool { return r == '/' || r == '.' })
}

// NewRootCommand creates and returns the root cobra command
func NewRootCommand() *cobra.Command {
	return NewRootCommandWithVersion("dev")
}

// NewRootCommandWithVersion creates and returns the root cobra command with a specific version
func NewRootCommandWithVersion(version string) *cobra.Command {
	var exclude []string
	var disableCategories []string
	var noDefaults bool

	cmd := &cobra.Command{
		Use:           fmt.Sprintf("%s [directory]", getAppName()),
		Short:         fmt.Sprintf("%s allows you to quickly create contexts to be given to GPT-like apps from your source code", getAppName()),
		Version:       version,
		Args:          cobra.MaximumNArgs(1),
		SilenceErrors: true,
		SilenceUsage:  true,
		RunE: func(cmd *cobra.Command, args []string) error {
			// Determine the directory to scan (default to current directory)
			directory := "."
			if len(args) > 0 {
				directory = args[0]
			}

			// Validate disabled category IDs
			if len(disableCategories) > 0 {
				invalid := filter.ValidateCategoryIDs(disableCategories)
				if len(invalid) > 0 {
					return fmt.Errorf("invalid category IDs: %s. Use 'list-exclusions' to see valid IDs", strings.Join(invalid, ", "))
				}
			}

			// Create the appropriate filter
			var f *filter.Filter
			if noDefaults {
				f = filter.New(exclude)
			} else {
				f = filter.NewWithDefaults(exclude, disableCategories)
			}

			// Create scanner and run
			s := scanner.New(f, os.Stdout)
			return s.Scan(directory)
		},
	}

	cmd.Flags().StringSliceVar(&exclude, "exclude", nil, "exclude files/folders matching these patterns (supports wildcards)")
	cmd.Flags().StringSliceVar(&disableCategories, "disable-category", nil, "disable default exclusion categories by ID (use list-exclusions to see IDs)")
	cmd.Flags().BoolVar(&noDefaults, "no-defaults", false, "disable default exclusions")

	// Add the list-exclusions subcommand
	listCmd := &cobra.Command{
		Use:   "list-exclusions",
		Short: "List all default exclusions organized by category",
		Run: func(cmd *cobra.Command, args []string) {
			categoryFilter, _ := cmd.Flags().GetString("category")
			patternsOnly, _ := cmd.Flags().GetBool("patterns-only")

			if categoryFilter != "" {
				// Validate the category ID
				invalid := filter.ValidateCategoryIDs([]string{categoryFilter})
				if len(invalid) > 0 {
					fmt.Fprintf(os.Stderr, "Error: invalid category ID: %s. Use 'list-exclusions' to see valid IDs\n", categoryFilter)
					os.Exit(1)
				}
				filter.PrintCategoryExclusions(categoryFilter)
			} else if patternsOnly {
				filter.PrintPatternsOnly()
			} else {
				filter.PrintExclusions()
			}
		},
	}
	listCmd.Flags().String("category", "", "show patterns for a specific category ID only")
	listCmd.Flags().Bool("patterns-only", false, "show only patterns ordered by category (wildcards first, then literals)")
	cmd.AddCommand(listCmd)

	return cmd
}
