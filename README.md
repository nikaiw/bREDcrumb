# RedBreadcrumb

A red team tool for generating, injecting, and tracking unique strings ("breadcrumbs") in binaries. Perfect for payload tracking, attribution, and detection rule generation.

## Features

- **String Generation**: Create unique tracking strings with custom prefixes
- **Binary Patching**: Inject strings into PE, ELF, and Mach-O executables
- **Code Generation**: Generate code snippets in 9 languages (C, C++, Python, Go, Rust, C#, JavaScript, PowerShell, Java)
- **YARA Rules**: Auto-generate detection rules for your tracking strings
- **Local Database**: Track all generated strings and patched binaries
- **WebAssembly Support**: Use in browser-based applications

## Installation

### From Source

```bash
git clone https://github.com/nikaiw/redbreadcrumb.git
cd redbreadcrumb
cargo build --release
```

The binary will be at `target/release/redbreadcrumb`.

### For WASM

```bash
cargo build --lib --target wasm32-unknown-unknown --release
```

## Quick Start

### Generate a Tracking String

```bash
# Generate a 16-character random string
redbreadcrumb generate -l 16

# With a custom prefix and tag
redbreadcrumb generate -l 12 --prefix "OP_" --tag "campaign-alpha"

# Use your own custom string
redbreadcrumb generate --custom "MY_CUSTOM_TRACKER_123" --tag "op-beta"
```

### Inject into a Binary

```bash
# Inject using code cave strategy (default)
redbreadcrumb patch ./payload.exe "TRACK_ABC123" -o ./payload_tracked.exe

# Available strategies: cave, section, extend, overlay
redbreadcrumb patch ./payload.elf "TRACK_XYZ" -s overlay
```

### Generate Detection Rules

```bash
# Generate YARA rule
redbreadcrumb yara "TRACK_ABC123" --ascii --wide

# Output:
# rule tracking_string_TRACK_ABC123 {
#     strings:
#         $s = "TRACK_ABC123" ascii wide
#     condition:
#         $s
# }
```

### Generate Code Snippets

```bash
# Generate C code
redbreadcrumb code "TRACK_ABC123" -l c

# Generate Python code
redbreadcrumb code "TRACK_ABC123" -l python

# Save to file
redbreadcrumb code "TRACK_ABC123" -l rust -o tracking.rs
```

### Manage Tracked Strings

```bash
# List all tracked strings
redbreadcrumb list

# Filter by tag
redbreadcrumb list --tag "campaign-alpha"

# Show details for a specific string
redbreadcrumb show <id-or-value>

# Export as JSON
redbreadcrumb list --json
```

## CLI Reference

```
Usage: redbreadcrumb [OPTIONS] <COMMAND>

Commands:
  generate  Generate a new tracking string
  yara      Generate a YARA rule for a tracking string
  code      Generate code snippet for a tracking string
  patch     Patch a binary with a tracking string
  list      List all tracked strings
  show      Show details for a tracked string

Options:
  -v, --verbose  Enable verbose output
  -h, --help     Print help
  -V, --version  Print version
```

## Supported Languages

| Language   | Flag          | Use Case                    |
|------------|---------------|-----------------------------|
| C          | `-l c`        | Native implants             |
| C++        | `-l cpp`      | Native implants             |
| Python     | `-l python`   | Scripts, tools              |
| Go         | `-l go`       | Cross-platform tools        |
| Rust       | `-l rust`     | Native implants             |
| C#         | `-l csharp`   | .NET assemblies             |
| JavaScript | `-l javascript` | Node.js, Electron         |
| PowerShell | `-l powershell` | Windows scripts           |
| Java       | `-l java`     | Cross-platform, Android     |

## Patch Strategies

| Strategy | Description                              | Formats       |
|----------|------------------------------------------|---------------|
| `cave`   | Use existing code caves (padding bytes)  | PE, ELF, Mach-O |
| `section`| Add to existing section padding          | PE, ELF, Mach-O |
| `extend` | Extend a section                         | PE, ELF, Mach-O |
| `overlay`| Append to file end (simplest)            | All           |

## License

MIT
