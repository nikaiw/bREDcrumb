# bREDcrumb

**Leave your mark in every binary.**

[![GitHub Pages](https://img.shields.io/badge/demo-GitHub%20Pages-blue)](https://nikaiw.github.io/bREDcrumb/)

bREDcrumb injects unique tracking strings into your red team tools and implants. At the end of an engagement, blue team can use these breadcrumbs to **distinguish with certainty** which binaries were deployed by your team versus actual threat actors. It also acts as a **safety net** — ensuring you can reliably identify and clean up all assets where your tools were deployed.

**[Try the online demo →](https://nikaiw.github.io/bREDcrumb/)**

## Features

- **Binary Patching**: Inject breadcrumbs into PE, ELF, and Mach-O executables
- **YARA Rules**: Auto-generate detection rules to share with blue team for cleanup
- **Tracking Database**: Keep track of all breadcrumbs and patched binaries
- **WebAssembly Support**: Use directly in the browser
- **Code Snippets**: Generate embeddable code for compiled languages

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

## Build Integration

Integrate bREDcrumb into your build pipeline to automatically tag all your tools.

### Makefile

```makefile
BREADCRUMB ?= YOURTEAM_$(shell git rev-parse --short HEAD)

build:
	$(CC) -o implant main.c

release: build
	bredcrumb patch ./implant "$(BREADCRUMB)" -s overlay
	bredcrumb yara "$(BREADCRUMB)" --ascii --wide -o implant.yar

clean:
	rm -f implant implant.yar
```

### Rust (Cargo)

Add a post-build script in your `Cargo.toml`:

```toml
[package.metadata.scripts]
post-build = "bredcrumb patch ./target/release/implant $BREADCRUMB -s overlay"
```

Or use a simple wrapper script `build.sh`:

```bash
#!/bin/bash
BREADCRUMB="YOURTEAM_$(git rev-parse --short HEAD)"
cargo build --release
bredcrumb patch ./target/release/implant "$BREADCRUMB" -s overlay
bredcrumb yara "$BREADCRUMB" --ascii --wide -o implant.yar
```

### Go

```bash
#!/bin/bash
BREADCRUMB="YOURTEAM_$(git rev-parse --short HEAD)"
go build -o implant .
bredcrumb patch ./implant "$BREADCRUMB" -s overlay
```

### CMake

```cmake
add_custom_command(TARGET implant POST_BUILD
    COMMAND bredcrumb patch $<TARGET_FILE:implant> "${BREADCRUMB}" -s overlay
    COMMENT "Injecting breadcrumb..."
)
```

### CI/CD

#### GitHub Actions

```yaml
name: Build with Breadcrumb

on: [push]

jobs:
  build:
    runs-on: ubuntu-latest
    steps:
      - uses: actions/checkout@v4

      - name: Setup Rust
        uses: dtolnay/rust-toolchain@stable

      - name: Build your tool
        run: make release

      - name: Install bREDcrumb
        run: cargo install --git https://github.com/nikaiw/bREDcrumb.git

      - name: Inject breadcrumb
        run: |
          BREADCRUMB="YOURTEAM_${GITHUB_SHA::8}"
          bredcrumb patch ./build/implant "$BREADCRUMB" -s overlay
          bredcrumb yara "$BREADCRUMB" --ascii --wide -o breadcrumb.yar

      - name: Upload artifacts
        uses: actions/upload-artifact@v4
        with:
          name: release
          path: |
            ./build/implant
            breadcrumb.yar
```

#### GitLab CI

```yaml
build:
  stage: build
  image: rust:latest
  script:
    - make release
    - cargo install --git https://github.com/nikaiw/bREDcrumb.git
    - BREADCRUMB="YOURTEAM_${CI_COMMIT_SHORT_SHA}"
    - bredcrumb patch ./build/implant "$BREADCRUMB" -s overlay
    - bredcrumb yara "$BREADCRUMB" --ascii --wide -o breadcrumb.yar
  artifacts:
    paths:
      - ./build/implant
      - breadcrumb.yar
```

### Tips

- Use commit SHA in breadcrumb for traceability: `TEAM_${COMMIT_SHA::8}`
- Store YARA rules as build artifacts for blue team handoff
- Use `overlay` strategy for simplicity, `cave` for stealth
- Tag builds with campaign name: `--tag "operation-name"`

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
