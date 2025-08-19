# Allsorts Development Guide

## Quick Start

### Prerequisites
- Rust 1.66.0 or higher
- Cargo (comes with Rust)

### Build Commands

```bash
# Build the library
cargo build

# Build with optimizations
cargo build --release

# Build only the library (no tests/examples)
cargo build --lib

# Check compilation without building
cargo check
```

### Testing

```bash
# Run all tests
cargo test

# Run specific test suite
cargo test subset_and_map

# Run with verbose output
cargo test -- --nocapture

# Run ignored tests (performance tests)
cargo test -- --ignored

# Run tests for a specific module
cargo test --lib subset
```

### Code Quality

```bash
# Format code (required before PR)
cargo fmt

# Check formatting without changing files
cargo fmt -- --check

# Run linter
cargo clippy

# Fix linter warnings automatically
cargo clippy --fix
```

### Documentation

```bash
# Build and open documentation
cargo doc --open

# Build docs with private items
cargo doc --document-private-items
```

### Examples

```bash
# Run specific example
cargo run --example subset_with_mapping

# List all examples
ls examples/
```

## Project Structure

```
allsorts/
├── src/
│   ├── lib.rs           # Main library entry point
│   ├── subset.rs        # Font subsetting (includes subset_and_map)
│   ├── font.rs          # Core font functionality
│   └── tables/          # OpenType table implementations
├── tests/               # Integration tests
├── examples/            # Usage examples
└── benches/            # Performance benchmarks
```

## Development Workflow

1. **Create feature branch**
   ```bash
   git checkout -b feat-your-feature
   ```

2. **Make changes and test**
   ```bash
   cargo test
   cargo fmt
   cargo clippy
   ```

3. **Commit with descriptive message**
   ```bash
   git add -A
   git commit -m "subset: Add subset_and_map function for glyph ID tracking"
   ```

4. **Push and create PR**
   ```bash
   git push origin feat-your-feature
   ```

## Common Tasks

### Adding a New Feature

1. Write tests first (TDD approach)
2. Implement functionality
3. Add documentation
4. Update examples if applicable

### Debugging Tests

```bash
# Run single test with output
cargo test test_name -- --exact --nocapture

# Get backtrace on failure
RUST_BACKTRACE=1 cargo test

# Run tests in single thread (helpful for debugging)
cargo test -- --test-threads=1
```

### Performance Testing

```bash
# Run benchmarks
cargo bench

# Profile with release optimizations
cargo build --release
cargo run --release --example your_example
```

## Key Modules

- **`subset`**: Font subsetting, including the new `subset_and_map` function
- **`font`**: Font loading and glyph mapping
- **`tables`**: OpenType/TrueType table parsing
- **`shaping`**: Text shaping engine
- **`woff`/`woff2`**: Web font format support

## Testing Guidelines

- Use fixture fonts from `tests/fonts/`
- Follow existing test patterns in `tests/`
- Test edge cases (empty input, missing glyphs, etc.)
- Verify backward compatibility

## Tips

- The codebase uses `rustc_hash::FxHashMap` for performance
- Most errors use `ParseError` or specific error enums
- Check `CONTRIBUTING.md` before submitting PRs
- Use `#[allow(dead_code)]` sparingly and with justification

## Useful Commands

```bash
# Find usage of a function
grep -r "subset_and_map" src/ tests/

# Check which tests cover a module
cargo test --lib subset 2>&1 | grep "test.*ok"

# See what would be included in package
cargo package --list

# Verify package builds correctly
cargo package --no-verify
```

## Resources

- [OpenType Specification](https://docs.microsoft.com/en-us/typography/opentype/spec/)
- [HarfBuzz](https://harfbuzz.github.io/) - Reference implementation
- [Font Tools](https://github.com/fonttools/fonttools) - Python font library