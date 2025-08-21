Read @README.md and the @docs/subsetting.md to understand this project and its current subsetting capabilities and APIs and then read @specs/250821_subset_and_map_with_context/README.md to understand the general context of the current upcoming changes and then implement the new changes below:

# Phase 2: API Enhancement & Convenience Wrappers (TDD Implementation)

## Executive Summary

Build on Phase 1's foundation by adding convenience APIs, better error handling, and improved ergonomics. This phase makes the context-aware API easier to use while maintaining backward compatibility.

**Prerequisites**: Phase 1 must be complete with all tests passing.

## TDD Implementation Steps

Following the TDD methodology from `specs/TDD.md`, this phase will be implemented in the following order:

1. **Preparation**: Verify Phase 1 completion and test health
2. **Analysis & Planning**: High-level design and task breakdown
3. **Define APIs & Signatures**: Draft interfaces without implementation
4. **Write Unit Tests (TDD)**: Create failing tests first
5. **Implement Functions Incrementally**: Minimal implementation to pass tests
6. **Full-Suite Integration**: Run all tests and fix regressions
7. **Documentation & Review**: Update docs and review

## 1. Preparation

### 1.1 Verify Phase 1 Dependencies

Before starting Phase 2, ensure Phase 1 is complete:

```bash
# Verify existing tests pass
cargo test subset::context::
cargo test subset::cid_map::

# Check Phase 1 components exist
ls src/subset/context.rs    # Should exist with FontContext, FontEncoding
ls src/subset/cid_map.rs    # Should exist with CID mapping logic
```

Required Phase 1 components:
- `FontEncoding` enum with Identity variants
- `FontContext` enum with PdfType0 variant
- `subset_and_map_with_context` function
- Identity encoding support in context system

### 1.2 Current Test Health Check

```bash
cargo test --all
```

All tests must pass before proceeding. If any fail, stop and fix them first.

## 2. Analysis & Planning

### 2.1 High-Level Design

**Goal**: Add PDF-specific convenience APIs that build on Phase 1's context system.

**Affected Components**:
- New: `src/subset/pdf.rs` - PDF-specific API
- New: `src/subset/builder.rs` - Builder pattern
- Enhanced: `src/subset/error.rs` - Additional error types
- Enhanced: `src/subset/mod.rs` - Public exports

**Data Flow**:
```
User Request → PdfFontContext → FontContext (Phase 1) → SubsetResult → PdfSubsetResult
```

### 2.2 Task Breakdown

1. **PDF Context Structure** - Wrapper around Phase 1 context system
2. **Main PDF API Function** - Single function for PDF use cases
3. **Builder Pattern** - Fluent API for complex configurations
4. **Enhanced Error Types** - Better error messages
5. **Statistics Calculation** - Metrics about subsetting
6. **Integration Layer** - Connects to Phase 1 APIs

### 2.3 Edge Cases & Error Scenarios

- Missing .notdef glyph (glyph_ids[0] != 0)
- Empty glyph lists
- Unsupported encodings
- Font type detection failures
- Invalid context combinations
- Statistics calculation edge cases
- Builder pattern validation

## 3. Define APIs & Signatures

| Component | Inputs | Outputs | Error Cases / Behaviors |
|-----------|--------|---------|------------------------|
| `PdfFontContext::identity_h()` | None | `PdfFontContext` | Cannot fail |
| `PdfFontContext::from_pdf_dict()` | `encoding_name: &str, flags: u32` | `Result<PdfFontContext, SubsetError>` | Invalid encoding name |
| `subset_and_map_for_pdf()` | `provider: &impl FontTableProvider, glyph_ids: &[u16], context: PdfFontContext` | `Result<PdfSubsetResult, SubsetError>` | Missing .notdef, font detection failure |
| `PdfSubsetBuilder::new()` | `provider: &dyn FontTableProvider` | `PdfSubsetBuilder` | Cannot fail |
| `PdfSubsetBuilder::build()` | `self` | `Result<PdfSubsetResult, SubsetError>` | Context validation, subsetting errors |
| `calculate_statistics()` | `provider, subset_data, glyph_count, mapping, cid_map` | `Result<SubsetStatistics, SubsetError>` | Size calculation errors |

### 3.2 Core Data Structures (Signatures Only)

**Note**: These are interface definitions only - no implementation yet.

// PDF-specific font type classification
pub enum PdfFontType {
    Simple,      // Type 1, TrueType
    CidType0,    // CID-keyed font with CFF outlines
    CidType2,    // CID-keyed font with TrueType outlines
}

// Context for PDF font subsetting
pub struct PdfFontContext {
    pub encoding: FontEncoding,
    pub max_cid: Option<u16>,
    pub preserve_identity: bool,
    pub is_symbolic: bool,
}

// Statistics about subsetting operation
pub struct SubsetStatistics {
    pub original_glyph_count: usize,
    pub subset_glyph_count: usize,
    pub original_size_estimate: usize,
    pub subset_size: usize,
    pub cid_map_size: usize,
    pub reduction_percentage: f32,
}

// Result of PDF-specific subsetting
pub struct PdfSubsetResult {
    pub font_data: Vec<u8>,
    pub glyph_mapping: HashMap<u16, u16>,
    pub cid_to_gid_map: Vec<u8>,
    pub font_type: PdfFontType,
    pub encoding_used: FontEncoding,
    pub statistics: SubsetStatistics,
}

// Main PDF API function (signature only)
pub fn subset_and_map_for_pdf(
    provider: &impl FontTableProvider,
    glyph_ids: &[u16],
    pdf_context: PdfFontContext,
) -> Result<PdfSubsetResult, SubsetError>;

// Builder for PDF font subsetting
pub struct PdfSubsetBuilder<'a> {
    provider: &'a dyn FontTableProvider,
    glyph_ids: Vec<u16>,
    encoding: Option<FontEncoding>,
    max_cid: Option<u16>,
    preserve_identity: bool,
    is_symbolic: bool,
}

// Enhanced error types
pub enum SubsetError {
    // ... existing variants from Phase 1 ...
    UnsupportedEncoding(String),
    InvalidContext(String),
    CidGenerationFailed(String),
    FontTypeDetection(String),
}
```

### 3.3 File Structure

```
src/subset/
├── mod.rs                    (existing, enhanced exports)
├── context.rs                (from Phase 1 - required)
├── cid_map.rs               (from Phase 1 - required)
├── pdf.rs                   (new - PDF-specific API)
├── builder.rs               (new - builder pattern)
└── error.rs                 (enhanced error types)
```

## 4. Write Unit Tests (TDD)

**Important**: These tests will FAIL initially. This is expected and correct TDD practice.
Implement tests first, then write minimal code to make them pass.

### 4.1 Test File Setup

```bash
# Create test files
touch src/subset/pdf.rs      # Will implement after tests
touch src/subset/builder.rs  # Will implement after tests
```

### 4.2 PDF Context Tests (WILL FAIL)

```rust
// tests/subset_pdf_context.rs
use allsorts::subset::pdf::{PdfFontContext, SubsetError};
use allsorts::subset::context::FontEncoding;

#[test]
fn test_pdf_context_identity_h_constructor() {
    // WILL FAIL - PdfFontContext::identity_h() not implemented yet
    let ctx = PdfFontContext::identity_h();
    assert_eq!(ctx.encoding, FontEncoding::Identity { vertical: false });
    assert_eq!(ctx.max_cid, None);
    assert!(!ctx.preserve_identity);
    assert!(!ctx.is_symbolic);
}

#[test] 
fn test_pdf_context_identity_v_constructor() {
    // WILL FAIL - PdfFontContext::identity_v() not implemented yet
    let ctx = PdfFontContext::identity_v();
    assert_eq!(ctx.encoding, FontEncoding::Identity { vertical: true });
}

#[test]
fn test_pdf_context_from_pdf_dict_valid() {
    // WILL FAIL - PdfFontContext::from_pdf_dict() not implemented yet
    let result = PdfFontContext::from_pdf_dict("Identity-H", 0);
    assert!(result.is_ok());
    
    let ctx = result.unwrap();
    assert_eq!(ctx.encoding, FontEncoding::Identity { vertical: false });
    assert!(!ctx.is_symbolic);
}

#[test]
fn test_pdf_context_from_pdf_dict_symbolic_flag() {
    // WILL FAIL - symbolic flag detection not implemented yet
    let result = PdfFontContext::from_pdf_dict("Identity-H", 0x04);
    assert!(result.is_ok());
    
    let ctx = result.unwrap();
    assert!(ctx.is_symbolic);  // 0x04 is symbolic flag
}

#[test]
fn test_pdf_context_from_pdf_dict_invalid_encoding() {
    // WILL FAIL - error handling not implemented yet
    let result = PdfFontContext::from_pdf_dict("Unknown-Encoding", 0);
    assert!(result.is_err());
    
    match result {
        Err(SubsetError::UnsupportedEncoding(enc)) => {
            assert_eq!(enc, "Unknown-Encoding");
        }
        _ => panic!("Expected UnsupportedEncoding error"),
    }
}

#[test]
fn test_pdf_context_builder_methods() {
    // WILL FAIL - builder methods not implemented yet
    let ctx = PdfFontContext::identity_h()
        .with_max_cid(1000)
        .preserve_identity();
    
    assert_eq!(ctx.max_cid, Some(1000));
    assert!(ctx.preserve_identity);
}
```

### 4.3 Main PDF API Tests (WILL FAIL)

```rust
// tests/subset_pdf_api.rs
use allsorts::subset::pdf::{
    subset_and_map_for_pdf, PdfFontContext, PdfSubsetResult, PdfFontType
};
use allsorts::subset::SubsetError;
use allsorts::tests::create_test_provider;

#[test]
fn test_subset_and_map_for_pdf_simple_case() {
    // WILL FAIL - subset_and_map_for_pdf() not implemented yet
    let provider = create_test_provider();
    let context = PdfFontContext::identity_h();
    
    let result = subset_and_map_for_pdf(
        &provider,
        &[0, 1, 2, 3],  // .notdef + 3 glyphs
        context,
    );
    
    assert!(result.is_ok());
    let pdf_result = result.unwrap();
    
    assert!(!pdf_result.font_data.is_empty());
    assert_eq!(pdf_result.glyph_mapping.len(), 4);
    assert!(pdf_result.glyph_mapping.contains_key(&0)); // .notdef mapped
    assert!(pdf_result.statistics.reduction_percentage >= 0.0);
}

#[test]
fn test_subset_and_map_for_pdf_missing_notdef_error() {
    // WILL FAIL - validation not implemented yet
    let provider = create_test_provider();
    let context = PdfFontContext::identity_h();
    
    let result = subset_and_map_for_pdf(
        &provider,
        &[1, 2, 3],  // Missing .notdef (glyph 0)
        context,
    );
    
    assert!(result.is_err());
    match result {
        Err(SubsetError::InvalidContext(msg)) => {
            assert!(msg.contains(".notdef"));
        }
        _ => panic!("Expected InvalidContext error"),
    }
}

#[test]
fn test_subset_and_map_for_pdf_empty_glyphs_error() {
    // WILL FAIL - validation not implemented yet
    let provider = create_test_provider();
    let context = PdfFontContext::identity_h();
    
    let result = subset_and_map_for_pdf(&provider, &[], context);
    assert!(result.is_err());
}

#[test]
fn test_pdf_subset_result_methods() {
    // WILL FAIL - PdfSubsetResult methods not implemented yet
    let provider = create_test_provider();
    let context = PdfFontContext::identity_h();
    
    let result = subset_and_map_for_pdf(&provider, &[0, 42], context).unwrap();
    
    // Test convenience methods
    assert!(result.size_reduction() >= 0.0);
    assert!(result.is_cid_font()); // Should be CID for Identity encoding
    assert!(result.cid_to_gid_map().is_some());
}
```

### 4.4 Builder Pattern Tests (WILL FAIL)

```rust
// tests/subset_pdf_builder.rs
use allsorts::subset::pdf::{subset_for_pdf, PdfSubsetBuilder};
use allsorts::subset::context::FontEncoding;
use allsorts::tests::create_test_provider;

#[test]
fn test_builder_basic_construction() {
    // WILL FAIL - PdfSubsetBuilder not implemented yet
    let provider = create_test_provider();
    let builder = PdfSubsetBuilder::new(&provider);
    
    // Should start with .notdef
    assert!(builder.glyph_ids.contains(&0));
}

#[test]
fn test_builder_with_glyphs() {
    // WILL FAIL - with_glyphs method not implemented yet
    let provider = create_test_provider();
    
    let result = subset_for_pdf(&provider)
        .with_glyphs(&[42, 43, 44])
        .identity_h()
        .build();
    
    assert!(result.is_ok());
    let pdf_result = result.unwrap();
    
    // Should include .notdef + requested glyphs
    assert!(pdf_result.glyph_mapping.len() >= 4);
    assert!(pdf_result.glyph_mapping.contains_key(&0));  // .notdef
    assert!(pdf_result.glyph_mapping.contains_key(&42));
}

#[test]
fn test_builder_with_text() {
    // WILL FAIL - with_text method not implemented yet
    let provider = create_test_provider();
    
    let result = subset_for_pdf(&provider)
        .with_text("ABC")
        .expect("Text mapping should work")
        .identity_h()
        .build();
    
    assert!(result.is_ok());
    let pdf_result = result.unwrap();
    
    // Should have glyphs for A, B, C plus .notdef
    assert!(pdf_result.glyph_mapping.len() >= 4);
}

#[test]
fn test_builder_fluent_api() {
    // WILL FAIL - fluent methods not implemented yet
    let provider = create_test_provider();
    
    let result = subset_for_pdf(&provider)
        .with_glyphs(&[10, 20, 30])
        .identity_v()                    // Vertical encoding
        .with_max_cid(5000)
        .preserve_identity()
        .symbolic()
        .build();
    
    assert!(result.is_ok());
    let pdf_result = result.unwrap();
    
    assert_eq!(pdf_result.encoding_used, FontEncoding::Identity { vertical: true });
}

#[test]
fn test_builder_default_encoding() {
    // WILL FAIL - default behavior not implemented yet
    let provider = create_test_provider();
    
    let result = subset_for_pdf(&provider)
        .with_glyphs(&[1, 2])
        .build();  // No explicit encoding - should default to Identity-H
    
    assert!(result.is_ok());
    let pdf_result = result.unwrap();
    
    assert_eq!(pdf_result.encoding_used, FontEncoding::Identity { vertical: false });
}
```

### 4.5 Statistics Tests (WILL FAIL)

```rust
// tests/subset_pdf_statistics.rs
use allsorts::subset::pdf::{subset_and_map_for_pdf, PdfFontContext, SubsetStatistics};
use allsorts::tests::create_test_provider;

#[test]
fn test_statistics_calculation() {
    // WILL FAIL - statistics calculation not implemented yet
    let provider = create_test_provider();
    let context = PdfFontContext::identity_h();
    
    let result = subset_and_map_for_pdf(&provider, &[0, 1, 2], context).unwrap();
    let stats = &result.statistics;
    
    assert!(stats.original_glyph_count > 0);
    assert_eq!(stats.subset_glyph_count, 3);
    assert!(stats.original_size_estimate > 0);
    assert!(stats.subset_size > 0);
    assert!(stats.reduction_percentage >= 0.0);
    assert!(stats.reduction_percentage <= 100.0);
}

#[test]
fn test_statistics_with_cid_map() {
    // WILL FAIL - CID map size tracking not implemented yet
    let provider = create_test_provider();
    let context = PdfFontContext::identity_h();
    
    let result = subset_and_map_for_pdf(&provider, &[0, 42, 100], context).unwrap();
    let stats = &result.statistics;
    
    if result.is_cid_font() {
        assert!(stats.cid_map_size > 0);
    } else {
        assert_eq!(stats.cid_map_size, 0);
    }
}
```

### 4.6 Error Handling Tests (WILL FAIL)

```rust
// tests/subset_pdf_errors.rs
use allsorts::subset::pdf::{subset_and_map_for_pdf, PdfFontContext};
use allsorts::subset::SubsetError;
use allsorts::tests::create_test_provider;

#[test]
fn test_enhanced_error_messages() {
    // WILL FAIL - enhanced error Display not implemented yet
    let error = SubsetError::UnsupportedEncoding("CustomEncoding".to_string());
    let msg = error.to_string();
    
    assert!(msg.contains("not supported"));
    assert!(msg.contains("CustomEncoding"));
    assert!(msg.contains("Identity-H"));  // Should suggest alternatives
}

#[test]
fn test_invalid_context_error() {
    // WILL FAIL - context validation not implemented yet
    let provider = create_test_provider();
    let context = PdfFontContext::identity_h();
    
    let result = subset_and_map_for_pdf(&provider, &[5, 10], context);
    
    match result {
        Err(SubsetError::InvalidContext(msg)) => {
            assert!(msg.contains(".notdef"));
        }
        _ => panic!("Expected InvalidContext error for missing .notdef"),
    }
}

#[test]
fn test_font_type_detection_error() {
    // WILL FAIL - font type detection not implemented yet
    // This test requires a malformed provider that fails detection
    // Implementation will define the exact conditions
}
```

### 4.7 Integration Tests (WILL FAIL)

```rust
// tests/subset_pdf_integration.rs
use allsorts::subset::pdf::{subset_and_map_for_pdf, subset_for_pdf, PdfFontContext};
use allsorts::subset::context::FontEncoding;
use allsorts::tests::{create_test_provider, TestFontProvider};

#[test]
fn test_phase1_integration() {
    // WILL FAIL - integration with Phase 1 not implemented yet
    let provider = create_test_provider();
    let context = PdfFontContext::identity_h();
    
    let result = subset_and_map_for_pdf(&provider, &[0, 1, 2], context).unwrap();
    
    // Verify it uses Phase 1 context system internally
    assert!(result.encoding_used == FontEncoding::Identity { vertical: false });
    
    // Verify CID mapping is consistent with Phase 1
    if result.is_cid_font() {
        assert!(!result.cid_to_gid_map.is_empty());
    }
}

#[test]
fn test_different_font_types() {
    // WILL FAIL - font type detection not implemented yet
    // Test with different providers (CFF, TrueType, etc.)
    // Will require different test providers for each font type
}
```

## 5. Implement Functions Incrementally

**TDD Rule**: Implement only the minimal code needed to make the NEXT failing test pass.

### 5.1 Implementation Order

1. **Phase 1 Verification** - Ensure all dependencies exist
2. **Basic Data Structures** - Minimal struct definitions
3. **Simple Constructors** - `identity_h()`, `identity_v()`
4. **Context Validation** - Error handling for invalid inputs
5. **Main API Function** - Core subsetting logic
6. **Statistics Calculation** - Size and reduction metrics
7. **Builder Pattern** - Fluent API implementation
8. **Enhanced Errors** - Better error messages

### 5.2 Verification Steps for Each Implementation

```bash
# After implementing each component:
cargo test subset::pdf::<component_name> --verbose

# Example:
cargo test subset::pdf::test_pdf_context_identity_h_constructor
```

**Success Criteria**: Each test should go from FAILING to PASSING as you implement the minimal code.

**Implementation Notes**:
- Follow the exact test specifications above
- Each function should have minimal implementation to pass its tests
- Don't implement features not covered by tests
- Use `todo!()` or `unimplemented!()` for placeholder implementations initially

### 6.1 PDF Context Implementation (`src/subset/pdf.rs`)

**Start with minimal implementation to pass constructor tests:**

```rust
// Minimal implementation to pass initial tests
impl PdfFontContext {
    pub fn identity_h() -> Self {
        // TODO: Implement to pass test_pdf_context_identity_h_constructor
        todo!("Implement after test is written")
    }
    
    pub fn identity_v() -> Self {
        // TODO: Implement to pass test_pdf_context_identity_v_constructor  
        todo!("Implement after test is written")
    }
    
    pub fn from_pdf_dict(encoding_name: &str, flags: u32) -> Result<Self, SubsetError> {
        // TODO: Implement to pass test_pdf_context_from_pdf_dict_valid
        todo!("Implement after test is written")
    }
    
    pub fn with_max_cid(self, max_cid: u16) -> Self {
        // TODO: Implement to pass test_pdf_context_builder_methods
        todo!("Implement after test is written")
    }
    
    pub fn preserve_identity(self) -> Self {
        // TODO: Implement to pass test_pdf_context_builder_methods
        todo!("Implement after test is written")
    }
}

impl PdfSubsetResult {
    pub fn size_reduction(&self) -> f32 {
        // TODO: Implement to pass test_pdf_subset_result_methods
        todo!("Implement after test is written")
    }
    
    pub fn is_cid_font(&self) -> bool {
        // TODO: Implement to pass test_pdf_subset_result_methods
        todo!("Implement after test is written")
    }
    
    pub fn cid_to_gid_map(&self) -> Option<&[u8]> {
        // TODO: Implement to pass test_pdf_subset_result_methods
        todo!("Implement after test is written")
    }
}
```

### 6.2 Main PDF API Implementation

**Implement incrementally to pass tests one by one:**

```rust
pub fn subset_and_map_for_pdf(
    provider: &impl FontTableProvider,
    glyph_ids: &[u16],
    pdf_context: PdfFontContext,
) -> Result<PdfSubsetResult, SubsetError> {
    // TODO: Implement validation to pass test_subset_and_map_for_pdf_missing_notdef_error
    // TODO: Implement main logic to pass test_subset_and_map_for_pdf_simple_case
    todo!("Implement after tests are written")
}

fn detect_pdf_font_type(provider: &impl FontTableProvider) -> Result<PdfFontType, SubsetError> {
    // TODO: Implement to pass test_different_font_types
    todo!("Implement after tests are written")
}

fn calculate_statistics(
    provider: &impl FontTableProvider,
    subset_data: &[u8],
    requested_glyphs: usize,
    glyph_mapping: &HashMap<u16, u16>,
    cid_map: &[u8],
) -> Result<SubsetStatistics, SubsetError> {
    // TODO: Implement to pass test_statistics_calculation
    todo!("Implement after tests are written")
}
```

### 6.3 Builder Pattern Implementation (`src/subset/builder.rs`)

**Start with minimal builder to pass basic tests:**

```rust
impl<'a> PdfSubsetBuilder<'a> {
    pub fn new(provider: &'a dyn FontTableProvider) -> Self {
        // TODO: Implement to pass test_builder_basic_construction
        todo!("Implement after test is written")
    }
    
    pub fn with_glyphs(self, glyph_ids: &[u16]) -> Self {
        // TODO: Implement to pass test_builder_with_glyphs
        todo!("Implement after test is written")
    }
    
    pub fn with_text(self, text: &str) -> Result<Self, SubsetError> {
        // TODO: Implement to pass test_builder_with_text
        todo!("Implement after test is written")
    }
    
    pub fn identity_h(self) -> Self {
        // TODO: Implement to pass test_builder_fluent_api
        todo!("Implement after test is written")
    }
    
    pub fn identity_v(self) -> Self {
        // TODO: Implement to pass test_builder_fluent_api
        todo!("Implement after test is written")
    }
    
    pub fn with_max_cid(self, max_cid: u16) -> Self {
        // TODO: Implement to pass test_builder_fluent_api
        todo!("Implement after test is written")
    }
    
    pub fn preserve_identity(self) -> Self {
        // TODO: Implement to pass test_builder_fluent_api
        todo!("Implement after test is written")
    }
    
    pub fn symbolic(self) -> Self {
        // TODO: Implement to pass test_builder_fluent_api
        todo!("Implement after test is written")
    }
    
    pub fn build(self) -> Result<PdfSubsetResult, SubsetError> {
        // TODO: Implement to pass test_builder_default_encoding
        todo!("Implement after test is written")
    }
}

pub fn subset_for_pdf(provider: &dyn FontTableProvider) -> PdfSubsetBuilder {
    // TODO: Implement convenience function
    todo!("Implement after tests are written")
}
```

### 6.4 Enhanced Error Implementation (`src/subset/error.rs`)

**Add new error variants and improved Display:**

```rust
// Add new error variants to existing SubsetError enum
// TODO: Implement to pass test_enhanced_error_messages
// TODO: Implement Display trait to pass error message tests

// New variants to add:
// UnsupportedEncoding(String),
// InvalidContext(String), 
// CidGenerationFailed(String),
// FontTypeDetection(String),

// TODO: Implement Display with helpful error messages
```

## 7. Full-Suite Integration

### 7.1 Integration Test Strategy

```bash
# Run all tests after each implementation milestone
cargo test --all

# Check Phase 1 integration specifically
cargo test subset::context::
cargo test subset::cid_map::

# Check Phase 2 new functionality
cargo test subset::pdf::
cargo test subset::builder::
```

### 7.2 Regression Prevention

- All Phase 1 tests must continue to pass
- No breaking changes to existing APIs
- Performance should not degrade significantly

## 8. Documentation & Review

### 8.1 Usage Examples

**After implementation is complete, add these examples:**

```rust
// Example 1: Simple Identity-H case
let result = subset_and_map_for_pdf(
    &provider,
    &glyph_ids,
    PdfFontContext::identity_h(),
)?;

// Example 2: With custom settings
let context = PdfFontContext::identity_h()
    .with_max_cid(65535)
    .preserve_identity();
    
let result = subset_and_map_for_pdf(&provider, &glyph_ids, context)?;

// Example 3: Using builder for text
let result = subset_for_pdf(&provider)
    .with_text("Sample text for subsetting")?
    .identity_h()
    .build()?;

// Example 4: From PDF dictionary
let context = PdfFontContext::from_pdf_dict(&pdf_font.encoding, pdf_font.flags)?;
let result = subset_and_map_for_pdf(&provider, &used_glyphs, context)?;
```

### 8.2 Performance Metrics

- Document performance impact vs Phase 1 baseline
- Include statistics calculation overhead
- Measure builder pattern efficiency

## 9. TDD Success Criteria

### 9.1 Test-Driven Requirements
- ✅ All tests written BEFORE implementation
- ✅ Tests initially FAIL as expected
- ✅ Minimal implementation makes tests pass
- ✅ No implementation without corresponding tests
- ✅ Full test suite passes after completion

### 9.2 Functional Requirements
- ✅ `subset_and_map_for_pdf` works for all Identity encodings
- ✅ Builder pattern supports common workflows
- ✅ Statistics provide useful metrics
- ✅ Error messages are actionable
- ✅ Phase 1 integration works correctly

### 9.3 Quality Metrics
- Test coverage > 95% for new code (higher due to TDD)
- All tests pass consistently
- Performance overhead < 5% vs Phase 1
- Zero regressions in Phase 1 functionality

## 10. Dependencies & Prerequisites

### 10.1 Phase 1 Requirements

**CRITICAL**: These must exist and be tested before starting Phase 2:

- `FontEncoding` enum with Identity variants
- `FontContext` enum with PdfType0 variant  
- `subset_and_map_with_context` function
- Identity encoding support in context system
- All Phase 1 tests passing

### 10.2 Verification Commands

```bash
# Verify Phase 1 completion
cargo test subset::context::test_font_context_pdf_type0
cargo test subset::context::test_identity_encoding
cargo test subset::cid_map::

# Ensure clean baseline
cargo test --all
```

## 11. Implementation Timeline (TDD)

| Step | Duration | TDD Activity | Verification |
|------|----------|--------------|-------------|
| **1. Test Setup** | 2 hours | Write all failing tests | `cargo test` shows expected failures |
| **2. Data Structures** | 1 hour | Minimal struct definitions | Constructor tests pass |
| **3. Context API** | 2 hours | PDF context implementation | Context tests pass |
| **4. Main API** | 3 hours | Core subsetting function | Main API tests pass |
| **5. Builder Pattern** | 3 hours | Fluent API implementation | Builder tests pass |
| **6. Statistics** | 2 hours | Metrics calculation | Statistics tests pass |
| **7. Error Handling** | 2 hours | Enhanced error types | Error tests pass |
| **8. Integration** | 1 hour | Phase 1 integration | All tests pass |
| **9. Documentation** | 1 hour | Update examples | Examples compile |
| **Total** | **17 hours** | Following strict TDD | Comprehensive test coverage |

## 12. TDD Implementation Checklist

### Phase 2 TDD Checklist

**Before Starting**
- [ ] All Phase 1 tests passing
- [ ] Phase 1 dependencies verified
- [ ] Test files created with failing tests

**During Implementation**
- [ ] PDF Context constructors (`identity_h`, `identity_v`, `from_pdf_dict`)
- [ ] PDF Context builder methods (`with_max_cid`, `preserve_identity`)
- [ ] Main API validation (missing .notdef, empty arrays)
- [ ] Main API core functionality (subsetting with context)
- [ ] Statistics calculation (size metrics, reduction percentage)
- [ ] Builder pattern basic construction
- [ ] Builder pattern fluent methods
- [ ] Builder pattern text support
- [ ] Enhanced error types and messages
- [ ] Font type detection
- [ ] CID mapping integration

**Testing Milestones**
- [ ] Each test goes from FAILING → PASSING
- [ ] No test skipped or ignored
- [ ] No implementation without test
- [ ] Full suite passes after each milestone

**Integration & Review**
- [ ] Phase 1 tests still pass
- [ ] Performance acceptable
- [ ] Documentation updated
- [ ] Examples compile and work
- [ ] Ready for Phase 3

**Success Validation**
- [ ] `cargo test --all` passes
- [ ] All usage examples work
- [ ] No breaking changes
- [ ] Statistics provide useful data
- [ ] Error messages are helpful

---

*Document Version: 2.0 - TDD Methodology*  
*Updated: 2024-08-21*  
*Phase: 2 of 5*  
*Priority: HIGH - Foundation for remaining phases*  
*Methodology: Test-Driven Development*