use allsorts::binary::read::ReadScope;
use allsorts::font_data::FontData;
use allsorts::subset::{subset_and_map, CmapTarget, SubsetProfile, SubsetResult};

#[test]
fn test_subset_should_work_with_all_glyphs() {
    // Load a test font
    let font_bytes = include_bytes!("fonts/gujarati/padmaa.ttf");

    let scope = ReadScope::new(font_bytes);
    let font_file = scope
        .read::<FontData<'_>>()
        .expect("Failed to parse font data");

    let provider = font_file
        .table_provider(0)
        .expect("Failed to get table provider");

    // This SHOULD work but DOESN'T due to the bug
    let glyphs_including_225: Vec<u16> = (0..=225).collect();

    // This call SHOULD succeed but WILL panic with Parse(BadIndex)
    let result = subset_and_map(
        &provider,
        &glyphs_including_225,
        &SubsetProfile::Pdf,
        CmapTarget::Unrestricted,
    )
    .expect("subset_and_map should handle all valid glyph IDs but fails at 225");

    match result {
        SubsetResult::Cid {
            font_data,
            glyph_mapping,
            cid_to_gid_map,
        } => {
            println!("✓ Successfully subsetted CID font");
            println!("  Font data: {} bytes", font_data.len());
            println!("  Glyph mapping: {} entries", glyph_mapping.len());
            println!("  CIDToGIDMap: {} bytes", cid_to_gid_map.len());
        }
        SubsetResult::Simple {
            font_data,
            glyph_mapping,
        } => {
            println!("✓ Successfully subsetted simple font");
            println!("  Font data: {} bytes", font_data.len());
            println!("  Glyph mapping: {} entries", glyph_mapping.len());
        }
    }
}

#[test]
fn test_subset_works_up_to_224_but_not_225() {
    let font_bytes = include_bytes!("fonts/gujarati/padmaa.ttf");

    let scope = ReadScope::new(font_bytes);
    let font_file = scope
        .read::<FontData<'_>>()
        .expect("Failed to parse font data");

    let provider = font_file
        .table_provider(0)
        .expect("Failed to get table provider");

    // First prove that it works with 224
    let glyphs_up_to_224: Vec<u16> = (0..=224).collect();
    let result_224 = subset_and_map(
        &provider,
        &glyphs_up_to_224,
        &SubsetProfile::Pdf,
        CmapTarget::Unrestricted,
    );

    assert!(
        result_224.is_ok(),
        "Subsetting with glyphs 0..=224 should work"
    );
    println!("✓ Subsetting with glyphs 0..=224 works");

    // Now show that adding just one more glyph (225) breaks it
    let glyphs_up_to_225: Vec<u16> = (0..=225).collect();
    let result_225 = subset_and_map(
        &provider,
        &glyphs_up_to_225,
        &SubsetProfile::Pdf,
        CmapTarget::Unrestricted,
    );

    // This assertion WILL FAIL, demonstrating the bug
    assert!(
        result_225.is_ok(),
        "Subsetting with glyphs 0..=225 should also work, but it fails with: {:?}",
        result_225.err()
    );
}

#[test]
fn test_minimal_case_glyph_225_should_work() {
    let font_bytes = include_bytes!("fonts/gujarati/padmaa.ttf");

    let scope = ReadScope::new(font_bytes);
    let font_file = scope
        .read::<FontData<'_>>()
        .expect("Failed to parse font data");

    let provider = font_file
        .table_provider(0)
        .expect("Failed to get table provider");

    // Minimal test: just .notdef (0) and glyph 225
    let result = subset_and_map(
        &provider,
        &[0, 225], // Just two glyphs
        &SubsetProfile::Pdf,
        CmapTarget::Unrestricted,
    );

    // This will fail and show the error
    assert!(
        result.is_ok(),
        "Failed to subset with glyphs [0, 225]. Error: {:?}",
        result.err()
    );
}
