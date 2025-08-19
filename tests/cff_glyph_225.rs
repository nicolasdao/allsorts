use allsorts::binary::read::ReadScope;
use allsorts::font_data::FontData;
use allsorts::subset::{subset_and_map, SubsetProfile, CmapTarget, SubsetResult};
use allsorts::tables::FontTableProvider;

#[test]
fn test_cff_subset_beyond_iso_adobe_charset() {
    // Test with a CFF font
    let font_bytes = include_bytes!("fonts/opentype/Klei.otf");
    
    let scope = ReadScope::new(font_bytes);
    let font_file = scope.read::<FontData<'_>>()
        .expect("Failed to parse font data");
    
    let provider = font_file.table_provider(0)
        .expect("Failed to get table provider");
    
    // Get the maximum glyph count
    let maxp_data = provider.read_table_data(allsorts::tag::MAXP)
        .expect("Failed to read maxp table");
    let maxp = ReadScope::new(&maxp_data)
        .read::<allsorts::tables::MaxpTable>()
        .expect("Failed to parse maxp table");
    
    let num_glyphs = maxp.num_glyphs;
    println!("Font has {} glyphs", num_glyphs);
    
    // Test subsetting with glyphs up to and including 225 (if the font has that many)
    let max_test_glyph = std::cmp::min(225, num_glyphs - 1);
    let glyphs: Vec<u16> = (0..=max_test_glyph).collect();
    
    println!("Testing subset with glyphs 0..={}", max_test_glyph);
    
    let result = subset_and_map(
        &provider,
        &glyphs,
        &SubsetProfile::Pdf,
        CmapTarget::Unrestricted,
    );
    
    assert!(
        result.is_ok(),
        "Failed to subset with glyphs 0..={}. Error: {:?}",
        max_test_glyph,
        result.err()
    );
    
    match result.unwrap() {
        SubsetResult::Cid { font_data, glyph_mapping, cid_to_gid_map } => {
            println!("✓ Successfully subsetted CID font");
            println!("  Font data: {} bytes", font_data.len());
            println!("  Glyph mapping: {} entries", glyph_mapping.len());
            println!("  CIDToGIDMap: {} bytes", cid_to_gid_map.len());
        }
        SubsetResult::Simple { font_data, glyph_mapping } => {
            println!("✓ Successfully subsetted simple font");
            println!("  Font data: {} bytes", font_data.len());
            println!("  Glyph mapping: {} entries", glyph_mapping.len());
        }
    }
}

#[test]
fn test_cff_subset_with_high_glyph_ids() {
    // Test with a CFF font, using sparse glyph IDs including high ones
    let font_bytes = include_bytes!("fonts/opentype/Klei.otf");
    
    let scope = ReadScope::new(font_bytes);
    let font_file = scope.read::<FontData<'_>>()
        .expect("Failed to parse font data");
    
    let provider = font_file.table_provider(0)
        .expect("Failed to get table provider");
    
    // Get the maximum glyph count
    let maxp_data = provider.read_table_data(allsorts::tag::MAXP)
        .expect("Failed to read maxp table");
    let maxp = ReadScope::new(&maxp_data)
        .read::<allsorts::tables::MaxpTable>()
        .expect("Failed to parse maxp table");
    
    let num_glyphs = maxp.num_glyphs;
    
    // Test with sparse glyphs including some high IDs
    let mut glyphs = vec![0]; // Always include .notdef
    
    // Add some glyphs that would be beyond ISOAdobe charset (228)
    for id in [100, 200, 225, 230, 240, 250].iter() {
        if *id < num_glyphs {
            glyphs.push(*id);
        }
    }
    
    println!("Testing subset with sparse glyphs: {:?}", glyphs);
    
    let result = subset_and_map(
        &provider,
        &glyphs,
        &SubsetProfile::Pdf,
        CmapTarget::Unrestricted,
    );
    
    assert!(
        result.is_ok(),
        "Failed to subset with sparse glyphs. Error: {:?}",
        result.err()
    );
    
    println!("✓ Successfully subsetted with sparse high glyph IDs");
}