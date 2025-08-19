# TDD Implementation Plan: `subset_and_map` Function

## Executive Summary

This document outlines the Test-Driven Development (TDD) implementation plan for adding a `subset_and_map` function to Allsorts that returns both the subset font data and a mapping of old-to-new glyph IDs. This addresses the critical need for tracking glyph ID remapping during subsetting, particularly for PDF document processing.

**TDD Approach**: Following Allsorts testing guidelines, we'll write comprehensive tests FIRST, then implement functionality incrementally to make each test pass.

## Current Architecture Analysis

### Key Components

1. **Main Entry Point**: `src/subset.rs::subset()`
   - Routes to TTF, CFF, or CFF2 subsetting based on font type
   - Lines 226-253

2. **TTF Subsetting**: `src/subset.rs::subset_ttf()`  
   - Uses `GlyfTable::subset()` to create `SubsetGlyf`
   - Lines 259-365

3. **CFF Subsetting**: `src/subset.rs::subset_cff()`
   - Uses `CFF::subset()` to create `SubsetCFF`  
   - Lines 367-402

4. **CFF2 Subsetting**: `src/subset.rs::subset_cff2()`
   - Uses `CFF2::subset_to_cff()` to create `SubsetCFF`
   - Lines 404-437

### Mapping Infrastructure Already Exists

The codebase already tracks old-to-new glyph ID mappings internally:

1. **`SubsetGlyf` (TTF fonts)** - `src/tables/glyf/subset.rs`
   - Contains `old_to_new_id: FxHashMap<u16, u16>` (line 21)
   - Implements `SubsetGlyphs` trait with mapping methods

2. **`SubsetCFF` (CFF/CFF2 fonts)** - `src/cff/subset.rs`
   - Contains `old_to_new_id: FxHashMap<u16, u16>` (line 18)
   - Implements `SubsetGlyphs` trait with mapping methods

3. **`SubsetGlyphs` trait** - `src/subset.rs`
   - Provides `old_id()` and `new_id()` mapping methods (lines 177-186)

## Phase 3: Define APIs & Signatures (TDD Step 1)

### 3.1 Public API Definition

Create internal functions that return both subset data and mappings:

```rust
// src/subset.rs - SIGNATURE ONLY, NO IMPLEMENTATION

/// Subset this font and return mapping of old to new glyph IDs
pub fn subset_and_map(
    provider: &impl FontTableProvider,
    glyph_ids: &[u16],
    profile: &SubsetProfile,
    cmap_target: CmapTarget,
) -> Result<(Vec<u8>, HashMap<u16, u16>), SubsetError>;

// Internal signatures
fn subset_with_mapping(
    provider: &impl FontTableProvider,
    glyph_ids: &[u16],
    profile: &SubsetProfile,
    cmap_target: CmapTarget,
) -> Result<(Vec<u8>, HashMap<u16, u16>), SubsetError> {
    let mappings_to_keep = MappingsToKeep::new(provider, glyph_ids, cmap_target)?;
    
    if provider.has_table(tag::CFF) {
        subset_cff_with_mapping(provider, glyph_ids, mappings_to_keep, true, profile)
    } else if provider.has_table(tag::CFF2) {
        subset_cff2_with_mapping(
            provider,
            glyph_ids,
            mappings_to_keep,
            false,
            OutputFormat::Type1OrCid,
            profile,
        )
    } else {
        subset_ttf_with_mapping(
            provider,
            glyph_ids,
            CmapStrategy::Generate(mappings_to_keep),
            profile,
        )
    }
}
```

### 3.2 Internal Function Signatures

```rust
// Function signatures only - no implementation
fn subset_ttf_with_mapping(
    provider: &impl FontTableProvider,
    glyph_ids: &[u16],
    cmap_strategy: CmapStrategy,
    profile: &SubsetProfile,
) -> Result<(Vec<u8>, HashMap<u16, u16>), SubsetError>;

fn subset_cff_with_mapping(
    provider: &impl FontTableProvider,
    glyph_ids: &[u16],
    mappings_to_keep: MappingsToKeep<OldIds>,
    convert_cff_to_cid_if_more_than_255_glyphs: bool,
    profile: &SubsetProfile,
) -> Result<(Vec<u8>, HashMap<u16, u16>), SubsetError>;

fn subset_cff2_with_mapping(
    provider: &impl FontTableProvider,
    glyph_ids: &[u16],
    mappings_to_keep: MappingsToKeep<OldIds>,
    convert_to_cff1: bool,
    output_format: OutputFormat,
    profile: &SubsetProfile,
) -> Result<(Vec<u8>, HashMap<u16, u16>), SubsetError>;

fn extract_mapping_from_subset(subset: &impl SubsetGlyphs) -> HashMap<u16, u16>;
```

## Phase 4: Write Unit Tests (TDD Step 2)

### 4.1 Test File Setup

Create `tests/subset_and_map.rs` following Allsorts testing patterns:


```rust
// tests/subset_and_map.rs
mod common;  // Import shared utilities per Allsorts guidelines

#[cfg(test)]
mod subset_and_map_tests {
    use crate::common;
    use allsorts::binary::read::ReadScope;
    use allsorts::tables::OpenTypeFont;
    use allsorts::subset::{subset_and_map, SubsetProfile, CmapTarget, SubsetError};
    use allsorts::Font;
    use std::collections::HashMap;

    // Test 1: Basic TTF subsetting with mapping
    #[test]
    fn test_subset_and_map_simple_ttf() {
        // Using common utilities per guidelines
        let buffer = common::read_fixture_font("opentype/Klei.otf");
        let scope = ReadScope::new(&buffer);
        let font_file = scope.read::<OpenTypeFont>().unwrap();
        let provider = font_file.table_provider(0).unwrap();
        
        let glyph_ids = vec![0, 10, 20, 30];
        let (data, mapping) = subset_and_map(
            &provider,
            &glyph_ids,
            &SubsetProfile::Pdf,
            CmapTarget::Unrestricted,
        ).unwrap();
        
        // Verify mapping correctness
        assert_eq!(mapping.len(), 4, "Expected 4 glyphs in mapping");
        assert_eq!(mapping[&0], 0, ".notdef should remain at index 0");
        assert_eq!(mapping[&10], 1, "GID 10 should map to 1");
        assert_eq!(mapping[&20], 2, "GID 20 should map to 2");
        assert_eq!(mapping[&30], 3, "GID 30 should map to 3");
        
        // Verify subset font validity
        let subset_scope = ReadScope::new(&data);
        let subset_font = subset_scope.read::<OpenTypeFont>().unwrap();
        let subset_provider = subset_font.table_provider(0).unwrap();
        let mut font = Font::new(Box::new(subset_provider)).unwrap();
        assert_eq!(font.num_glyphs(), 4, "Subset should have 4 glyphs");
    }

    // Test 2: Composite glyphs with dependencies
    #[test]
    fn test_subset_and_map_with_composites() {
        // Load a font with composite glyphs
        let buffer = common::read_fixture_font("opentype/OpenSans-Regular.ttf");
        let scope = ReadScope::new(&buffer);
        let font_file = scope.read::<OpenTypeFont>().unwrap();
        let provider = font_file.table_provider(0).unwrap();
        
        // Select glyphs that have composite dependencies
        // (will need to identify actual composite glyphs in the font)
        let glyph_ids = vec![0, 50];  // Assuming 50 has dependencies
        let (data, mapping) = subset_and_map(
            &provider,
            &glyph_ids,
            &SubsetProfile::Pdf,
            CmapTarget::Unrestricted,
        ).unwrap();
        
        // Verify mapping includes dependencies
        assert!(mapping.len() >= 2, "Mapping should include dependencies");
        assert!(mapping.contains_key(&0), "Should contain .notdef");
        assert!(mapping.contains_key(&50), "Should contain requested glyph");
    }

    // Test 3: CFF font subsetting
    #[test]
    fn test_subset_and_map_cff() {
        // Use a CFF font from fixtures
        let buffer = common::read_fixture_font("opentype/SourceSansPro-Regular.otf");
        let scope = ReadScope::new(&buffer);
        let font_file = scope.read::<OpenTypeFont>().unwrap();
        let provider = font_file.table_provider(0).unwrap();
        
        let glyph_ids = vec![0, 5, 10];
        let (data, mapping) = subset_and_map(
            &provider,
            &glyph_ids,
            &SubsetProfile::Pdf,
            CmapTarget::Unrestricted,
        ).unwrap();
        
        assert_eq!(mapping.len(), 3);
        assert_eq!(mapping[&0], 0);
        assert_eq!(mapping[&5], 1);
        assert_eq!(mapping[&10], 2);
    }

    // Test 4: CFF2 font subsetting
    #[test]
    #[ignore]  // Enable when CFF2 test font available
    fn test_subset_and_map_cff2() {
        // Will need a CFF2 variable font
    }

    // Test 5: Error case - missing .notdef
    #[test]
    fn test_missing_notdef_error() {
        let buffer = common::read_fixture_font("opentype/Klei.otf");
        let scope = ReadScope::new(&buffer);
        let font_file = scope.read::<OpenTypeFont>().unwrap();
        let provider = font_file.table_provider(0).unwrap();
        
        // Glyph IDs without 0 (.notdef)
        let glyph_ids = vec![10, 20, 30];
        let result = subset_and_map(
            &provider,
            &glyph_ids,
            &SubsetProfile::Pdf,
            CmapTarget::Unrestricted,
        );
        
        match result {
            Err(SubsetError::MissingNotdef) => {
                // Expected error
            }
            Ok(_) => panic!("Should have failed with MissingNotdef"),
            Err(e) => panic!("Wrong error type: {:?}", e),
        }
    }

    // Test 6: Backward compatibility
    #[test]
    fn test_backward_compatibility() {
        use allsorts::subset::subset;
        
        let buffer = common::read_fixture_font("opentype/Klei.otf");
        let scope = ReadScope::new(&buffer);
        let font_file = scope.read::<OpenTypeFont>().unwrap();
        let provider = font_file.table_provider(0).unwrap();
        
        let glyph_ids = vec![0, 10, 20];
        
        // Old API should still work
        let old_result = subset(
            &provider,
            &glyph_ids,
            &SubsetProfile::Pdf,
            CmapTarget::Unrestricted,
        );
        assert!(old_result.is_ok(), "subset() should still work");
        
        // New API should produce same font data
        let (new_data, _mapping) = subset_and_map(
            &provider,
            &glyph_ids,
            &SubsetProfile::Pdf,
            CmapTarget::Unrestricted,
        ).unwrap();
        
        assert_eq!(old_result.unwrap(), new_data, "Font data should match");
    }

    // Test 7: Large font performance
    #[test]
    #[ignore]  // Run with: cargo test -- --ignored
    fn test_performance_large_font() {
        use std::time::Instant;
        
        let buffer = common::read_fixture_font("noto/NotoSansCJKjp-Regular.otf");
        let scope = ReadScope::new(&buffer);
        let font_file = scope.read::<OpenTypeFont>().unwrap();
        let provider = font_file.table_provider(0).unwrap();
        
        // Select many glyphs
        let glyph_ids: Vec<u16> = (0..1000).collect();
        
        let start = Instant::now();
        let (_data, mapping) = subset_and_map(
            &provider,
            &glyph_ids,
            &SubsetProfile::Pdf,
            CmapTarget::Unrestricted,
        ).unwrap();
        let duration = start.elapsed();
        
        println!("Subset 1000 glyphs with mapping in {:?}", duration);
        assert!(mapping.len() >= 1000, "Should map all requested glyphs");
        assert!(duration.as_secs() < 1, "Should complete within 1 second");
    }
}
```

### 4.2 Run Tests (Should Fail)

```bash
cargo test subset_and_map
```

All tests should fail to compile since the functions don't exist yet.

## Phase 5: Implement Functions Incrementally (TDD Step 3)

### 5.1 Implementation Order

Implement each function to make tests pass one by one:

#### Step 1: Add HashMap import and helper function

```rust
// src/subset.rs
use std::collections::HashMap;

fn extract_mapping_from_subset(subset: &impl SubsetGlyphs) -> HashMap<u16, u16> {
    let mut mapping = HashMap::new();
    for new_id in 0..subset.len() as u16 {
        let old_id = subset.old_id(new_id);
        mapping.insert(old_id, new_id);
    }
    mapping
}
```

**Run specific test**: `cargo test test_subset_and_map_simple_ttf`

#### Step 2: Implement TTF subsetting with mapping

```rust
fn subset_ttf_with_mapping(
    provider: &impl FontTableProvider,
    glyph_ids: &[u16],
    cmap_strategy: CmapStrategy,
    profile: &SubsetProfile,
) -> Result<(Vec<u8>, HashMap<u16, u16>), SubsetError> {
    // Extract from existing subset_ttf() code
    // Add mapping extraction before consuming subset_glyphs
    let subset_glyphs = glyf.subset(glyph_ids)?;
    let mapping = extract_mapping_from_subset(&subset_glyphs);
    // Continue with font building...
    Ok((builder.data()?, mapping))
}
```

**Run test**: `cargo test test_subset_and_map_simple_ttf` - should pass

#### Step 3: Implement CFF subsetting with mapping

```rust
fn subset_cff_with_mapping(
    provider: &impl FontTableProvider,
    glyph_ids: &[u16],
    mappings_to_keep: MappingsToKeep<OldIds>,
    convert_cff_to_cid_if_more_than_255_glyphs: bool,
    profile: &SubsetProfile,
) -> Result<(Vec<u8>, HashMap<u16, u16>), SubsetError> {
    // Extract from existing subset_cff() code
    let cff_subset = cff.subset(glyph_ids, convert_cff_to_cid_if_more_than_255_glyphs)?;
    let mapping = extract_mapping_from_subset(&cff_subset);
    // Build font...
    Ok((font_data, mapping))
}
```

**Run test**: `cargo test test_subset_and_map_cff` - should pass

#### Step 4: Implement CFF2 subsetting (if needed)

#### Step 5: Wire up public API

```rust
pub fn subset_and_map(
    provider: &impl FontTableProvider,
    glyph_ids: &[u16],
    profile: &SubsetProfile,
    cmap_target: CmapTarget,
) -> Result<(Vec<u8>, HashMap<u16, u16>), SubsetError> {
    subset_with_mapping(provider, glyph_ids, profile, cmap_target)
}

fn subset_with_mapping(
    provider: &impl FontTableProvider,
    glyph_ids: &[u16],
    profile: &SubsetProfile,
    cmap_target: CmapTarget,
) -> Result<(Vec<u8>, HashMap<u16, u16>), SubsetError> {
    let mappings_to_keep = MappingsToKeep::new(provider, glyph_ids, cmap_target)?;
    
    if provider.has_table(tag::CFF) {
        subset_cff_with_mapping(provider, glyph_ids, mappings_to_keep, true, profile)
    } else if provider.has_table(tag::CFF2) {
        subset_cff2_with_mapping(
            provider,
            glyph_ids,
            mappings_to_keep,
            false,
            OutputFormat::Type1OrCid,
            profile,
        )
    } else {
        subset_ttf_with_mapping(
            provider,
            glyph_ids,
            CmapStrategy::Generate(mappings_to_keep),
            profile,
        )
    }
}
```

#### Step 6: Update existing subset() for backward compatibility

```rust
pub fn subset(
    provider: &impl FontTableProvider,
    glyph_ids: &[u16],
    profile: &SubsetProfile,
    cmap_target: CmapTarget,
) -> Result<Vec<u8>, SubsetError> {
    let (data, _mapping) = subset_with_mapping(provider, glyph_ids, profile, cmap_target)?;
    Ok(data)
}
```

**Run test**: `cargo test test_backward_compatibility` - should pass

## Phase 6: Full-Suite Integration

### 6.1 Run All Tests

```bash
# Run all subset tests
cargo test subset

# Run with verbose output to see any failures
cargo test subset -- --nocapture
```

### 6.2 Fix Any Regressions

Address any test failures, ensuring both new and existing tests pass.

## Phase 7: Documentation & Review

### 7.1 Add Documentation

```rust
/// Subset this font so that it only contains the glyphs with the supplied `glyph_ids`.
/// 
/// Returns both the subset font data and a mapping from old glyph IDs to new glyph IDs.
///
/// `glyph_ids` requirements:
///
/// * Glyph id 0, corresponding to the `.notdef` glyph must always be present.
/// * There must be no duplicate glyph ids.
///
/// The returned HashMap maps original glyph IDs to their new IDs in the subset font.
/// This includes all glyphs in the final subset, including any composite glyph dependencies
/// that were automatically added.
///
/// # Example
/// ```rust
/// use allsorts::subset::{subset_and_map, SubsetProfile, CmapTarget};
/// use std::collections::HashMap;
/// 
/// let glyph_ids = vec![0, 19, 21, 110, 143];
/// let (subset_data, mapping) = subset_and_map(
///     &provider,
///     &glyph_ids,
///     &SubsetProfile::Pdf,
///     CmapTarget::Unrestricted,
/// )?;
/// 
/// // Use mapping to update external references
/// assert_eq!(mapping.get(&19), Some(&1));   // GID 19 became GID 1
/// assert_eq!(mapping.get(&143), Some(&4));  // GID 143 became GID 4
/// ```
pub fn subset_and_map(...)
```

### 7.2 Export from lib.rs

```rust
// src/lib.rs
pub use subset::subset_and_map;
```

## File Changes Summary (TDD Order)

### Test-First Implementation Order

1. **Create test file first**: `tests/subset_and_map.rs` (300 lines)
2. **Add signatures only**: `src/subset.rs` (function signatures)
3. **Implement incrementally**: 
   - Add HashMap import
   - Implement `extract_mapping_from_subset()`
   - Implement `subset_ttf_with_mapping()`
   - Implement `subset_cff_with_mapping()`
   - Implement `subset_cff2_with_mapping()`
   - Implement `subset_with_mapping()`
   - Add public `subset_and_map()`
   - Update `subset()` for compatibility
4. **Export from lib.rs**: 1 line change

## TDD Implementation Checklist

### Pre-Implementation
- [ ] Run existing tests - all green
- [ ] Define API signatures (no implementation)
- [ ] Write comprehensive test suite
- [ ] Run tests - all should fail (no implementation)

### Incremental Implementation
- [ ] Add HashMap import
- [ ] Implement `extract_mapping_from_subset()` - run tests
- [ ] Implement `subset_ttf_with_mapping()` - TTF tests pass
- [ ] Implement `subset_cff_with_mapping()` - CFF tests pass  
- [ ] Implement `subset_cff2_with_mapping()` - CFF2 tests pass
- [ ] Implement `subset_with_mapping()` - integration tests pass
- [ ] Add public `subset_and_map()` - public API tests pass
- [ ] Update `subset()` - backward compatibility tests pass

### Post-Implementation
- [ ] All tests green
- [ ] Documentation complete
- [ ] Export from lib.rs
- [ ] Final test run: `cargo test`

## Risk Assessment

### Low Risk
- The mapping data already exists internally
- No changes to core subsetting logic
- Backward compatibility maintained
- Clear separation of concerns

### Mitigations
- Extensive testing with various font formats
- No modification of existing subsetting algorithms
- Simple extraction of existing data structures

## Performance Impact

- **Memory**: Additional HashMap with O(n) space where n = number of glyphs
- **Time**: Negligible - one additional iteration to build HashMap
- **Expected overhead**: <5% as specified in requirements

## Success Metrics

1. ✅ All existing tests pass unchanged
2. ✅ New function returns correct mappings for TTF, CFF, and CFF2
3. ✅ Composite glyph dependencies included in mapping
4. ✅ Performance overhead <5%
5. ✅ Clear documentation with PDF use case example
6. ✅ >95% test coverage for new code

## Timeline Estimate (TDD Approach)

- Write tests first: 2 hours
- Incremental implementation: 3-4 hours
- Documentation: 1 hour
- Total: ~6-7 hours

**TDD Benefits**:
- Tests guide implementation
- Immediate feedback on correctness
- No untested code paths
- Clear progress tracking

## Conclusion

This TDD implementation plan ensures high-quality, well-tested code by:
1. Writing comprehensive tests FIRST following Allsorts testing guidelines
2. Implementing functionality incrementally to pass each test
3. Maintaining backward compatibility with existing API
4. Leveraging existing mapping infrastructure

The test-first approach guarantees all code paths are tested and the implementation meets requirements before completion.