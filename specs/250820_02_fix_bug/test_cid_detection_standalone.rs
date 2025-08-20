// Standalone test to reproduce CID font detection issue in allsorts
// 
// To run this test:
// 1. Add to your Cargo.toml:
//    [dependencies]
//    allsorts = { git = "https://github.com/nicolasdao/allsorts.git", tag = "0.15.2" }
//    lopdf = "0.32"
//
// 2. Place this file in tests/ directory
// 3. Run: cargo test test_cid_detection_standalone
//
// Expected: Test should pass (CID font returns SubsetResult::Cid)
// Actual: Test fails (CID font returns SubsetResult::Simple)

use allsorts::subset::{subset_and_map, SubsetProfile, CmapTarget, SubsetResult};
use allsorts::binary::read::ReadScope;
use allsorts::font_data::FontData;
use lopdf::{Document, Object};

/// Extracts a real CID font from a PDF for testing
/// This ensures we're testing with actual production data
fn get_cid_font_data() -> Vec<u8> {
    // Option 1: Load from a test PDF if available
    if let Ok(doc) = Document::load("test_zone/page8.pdf") {
        for (_, object) in doc.objects.iter() {
            if let Ok(dict) = object.as_dict() {
                if dict.get(b"Type").and_then(|o| o.as_name()).ok() == Some(b"Font") {
                    if dict.get(b"Subtype").and_then(|o| o.as_name()).ok() == Some(b"Type0") {
                        if dict.get(b"Encoding").and_then(|o| o.as_name()).ok() == Some(b"Identity-H") {
                            // Found a CID font - extract its data
                            if let Ok(descendants) = dict.get(b"DescendantFonts").and_then(|o| o.as_array()) {
                                if let Some(Object::Reference(desc_ref)) = descendants.first() {
                                    if let Ok(desc_font) = doc.get_object(*desc_ref).and_then(|o| o.as_dict()) {
                                        if let Ok(Object::Reference(fd_ref)) = desc_font.get(b"FontDescriptor") {
                                            if let Ok(fd) = doc.get_object(*fd_ref).and_then(|o| o.as_dict()) {
                                                if let Ok(Object::Reference(ff_ref)) = fd.get(b"FontFile2") {
                                                    if let Ok(stream) = doc.get_object(*ff_ref) {
                                                        if let Ok(font_stream) = stream.as_stream() {
                                                            return font_stream.decompressed_content()
                                                                .unwrap_or(font_stream.content.clone());
                                                        }
                                                    }
                                                }
                                            }
                                        }
                                    }
                                }
                            }
                        }
                    }
                }
            }
        }
    }
    
    // Option 2: Use embedded test data (first 1000 bytes of Calibri CID font)
    // This is actual data from a CID font used with Identity-H encoding
    vec![
        0x00, 0x01, 0x00, 0x00, 0x00, 0x13, 0x01, 0x00, 0x00, 0x04, 0x00, 0x30,
        0x44, 0x53, 0x49, 0x47, 0xE2, 0x5A, 0x5D, 0xDD, 0x00, 0x00, 0x85, 0xC0,
        0x00, 0x00, 0x00, 0x08, 0x47, 0x44, 0x45, 0x46, 0x02, 0x71, 0x00, 0x23,
        // ... truncated for brevity
        // In production, include the full font data or load from file
    ]
}

#[test]
fn test_cid_font_returns_cid_result() {
    // Get actual CID font data
    let font_data = get_cid_font_data();
    assert!(!font_data.is_empty(), "Failed to get font data");
    
    // These glyph IDs are typical for CID fonts - note the sparse values
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
    
    println!("Testing with font data: {} bytes", font_data.len());
    println!("Glyph IDs to subset: {:?}", glyph_ids);
    
    // Parse the font
    let scope = ReadScope::new(&font_data);
    let font_file = scope.read::<FontData<'_>>()
        .expect("Failed to parse font data");
    
    let provider = font_file.table_provider(0)
        .expect("Failed to get table provider");
    
    // Call subset_and_map API
    let result = subset_and_map(
        &provider,
        &glyph_ids,
        &SubsetProfile::Pdf,
        CmapTarget::Unicode, // Recommended for CID fonts
    ).expect("subset_and_map failed");
    
    // Verify the result type
    match result {
        SubsetResult::Simple { font_data, glyph_mapping } => {
            // This is the BUG - CID fonts should NOT return Simple
            eprintln!("ERROR: CID font returned SubsetResult::Simple!");
            eprintln!("  Font data size: {} bytes", font_data.len());
            eprintln!("  Glyph mapping entries: {}", glyph_mapping.len());
            eprintln!("  This will cause rendering issues - missing CIDToGIDMap!");
            
            panic!("CID font with Identity-H encoding must return SubsetResult::Cid, not SubsetResult::Simple!");
        }
        SubsetResult::Cid { font_data, glyph_mapping, cid_to_gid_map } => {
            // This is the expected result for CID fonts
            println!("SUCCESS: Got SubsetResult::Cid as expected!");
            println!("  Subsetted font size: {} bytes", font_data.len());
            println!("  Glyph mapping entries: {}", glyph_mapping.len());
            println!("  CIDToGIDMap size: {} bytes", cid_to_gid_map.len());
            
            // Verify the CIDToGIDMap is properly sized
            // For Identity-H, this should cover the full CID range (0-65535)
            assert!(cid_to_gid_map.len() >= 131072, 
                "CIDToGIDMap should be at least 131072 bytes (65536 * 2) for full CID coverage");
            
            // Verify some mappings exist
            assert!(glyph_mapping.len() > 0, "Glyph mapping should not be empty");
            
            println!("Test PASSED - CID font correctly detected and processed!");
        }
    }
}

#[test]
fn test_sparse_glyph_ids_indicate_cid_font() {
    // This test demonstrates why sparse glyph IDs indicate CID font usage
    let font_data = get_cid_font_data();
    
    // Sparse glyph IDs are a clear indicator of CID font usage
    // The font might have only 200 glyphs, but CID 143 maps to GID 143
    let sparse_glyph_ids = vec![0, 143, 178];
    
    let scope = ReadScope::new(&font_data);
    let font_file = scope.read::<FontData<'_>>().unwrap();
    let provider = font_file.table_provider(0).unwrap();
    
    let result = subset_and_map(
        &provider,
        &sparse_glyph_ids,
        &SubsetProfile::Pdf,
        CmapTarget::Unicode,
    ).expect("subset_and_map failed");
    
    // Fonts with sparse glyph IDs MUST return Cid variant
    assert!(
        matches!(result, SubsetResult::Cid { .. }),
        "Sparse glyph IDs (0, 143, 178) indicate CID font usage - must return SubsetResult::Cid!"
    );
}

fn main() {
    println!("Run with: cargo test");
}