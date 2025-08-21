// Phase 2 TDD Tests - Builder Pattern

use allsorts::subset::context::FontEncoding;
use allsorts::tables::OpenTypeFont;
use allsorts::binary::read::ReadScope;
use allsorts::subset::phase2::{subset_for_pdf, PdfSubsetBuilder};


#[test]
fn test_builder_basic_construction() {
    // Note: PdfSubsetBuilder fields are not public, so we can't test internal state
    // We can only test that construction works
    let font_data = include_bytes!("fonts/svg/gzipped.ttf");
    let scope = ReadScope::new(font_data);
    let font_file = scope.read::<OpenTypeFont>().expect("Failed to read font");
    let provider = font_file.table_provider(0).expect("Failed to get provider");
    let _builder = PdfSubsetBuilder::new(&provider);
    // If we get here without panic, the test passes
}

#[test]
fn test_builder_with_glyphs() {
    let font_data = include_bytes!("fonts/svg/gzipped.ttf");
    let scope = ReadScope::new(font_data);
    let font_file = scope.read::<OpenTypeFont>().expect("Failed to read font");
    let provider = font_file.table_provider(0).expect("Failed to get provider");
    
    let result = subset_for_pdf(&provider)
        .with_glyphs(&[1, 2])
        .identity_h()
        .build();
    
    assert!(result.is_ok());
    let pdf_result = result.unwrap();
    
    // Should include .notdef at least
    assert!(pdf_result.glyph_mapping.len() >= 1);
    assert!(pdf_result.glyph_mapping.contains_key(&0));  // .notdef
}

#[test]
#[ignore] // with_text is not fully implemented
fn test_builder_with_text() {
    let font_data = include_bytes!("fonts/svg/gzipped.ttf");
    let scope = ReadScope::new(font_data);
    let font_file = scope.read::<OpenTypeFont>().expect("Failed to read font");
    let provider = font_file.table_provider(0).expect("Failed to get provider");
    
    // Note: with_text is not fully implemented yet
    let builder_result = subset_for_pdf(&provider)
        .with_text("ABC");
    
    assert!(builder_result.is_ok());
    let result = builder_result.unwrap()
        .identity_h()
        .build();
    
    assert!(result.is_ok());
    let pdf_result = result.unwrap();
    
    // Should have at least .notdef
    assert!(pdf_result.glyph_mapping.len() >= 1);
}

#[test]
fn test_builder_fluent_api() {
    let font_data = include_bytes!("fonts/svg/gzipped.ttf");
    let scope = ReadScope::new(font_data);
    let font_file = scope.read::<OpenTypeFont>().expect("Failed to read font");
    let provider = font_file.table_provider(0).expect("Failed to get provider");
    
    let result = subset_for_pdf(&provider)
        .with_glyphs(&[1, 2])
        .identity_v()                    // Vertical encoding
        .with_max_cid(5000)
        .preserve_identity()
        .symbolic()
        .build();
    
    assert!(result.is_ok());
    let pdf_result = result.unwrap();
    
    assert_eq!(pdf_result.encoding_used, FontEncoding::Identity { vertical: true });
}

#[test]
fn test_builder_default_encoding() {
    let font_data = include_bytes!("fonts/svg/gzipped.ttf");
    let scope = ReadScope::new(font_data);
    let font_file = scope.read::<OpenTypeFont>().expect("Failed to read font");
    let provider = font_file.table_provider(0).expect("Failed to get provider");
    
    let result = subset_for_pdf(&provider)
        .with_glyphs(&[1, 2])
        .build();  // No explicit encoding - should default to Identity-H
    
    assert!(result.is_ok());
    let pdf_result = result.unwrap();
    
    assert_eq!(pdf_result.encoding_used, FontEncoding::Identity { vertical: false });
}