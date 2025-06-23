# Context Generator

A command-line tool that generates copy-pastable context from your source code, perfect for providing to AI assistants like ChatGPT, Claude, or Copilot.

## What It Does

`context-generator` scans your project directory and creates a formatted output containing all your text files' contents. This output is specifically designed to be copy-pasted into AI chat interfaces, giving the AI complete context about your codebase for more accurate assistance.

## Why Use Context Generator?

When working with AI assistants on coding projects, you often need to provide file contents for context. Instead of manually copying and pasting multiple files, Context Generator:

- **Automates the process** - Scans entire directories in seconds
- **Smart filtering** - Excludes binary files, build artifacts, and common junk automatically
- **Highly configurable** - Customize exclusions with wildcards and patterns
- **AI-optimized format** - Outputs in a clear, structured format that AI tools understand perfectly
- **Universal compatibility** - Works with any AI that accepts text input

## Installation

### From Source

```bash
git clone https://github.com/patrickdappollonio/context-generator.git
cd context-generator
go build -o context-generator
```

### Using Go Install

```bash
go install github.com/patrickdappollonio/context-generator@latest
```

## Usage

```bash
# Scan current directory
context-generator

# Scan specific directory
context-generator /path/to/your/project
context-generator ./src

# Add custom exclusions
context-generator --exclude "*.backup" --exclude "temp/*"

# Disable default exclusions and use only custom ones
context-generator --no-defaults --exclude ".git" --exclude "*.log"

# Exclude entire subdirectories
context-generator --exclude "frontend/*" --exclude "legacy/*"

# Disable specific exclusion categories
context-generator --disable-category go      # Include go.sum and Go test files
context-generator --disable-category vcs     # Include .git directory contents
context-generator --disable-category go,logs # Disable multiple categories

# List all default exclusions organized by category
context-generator list-exclusions

# Show patterns for a specific category
context-generator list-exclusions --category go
context-generator list-exclusions --category python
context-generator list-exclusions --category python-ds
context-generator list-exclusions --category typescript
context-generator list-exclusions --category build

# Show only patterns (no headers) ordered by category
context-generator list-exclusions --patterns-only

# Preview files without generating output (dry-run mode)
context-generator --dry-run                               # Show what files would be processed
context-generator --dry-run --exclude "*.md"              # Preview with custom exclusions
context-generator --dry-run --disable-category go         # Preview with disabled categories

# Use with command-line tools for processing exclusions
context-generator list-exclusions | grep -A5 "ID: go"     # Show Go-specific exclusions
context-generator list-exclusions | grep "^\s\s" | wc -l  # Count total patterns
context-generator list-exclusions | grep "\.log"          # Find log-related patterns
```

## Command-Line Options

```
Usage:
  context-generator [directory] [flags]
  context-generator [command]

Available Commands:
  list-exclusions List all default exclusions organized by category

Flags:
      --disable-category strings   disable default exclusion categories by ID
      --dry-run                    show files that would be processed and excluded without generating output
      --exclude strings            exclude files/folders matching these patterns (supports wildcards)
      --no-defaults                disable default exclusions
  -h, --help                       help for context-generator
```

### Dry-Run Mode

The `--dry-run` flag lets you preview what files would be processed and which would be excluded without generating any output. This is useful for:

- **Verifying filters** - Check if your exclusion patterns work as expected
- **Understanding scope** - See exactly what files will be included in the context
- **Debugging exclusions** - Identify why specific files are being filtered out
- **Large projects** - Preview before processing huge codebases

The dry-run output shows:
1. **Files that would be processed** - In a tree structure, marking binary files that would be skipped
2. **Files that would be excluded** - With the specific category and pattern that caused the exclusion

**Example Output:**
```
$ context-generator --dry-run src/

Dry run for directory: src/

Files that would be processed:
  ├── components/
  ├── utils/
  ├── App.tsx
  └── main.ts
  components/
    ├── Button.tsx
    └── Header.tsx
  utils/
    └── helpers.ts

Files that would be excluded:
  ├── build/ [Build Artifacts: build]
  ├── node_modules/ [Dependencies: node_modules]
  └── app.log [Logs & Temporary: *.log]
```

### Wildcard Patterns

Context Generator supports shell-style wildcards in exclusion patterns:

- `*` - Matches any sequence of characters (e.g., `*.log`, `.env*`)
- `?` - Matches any single character
- `[...]` - Matches any character in brackets
- `temp/*` - Everything in the `temp` directory

## Exclusion Categories

Context Generator organizes exclusions into 20 categories with unique IDs for easy management:

| ID           | Category             | Examples                              |
| ------------ | -------------------- | ------------------------------------- |
| `vcs`        | Version Control      | `.git`, `.svn`, `.hg`                 |
| `deps`       | Dependencies         | `node_modules`, `vendor`              |
| `build`      | Build Artifacts      | `target`, `build`, `dist`, `*.exe`    |
| `go`         | Go Specific          | `go.sum`, `*.test`, `coverage.out`    |
| `js`         | JavaScript/Node.js   | `*.min.js`, `.nyc_output`             |
| `python`     | Python               | `__pycache__`, `*.pyc`, `.venv`       |
| `python-ds`  | Python Data Science  | `.ipynb_checkpoints`, `*.pkl`, `*.h5` |
| `typescript` | TypeScript           | `*.tsbuildinfo`, `*.d.ts.map`         |
| `php`        | PHP                  | `composer.lock`, `*.phar`             |
| `java`       | Java                 | `*.class`, `*.jar`, `.gradle`         |
| `c`          | C/C++                | `*.o`, `*.a`, `*.lib`                 |
| `rust`       | Rust                 | `Cargo.lock`                          |
| `ruby`       | Ruby                 | `Gemfile.lock`, `*.gem`               |
| `swift`      | Swift                | `*.xcworkspace`, `DerivedData`        |
| `kotlin`     | Kotlin               | `*.kt~`, `.kotlin`                    |
| `latex`      | LaTeX                | `*.aux`, `*.log`, `*.synctex.gz`      |
| `logs`       | Logs & Temporary     | `*.log`, `*.tmp`, `*.cache`           |
| `env`        | Environment & Config | `.env*`, `*.pem`, `*.key`             |
| `ide`        | IDE & Editors        | `.vscode`, `.idea`, `.DS_Store`       |
| `docs`       | Documentation        | `_site`, `docs/_build`                |

### Language Support

Context Generator provides comprehensive exclusion patterns for popular programming languages and frameworks:

**Core Languages**: Go, JavaScript/Node.js, TypeScript, Python, Java, C/C++, Rust, Ruby, Swift, Kotlin, PHP
**Specialized**: Python Data Science (Jupyter, ML models, datasets), LaTeX (document preparation)
**Frameworks & Tools**: Build systems, package managers, testing frameworks, linters, formatters

The tool automatically excludes language-specific build artifacts, dependency caches, and temporary files while preserving your source code.

### Viewing Exclusions

The `list-exclusions` command shows all categories and patterns in a clean, line-by-line format that's perfect for command-line processing:

```bash
# View all exclusions
context-generator list-exclusions

# Show patterns for a specific category (recommended)
context-generator list-exclusions --category go
context-generator list-exclusions --category python
context-generator list-exclusions --category python-ds
context-generator list-exclusions --category typescript
context-generator list-exclusions --category build

# Show only patterns without headers (great for scripting)
context-generator list-exclusions --patterns-only

# Extract specific category (alternative using grep)
context-generator list-exclusions | grep -A10 "ID: python"

# Count patterns in a category
context-generator list-exclusions --category build | wc -l

# Count all patterns
context-generator list-exclusions --patterns-only | wc -l

# Find all patterns matching a specific type
context-generator list-exclusions | grep "^\s\s.*\.log"

# Get just the pattern list (useful for scripting)
context-generator list-exclusions | grep "^\s\s"

# Save patterns for a category to file
context-generator list-exclusions --category go > go-exclusions.txt

# Save all patterns to file (clean format)
context-generator list-exclusions --patterns-only > all-patterns.txt
```

**Pattern Ordering**: The `--patterns-only` flag outputs patterns grouped by category, with wildcards (like `*.log`, `*.tmp`) listed before literal values (like `.git`, `node_modules`) within each category.

### Selective Category Disabling

Sometimes you want to include files that are normally excluded. Use `--disable-category` to selectively disable specific exclusion categories:

```bash
# Include Go-specific files (go.sum, test files, etc.)
context-generator --disable-category go

# Include version control files (.git directory contents)
context-generator --disable-category vcs

# Include both Go files and logs
context-generator --disable-category go,logs

# Include data science files (Jupyter notebooks, model files, etc.)
context-generator --disable-category python-ds

# Include TypeScript build files
context-generator --disable-category typescript

# Include build artifacts for debugging
context-generator --disable-category build
```

This gives you fine-grained control without having to disable all defaults or manually specify many exclusions.

## Output Format

The tool generates output in this format:

```
--------------------
file: src/main.go
--------------------
    package main

    import "fmt"

    func main() {
        fmt.Println("Hello, World!")
    }
--------------------
file: src/utils.go
--------------------
    package main

    func helper() string {
        return "helper function"
    }
--------------------
```

This format makes it easy for AI assistants to identify individual files, understand the project structure, and reference specific files in their responses.

## Use Cases

Perfect for various AI-assisted development scenarios:

- **Code Review**: Get comprehensive feedback on your entire codebase
- **Documentation**: Generate API docs or explanations for complex code
- **Debugging**: Provide full context when asking for help with errors
- **Code Explanation**: Help others understand how your project works
- **Refactoring**: Get suggestions for improving code structure across multiple files

## Tips for AI Interaction

1. **Preview First**: Use `--dry-run` to verify you're including the right files before generating context
2. **Be Specific**: After pasting the context, ask specific questions about the code
3. **Mention File Names**: Reference specific files when asking questions
4. **Update Context**: Re-run the tool when your code changes significantly
5. **Size Awareness**: Very large codebases might hit AI token limits - use exclusions to focus on relevant parts
6. **Use Categories**: Disable specific categories (like `logs` or `build`) to focus on source code

## Command-Line Integration

The tool is designed to work well with standard Unix command-line tools:

```bash
# Preview before generating context
context-generator --dry-run
context-generator --dry-run --exclude "*.test.js" --disable-category logs

# Save context to file
context-generator > project-context.txt

# Save patterns for a specific category
context-generator list-exclusions --category python > python-patterns.txt

# Save all patterns in clean format
context-generator list-exclusions --patterns-only > all-exclusions.txt

# Filter exclusions and save to file
context-generator list-exclusions | grep "^\s\s" > patterns.txt

# Count patterns in a category
context-generator list-exclusions --category build | wc -l

# Count all patterns
context-generator list-exclusions --patterns-only | wc -l

# Count files that would be included
context-generator --no-defaults | grep "^file:" | wc -l

# Combine with other tools
context-generator | grep -A5 "file: main.go"

# Use category patterns in shell scripts
for pattern in $(context-generator list-exclusions --category logs); do
  echo "Would exclude: $pattern"
done

# Create .gitignore from patterns
echo "# Generated exclusions" > .gitignore
context-generator list-exclusions --patterns-only >> .gitignore
```

## Contributing

Contributions are welcome! The exclusion categories are organized in `internal/filter/filter.go` making it easy to add new patterns for additional languages or tools.

---

**Pro Tip**: Use `context-generator list-exclusions --category <id>` to quickly see patterns for a specific category, or `context-generator list-exclusions | grep "^\s\s"` to get a clean list of all patterns for scripting!

## Project Structure

```
context-generator/
├── cmd/
│   └── root.go          # CLI command setup
├── internal/
│   ├── filter/
│   │   └── filter.go    # Exclusion logic with wildcards
│   └── scanner/
│       └── scanner.go   # File scanning and processing
├── main.go              # Application entry point
├── go.mod
├── go.sum
└── README.md
```

Use `context-generator list-exclusions` to see the complete list with all patterns.
