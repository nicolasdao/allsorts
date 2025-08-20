// Integration tests for CID font detection issue
// Tests that TrueType fonts used as CID fonts in PDFs are correctly detected

mod common;

use allsorts::binary::read::ReadScope;
use allsorts::font_data::FontData;
use allsorts::subset::{subset_and_map, CmapTarget, SubsetProfile, SubsetResult};
use std::fs;
use std::path::PathBuf;

/// Load the actual CID font binary from specs directory
fn load_calibri_cid_font() -> Vec<u8> {
    let mut path = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
    path.push("specs/250820_02_fix_bug/calibri_cid_font.bin");

    fs::read(&path).unwrap_or_else(|e| {
        panic!("Failed to read calibri_cid_font.bin from {:?}: {}", path, e);
    })
}

#[test]
fn test_truetype_cid_font_detection() {
    // Load the actual Calibri CID font (TrueType font used as CIDFontType2 in PDF)
    let font_data = load_calibri_cid_font();
    assert_eq!(
        font_data.len(),
        34240,
        "Expected calibri_cid_font.bin to be 34240 bytes"
    );

    // These glyph IDs are from actual PDF usage - note the sparse values
    let glyph_ids: Vec<u16> = vec![
        0,   // .notdef
        3,   // Common ASCII
        43,  // Plus sign
        45,  // Hyphen
        46,  // Period
        54,  // Digit 6
        69,  // Letter E
        71,  // Letter G
        80,  // Letter P
        91,  // Left bracket
        93,  // Right bracket
        100, // Letter d
        102, // Letter f
        104, // Letter h
        106, // Letter j
        107, // Letter k
        143, // Bullet (•) - sparse GID!
        159, // Another special char - sparse GID!
        178, // Another special char - sparse GID!
    ];

    println!("Testing TrueType CID font detection");
    println!("  Font data size: {} bytes", font_data.len());
    println!("  Glyph IDs to subset: {:?}", glyph_ids);

    // Parse the font
    let scope = ReadScope::new(&font_data);
    let font_file = scope
        .read::<FontData<'_>>()
        .expect("Failed to parse font data");

    let provider = font_file
        .table_provider(0)
        .expect("Failed to get table provider");

    // Call subset_and_map API
    let result = subset_and_map(
        &provider,
        &glyph_ids,
        &SubsetProfile::Pdf,
        CmapTarget::Unicode, // Recommended for CID fonts
    )
    .expect("subset_and_map failed");

    // Verify the result type
    match result {
        SubsetResult::Simple {
            font_data,
            glyph_mapping,
        } => {
            // This is the BUG - CID fonts should NOT return Simple
            eprintln!("ERROR: TrueType CID font returned SubsetResult::Simple!");
            eprintln!("  Font data size: {} bytes", font_data.len());
            eprintln!("  Glyph mapping entries: {}", glyph_mapping.len());
            eprintln!("  This will cause rendering issues - missing CIDToGIDMap!");

            panic!("TrueType font used as CID font must return SubsetResult::Cid, not SubsetResult::Simple!");
        }
        SubsetResult::Cid {
            font_data,
            glyph_mapping,
            cid_to_gid_map,
        } => {
            // This is the expected result for CID fonts
            println!("SUCCESS: Got SubsetResult::Cid as expected!");
            println!("  Subsetted font size: {} bytes", font_data.len());
            println!("  Glyph mapping entries: {}", glyph_mapping.len());
            println!("  CIDToGIDMap size: {} bytes", cid_to_gid_map.len());

            // Verify the CIDToGIDMap is properly sized
            // For sparse glyph usage, we should have a map up to the highest GID
            assert!(
                cid_to_gid_map.len() >= 178 * 2,
                "CIDToGIDMap should cover at least up to GID 178"
            );

            // Verify mappings exist
            assert!(
                !glyph_mapping.is_empty(),
                "Glyph mapping should not be empty"
            );

            println!("Test PASSED - TrueType CID font correctly detected and processed!");
        }
    }
}

#[test]
fn test_sparse_glyph_ids_indicate_cid_font() {
    // This test demonstrates why sparse glyph IDs indicate CID font usage
    let font_data = load_calibri_cid_font();

    // Sparse glyph IDs are a clear indicator of CID font usage
    // The font might have only 225 glyphs, but uses GID 143, 178 for special chars
    let sparse_glyph_ids = vec![0, 143, 178];

    println!("Testing sparse glyph ID detection");
    println!("  Sparse glyph IDs: {:?}", sparse_glyph_ids);

    let scope = ReadScope::new(&font_data);
    let font_file = scope.read::<FontData<'_>>().unwrap();
    let provider = font_file.table_provider(0).unwrap();

    let result = subset_and_map(
        &provider,
        &sparse_glyph_ids,
        &SubsetProfile::Pdf,
        CmapTarget::Unicode,
    )
    .expect("subset_and_map failed");

    // Fonts with sparse glyph IDs MUST return Cid variant
    assert!(
        matches!(result, SubsetResult::Cid { .. }),
        "Sparse glyph IDs (0, 143, 178) indicate CID font usage - must return SubsetResult::Cid!"
    );
}

#[test]
fn test_subset_and_map_with_hint() {
    // Test the new API that allows explicit CID font specification
    use allsorts::subset::subset_and_map_with_hint;

    let font_data = load_calibri_cid_font();

    // Use regular ASCII glyphs that normally wouldn't trigger CID detection
    let ascii_glyph_ids = vec![0, 1, 2, 3, 4, 5];

    println!("Testing subset_and_map_with_hint API");
    println!("  ASCII glyph IDs: {:?}", ascii_glyph_ids);

    let scope = ReadScope::new(&font_data);
    let font_file = scope.read::<FontData<'_>>().unwrap();
    let provider = font_file.table_provider(0).unwrap();

    // Without hint, these ASCII glyphs shouldn't trigger CID detection
    let result_without_hint = subset_and_map(
        &provider,
        &ascii_glyph_ids,
        &SubsetProfile::Pdf,
        CmapTarget::Unicode,
    )
    .expect("subset_and_map failed");

    assert!(
        matches!(result_without_hint, SubsetResult::Simple { .. }),
        "ASCII glyphs alone should not trigger CID detection"
    );

    // With hint, force CID treatment
    let result_with_hint = subset_and_map_with_hint(
        &provider,
        &ascii_glyph_ids,
        &SubsetProfile::Pdf,
        CmapTarget::Unicode,
        true, // Force CID treatment
    )
    .expect("subset_and_map_with_hint failed");

    assert!(
        matches!(result_with_hint, SubsetResult::Cid { .. }),
        "With force_cid=true, should return SubsetResult::Cid"
    );

    println!("Test PASSED - subset_and_map_with_hint API works correctly!");
}
