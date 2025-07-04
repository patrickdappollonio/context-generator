# Context Generator

A fast, efficient command-line tool written in Rust that generates copy-pastable context from your source code, perfect for providing to AI assistants like ChatGPT, Claude, or Copilot.

## What It Does

`context-generator` scans your project directory and creates a formatted output containing all your text files' contents. This output is specifically designed to be copy-pasted into AI chat interfaces, giving the AI complete context about your codebase for more accurate assistance.

## Why Use Context Generator?

When working with AI assistants on coding projects, you often need to provide file contents for context. Instead of manually copying and pasting multiple files, Context Generator:

- **Automates the process** - Scans entire directories in milliseconds
- **Smart filtering** - Excludes binary files, build artifacts, and common junk automatically
- **Highly configurable** - Customize exclusions with wildcards and patterns
- **AI-optimized format** - Outputs in a clear, structured format that AI tools understand perfectly
- **Universal compatibility** - Works with any AI that accepts text input
- **Memory efficient** - Built in Rust for optimal performance and safety

## Installation

### From Releases (Recommended)

Download a pre-built binary from the [releases page](https://github.com/patrickdappollonio/context-generator/releases/latest):

- **Linux x86_64**: `context-generator-linux-x86_64.tar.gz`
- **Linux ARM64**: `context-generator-linux-arm64.tar.gz`
- **macOS x86_64**: `context-generator-darwin-x86_64.tar.gz`
- **macOS ARM64 (M+)**: `context-generator-darwin-arm64.tar.gz`
- **Windows x86_64**: `context-generator-windows-x86_64.zip`

### From Source

```bash
git clone https://github.com/patrickdappollonio/context-generator.git
cd context-generator
cargo build --release
# Binary will be at target/release/context-generator
```

### Using Cargo

```bash
cargo install --git https://github.com/patrickdappollonio/context-generator
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
Generate copy-pastable context from your source code for AI assistants

Usage: context-generator [OPTIONS] [DIRECTORY] [COMMAND]

Commands:
  list-exclusions  List all default exclusions organized by category
  help             Print this message or the help of the given subcommand(s)

Arguments:
  [DIRECTORY]  Directory to scan (defaults to current directory)

Options:
      --exclude <PATTERN>      Exclude files/folders matching these patterns (supports wildcards)
      --disable-category <ID>  Disable default exclusion categories by ID (use list-exclusions to see IDs)
      --no-defaults            Disable default exclusions
      --dry-run                Show files that would be processed and excluded without generating output
  -h, --help                   Print help
  -V, --version                Print version
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
```bash
$ context-generator --dry-run src/

Dry run for directory: src/

Files that would be processed:
  ├── cli.rs
  ├── filter.rs
  ├── lib.rs
  ├── main.rs
  └── scanner.rs

Files that would be excluded:
  (none)
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
file: src/main.rs
--------------------
    use cli::run_cli;
    use std::process;

    mod cli;
    mod filter;
    mod scanner;

    fn main() {
        if let Err(e) = run_cli() {
            eprintln!("Error: {}", e);
            process::exit(1);
        }
    }
--------------------
file: src/cli.rs
--------------------
    use clap::{Parser, Subcommand};
    use std::io;
    // ... rest of file content
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
2. **Mention File Names**: Reference specific files when asking questions
3. **Update Context**: Re-run the tool when your code changes significantly
4. **Size Awareness**: Very large codebases might hit AI token limits - use exclusions to focus on relevant parts
5. **Use Categories**: Disable specific categories (like `logs` or `build`) to focus on source code

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
context-generator | grep -A5 "file: main.rs"

# Use category patterns in shell scripts
for pattern in $(context-generator list-exclusions --category logs); do
  echo "Would exclude: $pattern"
done

# Create .gitignore from patterns
echo "# Generated exclusions" > .gitignore
context-generator list-exclusions --patterns-only >> .gitignore
```

## Performance

Context Generator is built in Rust for optimal performance:

- **Fast scanning** - Processes large codebases in milliseconds
- **Memory efficient** - Minimal memory usage even on huge projects
- **Small binary** - Single ~800KB executable with no dependencies
- **Cross-platform** - Available for Linux, macOS, and Windows

## Contributing

Contributions are welcome! We especially encourage contributions to expand language support and exclusion patterns.

### Adding New Exclusion Categories

The exclusion categories are defined in `exclusions.yaml` in the root directory. This makes it easy for developers of any language to contribute new patterns without needing Rust knowledge.

To add a new exclusion category:

1. **Edit `exclusions.yaml`** - Add your new category following this structure, at the bottom of the file:

```yaml
- id: your-language-id
  name: Your Language Name
  description: Brief description of what files this category excludes
  patterns:
    - "*.your-ext"
    - "build-dir"
    - "*.generated"
```

2. **Test your changes** - Run the tool to verify your patterns work:

```bash
# Build and test
cargo build --release

# Test your new category
./target/release/context-generator list-exclusions --category your-language-id

# Test exclusions work as expected
./target/release/context-generator --dry-run /path/to/test/project
```

3. **Submit a pull request** - Your contribution will be automatically embedded in the binary at compile time.

### Exclusion Category Guidelines

- **Use descriptive IDs**: Short, lowercase, hyphen-separated (e.g., `python-ds`, `web-frameworks`)
- **Clear descriptions**: Explain what type of files are excluded
- **Comprehensive patterns**: Include common file extensions, directories, and build artifacts
- **Test thoroughly**: Ensure patterns work with real projects in that language/framework

### Examples of Good Contributions

```yaml
# Good: Comprehensive mobile development category
- id: flutter
  name: Flutter
  description: Flutter mobile development framework files
  patterns:
    - "*.g.dart"
    - "*.freezed.dart"
    - "*.mocks.dart"
    - ".flutter-plugins"
    - ".flutter-plugins-dependencies"
    - "build/"
    - ".dart_tool/"
    - "ios/Flutter/Generated.xcconfig"
    - "ios/Flutter/flutter_export_environment.sh"

# Good: Specific build system category
- id: cmake
  name: CMake
  description: CMake build system files
  patterns:
    - "CMakeCache.txt"
    - "CMakeFiles/"
    - "cmake_install.cmake"
    - "install_manifest.txt"
    - "*.cmake"
    - "build/"
```

The YAML format is embedded at compile time, so there's no runtime performance cost and the binary remains self-contained.

---

**Pro Tip**: Use `context-generator list-exclusions --category <id>` to quickly see patterns for a specific category, or `context-generator list-exclusions | grep "^\s\s"` to get a clean list of all patterns for scripting!

## Project Structure

```
context-generator/
├── src/
│   ├── cli.rs           # CLI command setup with clap
│   ├── filter.rs        # Exclusion logic and YAML parsing
│   ├── lib.rs           # Library exports
│   ├── main.rs          # Application entry point
│   └── scanner.rs       # File scanning and processing
├── exclusions.yaml      # Exclusion categories and patterns (embedded at compile time)
├── Cargo.toml           # Rust project configuration
├── Cargo.lock           # Dependency lock file
├── Dockerfile           # Container build instructions
└── README.md
```

Use `context-generator list-exclusions` to see the complete list with all patterns.
