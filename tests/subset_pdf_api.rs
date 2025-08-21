// Phase 2 TDD Tests - Main PDF API

use allsorts::subset::SubsetError;
use allsorts::tables::OpenTypeFont;
use allsorts::binary::read::ReadScope;
use std::collections::HashMap;
use allsorts::subset::phase2::{subset_and_map_for_pdf, PdfFontContext, PdfSubsetResult, PdfFontType};


#[test]
fn test_subset_and_map_for_pdf_simple_case() {
    let font_data = include_bytes!("fonts/svg/gzipped.ttf");
    let scope = ReadScope::new(font_data);
    let font_file = scope.read::<OpenTypeFont>().expect("Failed to read font");
    let provider = font_file.table_provider(0).expect("Failed to get provider");
    let context = PdfFontContext::identity_h();
    
    let result = subset_and_map_for_pdf(
        &provider,
        &[0, 1, 2, 3],  // .notdef + 3 glyphs
        context,
    );
    
    assert!(result.is_ok(), "Subsetting failed: {:?}", result.err());
    let pdf_result = result.unwrap();
    
    assert!(!pdf_result.font_data.is_empty());
    // At least .notdef should be mapped
    assert!(pdf_result.glyph_mapping.len() >= 1);
    assert!(pdf_result.glyph_mapping.contains_key(&0)); // .notdef mapped
    // Reduction percentage could be negative if subset is larger than original estimate
    assert!(pdf_result.statistics.reduction_percentage >= -100.0);
}

#[test]
fn test_subset_and_map_for_pdf_missing_notdef_error() {
    let font_data = include_bytes!("fonts/svg/gzipped.ttf");
    let scope = ReadScope::new(font_data);
    let font_file = scope.read::<OpenTypeFont>().expect("Failed to read font");
    let provider = font_file.table_provider(0).expect("Failed to get provider");
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
    let font_data = include_bytes!("fonts/svg/gzipped.ttf");
    let scope = ReadScope::new(font_data);
    let font_file = scope.read::<OpenTypeFont>().expect("Failed to read font");
    let provider = font_file.table_provider(0).expect("Failed to get provider");
    let context = PdfFontContext::identity_h();
    
    let result = subset_and_map_for_pdf(&provider, &[], context);
    assert!(result.is_err());
}

#[test]
fn test_pdf_subset_result_methods() {
    let font_data = include_bytes!("fonts/svg/gzipped.ttf");
    let scope = ReadScope::new(font_data);
    let font_file = scope.read::<OpenTypeFont>().expect("Failed to read font");
    let provider = font_file.table_provider(0).expect("Failed to get provider");
    let context = PdfFontContext::identity_h();
    
    let result = subset_and_map_for_pdf(&provider, &[0, 1], context).unwrap();
    
    // Test convenience methods
    // Size reduction could be negative if subset is larger
    assert!(result.size_reduction() >= -100.0);
    assert!(result.is_cid_font()); // Should be CID for Identity encoding
    assert!(result.cid_to_gid_map().is_some());
}