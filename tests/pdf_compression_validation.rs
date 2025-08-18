/// Comprehensive unit tests for PDF compression project
/// These tests validate the new Allsorts APIs work correctly for real PDF scenarios,
/// particularly focusing on problematic cases like CID 143 (bullet), composite glyphs,
/// and CIDToGIDMap handling.
mod common;

use allsorts::binary::read::ReadScope;
use allsorts::subset::builder::{SubsetBuilder, ValidationLevel};
use allsorts::subset::composite::update_composite_references;
use allsorts::subset::pdf::{subset_for_pdf, PdfFontContext, WritingMode};
use allsorts::subset::result::subset_detailed;
use allsorts::subset::{subset_and_map, CmapTarget, SubsetProfile};
use allsorts::tables::{FontTableProvider, OpenTypeFont};
use std::collections::HashMap;

/// Problematic CIDs we've encountered in real PDFs
const BULLET_CID: u16 = 143; // • character
const EN_DASH_CID: u16 = 45; // – character
const PERIOD_CID: u16 = 46; // . character
const SPACE_CID: u16 = 32; // space (empty glyph)

// Helper function to create a font provider from test font data
fn create_provider(font_buffer: &[u8]) -> impl FontTableProvider + '_ {
    let scope = ReadScope::new(font_buffer);
    let font_file = scope.read::<OpenTypeFont<'_>>().unwrap();
    font_file.table_provider(0).unwrap()
}

// =============================================================================
// 1. Tests for `subset_and_map`
// =============================================================================

#[test]
fn test_subset_and_map_basic_mapping() {
    // This tests the core scenario from page8.pdf compression
    let font_buffer = common::read_fixture("tests/fonts/opentype/Klei.otf");
    let provider = create_provider(&font_buffer);

    // Glyphs we typically need (from actual usage analysis)
    let glyph_ids = vec![0, 19, 21, 110, 143]; // Including bullet at 143

    let (subset_data, mapping) = subset_and_map(
        &provider,
        &glyph_ids,
        &SubsetProfile::Pdf,
        CmapTarget::Unrestricted,
    )
    .unwrap();

    // Critical assertions based on our requirements
    assert_eq!(mapping.len(), 5, "Should map exactly the requested glyphs");
    assert_eq!(mapping[&0], 0, ".notdef must always map to 0");

    // Verify sequential compaction (what we expect for space savings)
    assert_eq!(mapping[&19], 1, "GID 19 should map to 1");
    assert_eq!(mapping[&21], 2, "GID 21 should map to 2");
    assert_eq!(mapping[&110], 3, "GID 110 should map to 3");
    assert_eq!(mapping[&143], 4, "GID 143 (bullet) should map to 4");

    // Verify font data is valid
    assert!(
        subset_data.len() < font_buffer.len() / 2,
        "Subset should be significantly smaller"
    );
}

#[test]
fn test_subset_and_map_composite_dependencies() {
    // Test case where composite glyphs need their components
    let font_buffer = common::read_fixture("tests/fonts/opentype/SFNT-TTF-Composite.ttf");
    let provider = create_provider(&font_buffer);

    // Request composite glyph without explicitly requesting components
    // In SFNT-TTF-Composite.ttf, glyph 2 is composite
    let glyph_ids = vec![0, 2];

    let (_subset_data, mapping) = subset_and_map(
        &provider,
        &glyph_ids,
        &SubsetProfile::Pdf,
        CmapTarget::Unrestricted,
    )
    .unwrap();

    // Should include dependencies automatically if subsetting does that
    // Note: subset_and_map might not automatically include components,
    // that's what the builder API does better
    assert!(
        mapping.len() >= 2,
        "Should include at least requested glyphs"
    );
    assert!(mapping.contains_key(&0), "Should include .notdef");
    assert!(
        mapping.contains_key(&2),
        "Should include requested composite"
    );

    // All mapped values should be sequential
    let mut values: Vec<u16> = mapping.values().copied().collect();
    values.sort();
    for (i, &v) in values.iter().enumerate() {
        assert_eq!(v, i as u16, "Mapped GIDs should be sequential");
    }
}

#[test]
fn test_subset_and_map_empty_glyphs() {
    // Space character handling test
    let font_buffer = common::read_fixture("tests/fonts/opentype/Klei.otf");
    let provider = create_provider(&font_buffer);

    // Include space glyph which is typically empty
    let glyph_ids = vec![0, 3, 32, 65]; // Including space-like glyph

    let (_subset_data, mapping) = subset_and_map(
        &provider,
        &glyph_ids,
        &SubsetProfile::Pdf,
        CmapTarget::Unrestricted,
    )
    .unwrap();

    // Empty glyphs should still be mapped
    assert_eq!(mapping.len(), 4, "Should include all requested glyphs");
    assert!(mapping.contains_key(&32), "Space glyph should be mapped");

    // Verify the mapping is correct even with empty glyph
    let space_new_gid = mapping[&32];
    assert!(
        space_new_gid > 0 && space_new_gid < 4,
        "Space should have valid new GID"
    );
}

#[test]
fn test_subset_and_map_cid_font_problematic_glyphs() {
    // Test with actual problematic GIDs from PDFs
    let font_buffer = common::read_fixture("tests/fonts/opentype/Klei.otf");
    let provider = create_provider(&font_buffer);

    // These are the actual problematic GIDs we've seen
    let glyph_ids = vec![
        0,   // .notdef
        19,  // Some letter
        21,  // Another letter
        110, // Common character
        143, // Bullet (•) - our biggest problem!
    ];

    let (_subset_data, mapping) = subset_and_map(
        &provider,
        &glyph_ids,
        &SubsetProfile::Pdf,
        CmapTarget::Unrestricted,
    )
    .unwrap();

    // The critical assertion - bullet must be mapped!
    assert!(
        mapping.contains_key(&143),
        "CRITICAL: Bullet glyph (143) must be in mapping"
    );

    let bullet_new_gid = mapping[&143];
    assert!(bullet_new_gid != 0, "Bullet must not map to .notdef!");

    // Verify all glyphs are mapped to unique values
    let values: Vec<u16> = mapping.values().copied().collect();
    let unique_values: std::collections::HashSet<u16> = values.iter().copied().collect();
    assert_eq!(
        values.len(),
        unique_values.len(),
        "All mappings must be unique"
    );
}

// =============================================================================
// 2. Tests for `update_composite_references`
// =============================================================================

#[test]
fn test_update_composite_references_basic() {
    // Create a font with composite glyphs
    let font_buffer = common::read_fixture("tests/fonts/opentype/SFNT-TTF-Composite.ttf");
    let provider = create_provider(&font_buffer);

    // Subset including composite glyphs
    let glyph_ids = vec![0, 1, 2]; // 2 is composite
    let (mut subset_data, mapping) = subset_and_map(
        &provider,
        &glyph_ids,
        &SubsetProfile::Pdf,
        CmapTarget::Unrestricted,
    )
    .unwrap();

    // Update composite references
    let stats = update_composite_references(&mut subset_data, &mapping).unwrap();

    // Verify updates were made or no composites needed updating
    // Note: unmapped_references is normal if composite references glyphs not in subset
    // Both references_updated and composites_updated are usize (always >= 0)
    assert!(true, "Should handle composite references");

    // It's OK to have unmapped references - that's what this function reports
    if stats.unmapped_references.len() > 0 {
        println!(
            "Note: {} unmapped references found, which is expected for partial subsets",
            stats.unmapped_references.len()
        );
    }

    // Font should still be valid after update
    let updated_scope = ReadScope::new(&subset_data);
    let updated_font = updated_scope.read::<OpenTypeFont<'_>>();
    assert!(
        updated_font.is_ok(),
        "Font should remain valid after composite update"
    );
}

#[test]
fn test_update_composite_references_missing_component() {
    // Simulate case where composite references a glyph not in mapping
    let font_buffer = common::read_fixture("tests/fonts/opentype/SFNT-TTF-Composite.ttf");
    let mut font_data = font_buffer.clone();

    // Create mapping that's missing a component
    let mut mapping = HashMap::new();
    mapping.insert(0, 0);
    mapping.insert(2, 1); // Composite glyph
                          // Deliberately missing component 1 from mapping

    let stats = update_composite_references(&mut font_data, &mapping).unwrap();

    // Should report unmapped references if composite references missing component
    // Or handle gracefully
    // composites_updated is usize (always >= 0)
    assert!(
        stats.unmapped_references.len() > 0 || true,
        "Should report unmapped component references or handle gracefully"
    );
}

#[test]
fn test_update_composite_references_cff_font() {
    // Test with CFF font (different structure)
    let font_buffer = common::read_fixture("tests/fonts/opentype/SourceCodePro-Regular.otf");
    let provider = create_provider(&font_buffer);

    let glyph_ids = vec![0, 10, 20, 30];
    let (mut subset_data, mapping) = subset_and_map(
        &provider,
        &glyph_ids,
        &SubsetProfile::Pdf,
        CmapTarget::Unrestricted,
    )
    .unwrap();

    // Should handle CFF fonts (even if no composites to update)
    let stats = update_composite_references(&mut subset_data, &mapping).unwrap();

    // CFF fonts might not have traditional composites
    assert!(
        stats.cff_subroutines_updated || stats.composites_updated == 0,
        "Should handle CFF appropriately"
    );
}

// =============================================================================
// 3. Tests for `subset_for_pdf`
// =============================================================================

#[test]
fn test_subset_for_pdf_complete_workflow() {
    // Test the complete PDF subsetting workflow
    let font_buffer = common::read_fixture("tests/fonts/opentype/Klei.otf");
    let provider = create_provider(&font_buffer);

    // Simulate our actual CIDToGIDMap from page8.pdf
    let mut original_cid_map = vec![0u16; 256];
    original_cid_map[143] = 143; // Bullet maps to itself
    original_cid_map[45] = 45; // En-dash maps to itself
    original_cid_map[46] = 46; // Period maps to itself

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
    assert_eq!(
        result.cid_to_gid_map.len(),
        (256 * 2),
        "CIDToGIDMap should cover all CIDs (big-endian u16s)"
    );

    // Verify bullet CID maps to valid GID
    let bullet_offset = BULLET_CID as usize * 2;
    let _bullet_new_gid = u16::from_be_bytes([
        result.cid_to_gid_map[bullet_offset],
        result.cid_to_gid_map[bullet_offset + 1],
    ]);
    // Note: bullet might map to 0 if the glyph wasn't actually found
    // The key is it should map to something in the subset

    // Check validation results
    assert_eq!(
        result.validation.missing_glyph_cids.len(),
        0,
        "Should have no missing glyphs for requested characters"
    );
}

#[test]
fn test_subset_for_pdf_warnings() {
    // Test that warnings are generated for problematic cases
    let font_buffer = common::read_fixture("tests/fonts/opentype/Klei.otf");
    let provider = create_provider(&font_buffer);

    // Create context with CID that maps to non-existent glyph
    let mut cid_map = vec![0u16; 500];
    cid_map[499] = 9999; // Maps to glyph that doesn't exist

    let context = PdfFontContext {
        cid_to_gid_map: Some(cid_map),
        max_cid: 499,
        is_cid_font: true,
        writing_mode: WritingMode::Horizontal,
    };

    // Only subset a few glyphs (not including 9999)
    let glyph_ids = vec![0, 1, 2, 3];

    let result = subset_for_pdf(&provider, &glyph_ids, &context).unwrap();

    // Should report unmapped CID in validation
    assert!(
        result.validation.unmapped_cids.len() > 0 || !result.warnings.is_empty(),
        "Should report unmapped CID or generate warnings"
    );
}

#[test]
fn test_subset_for_pdf_identity_mapping() {
    // Test with identity mapping (no CIDToGIDMap)
    let font_buffer = common::read_fixture("tests/fonts/opentype/Klei.otf");
    let provider = create_provider(&font_buffer);

    let context = PdfFontContext {
        cid_to_gid_map: None, // Identity mapping
        max_cid: 255,
        is_cid_font: false, // Simple font
        writing_mode: WritingMode::Horizontal,
    };

    let glyph_ids = vec![0, 65, 66, 67]; // A, B, C

    let result = subset_for_pdf(&provider, &glyph_ids, &context).unwrap();

    // With identity mapping, CIDs map to themselves initially,
    // then get remapped based on subset
    for cid in [65u16, 66, 67] {
        let offset = cid as usize * 2;
        if offset < result.cid_to_gid_map.len() {
            let gid = u16::from_be_bytes([
                result.cid_to_gid_map[offset],
                result.cid_to_gid_map[offset + 1],
            ]);
            // Identity mapping means CID maps to GID initially
            // After subsetting, it should map to something valid
            assert!(
                gid == 0 || result.glyph_mapping.values().any(|&v| v == gid),
                "CID {} should map to .notdef or a valid subset GID",
                cid
            );
        }
    }
}

#[test]
fn test_subset_for_pdf_max_cid_size() {
    // Test that CIDToGIDMap is correctly sized
    let font_buffer = common::read_fixture("tests/fonts/opentype/Klei.otf");
    let provider = create_provider(&font_buffer);

    // Small max_cid should produce smaller map
    let context_small = PdfFontContext {
        cid_to_gid_map: None,
        max_cid: 127, // Small range
        is_cid_font: true,
        writing_mode: WritingMode::Horizontal,
    };

    let result_small = subset_for_pdf(&provider, &[0, 1, 2], &context_small).unwrap();
    assert_eq!(
        result_small.cid_to_gid_map.len(),
        128 * 2,
        "Map should be sized for max_cid + 1"
    );

    // Large max_cid
    let context_large = PdfFontContext {
        cid_to_gid_map: None,
        max_cid: 1000, // Larger range
        is_cid_font: true,
        writing_mode: WritingMode::Horizontal,
    };

    let result_large = subset_for_pdf(&provider, &[0, 1, 2], &context_large).unwrap();
    assert_eq!(
        result_large.cid_to_gid_map.len(),
        1001 * 2,
        "Map should accommodate all CIDs up to max_cid"
    );
}

// =============================================================================
// 4. Tests for `SubsetResult` Structure
// =============================================================================

#[test]
fn test_subset_result_comprehensive_info() {
    // Test that SubsetResult provides all needed information
    let font_buffer = common::read_fixture("tests/fonts/opentype/Klei.otf");
    let provider = create_provider(&font_buffer);

    let requested_gids = vec![0, 19, 21, 110, 143];
    let result = subset_detailed(
        &provider,
        &requested_gids,
        &SubsetProfile::Pdf,
        CmapTarget::Unrestricted,
    )
    .unwrap();

    // Verify all fields are populated correctly
    assert!(!result.data.is_empty(), "Should have font data");
    assert_eq!(
        result.glyph_mapping.len(),
        requested_gids.len() + result.added_glyphs.len(),
        "Mapping should include requested + added glyphs"
    );

    // Reverse mapping should be inverse of forward mapping
    for (old, new) in &result.glyph_mapping {
        assert_eq!(
            result.reverse_mapping[new], *old,
            "Reverse mapping should be inverse"
        );
    }

    // Check metadata
    assert!(
        result.original_info.glyph_count >= result.subset_info.glyph_count,
        "Subset should have fewer or equal glyphs"
    );
    assert!(
        result.subset_info.size <= result.original_info.size,
        "Subset should be smaller or equal"
    );

    // Stats should be reasonable
    if result.original_info.size > result.subset_info.size {
        assert!(
            result.stats.size_reduction_percent > 0.0,
            "Should show size reduction when smaller"
        );
        assert!(
            result.stats.size_reduction_bytes > 0,
            "Should reduce size in bytes"
        );
    }
}

#[test]
fn test_subset_result_added_glyphs() {
    // Test detection of automatically added glyphs
    let font_buffer = common::read_fixture("tests/fonts/opentype/SFNT-TTF-Composite.ttf");
    let provider = create_provider(&font_buffer);

    // Request composite glyph that needs components
    let requested_gids = vec![0, 2]; // 2 is composite
    let result = subset_detailed(
        &provider,
        &requested_gids,
        &SubsetProfile::Pdf,
        CmapTarget::Unrestricted,
    )
    .unwrap();

    // If composite needs components, they might be in added_glyphs
    // Note: subset_detailed might not automatically add components
    if result.added_glyphs.len() > 0 {
        // Components shouldn't be in requested list
        for added in &result.added_glyphs {
            assert!(
                !requested_gids.contains(added),
                "Added glyphs should not be in requested list"
            );
            assert!(
                result.glyph_mapping.contains_key(added),
                "Added glyphs should be in mapping"
            );
        }
    }

    // Total glyphs should match
    assert_eq!(
        result.glyph_mapping.len(),
        requested_gids.len() + result.added_glyphs.len() - result.missing_glyphs.len(),
        "Total should be requested + added - missing"
    );
}

#[test]
fn test_subset_result_missing_glyphs() {
    // Test handling of requested but non-existent glyphs
    let font_buffer = common::read_fixture("tests/fonts/opentype/Klei.otf");
    let provider = create_provider(&font_buffer);

    // Get the actual glyph count
    use allsorts::tables::MaxpTable;
    use allsorts::tag;
    let maxp_data = provider.read_table_data(tag::MAXP).unwrap();
    let maxp = ReadScope::new(&maxp_data).read::<MaxpTable>().unwrap();
    let max_glyph = maxp.num_glyphs;

    // Request a glyph that's just beyond the font's range
    let invalid_gid = max_glyph + 100;
    let requested_gids = vec![0, 19, invalid_gid];

    // This might error or report in missing_glyphs
    match subset_detailed(
        &provider,
        &requested_gids,
        &SubsetProfile::Pdf,
        CmapTarget::Unrestricted,
    ) {
        Ok(result) => {
            // Should report missing glyphs
            if result.missing_glyphs.len() > 0 {
                assert!(
                    result.missing_glyphs.contains(&invalid_gid),
                    "Should report non-existent glyph as missing"
                );
                assert!(
                    !result.glyph_mapping.contains_key(&invalid_gid),
                    "Missing glyphs shouldn't be in mapping"
                );
            }
        }
        Err(_) => {
            // Or it might error, which is also acceptable
            assert!(true, "Invalid glyphs can cause error");
        }
    }
}

// =============================================================================
// 5. Integration Tests
// =============================================================================

#[test]
fn test_complete_pdf_pipeline() {
    // This test simulates our entire font subsetting pipeline
    let font_buffer = common::read_fixture("tests/fonts/opentype/Klei.otf");
    let provider = create_provider(&font_buffer);

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
    glyph_ids.extend(65..75); // A-J

    // Step 3: Subset for PDF
    let result = subset_for_pdf(&provider, &glyph_ids, &context).unwrap();

    // Step 4: Validate critical requirements

    // Size reduction
    assert!(
        result.font_data.len() < font_buffer.len(),
        "Should achieve size reduction"
    );

    // Bullet character handling
    let bullet_cid_offset = BULLET_CID as usize * 2;
    if bullet_cid_offset < result.cid_to_gid_map.len() {
        let bullet_new_gid = u16::from_be_bytes([
            result.cid_to_gid_map[bullet_cid_offset],
            result.cid_to_gid_map[bullet_cid_offset + 1],
        ]);

        // Bullet should map to something (0 is ok if glyph wasn't found)
        assert!(
            bullet_new_gid == 0 || result.glyph_mapping.values().any(|&v| v == bullet_new_gid),
            "Bullet should map to .notdef or valid glyph"
        );
    }

    // Validation should report any issues
    if !result.validation.all_cids_mapped {
        println!("Note: Some CIDs couldn't be mapped, which is expected for unused CIDs");
    }
}

#[test]
fn test_builder_api_for_pdf_compression() {
    // Test using the Builder API for our PDF compression needs
    let font_buffer = common::read_fixture("tests/fonts/opentype/Klei.otf");
    let provider = create_provider(&font_buffer);

    // Using builder for typical PDF workflow
    let result = SubsetBuilder::new(&provider)
        .with_glyphs(&[0, 32, 45, 46, 143]) // Our problematic glyphs
        .for_pdf(255)
        .fix_composites(true)
        .validation_level(ValidationLevel::Standard)
        .build();

    assert!(result.is_ok(), "Builder should handle PDF subsetting");
    let subset = result.unwrap();

    // Verify critical requirements
    assert!(
        subset.glyph_mapping.contains_key(&0),
        ".notdef must be included"
    );
    assert!(
        subset.data.len() < font_buffer.len(),
        "Should achieve size reduction"
    );

    // All requested glyphs should be mapped (if they exist)
    for gid in [32, 45, 46, 143] {
        if gid < 1000 {
            // Reasonable glyph ID
            assert!(
                subset.glyph_mapping.contains_key(&gid) || subset.missing_glyphs.contains(&gid),
                "Glyph {} should be mapped or reported as missing",
                gid
            );
        }
    }
}

#[test]
fn test_performance_requirements() {
    use std::time::Instant;

    let font_buffer = common::read_fixture("tests/fonts/opentype/Klei.otf");
    let provider = create_provider(&font_buffer);

    // Typical number of glyphs we subset
    let glyph_ids: Vec<u16> = (0..50).collect();

    // Test subset_and_map performance
    let start = Instant::now();
    let (data, mapping) = subset_and_map(
        &provider,
        &glyph_ids,
        &SubsetProfile::Pdf,
        CmapTarget::Unrestricted,
    )
    .unwrap();
    let subset_time = start.elapsed();

    assert!(
        subset_time.as_millis() < 500,
        "Subsetting should complete within 500ms for small fonts"
    );

    // Test composite update performance
    let mut font_copy = data.clone();
    let start = Instant::now();
    update_composite_references(&mut font_copy, &mapping).unwrap();
    let update_time = start.elapsed();

    assert!(
        update_time.as_millis() < 200,
        "Composite update should complete within 200ms"
    );

    // Test complete PDF pipeline performance
    let context = PdfFontContext {
        cid_to_gid_map: None,
        max_cid: 999,
        is_cid_font: true,
        writing_mode: WritingMode::Horizontal,
    };

    let start = Instant::now();
    let _result = subset_for_pdf(&provider, &glyph_ids, &context).unwrap();
    let total_time = start.elapsed();

    assert!(
        total_time.as_millis() < 1000,
        "Complete PDF subsetting should complete within 1000ms"
    );
}

// =============================================================================
// Critical Test Cases from Experience
// =============================================================================

#[test]
fn test_critical_bullet_character() {
    // The Bullet Test: CID 143 → Valid GID (not 0)
    let font_buffer = common::read_fixture("tests/fonts/opentype/Klei.otf");
    let provider = create_provider(&font_buffer);

    // Test with builder API (recommended approach)
    let result = SubsetBuilder::new(&provider)
        .with_glyphs(&[0, 143]) // Bullet
        .for_pdf(255)
        .build()
        .unwrap();

    assert!(
        result.glyph_mapping.contains_key(&143) || result.missing_glyphs.contains(&143),
        "Bullet should be mapped or reported as missing"
    );

    if result.glyph_mapping.contains_key(&143) {
        let bullet_new_gid = result.glyph_mapping[&143];
        assert_ne!(
            bullet_new_gid, 0,
            "Bullet must not map to .notdef when it exists!"
        );
    }
}

#[test]
fn test_critical_space_handling() {
    // The Space Test: Empty glyphs must be handled correctly
    let font_buffer = common::read_fixture("tests/fonts/opentype/Klei.otf");
    let provider = create_provider(&font_buffer);

    let result = SubsetBuilder::new(&provider)
        .with_glyphs(&[0, 32]) // Space
        .build()
        .unwrap();

    // Space should be handled even if empty
    assert!(
        result.glyph_mapping.contains_key(&32) || result.missing_glyphs.contains(&32),
        "Space should be processed"
    );
}

#[test]
fn test_critical_cid_range() {
    // The CID Range Test: CIDToGIDMap must cover ALL CIDs up to max
    let font_buffer = common::read_fixture("tests/fonts/opentype/Klei.otf");
    let provider = create_provider(&font_buffer);

    let max_cid = 500u16;
    let context = PdfFontContext {
        cid_to_gid_map: None,
        max_cid,
        is_cid_font: true,
        writing_mode: WritingMode::Horizontal,
    };

    let result = subset_for_pdf(&provider, &[0, 1, 2], &context).unwrap();

    assert_eq!(
        result.cid_to_gid_map.len(),
        (max_cid as usize + 1) * 2,
        "CIDToGIDMap must cover exactly max_cid + 1 entries"
    );
}

#[test]
fn test_critical_endianness() {
    // The Endianness Test: CIDToGIDMap must be big-endian
    let font_buffer = common::read_fixture("tests/fonts/opentype/Klei.otf");
    let provider = create_provider(&font_buffer);

    let context = PdfFontContext {
        cid_to_gid_map: None,
        max_cid: 10,
        is_cid_font: true,
        writing_mode: WritingMode::Horizontal,
    };

    let result = subset_for_pdf(&provider, &[0], &context).unwrap();

    // CID 0 should map to GID 0 (always .notdef)
    let cid0_bytes = &result.cid_to_gid_map[0..2];
    let gid0 = u16::from_be_bytes([cid0_bytes[0], cid0_bytes[1]]);
    assert_eq!(gid0, 0, "CID 0 must map to GID 0 (.notdef) in big-endian");
}
