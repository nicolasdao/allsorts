# Phase 3 Implementation Plan: Builder Pattern API for Allsorts Font Subsetting

## Executive Summary

This document provides a complete implementation plan for Phase 3 of the PDF-specific font subsetting features for the Allsorts library. The goal is to create a convenient Builder Pattern API that unifies all the subsetting features developed in Phases 1 and 2.

## Project Context

### What is Allsorts?
Allsorts is a Rust library for parsing, manipulating, and subsetting OpenType fonts. It's used in production environments for PDF generation, web font optimization, and other font processing tasks.

### Current Project Status
We are enhancing Allsorts with PDF-specific font subsetting capabilities. Two phases have been completed:

**Phase 1: Core Features (COMPLETED)**
- Composite glyph reference updater (`src/subset/composite.rs`)
- Enhanced subset result structure (`src/subset/result.rs`)

**Phase 2: PDF-Specific Features (COMPLETED)**
- PDF-specific subsetting helper (`src/subset/pdf.rs`)
- Validation and debugging utilities (`src/subset/validation.rs`)

**Phase 3: Builder Pattern API (TO BE IMPLEMENTED)**
- Fluent interface for font subsetting
- Unified API for all subsetting features
- Character-to-glyph mapping support
- Validation level configuration

## Project Structure

```
allsorts/
├── src/
│   ├── subset.rs                 # Main subsetting module
│   ├── subset/
│   │   ├── composite.rs         # ✅ Composite reference updater
│   │   ├── result.rs           # ✅ Enhanced result structures
│   │   ├── pdf.rs              # ✅ PDF-specific features
│   │   ├── validation.rs       # ✅ Validation utilities
│   │   └── builder.rs          # 🔨 TO BE CREATED - Builder API
│   └── ...
├── tests/
│   ├── composite_update.rs     # ✅ Tests for composite updates
│   ├── subset_result.rs        # ✅ Tests for result structures
│   ├── pdf_subset.rs           # ✅ Tests for PDF features
│   ├── validation.rs           # ✅ Tests for validation
│   ├── builder.rs              # 🔨 TO BE CREATED - Builder tests
│   └── fonts/                  # Test font files
└── Cargo.toml
```

## Implementation Methodology: Test-Driven Development (TDD)

**IMPORTANT**: This project follows strict TDD principles. The implementation MUST follow this order:

1. **Write API signatures** with `todo!()` implementations
2. **Write comprehensive tests** that will initially fail
3. **Implement the actual code** to make tests pass
4. **Refactor** while keeping tests green

## Phase 3 Detailed Implementation Plan

### Overview
The Builder Pattern API provides a fluent interface for configuring and executing font subsetting operations. It wraps all existing functionality in an easy-to-use API.

### Step 1: Create API Signatures (No Implementation)

Create `src/subset/builder.rs`:

```rust
use std::collections::HashSet;
use crate::tables::FontTableProvider;
use crate::subset::{SubsetProfile, CmapTarget, SubsetError};
use crate::subset::result::SubsetResult;
use crate::subset::pdf::{PdfFontContext, WritingMode};

/// Builder for configuring font subsetting operations
pub struct SubsetBuilder<'a> {
    provider: &'a dyn FontTableProvider,
    glyph_ids: Vec<u16>,
    cmap_target: CmapTarget,
    profile: SubsetProfile,
    fix_composites: bool,
    validation_level: ValidationLevel,
    pdf_context: Option<PdfFontContext>,
}

/// Validation strictness levels
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum ValidationLevel {
    /// No validation
    None,
    /// Basic validation (check .notdef)
    Basic,
    /// Standard validation (check glyph IDs exist)
    Standard,
    /// Strict validation (comprehensive checks)
    Strict,
}

impl<'a> SubsetBuilder<'a> {
    /// Create a new builder with a font provider
    pub fn new(provider: &'a dyn FontTableProvider) -> Self {
        todo!("Implementation will come after tests")
    }
    
    /// Add glyphs by their IDs
    pub fn with_glyphs(self, glyph_ids: &[u16]) -> Self {
        todo!("Implementation will come after tests")
    }
    
    /// Add glyphs for specific characters
    pub fn with_characters(self, chars: &str) -> Result<Self, SubsetError> {
        todo!("Implementation will come after tests")
    }
    
    /// Configure for PDF embedding
    pub fn for_pdf(self, max_cid: u16) -> Self {
        todo!("Implementation will come after tests")
    }
    
    /// Set CID-to-GID mapping for PDF
    pub fn with_cid_map(self, map: &[u16]) -> Self {
        todo!("Implementation will come after tests")
    }
    
    /// Enable/disable composite glyph fixing
    pub fn fix_composites(self, enabled: bool) -> Self {
        todo!("Implementation will come after tests")
    }
    
    /// Set validation level
    pub fn validation_level(self, level: ValidationLevel) -> Self {
        todo!("Implementation will come after tests")
    }
    
    /// Set subsetting profile
    pub fn with_profile(self, profile: SubsetProfile) -> Self {
        todo!("Implementation will come after tests")
    }
    
    /// Set cmap target format
    pub fn with_cmap_target(self, target: CmapTarget) -> Self {
        todo!("Implementation will come after tests")
    }
    
    /// Build and execute the subsetting operation
    pub fn build(self) -> Result<SubsetResult, SubsetError> {
        todo!("Implementation will come after tests")
    }
}
```

### Step 2: Write Comprehensive Tests

Create `tests/builder.rs`:

```rust
mod common;

use allsorts::subset::builder::{SubsetBuilder, ValidationLevel};
use allsorts::subset::{SubsetProfile, CmapTarget};
use allsorts::tables::{OpenTypeFont, FontTableProvider};
use allsorts::binary::read::ReadScope;

fn create_provider(font_buffer: &[u8]) -> impl FontTableProvider + '_ {
    let scope = ReadScope::new(font_buffer);
    let font_file = scope.read::<OpenTypeFont<'_>>().unwrap();
    font_file.table_provider(0).unwrap()
}

#[test]
fn test_builder_basic() {
    let font_buffer = common::read_fixture("tests/fonts/opentype/Klei.otf");
    let provider = create_provider(&font_buffer);
    
    let result = SubsetBuilder::new(&provider)
        .with_glyphs(&[0, 1, 2])
        .build();
    
    assert!(result.is_ok());
    let subset = result.unwrap();
    assert_eq!(subset.glyph_mapping.len(), 3);
}

#[test]
fn test_builder_with_characters() {
    let font_buffer = common::read_fixture("tests/fonts/opentype/Klei.otf");
    let provider = create_provider(&font_buffer);
    
    let result = SubsetBuilder::new(&provider)
        .with_characters("Hello")
        .unwrap()
        .build();
    
    assert!(result.is_ok());
    let subset = result.unwrap();
    assert!(subset.glyph_mapping.len() >= 5); // At least H, e, l, o + .notdef
}

#[test]
fn test_builder_for_pdf() {
    let font_buffer = common::read_fixture("tests/fonts/opentype/Klei.otf");
    let provider = create_provider(&font_buffer);
    
    let result = SubsetBuilder::new(&provider)
        .with_glyphs(&[0, 1, 2])
        .for_pdf(255)
        .build();
    
    assert!(result.is_ok());
    // PDF mode should auto-enable composite fixing
}

#[test]
fn test_builder_validation_levels() {
    let font_buffer = common::read_fixture("tests/fonts/opentype/Klei.otf");
    let provider = create_provider(&font_buffer);
    
    // Test None validation - should accept anything
    let result = SubsetBuilder::new(&provider)
        .with_glyphs(&[9999]) // Invalid glyph
        .validation_level(ValidationLevel::None)
        .build();
    // Should succeed despite invalid glyph
    
    // Test Strict validation - should reject invalid
    let result = SubsetBuilder::new(&provider)
        .with_glyphs(&[9999]) // Invalid glyph
        .validation_level(ValidationLevel::Strict)
        .build();
    // Should fail due to invalid glyph
}

#[test]
fn test_builder_chaining() {
    let font_buffer = common::read_fixture("tests/fonts/opentype/Klei.otf");
    let provider = create_provider(&font_buffer);
    
    let result = SubsetBuilder::new(&provider)
        .with_glyphs(&[0, 1, 2])
        .with_profile(SubsetProfile::Pdf)
        .with_cmap_target(CmapTarget::Unicode)
        .fix_composites(true)
        .validation_level(ValidationLevel::Standard)
        .build();
    
    assert!(result.is_ok());
}
```

### Step 3: Update Module Exports

Add to `src/subset.rs`:
```rust
/// Builder pattern API for font subsetting
pub mod builder;
```

### Step 4: Implement the Builder

After tests are written and failing, implement `src/subset/builder.rs`:

```rust
impl<'a> SubsetBuilder<'a> {
    pub fn new(provider: &'a dyn FontTableProvider) -> Self {
        Self {
            provider,
            glyph_ids: vec![0],  // Always include .notdef
            cmap_target: CmapTarget::Unrestricted,
            profile: SubsetProfile::Minimal,
            fix_composites: false,
            validation_level: ValidationLevel::Standard,
            pdf_context: None,
        }
    }
    
    pub fn with_glyphs(mut self, glyph_ids: &[u16]) -> Self {
        // Ensure .notdef is first
        if !glyph_ids.is_empty() && glyph_ids[0] != 0 {
            self.glyph_ids = vec![0];
            self.glyph_ids.extend_from_slice(glyph_ids);
        } else {
            self.glyph_ids = glyph_ids.to_vec();
        }
        
        // Remove duplicates while preserving order
        let mut seen = HashSet::new();
        self.glyph_ids.retain(|&gid| seen.insert(gid));
        
        self
    }
    
    pub fn with_characters(mut self, chars: &str) -> Result<Self, SubsetError> {
        // Map characters to glyph IDs using cmap
        use crate::Font;
        use crate::font::MatchingPresentation;
        
        let mut font = Font::new(self.provider.box_clone())?;
        
        for ch in chars.chars() {
            let (glyph_id, _) = font.lookup_glyph_index(
                ch,
                MatchingPresentation::NotRequired,
                None,
            );
            if glyph_id != 0 && !self.glyph_ids.contains(&glyph_id) {
                self.glyph_ids.push(glyph_id);
            }
        }
        
        Ok(self)
    }
    
    pub fn for_pdf(mut self, max_cid: u16) -> Self {
        self.profile = SubsetProfile::Pdf;
        self.pdf_context = Some(PdfFontContext {
            cid_to_gid_map: None,
            max_cid,
            is_cid_font: true,
            writing_mode: WritingMode::Horizontal,
        });
        self.fix_composites = true;  // Auto-enable for PDF
        self
    }
    
    pub fn with_cid_map(mut self, map: &[u16]) -> Self {
        if let Some(ref mut ctx) = self.pdf_context {
            ctx.cid_to_gid_map = Some(map.to_vec());
        }
        self
    }
    
    pub fn fix_composites(mut self, enabled: bool) -> Self {
        self.fix_composites = enabled;
        self
    }
    
    pub fn validation_level(mut self, level: ValidationLevel) -> Self {
        self.validation_level = level;
        self
    }
    
    pub fn with_profile(mut self, profile: SubsetProfile) -> Self {
        self.profile = profile;
        self
    }
    
    pub fn with_cmap_target(mut self, target: CmapTarget) -> Self {
        self.cmap_target = target;
        self
    }
    
    pub fn build(self) -> Result<SubsetResult, SubsetError> {
        // Validate input based on level
        match self.validation_level {
            ValidationLevel::None => {},
            ValidationLevel::Basic => {
                if self.glyph_ids.is_empty() || self.glyph_ids[0] != 0 {
                    return Err(SubsetError::NotDef);
                }
            },
            ValidationLevel::Standard => {
                validate_glyph_ids(&self.glyph_ids, self.provider)?;
            },
            ValidationLevel::Strict => {
                strict_validate_glyph_ids(&self.glyph_ids, self.provider)?;
            },
        }
        
        // Perform subsetting
        let result = if let Some(pdf_ctx) = self.pdf_context {
            // PDF-specific subsetting
            use crate::subset::pdf::subset_for_pdf;
            let pdf_result = subset_for_pdf(
                self.provider,
                &self.glyph_ids,
                &pdf_ctx,
            )?;
            
            // Convert to SubsetResult
            // ... conversion code ...
        } else {
            // Standard subsetting
            use crate::subset::result::subset_detailed;
            subset_detailed(
                self.provider,
                &self.glyph_ids,
                &self.profile,
                self.cmap_target,
            )?
        };
        
        // Fix composites if requested
        if self.fix_composites {
            use crate::subset::composite::update_composite_references;
            let mut data = result.data;
            update_composite_references(&mut data, &result.glyph_mapping)?;
            return Ok(SubsetResult { data, ..result });
        }
        
        Ok(result)
    }
}
```

## Important Implementation Notes

### 1. Error Handling
- Use `SubsetError` for all error returns
- The error type already has `From` implementations for `ParseError`, `WriteError`, etc.
- Use `?` operator for propagating errors

### 2. Import Statements
Common imports needed:
```rust
use crate::Font;  // For character-to-glyph mapping
use crate::font::MatchingPresentation;  // For glyph lookup
use crate::tables::{MaxpTable, FontTableProvider};
use crate::binary::read::ReadScope;
use crate::tag;  // For table tags like tag::MAXP
```

### 3. Testing Requirements
- All tests must pass before proceeding
- Run tests with: `cargo test --test builder`
- Ensure no regression in existing tests:
  - `cargo test --lib` (320 tests)
  - `cargo test --test composite_update` (6 tests)
  - `cargo test --test subset_result` (4 tests)
  - `cargo test --test pdf_subset` (5 tests)
  - `cargo test --test validation` (5 tests)

### 4. Common Pitfalls to Avoid

**Pitfall 1: Missing Documentation**
- All public items need documentation comments
- Enum variants with fields need field documentation
- Use `///` for public items, `//` for internal comments

**Pitfall 2: Borrow Checker Issues**
- When reading font data, collect needed values before mutating
- Use scoped blocks `{ }` to limit borrows
- Example pattern from composite.rs:
```rust
let (glyf_offset, loca_offsets) = {
    let scope = ReadScope::new(&*font_data);
    // ... parse and collect data ...
    (offset, offsets)
};
// Now font_data can be mutated
```

**Pitfall 3: Type Conversions**
- Table offsets are often `u32` but Rust uses `usize`
- Use `as usize` for conversions
- Example: `record.offset as usize`

### 5. Validation Functions to Implement

```rust
fn validate_glyph_ids(
    glyph_ids: &[u16],
    provider: &dyn FontTableProvider,
) -> Result<(), SubsetError> {
    // Check .notdef is first
    if glyph_ids.is_empty() || glyph_ids[0] != 0 {
        return Err(SubsetError::NotDef);
    }
    
    // Check all IDs are valid
    let maxp_data = provider.read_table_data(tag::MAXP)?;
    let maxp = ReadScope::new(&maxp_data).read::<MaxpTable>()?;
    for &gid in glyph_ids {
        if gid >= maxp.num_glyphs {
            return Err(SubsetError::InvalidGlyphId(gid));
        }
    }
    
    Ok(())
}

fn strict_validate_glyph_ids(
    glyph_ids: &[u16],
    provider: &dyn FontTableProvider,
) -> Result<(), SubsetError> {
    // Basic validation first
    validate_glyph_ids(glyph_ids, provider)?;
    
    // Additional strict checks
    // - Check for duplicate glyph IDs
    // - Verify glyphs actually exist in glyf/CFF table
    // - Check composite dependencies are included
    
    Ok(())
}
```

### 6. Integration Testing

After implementation, create an integration test:

```rust
#[test]
fn test_full_pdf_workflow() {
    let font_buffer = common::read_fixture("tests/fonts/opentype/Klei.otf");
    let provider = create_provider(&font_buffer);
    
    // Complete PDF workflow
    let result = SubsetBuilder::new(&provider)
        .with_characters("Hello, World!")
        .unwrap()
        .for_pdf(255)
        .fix_composites(true)
        .validation_level(ValidationLevel::Strict)
        .build();
    
    assert!(result.is_ok());
    let subset = result.unwrap();
    
    // Verify PDF-specific features
    assert!(subset.glyph_mapping.contains_key(&0)); // .notdef
    assert!(subset.stats.size_reduction_percent > 0.0);
}
```

## Success Criteria

The implementation is complete when:

1. ✅ All builder API methods are implemented
2. ✅ All tests in `tests/builder.rs` pass
3. ✅ No regression in existing tests (340+ tests still passing)
4. ✅ Documentation is complete for all public items
5. ✅ Code compiles without warnings
6. ✅ The builder provides access to all Phase 1 & 2 features

## Testing Commands

```bash
# Run builder tests
cargo test --test builder

# Run all tests to ensure no regression
cargo test

# Check for warnings
cargo build --all-features

# Generate documentation
cargo doc --no-deps

# Run clippy for code quality
cargo clippy -- -D warnings
```

## Example Usage After Implementation

```rust
use allsorts::subset::builder::{SubsetBuilder, ValidationLevel};

// Simple subsetting
let subset = SubsetBuilder::new(&provider)
    .with_glyphs(&[0, 1, 2, 3])
    .build()?;

// PDF subsetting with characters
let subset = SubsetBuilder::new(&provider)
    .with_characters("Hello, World!")
    .for_pdf(255)
    .fix_composites(true)
    .build()?;

// Advanced configuration
let subset = SubsetBuilder::new(&provider)
    .with_glyphs(&[0, 10, 20, 30])
    .with_profile(SubsetProfile::Pdf)
    .with_cmap_target(CmapTarget::Unicode)
    .fix_composites(true)
    .validation_level(ValidationLevel::Strict)
    .build()?;
```

## Timeline Estimate

- **Step 1**: Create API signatures (30 minutes)
- **Step 2**: Write comprehensive tests (1 hour)
- **Step 3**: Implement builder methods (2 hours)
- **Step 4**: Fix any failing tests (1 hour)
- **Step 5**: Integration testing (30 minutes)
- **Total**: ~5 hours

## Additional Resources

- Allsorts documentation: https://docs.rs/allsorts/
- OpenType specification: https://docs.microsoft.com/en-us/typography/opentype/spec/
- Rust error handling: https://doc.rust-lang.org/book/ch09-00-error-handling.html
- Builder pattern in Rust: https://rust-unofficial.github.io/patterns/patterns/creational/builder.html

## Conclusion

This plan provides everything needed to implement Phase 3 of the PDF-specific font subsetting features. Follow the TDD methodology strictly, ensure all tests pass, and maintain backward compatibility with existing functionality. The builder pattern will provide a clean, intuitive API that unifies all the powerful subsetting features developed in earlier phases.