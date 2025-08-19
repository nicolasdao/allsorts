# Allsorts Testing Guidelines

This document outlines the testing conventions, patterns, and style guidelines for the Allsorts font library. Follow these guidelines to ensure your tests integrate seamlessly with the existing test suite.

## Table of Contents
1. [Test Organization](#test-organization)
2. [Running Tests](#running-tests)
3. [Test File Structure](#test-file-structure)
4. [Unit Tests](#unit-tests)
5. [Integration Tests](#integration-tests)
6. [Test Fixtures and Data](#test-fixtures-and-data)
7. [Common Testing Patterns](#common-testing-patterns)
8. [Script-Specific Tests](#script-specific-tests)
9. [Error Testing](#error-testing)
10. [Performance and Benchmarking](#performance-and-benchmarking)

## Test Organization

### Directory Structure
```
allsorts/
├── src/                  # Source files with inline unit tests
│   └── *.rs             # Use #[cfg(test)] modules for unit tests
├── tests/               # Integration tests
│   ├── common.rs        # Shared test utilities (not a test file itself)
│   ├── shape.rs         # Shared shaping utilities
│   ├── *.rs             # Integration test files
│   └── fonts/           # Test font fixtures
│       └── */           # Organized by script/feature
└── criterion/           # Benchmark tests (currently disabled)
```

### Test Categories

1. **Unit Tests**: Inside source files, testing individual functions/methods
2. **Integration Tests**: In `tests/` directory, testing feature combinations
3. **Script Tests**: Script-specific shaping tests (arabic.rs, indic.rs, etc.)
4. **Format Tests**: Font format parsing tests (woff.rs, woff2.rs, cff.rs)
5. **AOTS Tests**: Adobe OpenType Specification compliance tests

## Running Tests

### Basic Commands
```bash
# Run all tests
cargo test

# Run tests for a specific module
cargo test tag         # Runs tests matching "tag"

# Run with verbose output
cargo test -- --nocapture

# Run a specific test
cargo test test_decode_head

# Run tests with specific features
cargo test --features prince

# Run only doc tests
cargo test --doc

# Run tests in release mode (optimized)
cargo test --release
```

### Test Features
- `default`: Standard configuration with zlib compression
- `prince`: Enables Prince-specific test paths and behaviors
- `flate2_rust`: Use pure Rust compression (for WASM compatibility)

## Test File Structure

### Integration Test Template
```rust
// tests/my_feature.rs
mod common;  // Import shared utilities
mod shape;   // Import shaping utilities if needed

#[cfg(test)]  // Optional for integration tests, but good practice
mod my_feature_tests {
    use crate::common;
    use allsorts::binary::read::ReadScope;
    use allsorts::tables::OpenTypeFont;
    use allsorts::Font;
    // Other imports...

    #[test]
    fn test_basic_functionality() {
        // Test implementation
    }

    #[test]
    #[should_panic(expected = "specific error message")]
    fn test_error_condition() {
        // Test that should panic
    }
}
```

### Unit Test Template
```rust
// src/my_module.rs

// ... module implementation ...

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_internal_function() {
        // Test implementation
    }
}
```

## Unit Tests

### Style Guidelines

1. **Location**: Place unit tests at the bottom of the source file in a `#[cfg(test)]` module
2. **Naming**: Use descriptive names prefixed with `test_`
3. **Scope**: Test individual functions, focusing on edge cases
4. **Dependencies**: Minimize external dependencies; use mocks where appropriate

### Example Unit Test
```rust
#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_from_string_four_chars() {
        let tag = from_string("beng").expect("invalid tag");
        assert_eq!(tag, 1650814567);
    }

    #[test]
    fn test_from_string_padding() {
        let tag = from_string("BEN").expect("invalid tag");
        assert_eq!(tag, 1111838240); // "BEN " with space padding
    }

    #[test]
    fn test_from_string_too_long() {
        assert!(from_string("12345").is_err());
    }
}
```

## Integration Tests

### Loading Font Fixtures

Use the common utilities for consistent fixture loading:

```rust
use crate::common::{read_fixture, read_fixture_font, fixture_path};

#[test]
fn test_font_parsing() {
    // Load font from tests/fonts/ directory
    let font_data = read_fixture_font("opentype/test-font.ttf");
    
    // Load arbitrary fixture
    let data = read_fixture("tests/data/sample.bin");
    
    // Get full path to fixture
    let path = fixture_path("tests/fonts/arabic/amiri-regular.ttf");
}
```

### Font Setup Pattern
```rust
#[test]
fn test_font_feature() {
    // Standard font loading and setup
    let font_buffer = common::read_fixture_font("path/to/font.ttf");
    let scope = ReadScope::new(&font_buffer);
    let font_file = scope.read::<OpenTypeFont>().unwrap();
    let provider = font_file.table_provider(0).unwrap();
    let mut font = Font::new(Box::new(provider))
        .expect("error reading font data");
    
    // Test font operations
    assert_eq!(font.num_glyphs(), 100);
}
```

## Test Fixtures and Data

### Font Fixtures Organization
```
tests/fonts/
├── arabic/          # Arabic script test fonts
├── bengali/         # Bengali script test fonts
├── devanagari/      # Devanagari script test fonts
├── khmer/           # Khmer script test fonts
├── myanmar/         # Myanmar script test fonts
├── noto/            # Google Noto fonts
├── opentype/        # General OpenType test fonts
├── svg/             # SVG glyph test fonts
├── variable/        # Variable font tests
├── woff1/           # WOFF format tests
├── woff2/           # WOFF2 format tests
└── licenses/        # Font licenses
```

### Test Data Files

For script-specific tests, use companion data files:

```
tests/indic/
├── bad.bn           # Invalid Bengali sequences
├── good.bn          # Valid Bengali sequences
├── harfbuzz/        # HarfBuzz comparison data
│   └── good-*.bn    # Expected outputs
└── directwrite/     # DirectWrite comparison data
```

### Data File Format
```
# Input file (good.bn)
নমস্কার
বাংলা

# Output file (expected glyph indices)
[179|259|332|480|235]
[365|147|404|296|147]
```

## Common Testing Patterns

### 1. Table Parsing Tests
```rust
#[test]
fn test_decode_table() {
    let buffer = read_fixture("tests/fonts/opentype/test-font.ttf");
    let file = ReadScope::new(&buffer).read::<OpenTypeFont>().unwrap();
    
    match file.data {
        OpenTypeData::Single(ttf) => {
            let table_data = ttf
                .read_table(&file.scope, tag::HEAD)
                .expect("unable to read table")
                .expect("table not found");
            let head = table_data
                .read::<HeadTable>()
                .expect("error parsing table");
            
            assert_eq!(head.units_per_em, 2048);
        }
        OpenTypeData::Collection(_) => unreachable!(),
    }
}
```

### 2. Shaping Tests
```rust
fn test_shaping_output(
    font_path: &str,
    text: &str,
    expected_glyphs: Vec<u16>,
) {
    let font_buffer = common::read_fixture_font(font_path);
    let scope = ReadScope::new(&font_buffer);
    let font_file = scope.read::<OpenTypeFont>().unwrap();
    let provider = font_file.table_provider(0).unwrap();
    let mut font = Font::new(Box::new(provider)).unwrap();
    
    let glyphs = font.map_glyphs(text, tag::LATN, MatchingPresentation::Required);
    let shaped = font.shape(
        glyphs,
        tag::LATN,
        None,
        &Features::default(),
        None,
        true,
    ).unwrap();
    
    let glyph_ids: Vec<u16> = shaped.iter()
        .map(|info| info.glyph.glyph_index)
        .collect();
    
    assert_eq!(glyph_ids, expected_glyphs);
}
```

### 3. Comparison Testing
```rust
fn run_comparison_test(
    test_data: &TestData,
    expected_outputs_path: &str,
    font_path: &str,
) {
    let inputs = common::read_inputs("tests/indic", test_data.inputs_path);
    let expected = common::parse_expected_outputs(
        "tests/indic",
        expected_outputs_path,
        &[], // glyphs to ignore
    );
    
    for (input, (expected_output, reason)) in inputs.iter().zip(expected.iter()) {
        let actual = shape_text(&font, input);
        
        match (&actual, expected_output) {
            (Ok(actual), expected) if actual == expected => {
                // Test passed
            }
            _ => {
                println!("Test failed for input: {}", input);
                println!("  Expected: {:?}", expected_output);
                println!("  Actual: {:?}", actual);
                if let Some(reason) = reason {
                    println!("  Reason: {}", reason);
                }
                panic!("Shaping mismatch");
            }
        }
    }
}
```

## Script-Specific Tests

### Arabic/Syriac Pattern
```rust
#[test]
fn test_arabic_shaping() {
    test(
        Some(tag::ARA),  // Language tag
        vec![
            (
                "tests/fonts/arabic/font.ttf",
                "السلام",  // Input text
                vec![965, 994, 1330, 982, 147],  // Expected glyph indices
            ),
        ],
    );
}
```

### Indic Scripts Pattern
```rust
struct TestData {
    font_tag: &'static str,
    font_path: &'static str,
    script_tag: &'static str,
    lang_tag: &'static str,
    inputs_path: &'static str,
}

#[test]
fn test_devanagari() {
    let test_data = TestData {
        font_tag: "lohit_hi",
        font_path: "devanagari/lohit_hi.ttf",
        script_tag: "deva",
        lang_tag: "HIN",
        inputs_path: "good.hi",
    };
    
    run_test(
        &test_data,
        "harfbuzz/good-lohit.hi",
        test_data.font_path,
        &[],  // glyphs to ignore
        0,    // expected failures
    );
}
```

## Error Testing

### Expected Failures
```rust
#[test]
#[should_panic(expected = "MissingTable")]
fn test_missing_required_table() {
    let font_data = create_font_without_cmap();
    Font::new(Box::new(font_data)).unwrap();
}
```

### Error Propagation
```rust
#[test]
fn test_error_handling() {
    let result = parse_invalid_data();
    
    match result {
        Err(ParseError::BadValue) => {
            // Expected error
        }
        Ok(_) => panic!("Should have failed"),
        Err(e) => panic!("Wrong error type: {:?}", e),
    }
}
```

### Validation Tests
```rust
#[test]
fn test_bounds_checking() {
    let font = load_test_font();
    
    // Test invalid glyph index
    assert_eq!(font.horizontal_advance(u16::MAX), None);
    
    // Test empty input
    let glyphs = font.map_glyphs("", tag::LATN, MatchingPresentation::Required);
    assert!(glyphs.is_empty());
}
```

## Performance and Benchmarking

### Criterion Benchmarks (Currently Disabled)
```rust
// criterion/bench-shape.rs
use criterion::{black_box, criterion_group, criterion_main, Criterion};

fn bench_arabic_shaping(c: &mut Criterion) {
    let font = setup_font();
    let text = "مرحبا بالعالم";
    
    c.bench_function("arabic_shaping", |b| {
        b.iter(|| {
            font.shape(
                black_box(text),
                tag::ARAB,
                None,
                &Features::default(),
                None,
                true,
            )
        })
    });
}

criterion_group!(benches, bench_arabic_shaping);
criterion_main!(benches);
```

### Manual Performance Testing
```rust
#[test]
#[ignore]  // Run with: cargo test -- --ignored
fn test_performance_large_text() {
    use std::time::Instant;
    
    let font = load_test_font();
    let text = "a".repeat(10000);
    
    let start = Instant::now();
    let _shaped = font.shape_text(&text);
    let duration = start.elapsed();
    
    println!("Shaped 10000 characters in {:?}", duration);
    assert!(duration.as_secs() < 1, "Shaping took too long");
}
```

## Best Practices

### 1. Test Naming
- Use descriptive names that indicate what is being tested
- Prefix with `test_` for consistency
- Group related tests in modules

### 2. Assertions
```rust
// Prefer specific assertions
assert_eq!(actual, expected, "Custom failure message");

// Use custom messages for clarity
assert!(
    glyphs.len() > 0,
    "Expected non-empty glyph list for text: {}",
    text
);

// Use assert_matches! for enums (requires nightly)
// assert_matches!(result, ParseError::BadValue);
```

### 3. Test Data
- Use small, focused test cases
- Document why specific values are expected
- Include edge cases (empty input, maximum values, etc.)

### 4. Test Independence
- Each test should be independent
- Don't rely on test execution order
- Clean up any temporary files created

### 5. Documentation
```rust
/// Tests that composite glyphs are correctly resolved
/// when their components reference other composite glyphs.
/// 
/// This specifically tests the case described in issue #123
#[test]
fn test_nested_composite_glyphs() {
    // Test implementation
}
```

### 6. Feature Gating
```rust
#[test]
#[cfg(feature = "prince")]
fn test_prince_specific_behavior() {
    // Prince-specific test
}

#[test]
#[cfg(not(feature = "prince"))]
fn test_standard_behavior() {
    // Standard behavior test
}
```

## Adding New Tests

### Checklist for New Tests

1. **Choose appropriate location**:
   - Unit test: In source file's `#[cfg(test)]` module
   - Integration test: New file in `tests/` or add to existing

2. **Follow naming conventions**:
   - File: `tests/feature_name.rs`
   - Test function: `test_specific_behavior()`

3. **Add test fixtures** if needed:
   - Place in appropriate `tests/fonts/` subdirectory
   - Include license file if adding new font

4. **Use common utilities**:
   - Import `mod common;` for fixture loading
   - Use existing helper functions

5. **Document complex tests**:
   - Add comments explaining the test scenario
   - Reference issues or specifications

6. **Verify test passes**:
   ```bash
   cargo test your_test_name
   ```

7. **Check test coverage**:
   ```bash
   cargo tarpaulin  # If installed
   ```

## Continuous Integration

Tests are automatically run on:
- Linux
- macOS  
- Windows

Ensure your tests work cross-platform by:
- Using `Path` and `PathBuf` for file paths
- Avoiding platform-specific assumptions
- Testing with both compression backends

## Troubleshooting

### Common Issues

1. **File not found errors**:
   - Use `common::read_fixture()` functions
   - Paths are relative to `CARGO_MANIFEST_DIR`

2. **Feature-related failures**:
   - Check if test needs feature gating
   - Run with appropriate features enabled

3. **Intermittent failures**:
   - Ensure test independence
   - Check for timing dependencies
   - Verify no shared mutable state

4. **Platform-specific failures**:
   - Test locally on Docker/VM if needed
   - Check path separator usage
   - Verify endianness assumptions

## Resources

- [Rust Testing Documentation](https://doc.rust-lang.org/book/ch11-00-testing.html)
- [OpenType Specification](https://docs.microsoft.com/en-us/typography/opentype/spec/)
- [HarfBuzz Test Suite](https://github.com/harfbuzz/harfbuzz/tree/main/test) (for comparison)
- [Adobe AOTS](https://github.com/adobe-fonts/aots) (compliance tests)