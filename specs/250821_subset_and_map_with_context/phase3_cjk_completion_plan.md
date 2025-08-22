Read @README.md and the @docs/subsetting.md to understand this project and its current subsetting capabilities and APIs and then implement the new changes below:

# Phase 3 CJK Support Completion Plan

## Executive Summary

Phase 3 CJK support is currently **50% complete**. While CJK encoding detection and the `build_cjk_cid_map()` function exist with passing tests, they are **not integrated** into the main subsetting pipeline. The main entry point (`build_cid_to_gid_map_for_encoding`) returns an error for CJK encodings instead of calling the implemented functionality.

This plan completes Phase 3 by connecting the existing CJK implementation to the main pipeline and ensuring end-to-end CJK subsetting works.

## Current State Analysis

### ✅ What's Complete
1. **CJK Type Definitions** (`src/subset/context.rs`)
   - `CJKLanguage` enum with Chinese/Japanese/Korean variants
   - `FontEncoding::CJK` variant with all fields
   - `FontEncoding::from_pdf_name()` correctly detects CJK encodings

2. **CJK Implementation** (`src/subset/cjk/`)
   - `build_cjk_cid_map()` function exists
   - Language-specific modules (chinese.rs, japanese.rs, korean.rs)
   - CMap provider interfaces
   - 18 unit tests passing in `tests/subset/cjk_encodings.rs`

### ❌ What's Missing
1. **Pipeline Integration** (`src/subset/cid_map.rs`)
   - `build_cid_to_gid_map_for_encoding()` returns error for CJK
   - Never calls the existing `build_cjk_cid_map()` function

2. **End-to-End Testing**
   - No integration tests that verify CJK subsetting works through the main APIs
   - Auto-detection tests skip CJK (using Identity encoding instead)

## TDD Implementation Plan

### Step 1: Preparation (30 minutes)

#### 1.1 Verify Current Test Health
```bash
# Run all existing tests to ensure clean baseline
cargo test --lib
cargo test --test cjk_encodings
cargo test subset::detection

# Expected: All pass except integration tests using CJK
```

#### 1.2 Document Current Failure Points
```bash
# Create a test that demonstrates the current failure
cargo test --test test_cjk_integration_failure 2>&1 | grep "not yet implemented"
```

### Step 2: Analysis & Planning (1 hour)

#### 2.1 Component Analysis

| Component | Current State | Required Change |
|-----------|--------------|-----------------|
| `build_cid_to_gid_map_for_encoding()` | Returns error for CJK | Call `build_cjk_cid_map()` |
| `build_cjk_cid_map()` | Implemented, tested | No change needed |
| CMap providers | Implemented | Verify data loading works |
| Integration tests | Use Identity encoding | Update to test CJK |

#### 2.2 Data Flow

```
User calls auto_subset_for_pdf() with CJK font
    ↓
AutoSubsetBuilder::build()
    ↓
subset_and_map_for_pdf()
    ↓
subset_and_map_with_context()
    ↓
build_cid_to_gid_map_for_encoding() ← FAILS HERE
    ↓ (should call)
build_cjk_cid_map() ← EXISTS BUT NOT CALLED
```

#### 2.3 Edge Cases to Test
- CJK encoding without CMap provider
- CJK encoding with invalid CMap data
- Mixed CJK/ASCII glyphs
- Large CID ranges (>10000)
- Vertical vs horizontal CJK text

### Step 3: Define APIs & Signatures (30 minutes)

No new APIs needed. We're connecting existing components:

| Function | Current Behavior | New Behavior |
|----------|-----------------|--------------|
| `build_cid_to_gid_map_for_encoding(FontEncoding::CJK)` | Returns `Err(UnsupportedEncoding)` | Calls `build_cjk_cid_map()` |

### Step 4: Write Unit Tests (TDD) (2 hours)

#### 4.1 Create Integration Test File
```rust
// tests/subset/cjk_integration.rs

use allsorts::subset::context::{FontEncoding, CJKLanguage, ChineseVariant};
use allsorts::subset::pdf::{subset_and_map_for_pdf, PdfFontContext};
use allsorts::subset::auto::auto_subset_for_pdf;
use allsorts::subset::cjk::BuiltinCMapProvider;
use allsorts::binary::read::ReadScope;
use allsorts::tables::OpenTypeFont;

// WILL FAIL - CJK not integrated
#[test]
fn test_chinese_gb_subsetting_end_to_end() {
    let font_data = include_bytes!("../../tests/font_specimen/fonts/SymbolTest-Regular.ttf");
    let scope = ReadScope::new(font_data);
    let font_file = scope.read::<OpenTypeFont>().unwrap();
    let provider = font_file.table_provider(0).unwrap();
    
    let mut context = PdfFontContext::from_pdf_dict("GB-EUC-H", 0).unwrap();
    context = context.with_max_cid(100);
    let cmap_provider = Box::new(BuiltinCMapProvider::new());
    context = context.with_cmap_provider(cmap_provider);
    
    let result = subset_and_map_for_pdf(&provider, &[0, 1, 2], context);
    
    // Should succeed, not return "not yet implemented"
    assert!(result.is_ok(), "CJK subsetting should work");
    
    let pdf_result = result.unwrap();
    assert!(pdf_result.cid_to_gid_map.len() > 0);
    assert!(matches!(pdf_result.encoding_used, FontEncoding::CJK { .. }));
}

// WILL FAIL - CJK not integrated
#[test]
fn test_japanese_subsetting_end_to_end() {
    let font_data = include_bytes!("../../tests/font_specimen/fonts/SymbolTest-Regular.ttf");
    let scope = ReadScope::new(font_data);
    let font_file = scope.read::<OpenTypeFont>().unwrap();
    let provider = font_file.table_provider(0).unwrap();
    
    let mut context = PdfFontContext::from_pdf_dict("90ms-RKSJ-H", 0).unwrap();
    context = context.with_max_cid(200);
    let cmap_provider = Box::new(BuiltinCMapProvider::new());
    context = context.with_cmap_provider(cmap_provider);
    
    let result = subset_and_map_for_pdf(&provider, &[0, 1, 2, 3], context);
    assert!(result.is_ok(), "Japanese subsetting should work");
}

// WILL FAIL - CJK not integrated
#[test]
fn test_korean_subsetting_end_to_end() {
    let font_data = include_bytes!("../../tests/font_specimen/fonts/SymbolTest-Regular.ttf");
    let scope = ReadScope::new(font_data);
    let font_file = scope.read::<OpenTypeFont>().unwrap();
    let provider = font_file.table_provider(0).unwrap();
    
    let mut context = PdfFontContext::from_pdf_dict("KSCms-UHC-H", 0).unwrap();
    context = context.with_max_cid(150);
    let cmap_provider = Box::new(BuiltinCMapProvider::new());
    context = context.with_cmap_provider(cmap_provider);
    
    let result = subset_and_map_for_pdf(&provider, &[0, 1, 2], context);
    assert!(result.is_ok(), "Korean subsetting should work");
}

// WILL FAIL - CJK not integrated
#[test]
fn test_cjk_auto_detection_and_subsetting() {
    let font_data = include_bytes!("../../tests/font_specimen/fonts/SymbolTest-Regular.ttf");
    let scope = ReadScope::new(font_data);
    let font_file = scope.read::<OpenTypeFont>().unwrap();
    let provider = font_file.table_provider(0).unwrap();
    
    // Provide PDF info with CJK encoding
    let pdf_info = allsorts::subset::detection::PdfFontInfo {
        encoding_name: Some("GB-EUC-H".to_string()),
        ..Default::default()
    };
    
    let result = auto_subset_for_pdf(&provider)
        .with_glyphs(&[0, 1, 2])
        .with_pdf_info(pdf_info)
        .build();
    
    assert!(result.is_ok(), "Auto-detection with CJK should work");
    
    let auto_result = result.unwrap();
    assert!(matches!(auto_result.detection.encoding, FontEncoding::CJK { .. }));
}

// WILL FAIL - CJK not integrated
#[test]
fn test_cjk_without_cmap_provider_fails_gracefully() {
    let font_data = include_bytes!("../../tests/font_specimen/fonts/SymbolTest-Regular.ttf");
    let scope = ReadScope::new(font_data);
    let font_file = scope.read::<OpenTypeFont>().unwrap();
    let provider = font_file.table_provider(0).unwrap();
    
    let mut context = PdfFontContext::from_pdf_dict("GB-EUC-H", 0).unwrap();
    context = context.with_max_cid(100);
    // Note: No CMap provider set
    
    let result = subset_and_map_for_pdf(&provider, &[0, 1, 2], context);
    
    // Should fail with appropriate error
    assert!(result.is_err());
    let err = result.unwrap_err();
    assert!(err.to_string().contains("CMap") || err.to_string().contains("required"));
}

// WILL FAIL - CJK not integrated
#[test]
fn test_cjk_vertical_text_handling() {
    let font_data = include_bytes!("../../tests/font_specimen/fonts/SymbolTest-Regular.ttf");
    let scope = ReadScope::new(font_data);
    let font_file = scope.read::<OpenTypeFont>().unwrap();
    let provider = font_file.table_provider(0).unwrap();
    
    // Vertical Chinese text
    let mut context = PdfFontContext::from_pdf_dict("GB-EUC-V", 0).unwrap();
    context = context.with_max_cid(100);
    let cmap_provider = Box::new(BuiltinCMapProvider::new());
    context = context.with_cmap_provider(cmap_provider);
    
    let result = subset_and_map_for_pdf(&provider, &[0, 1, 2], context);
    assert!(result.is_ok(), "Vertical CJK should work");
    
    let pdf_result = result.unwrap();
    assert!(matches!(pdf_result.encoding_used, 
        FontEncoding::CJK { vertical: true, .. }));
}
```

#### 4.2 Run Tests to Verify They Fail
```bash
cargo test --test cjk_integration 2>&1 | grep "FAILED"
# Expected: All 6 tests fail with "not yet implemented" error
```

### Step 5: Implement Functions Incrementally (2 hours)

#### 5.1 Fix `build_cid_to_gid_map_for_encoding()`

```rust
// src/subset/cid_map.rs

pub fn build_cid_to_gid_map_for_encoding(
    encoding: &FontEncoding,
    glyph_mapping: &HashMap<u16, u16>,
    max_cid: u16,
) -> Result<Vec<u8>, SubsetError> {
    match encoding {
        FontEncoding::Identity { .. } => {
            Ok(build_identity_cid_map(glyph_mapping, max_cid))
        }
        FontEncoding::CJK { .. } => {
            // CHANGE: Call the existing implementation instead of returning error
            use crate::subset::cjk::build_cjk_cid_map;
            
            // Note: We need a CMap provider for some CJK encodings
            // This should be passed through the context
            // For now, use the builtin provider as default
            let provider = crate::subset::cjk::BuiltinCMapProvider::new();
            build_cjk_cid_map(encoding, glyph_mapping, max_cid, Some(&provider))
        }
        FontEncoding::AdobeCollection { .. } => {
            // Adobe collections will be handled similarly to CJK
            Err(SubsetError::UnsupportedEncoding("Adobe collections not yet implemented".to_string()))
        }
        FontEncoding::Custom(_) => {
            // Custom encodings are not supported for CID mapping
            Err(SubsetError::UnsupportedEncoding("Custom encodings not supported".to_string()))
        }
    }
}
```

#### 5.2 Update to Pass CMap Provider Through Context

```rust
// src/subset/pdf.rs - Update generate_cid_to_gid_map function

fn generate_cid_to_gid_map(
    context: &PdfFontContext,
    mapping: &HashMap<u16, u16>,
) -> Result<(Vec<u8>, ValidationResult), SubsetError> {
    let max_cid = context.max_cid.unwrap_or(255);
    
    // Use encoding-specific CID map generation
    let cid_map = match &context.encoding {
        FontEncoding::CJK { .. } => {
            // Use CJK-aware CID mapping with CMap provider
            use crate::subset::cjk::build_cjk_cid_map;
            build_cjk_cid_map(
                &context.encoding,
                mapping,
                max_cid,
                context.cmap_provider.as_deref(),  // Pass the provider
            )?
        }
        _ => {
            // Use standard CID map generation for Identity and other encodings
            use crate::subset::cid_map::build_cid_to_gid_map_for_encoding;
            build_cid_to_gid_map_for_encoding(&context.encoding, mapping, max_cid)?
        }
    };
    
    // ... rest of validation
}
```

#### 5.3 Run Tests Incrementally
```bash
# Test just the basic integration
cargo test --test cjk_integration test_chinese_gb_subsetting_end_to_end

# If passes, test Japanese
cargo test --test cjk_integration test_japanese_subsetting_end_to_end

# Continue with each test...
```

### Step 6: Full-Suite Integration (1 hour)

#### 6.1 Run All Tests
```bash
# Run all CJK tests
cargo test cjk

# Run all subset tests
cargo test subset

# Run full test suite
cargo test
```

#### 6.2 Fix Any Regressions
- Check that Identity encoding still works
- Verify auto-detection doesn't break
- Ensure Phase 4 detection tests still pass

#### 6.3 Update Broken Tests
Update tests that were changed to avoid CJK:

```rust
// src/subset/detection/tests/auto_builder_tests.rs
// Restore the original CJK test now that it works

#[test]
fn test_auto_builder_override_encoding() {
    let override_encoding = FontEncoding::CJK {
        language: CJKLanguage::Japanese(JapaneseVariant::Unicode),
        encoding_name: "UniJIS-UTF16-H".to_string(),
        vertical: false,
        requires_cmap_data: false,
    };
    
    let provider = create_test_provider();
    let result = auto_subset_for_pdf(provider.as_ref())
        .with_glyphs(&[0, 1, 2])
        .override_encoding(override_encoding.clone())
        .build()
        .unwrap();
    
    assert_eq!(result.detection.encoding, override_encoding);
    assert_eq!(result.detection.confidence, DetectionConfidence::Certain);
}
```

### Step 7: Documentation & Review (1 hour)

#### 7.1 Update Documentation

**docs/subsetting.md:**
- Update Phase 3 section to show CJK is fully working
- Add examples of CJK subsetting
- Document CMap provider usage

**specs/250821_subset_and_map_with_context/README.md:**
- Mark Phase 3 as 100% complete
- Note that CJK subsetting is now functional

#### 7.2 Update CHANGELOG
```markdown
### Fixed
- CJK font subsetting now works end-to-end (was returning "not implemented" error)
- Connected existing CJK implementation to main subsetting pipeline
```

#### 7.3 Add Integration Examples
```rust
// examples/cjk_subsetting.rs
// Complete example showing CJK font subsetting for PDF
```

## Success Criteria

### ✅ All Tests Pass
- [ ] 18 existing CJK unit tests still pass
- [ ] 6 new CJK integration tests pass  
- [ ] No regressions in other tests
- [ ] Full test suite green

### ✅ End-to-End CJK Works
- [ ] Chinese (GB-EUC-H) subsetting works
- [ ] Japanese (90ms-RKSJ-H) subsetting works
- [ ] Korean (KSCms-UHC-H) subsetting works
- [ ] Vertical text variants work
- [ ] Auto-detection with CJK works

### ✅ Error Handling
- [ ] Missing CMap provider gives clear error
- [ ] Invalid CMap data handled gracefully
- [ ] Unsupported CJK variants documented

## Timeline

| Step | Duration | Cumulative |
|------|----------|------------|
| Preparation | 30 min | 30 min |
| Analysis & Planning | 1 hour | 1.5 hours |
| Define APIs | 30 min | 2 hours |
| Write Tests | 2 hours | 4 hours |
| Implementation | 2 hours | 6 hours |
| Integration | 1 hour | 7 hours |
| Documentation | 1 hour | 8 hours |

**Total: 1 working day**

## Risk Mitigation

| Risk | Mitigation |
|------|------------|
| CMap data not available | Use simplified identity mapping as fallback |
| Performance impact | Cache CMap parsing results |
| Breaking changes | Keep existing error for Adobe collections |
| Test font limitations | Document that tests use simplified mappings |

## Implementation Checklist

- [ ] Verify all existing tests pass before starting
- [ ] Write 6 CJK integration tests (all fail initially)
- [ ] Update `build_cid_to_gid_map_for_encoding()` to call CJK implementation
- [ ] Update `generate_cid_to_gid_map()` to pass CMap provider
- [ ] Run tests incrementally, fixing issues
- [ ] Update previously modified tests to use CJK again
- [ ] Run full test suite
- [ ] Update documentation
- [ ] Create PR with clear description of changes

## Notes

This completion plan focuses on connecting the existing CJK implementation to the main pipeline. The actual CJK mapping logic (`build_cjk_cid_map`) already exists and has tests, so we're primarily doing integration work rather than new implementation.

The key insight is that Phase 3 created the CJK infrastructure but didn't complete the final step of removing the error stub in the main entry point. This plan fixes that gap with minimal risk and maximum test coverage.