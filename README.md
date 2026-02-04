# bREDcrumb

**Leave your mark in every binary.** bREDcrumb helps red teams inject unique tracking strings into compiled binaries for attribution and tracking. When blue team discovers your implant, they'll find your breadcrumb — and you'll have the YARA rule ready.

## Features

- **String Generation**: Create unique tracking strings with custom prefixes
- **Binary Patching**: Inject strings into PE, ELF, and Mach-O executables
- **YARA Rules**: Auto-generate detection rules for your tracking strings
- **Local Database**: Track all generated strings and patched binaries
- **Code Generation**: Generate code snippets in compiled languages (C, C++, Go, Rust, C#, Java)
- **WebAssembly Support**: Use in browser-based applications

## Installation

### From Source

```bash
git clone https://github.com/nikaiw/bREDcrumb.git
cd bREDcrumb
cargo build --release
```

The binary will be at `target/release/bredcrumb`.

### For WASM

```bash
wasm-pack build --target web --out-dir docs/pkg
```

## Quick Start

### Generate a Tracking String

```bash
# Generate a 16-character random string
bredcrumb generate -l 16

# With a custom prefix and tag
bredcrumb generate -l 12 --prefix "OP_" --tag "campaign-alpha"

# Use your own custom string
bredcrumb generate --custom "MY_CUSTOM_TRACKER_123" --tag "op-beta"
```

### Inject into a Binary

```bash
# Inject using code cave strategy (default)
bredcrumb patch ./payload.exe "TRACK_ABC123" -o ./payload_tracked.exe

# Available strategies: cave, section, extend, overlay
bredcrumb patch ./payload.elf "TRACK_XYZ" -s overlay
```

### Generate Detection Rules

```bash
# Generate YARA rule
bredcrumb yara "TRACK_ABC123" --ascii --wide

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
# Generate C code with DCE prevention
bredcrumb code "TRACK_ABC123" -l c

# Generate Rust code
bredcrumb code "TRACK_ABC123" -l rust

# Save to file
bredcrumb code "TRACK_ABC123" -l go -o tracking.go
```

### Manage Tracked Strings

```bash
# List all tracked strings
bredcrumb list

# Filter by tag
bredcrumb list --tag "campaign-alpha"

# Show details for a specific string
bredcrumb show <id-or-value>

# Export as JSON
bredcrumb list --json
```

## CLI Reference

```
Usage: bredcrumb [OPTIONS] <COMMAND>

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

| Language | Flag        | Use Case                |
|----------|-------------|-------------------------|
| C        | `-l c`      | Native implants         |
| C++      | `-l cpp`    | Native implants         |
| Go       | `-l go`     | Cross-platform tools    |
| Rust     | `-l rust`   | Native implants         |
| C#       | `-l csharp` | .NET assemblies         |
| Java     | `-l java`   | Cross-platform, Android |

## Patch Strategies

| Strategy | Description                              | Formats         |
|----------|------------------------------------------|-----------------|
| `cave`   | Use existing code caves (padding bytes)  | PE, ELF, Mach-O |
| `section`| Add to existing section padding          | PE, ELF, Mach-O |
| `extend` | Extend a section                         | PE, ELF, Mach-O |
| `overlay`| Append to file end (simplest)            | All             |

## License

MIT
