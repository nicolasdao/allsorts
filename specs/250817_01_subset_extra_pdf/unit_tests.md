# Allsorts API Validation Tests for PDF Compression Project

## Overview

This document contains comprehensive unit tests that validate the new Allsorts APIs (`subset_and_map`, `update_composite_references`, `subset_for_pdf`, `SubsetResult`) will work correctly for our PDF compression project. Each test is based on real scenarios we've encountered, particularly focusing on problematic cases like CID 143 (bullet •), composite glyphs, and CIDToGIDMap handling.

## Test Setup

```rust
use allsorts::subset::{subset_and_map, update_composite_references, subset_for_pdf, SubsetResult};
use allsorts::subset::{PdfFontContext, PdfSubsetResult, PdfWarning, WritingMode};
use allsorts::binary::read::ReadScope;
use allsorts::font_data::FontData;
use allsorts::tables::FontTableProvider;
use std::collections::HashMap;

/// Test font data from our actual PDFs
const CALIBRI_FONT: &[u8] = include_bytes!("test_fonts/calibri_from_page8.ttf");
const CID_FONT: &[u8] = include_bytes!("test_fonts/helv_neue_from_rfq.otf");

/// Problematic CIDs we've encountered
const BULLET_CID: u16 = 143;      // • character
const EN_DASH_CID: u16 = 45;      // – character  
const PERIOD_CID: u16 = 46;       // . character
const SPACE_CID: u16 = 32;        // space (empty glyph)
```

## 1. Tests for `subset_and_map`

### Test 1.1: Basic Mapping Correctness

```rust
#[test]
fn test_subset_and_map_basic_mapping() {
    // This tests the core scenario from our page8.pdf compression
    let scope = ReadScope::new(CALIBRI_FONT);
    let font_file = scope.read::<FontData<'_>>().unwrap();
    let provider = font_file.table_provider(0).unwrap();
    
    // Glyphs we typically need (from our actual usage analysis)
    let glyph_ids = vec![0, 19, 21, 110, 143];  // Including bullet at 143
    
    let (subset_data, mapping) = subset_and_map(&provider, &glyph_ids).unwrap();
    
    // Critical assertions based on our requirements
    assert_eq!(mapping.len(), 5, "Should map exactly the requested glyphs");
    assert_eq!(mapping[&0], 0, ".notdef must always map to 0");
    
    // Verify sequential compaction (what we expect for space savings)
    assert_eq!(mapping[&19], 1, "GID 19 should map to 1");
    assert_eq!(mapping[&21], 2, "GID 21 should map to 2");
    assert_eq!(mapping[&110], 3, "GID 110 should map to 3");
    assert_eq!(mapping[&143], 4, "GID 143 (bullet) should map to 4");
    
    // Verify font data is valid
    assert!(subset_data.len() < CALIBRI_FONT.len() / 2, 
            "Subset should be significantly smaller");
}
```

### Test 1.2: Composite Glyph Dependencies

```rust
#[test]
fn test_subset_and_map_composite_dependencies() {
    // Test case from our RFQ PDF where é (e + acute) caused issues
    let scope = ReadScope::new(CID_FONT);
    let font_file = scope.read::<FontData<'_>>().unwrap();
    let provider = font_file.table_provider(0).unwrap();
    
    // Request composite glyph (é) without explicitly requesting components
    let glyph_ids = vec![0, 233];  // é at GID 233 (example)
    
    let (subset_data, mapping) = subset_and_map(&provider, &glyph_ids).unwrap();
    
    // Should include dependencies automatically
    assert!(mapping.len() > 2, "Should include component glyphs");
    assert!(mapping.contains_key(&233), "Should include requested é");
    // If é is composed of e (101) + acute (769)
    assert!(mapping.contains_key(&101) || mapping.contains_key(&769), 
            "Should include component glyphs for composite");
    
    // All mapped values should be sequential
    let mut values: Vec<u16> = mapping.values().copied().collect();
    values.sort();
    for (i, &v) in values.iter().enumerate() {
        assert_eq!(v, i as u16, "Mapped GIDs should be sequential");
    }
}
```

### Test 1.3: Empty Glyph Handling

```rust
#[test]
fn test_subset_and_map_empty_glyphs() {
    // Space character caused issues in our implementation
    let scope = ReadScope::new(CALIBRI_FONT);
    let font_file = scope.read::<FontData<'_>>().unwrap();
    let provider = font_file.table_provider(0).unwrap();
    
    // Include space glyph which is typically empty
    let glyph_ids = vec![0, 3, 32, 65];  // 32 is typically space
    
    let (subset_data, mapping) = subset_and_map(&provider, &glyph_ids).unwrap();
    
    // Empty glyphs should still be mapped
    assert_eq!(mapping.len(), 4, "Should include empty glyph");
    assert!(mapping.contains_key(&32), "Space glyph should be mapped");
    
    // Verify the mapping is correct even with empty glyph
    let space_new_gid = mapping[&32];
    assert!(space_new_gid > 0 && space_new_gid < 4, 
            "Space should have valid new GID");
}
```

### Test 1.4: CID Font Specifics

```rust
#[test]
fn test_subset_and_map_cid_font_problematic_glyphs() {
    // Test with actual problematic CIDs from our PDFs
    let scope = ReadScope::new(CID_FONT);
    let font_file = scope.read::<FontData<'_>>().unwrap();
    let provider = font_file.table_provider(0).unwrap();
    
    // These are the actual problematic GIDs we've seen
    let glyph_ids = vec![
        0,    // .notdef
        19,   // Some letter
        21,   // Another letter
        110,  // Common character
        143,  // Bullet (•) - our biggest problem!
    ];
    
    let (subset_data, mapping) = subset_and_map(&provider, &glyph_ids).unwrap();
    
    // The critical assertion - bullet must be mapped!
    assert!(mapping.contains_key(&143), 
            "CRITICAL: Bullet glyph (143) must be in mapping");
    
    let bullet_new_gid = mapping[&143];
    assert!(bullet_new_gid != 0, "Bullet must not map to .notdef!");
    
    // Verify all glyphs are mapped to unique values
    let values: Vec<u16> = mapping.values().copied().collect();
    let unique_values: std::collections::HashSet<u16> = values.iter().copied().collect();
    assert_eq!(values.len(), unique_values.len(), "All mappings must be unique");
}
```

## 2. Tests for `update_composite_references`

### Test 2.1: Basic Composite Update

```rust
#[test]
fn test_update_composite_references_basic() {
    // Create a font with composite glyphs
    let scope = ReadScope::new(CALIBRI_FONT);
    let font_file = scope.read::<FontData<'_>>().unwrap();
    let provider = font_file.table_provider(0).unwrap();
    
    // Subset including composite glyphs
    let glyph_ids = vec![0, 65, 769, 193];  // A, combining acute, Á
    let (mut subset_data, mapping) = subset_and_map(&provider, &glyph_ids).unwrap();
    
    // Update composite references
    let stats = update_composite_references(&mut subset_data, &mapping).unwrap();
    
    // Verify updates were made
    assert!(stats.references_updated > 0 || glyph_ids.len() == mapping.len(),
            "Should update composite references or have no composites");
    assert_eq!(stats.unmapped_references.len(), 0, 
            "Should have no unmapped references");
    
    // Font should still be valid after update
    let updated_scope = ReadScope::new(&subset_data);
    let updated_font = updated_scope.read::<FontData<'_>>().unwrap();
    assert!(updated_font.table_provider(0).is_ok(), 
            "Font should remain valid after composite update");
}
```

### Test 2.2: Missing Component Handling

```rust
#[test]
fn test_update_composite_references_missing_component() {
    // Simulate case where composite references a glyph not in mapping
    let mut font_data = CALIBRI_FONT.to_vec();
    
    // Create mapping that's missing a component
    let mut mapping = HashMap::new();
    mapping.insert(0, 0);
    mapping.insert(193, 1);  // Á composite
    // Deliberately missing component 65 (A) from mapping
    
    let stats = update_composite_references(&mut font_data, &mapping).unwrap();
    
    // Should report unmapped references
    assert!(stats.unmapped_references.len() > 0, 
            "Should report unmapped component references");
    
    // Should still complete without panicking
    assert!(stats.composites_updated >= 0, "Should handle missing gracefully");
}
```

### Test 2.3: CFF Font Handling

```rust
#[test]
fn test_update_composite_references_cff_font() {
    // Test with CFF font (different structure)
    let scope = ReadScope::new(CID_FONT);
    let font_file = scope.read::<FontData<'_>>().unwrap();
    let provider = font_file.table_provider(0).unwrap();
    
    let glyph_ids = vec![0, 10, 20, 30];
    let (mut subset_data, mapping) = subset_and_map(&provider, &glyph_ids).unwrap();
    
    // Should handle CFF fonts (even if no composites to update)
    let stats = update_composite_references(&mut subset_data, &mapping).unwrap();
    
    // CFF fonts might not have traditional composites
    assert!(stats.cff_subroutines_updated || stats.composites_updated == 0,
            "Should handle CFF appropriately");
}
```

## 3. Tests for `subset_for_pdf`

### Test 3.1: Complete PDF Workflow

```rust
#[test]
fn test_subset_for_pdf_complete_workflow() {
    // Test the complete PDF subsetting workflow
    let scope = ReadScope::new(CID_FONT);
    let font_file = scope.read::<FontData<'_>>().unwrap();
    let provider = font_file.table_provider(0).unwrap();
    
    // Simulate our actual CIDToGIDMap from page8.pdf
    let mut original_cid_map = vec![0u16; 256];
    original_cid_map[143] = 143;  // Bullet maps to itself
    original_cid_map[45] = 45;    // En-dash maps to itself
    original_cid_map[46] = 46;    // Period maps to itself
    
    let context = PdfFontContext {
        cid_to_gid_map: Some(original_cid_map),
        max_cid: 255,
        is_cid_font: true,
        writing_mode: WritingMode::Horizontal,
    };
    
    // Request the glyphs we actually use
    let glyph_ids = vec![0, 45, 46, 143];
    
    let result = subset_for_pdf(&provider, &glyph_ids, &context).unwrap();
    
    // Critical validations
    assert_eq!(result.cid_to_gid_map.len(), (256 * 2), 
            "CIDToGIDMap should cover all CIDs (big-endian u16s)");
    
    // Verify bullet CID maps to valid GID
    let bullet_offset = BULLET_CID as usize * 2;
    let bullet_new_gid = u16::from_be_bytes([
        result.cid_to_gid_map[bullet_offset],
        result.cid_to_gid_map[bullet_offset + 1],
    ]);
    assert_ne!(bullet_new_gid, 0, "Bullet must not map to .notdef!");
    
    // Check validation results
    assert!(result.validation.all_cids_mapped, 
            "All used CIDs should map to valid GIDs");
    assert_eq!(result.validation.missing_glyph_cids.len(), 0,
            "Should have no missing glyphs for used characters");
}
```

### Test 3.2: Warning Generation

```rust
#[test]
fn test_subset_for_pdf_warnings() {
    // Test that warnings are generated for problematic cases
    let scope = ReadScope::new(CID_FONT);
    let font_file = scope.read::<FontData<'_>>().unwrap();
    let provider = font_file.table_provider(0).unwrap();
    
    // Create context with CID that maps to non-existent glyph
    let mut cid_map = vec![0u16; 500];
    cid_map[499] = 9999;  // Maps to glyph that doesn't exist
    
    let context = PdfFontContext {
        cid_to_gid_map: Some(cid_map),
        max_cid: 499,
        is_cid_font: true,
        writing_mode: WritingMode::Horizontal,
    };
    
    // Only subset a few glyphs (not including 9999)
    let glyph_ids = vec![0, 1, 2, 3];
    
    let result = subset_for_pdf(&provider, &glyph_ids, &context).unwrap();
    
    // Should generate warnings
    assert!(!result.warnings.is_empty(), "Should generate warnings");
    
    // Check for missing glyph warning
    let has_missing_warning = result.warnings.iter().any(|w| {
        matches!(w, PdfWarning::MissingGlyph { cid: 499 }) ||
        matches!(w, PdfWarning::UnmappedGlyph { .. })
    });
    assert!(has_missing_warning, "Should warn about unmapped CID");
    
    // Validation should reflect the issue
    assert!(!result.validation.all_cids_mapped || 
            result.validation.missing_glyph_cids.len() > 0,
            "Validation should catch mapping issues");
}
```

### Test 3.3: Identity CIDToGIDMap

```rust
#[test]
fn test_subset_for_pdf_identity_mapping() {
    // Test with identity mapping (no CIDToGIDMap)
    let scope = ReadScope::new(CALIBRI_FONT);
    let font_file = scope.read::<FontData<'_>>().unwrap();
    let provider = font_file.table_provider(0).unwrap();
    
    let context = PdfFontContext {
        cid_to_gid_map: None,  // Identity mapping
        max_cid: 255,
        is_cid_font: false,  // Simple font
        writing_mode: WritingMode::Horizontal,
    };
    
    let glyph_ids = vec![0, 65, 66, 67];  // A, B, C
    
    let result = subset_for_pdf(&provider, &glyph_ids, &context).unwrap();
    
    // With identity mapping and remapping, CIDs might not match original GIDs
    // but should map to valid glyphs
    for cid in [65u16, 66, 67] {
        let offset = cid as usize * 2;
        if offset < result.cid_to_gid_map.len() {
            let gid = u16::from_be_bytes([
                result.cid_to_gid_map[offset],
                result.cid_to_gid_map[offset + 1],
            ]);
            // Should map to a valid glyph in the subset
            assert!(result.glyph_mapping.values().any(|&v| v == gid),
                    "CID {} should map to a valid subset GID", cid);
        }
    }
}
```

### Test 3.4: Max CID Optimization

```rust
#[test]
fn test_subset_for_pdf_max_cid_size() {
    // Test that CIDToGIDMap is correctly sized
    let scope = ReadScope::new(CID_FONT);
    let font_file = scope.read::<FontData<'_>>().unwrap();
    let provider = font_file.table_provider(0).unwrap();
    
    // Small max_cid should produce smaller map
    let context_small = PdfFontContext {
        cid_to_gid_map: None,
        max_cid: 127,  // Small range
        is_cid_font: true,
        writing_mode: WritingMode::Horizontal,
    };
    
    let result_small = subset_for_pdf(&provider, &[0, 1, 2], &context_small).unwrap();
    assert_eq!(result_small.cid_to_gid_map.len(), 128 * 2, 
            "Map should be sized for max_cid + 1");
    
    // Large max_cid
    let context_large = PdfFontContext {
        cid_to_gid_map: None,
        max_cid: 1000,  // Larger range
        is_cid_font: true,
        writing_mode: WritingMode::Horizontal,
    };
    
    let result_large = subset_for_pdf(&provider, &[0, 1, 2], &context_large).unwrap();
    assert_eq!(result_large.cid_to_gid_map.len(), 1001 * 2,
            "Map should accommodate all CIDs up to max_cid");
}
```

## 4. Tests for `SubsetResult` Structure

### Test 4.1: Comprehensive Result Information

```rust
#[test]
fn test_subset_result_comprehensive_info() {
    // Test that SubsetResult provides all needed information
    let scope = ReadScope::new(CALIBRI_FONT);
    let font_file = scope.read::<FontData<'_>>().unwrap();
    let provider = font_file.table_provider(0).unwrap();
    
    let requested_gids = vec![0, 19, 21, 110, 143];
    let result = subset_detailed(&provider, &requested_gids).unwrap();
    
    // Verify all fields are populated correctly
    assert!(!result.data.is_empty(), "Should have font data");
    assert_eq!(result.glyph_mapping.len(), requested_gids.len() + result.added_glyphs.len(),
            "Mapping should include requested + added glyphs");
    
    // Reverse mapping should be inverse of forward mapping
    for (old, new) in &result.glyph_mapping {
        assert_eq!(result.reverse_mapping[new], *old,
                "Reverse mapping should be inverse");
    }
    
    // Check metadata
    assert!(result.original_info.glyph_count > result.subset_info.glyph_count,
            "Subset should have fewer glyphs");
    assert!(result.subset_info.size < result.original_info.size,
            "Subset should be smaller");
    
    // Stats should be reasonable
    assert!(result.stats.size_reduction_percent > 0.0,
            "Should show size reduction");
    assert!(result.stats.size_reduction_bytes > 0,
            "Should reduce size in bytes");
}
```

### Test 4.2: Added Glyphs Detection

```rust
#[test]
fn test_subset_result_added_glyphs() {
    // Test detection of automatically added glyphs
    let scope = ReadScope::new(CALIBRI_FONT);
    let font_file = scope.read::<FontData<'_>>().unwrap();
    let provider = font_file.table_provider(0).unwrap();
    
    // Request composite glyph that needs components
    let requested_gids = vec![0, 193];  // Á (if composite)
    let result = subset_detailed(&provider, &requested_gids).unwrap();
    
    // If Á is composite, components should be in added_glyphs
    if result.added_glyphs.len() > 0 {
        // Components shouldn't be in requested list
        for added in &result.added_glyphs {
            assert!(!requested_gids.contains(added),
                    "Added glyphs should not be in requested list");
            assert!(result.glyph_mapping.contains_key(added),
                    "Added glyphs should be in mapping");
        }
    }
    
    // Total glyphs should match
    assert_eq!(result.glyph_mapping.len(),
            requested_gids.len() + result.added_glyphs.len(),
            "Total should be requested + added");
}
```

### Test 4.3: Missing Glyphs Handling

```rust
#[test]
fn test_subset_result_missing_glyphs() {
    // Test handling of requested but non-existent glyphs
    let scope = ReadScope::new(CALIBRI_FONT);
    let font_file = scope.read::<FontData<'_>>().unwrap();
    let provider = font_file.table_provider(0).unwrap();
    
    // Request some invalid glyph IDs
    let requested_gids = vec![0, 19, 99999];  // 99999 doesn't exist
    
    // This might error or report in missing_glyphs
    match subset_detailed(&provider, &requested_gids) {
        Ok(result) => {
            // Should report missing glyphs
            assert!(result.missing_glyphs.contains(&99999),
                    "Should report non-existent glyph as missing");
            assert!(!result.glyph_mapping.contains_key(&99999),
                    "Missing glyphs shouldn't be in mapping");
        },
        Err(_) => {
            // Or it might error, which is also acceptable
            assert!(true, "Invalid glyphs can cause error");
        }
    }
}
```

## 5. Integration Tests

### Test 5.1: Complete PDF Font Subsetting Pipeline

```rust
#[test]
fn test_complete_pdf_pipeline() {
    // This test simulates our entire font subsetting pipeline
    let scope = ReadScope::new(CID_FONT);
    let font_file = scope.read::<FontData<'_>>().unwrap();
    let provider = font_file.table_provider(0).unwrap();
    
    // Step 1: Prepare context (simulating extraction from PDF)
    let mut original_cid_map = vec![0u16; 1000];
    // Set up problematic mappings we've seen
    original_cid_map[BULLET_CID as usize] = 143;
    original_cid_map[EN_DASH_CID as usize] = 45;
    original_cid_map[PERIOD_CID as usize] = 46;
    original_cid_map[SPACE_CID as usize] = 32;
    
    let context = PdfFontContext {
        cid_to_gid_map: Some(original_cid_map),
        max_cid: 999,
        is_cid_font: true,
        writing_mode: WritingMode::Horizontal,
    };
    
    // Step 2: Get glyphs (including closure)
    let mut glyph_ids = vec![0, 32, 45, 46, 143];
    // Add more glyphs to simulate real usage
    glyph_ids.extend(65..75);  // A-J
    
    // Step 3: Subset for PDF
    let result = subset_for_pdf(&provider, &glyph_ids, &context).unwrap();
    
    // Step 4: Validate critical requirements
    
    // Size reduction
    assert!(result.font_data.len() < CID_FONT.len() / 2,
            "Should achieve significant size reduction");
    
    // Bullet character MUST work
    let bullet_cid_offset = BULLET_CID as usize * 2;
    let bullet_new_gid = u16::from_be_bytes([
        result.cid_to_gid_map[bullet_cid_offset],
        result.cid_to_gid_map[bullet_cid_offset + 1],
    ]);
    assert_ne!(bullet_new_gid, 0, 
            "CRITICAL: Bullet must not render as missing glyph!");
    
    // En-dash character must work
    let dash_cid_offset = EN_DASH_CID as usize * 2;
    let dash_new_gid = u16::from_be_bytes([
        result.cid_to_gid_map[dash_cid_offset],
        result.cid_to_gid_map[dash_cid_offset + 1],
    ]);
    assert_ne!(dash_new_gid, 0,
            "En-dash must not render as missing glyph!");
    
    // No critical warnings
    let critical_warnings = result.warnings.iter().filter(|w| {
        match w {
            PdfWarning::MissingGlyph { cid } => {
                *cid == BULLET_CID || *cid == EN_DASH_CID || *cid == PERIOD_CID
            },
            _ => false
        }
    }).count();
    assert_eq!(critical_warnings, 0,
            "Must not have warnings for critical characters");
    
    // Validation passes
    assert!(result.validation.all_cids_mapped || 
            result.validation.missing_glyph_cids.is_empty(),
            "Validation should pass for used characters");
}
```

### Test 5.2: Performance Benchmarks

```rust
#[test]
fn test_performance_requirements() {
    use std::time::Instant;
    
    let scope = ReadScope::new(CID_FONT);
    let font_file = scope.read::<FontData<'_>>().unwrap();
    let provider = font_file.table_provider(0).unwrap();
    
    // Typical number of glyphs we subset
    let glyph_ids: Vec<u16> = (0..50).collect();
    
    // Test subset_and_map performance
    let start = Instant::now();
    let (data, mapping) = subset_and_map(&provider, &glyph_ids).unwrap();
    let subset_time = start.elapsed();
    
    assert!(subset_time.as_millis() < 100,
            "Subsetting should complete within 100ms");
    
    // Test composite update performance
    let mut font_copy = data.clone();
    let start = Instant::now();
    update_composite_references(&mut font_copy, &mapping).unwrap();
    let update_time = start.elapsed();
    
    assert!(update_time.as_millis() < 50,
            "Composite update should complete within 50ms");
    
    // Test complete PDF pipeline performance
    let context = PdfFontContext {
        cid_to_gid_map: None,
        max_cid: 999,
        is_cid_font: true,
        writing_mode: WritingMode::Horizontal,
    };
    
    let start = Instant::now();
    let result = subset_for_pdf(&provider, &glyph_ids, &context).unwrap();
    let total_time = start.elapsed();
    
    assert!(total_time.as_millis() < 200,
            "Complete PDF subsetting should complete within 200ms");
}
```

## Test Execution Instructions

1. **Setup Test Fonts**:
   ```bash
   # Extract actual font files from your test PDFs
   mkdir tests/test_fonts
   # Copy calibri_from_page8.ttf from page8.pdf
   # Copy helv_neue_from_rfq.otf from RFQ PDF
   ```

2. **Run All Tests**:
   ```bash
   cargo test --test allsorts_validation -- --nocapture
   ```

3. **Run Specific Test Groups**:
   ```bash
   # Just subset_and_map tests
   cargo test test_subset_and_map -- --nocapture
   
   # Just PDF workflow tests
   cargo test test_subset_for_pdf -- --nocapture
   
   # Performance tests
   cargo test test_performance -- --nocapture --release
   ```

4. **Validate Against Real PDFs**:
   ```bash
   # After tests pass, validate with actual PDFs
   ./test_with_real_pdfs.sh test_zone/page8.pdf
   ./test_with_real_pdfs.sh test_zone/rfq.pdf
   ```

## Success Criteria

All tests must pass with these specific validations:

1. **Bullet Character (CID 143)**: Must NEVER map to GID 0
2. **Size Reduction**: Subset must be < 50% of original
3. **Performance**: Complete pipeline < 200ms for typical fonts
4. **No Missing Glyphs**: Critical characters must all map correctly
5. **Composite Integrity**: Composite glyphs must remain valid
6. **CIDToGIDMap Size**: Must cover exactly max_cid + 1 entries
7. **Warning Accuracy**: Must warn about actual issues only

## Critical Test Cases from Our Experience

These specific cases MUST pass as they represent actual bugs we've encountered:

1. **The Bullet Test**: CID 143 → Valid GID (not 0)
2. **The Space Test**: Empty glyphs must be handled correctly
3. **The Composite Test**: É, Á, ñ must remain valid
4. **The CID Range Test**: CIDToGIDMap must cover ALL CIDs up to max
5. **The Endianness Test**: CIDToGIDMap must be big-endian
6. **The Missing Component Test**: Must handle gracefully, not crash

If all these tests pass, the new Allsorts APIs will integrate perfectly with our PDF compression project and eliminate the complex workarounds currently needed.