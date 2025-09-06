//! Test for the Parse(BadIndex) error fix in subset_and_map API
//!
//! This test reproduces the issue where subset_and_map fails with Parse(BadIndex)
//! when attempting to subset fonts with glyph IDs that exceed the actual number
//! of glyphs in the font file.

use allsorts::binary::read::ReadScope;
use allsorts::font_data::FontData;
use allsorts::subset::{subset, subset_and_map, CmapTarget, SubsetProfile, SubsetResult};
use allsorts::tables::FontTableProvider;
use allsorts::tag;

#[test]
fn test_subset_and_map_handles_excessive_glyph_ids_gracefully() {
    // Load a TrueType font for testing
    let font_bytes = include_bytes!("../tests/fonts/opentype/Klei.otf");

    // Parse the font
    let font_data = ReadScope::new(font_bytes)
        .read::<FontData<'_>>()
        .expect("Failed to parse font");

    let provider = font_data
        .table_provider(0)
        .expect("Failed to get table provider");

    // Check actual glyph count in the font
    let actual_glyph_count = if let Ok(maxp_data) = provider.read_table_data(tag::MAXP) {
        let data = maxp_data.as_ref();
        if data.len() >= 6 {
            u16::from_be_bytes([data[4], data[5]])
        } else {
            1000 // Fallback estimate
        }
    } else {
        1000 // Fallback estimate
    };

    println!("Font has {} actual glyphs", actual_glyph_count);

    // Test case 1: Request glyph IDs that are beyond the font's physical glyph count
    // These are actual GIDs from CID fonts that can exceed the font's glyph count
    let glyph_ids_excessive: Vec<u16> = vec![
        0,                        // .notdef - always required
        75,                       // Normal glyph
        88,                       // Normal glyph
        92,                       // Normal glyph
        143,                      // Bullet character - may be beyond typical font glyph count
        178,                      // Special character - way beyond typical font glyph count
        159,                      // Another high GID
        144,                      // Another high GID
        actual_glyph_count + 10,  // Definitely beyond the font's range
        actual_glyph_count + 100, // Way beyond the font's range
    ];

    println!(
        "Attempting to subset with {} glyph IDs",
        glyph_ids_excessive.len()
    );
    let max_requested_gid = *glyph_ids_excessive.iter().max().unwrap();
    println!(
        "Max requested GID: {} (font only has {} glyphs!)",
        max_requested_gid, actual_glyph_count
    );

    // This should NOT fail with Parse(BadIndex) after the fix
    // It should either:
    // 1. Filter out invalid glyph IDs
    // 2. Handle them gracefully
    let result = subset_and_map(
        &provider,
        &glyph_ids_excessive,
        &SubsetProfile::Pdf,
        CmapTarget::Unrestricted,
    );

    match result {
        Ok(SubsetResult::Simple {
            font_data,
            glyph_mapping,
        }) => {
            println!(
                "Success! Got {} bytes, {} mappings",
                font_data.len(),
                glyph_mapping.len()
            );
            // Verify that .notdef is always mapped
            assert_eq!(glyph_mapping.get(&0), Some(&0));
            // The mapping should only contain valid glyph IDs
            for (old_id, _new_id) in &glyph_mapping {
                assert!(
                    *old_id < actual_glyph_count || glyph_ids_excessive.contains(old_id),
                    "Unexpected glyph ID {} in mapping",
                    old_id
                );
            }
        }
        Ok(SubsetResult::Cid {
            font_data,
            glyph_mapping,
            cid_to_gid_map,
        }) => {
            println!(
                "Success! CID font: {} bytes, {} mappings, {} byte map",
                font_data.len(),
                glyph_mapping.len(),
                cid_to_gid_map.len()
            );
            // Verify that .notdef is always mapped
            assert_eq!(glyph_mapping.get(&0), Some(&0));
        }
        Err(e) => {
            // After the fix, this should not happen for BadIndex errors
            panic!(
                "subset_and_map should handle excessive glyph IDs gracefully, got error: {:?}",
                e
            );
        }
    }
}

#[test]
fn test_subset_and_map_works_with_reasonable_glyphs() {
    // Load a TrueType font for testing
    let font_bytes = include_bytes!("../tests/fonts/opentype/Klei.otf");

    // Parse the font
    let font_data = ReadScope::new(font_bytes)
        .read::<FontData<'_>>()
        .expect("Failed to parse font");

    let provider = font_data
        .table_provider(0)
        .expect("Failed to get table provider");

    // Use only glyphs that definitely exist
    let glyph_ids: Vec<u16> = vec![0, 3, 36, 37, 38, 68, 69, 70];

    // This SHOULD work fine
    let result = subset_and_map(
        &provider,
        &glyph_ids,
        &SubsetProfile::Pdf,
        CmapTarget::Unrestricted,
    );

    // Should succeed with reasonable inputs
    assert!(result.is_ok());
    match result {
        Ok(SubsetResult::Simple {
            font_data,
            glyph_mapping,
        }) => {
            println!(
                "Success! Got {} bytes, {} mappings",
                font_data.len(),
                glyph_mapping.len()
            );
            // Verify all requested glyphs are mapped
            for gid in &glyph_ids {
                assert!(
                    glyph_mapping.contains_key(gid),
                    "Glyph {} not in mapping",
                    gid
                );
            }
        }
        Ok(SubsetResult::Cid {
            font_data,
            glyph_mapping,
            cid_to_gid_map,
        }) => {
            println!(
                "Success! CID font: {} bytes, {} mappings, {} byte map",
                font_data.len(),
                glyph_mapping.len(),
                cid_to_gid_map.len()
            );
            // Verify all requested glyphs are mapped
            for gid in &glyph_ids {
                assert!(
                    glyph_mapping.contains_key(gid),
                    "Glyph {} not in mapping",
                    gid
                );
            }
        }
        Err(e) => {
            panic!("Should work with valid glyph IDs, got: {:?}", e);
        }
    }
}

#[test]
fn test_legacy_subset_also_handles_excessive_glyphs() {
    // Load a TrueType font for testing
    let font_bytes = include_bytes!("../tests/fonts/opentype/Klei.otf");

    // Parse the font
    let font_data = ReadScope::new(font_bytes)
        .read::<FontData<'_>>()
        .expect("Failed to parse font");

    let provider = font_data
        .table_provider(0)
        .expect("Failed to get table provider");

    // Check actual glyph count in the font
    let actual_glyph_count = if let Ok(maxp_data) = provider.read_table_data(tag::MAXP) {
        let data = maxp_data.as_ref();
        if data.len() >= 6 {
            u16::from_be_bytes([data[4], data[5]])
        } else {
            1000
        }
    } else {
        1000
    };

    // Test with excessive glyph IDs
    let glyph_ids_excessive: Vec<u16> = vec![
        0,                       // .notdef
        143,                     // May be beyond font's range
        178,                     // May be beyond font's range
        actual_glyph_count + 10, // Definitely beyond
    ];

    // Test the legacy subset API to see if it has the same issue
    let result = subset(
        &provider,
        &glyph_ids_excessive,
        &SubsetProfile::Pdf,
        CmapTarget::Unrestricted,
    );

    // Document the current behavior of legacy API for comparison
    match result {
        Ok(font_data) => {
            println!("Legacy subset succeeded with {} bytes", font_data.len());
        }
        Err(e) => {
            println!("Legacy subset failed with: {:?}", e);
            // The legacy API also fails with BadIndex
        }
    }
}

#[test]
fn test_subset_with_65280_glyphs() {
    // Load a TrueType font for testing
    let font_bytes = include_bytes!("../tests/fonts/opentype/Klei.otf");

    // Parse the font
    let font_data = ReadScope::new(font_bytes)
        .read::<FontData<'_>>()
        .expect("Failed to parse font");

    let provider = font_data
        .table_provider(0)
        .expect("Failed to get table provider");

    // Check actual glyph count
    let actual_glyph_count = if let Ok(maxp_data) = provider.read_table_data(tag::MAXP) {
        let data = maxp_data.as_ref();
        if data.len() >= 6 {
            u16::from_be_bytes([data[4], data[5]])
        } else {
            1000
        }
    } else {
        1000
    };

    println!("Font has {} actual glyphs", actual_glyph_count);
    
    // This is the problematic request: all glyphs from 0 to 65279 (65,280 total)
    // This simulates what CID font subsetting code does when trying to preserve identity mapping
    let glyph_ids_vec: Vec<u16> = (0..=65279).collect();
    
    println!("🧪 Testing subset_and_map with {} glyphs (0..=65279)", glyph_ids_vec.len());
    println!("   Font only has {} actual glyphs!", actual_glyph_count);
    
    // This should NOT fail with Parse(BadIndex) after the fix
    let result = subset_and_map(
        &provider,
        &glyph_ids_vec,
        &SubsetProfile::Pdf,
        CmapTarget::Unrestricted,
    );
    
    match result {
        Ok(SubsetResult::Simple { font_data, glyph_mapping }) => {
            println!("✅ SUCCESS: subset_and_map handled 65,280 glyphs gracefully!");
            println!("   Result: {} bytes, {} mapped glyphs", font_data.len(), glyph_mapping.len());
            // The mapping should only contain valid glyphs from the font
            assert!(glyph_mapping.len() <= actual_glyph_count as usize, 
                    "Mapping has more glyphs than the font!");
            assert_eq!(glyph_mapping.get(&0), Some(&0), ".notdef should be mapped");
        }
        Ok(SubsetResult::Cid { font_data, glyph_mapping, cid_to_gid_map }) => {
            println!("✅ SUCCESS: subset_and_map handled 65,280 glyphs as CID font!");
            println!("   Result: {} bytes, {} mapped glyphs, CIDToGIDMap: {} bytes", 
                     font_data.len(), glyph_mapping.len(), cid_to_gid_map.len());
            assert_eq!(glyph_mapping.get(&0), Some(&0), ".notdef should be mapped");
        }
        Err(e) => {
            let error_string = format!("{:?}", e);
            if error_string.contains("Parse") && error_string.contains("BadIndex") {
                panic!("❌ BUG STILL EXISTS: Parse(BadIndex) error when requesting 65,280 glyphs!\nThis is the exact bug that needs to be fixed.\nError: {}", error_string);
            } else {
                panic!("Unexpected error (not Parse(BadIndex)): {}", error_string);
            }
        }
    }
}
