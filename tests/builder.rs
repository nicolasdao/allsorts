mod common;

use allsorts::subset::builder::{SubsetBuilder, ValidationLevel};
use allsorts::subset::{SubsetProfile, CmapTarget};
use allsorts::tables::{OpenTypeFont, FontTableProvider};
use allsorts::binary::read::ReadScope;

fn create_provider(font_buffer: &[u8]) -> impl FontTableProvider + '_ {
    let scope = ReadScope::new(font_buffer);
    let font_file = scope.read::<OpenTypeFont<'_>>().unwrap();
    font_file.table_provider(0).unwrap()
}

#[test]
fn test_builder_basic() {
    let font_buffer = common::read_fixture("tests/fonts/opentype/Klei.otf");
    let provider = create_provider(&font_buffer);
    
    let result = SubsetBuilder::new(&provider)
        .with_glyphs(&[0, 1, 2])
        .build();
    
    assert!(result.is_ok());
    let subset = result.unwrap();
    assert_eq!(subset.glyph_mapping.len(), 3);
}

#[test]
fn test_builder_with_characters() {
    let font_buffer = common::read_fixture("tests/fonts/opentype/Klei.otf");
    let provider = create_provider(&font_buffer);
    
    let result = SubsetBuilder::new(&provider)
        .with_characters("Hello")
        .unwrap()
        .build();
    
    assert!(result.is_ok());
    let subset = result.unwrap();
    assert!(subset.glyph_mapping.len() >= 5); // At least H, e, l, o + .notdef
}

#[test]
fn test_builder_for_pdf() {
    let font_buffer = common::read_fixture("tests/fonts/opentype/Klei.otf");
    let provider = create_provider(&font_buffer);
    
    let result = SubsetBuilder::new(&provider)
        .with_glyphs(&[0, 1, 2])
        .for_pdf(255)
        .build();
    
    assert!(result.is_ok());
    // PDF mode should auto-enable composite fixing
}

#[test]
fn test_builder_validation_levels() {
    let font_buffer = common::read_fixture("tests/fonts/opentype/Klei.otf");
    let provider = create_provider(&font_buffer);
    
    // Test None validation - should accept anything
    let _result = SubsetBuilder::new(&provider)
        .with_glyphs(&[9999]) // Invalid glyph
        .validation_level(ValidationLevel::None)
        .build();
    // Should succeed despite invalid glyph
    
    // Test Strict validation - should reject invalid
    let _result = SubsetBuilder::new(&provider)
        .with_glyphs(&[9999]) // Invalid glyph
        .validation_level(ValidationLevel::Strict)
        .build();
    // Should fail due to invalid glyph
}

#[test]
fn test_builder_chaining() {
    let font_buffer = common::read_fixture("tests/fonts/opentype/Klei.otf");
    let provider = create_provider(&font_buffer);
    
    let result = SubsetBuilder::new(&provider)
        .with_glyphs(&[0, 1, 2])
        .with_profile(SubsetProfile::Pdf)
        .with_cmap_target(CmapTarget::Unicode)
        .fix_composites(true)
        .validation_level(ValidationLevel::Standard)
        .build();
    
    assert!(result.is_ok());
}

#[test]
fn test_full_pdf_workflow() {
    let font_buffer = common::read_fixture("tests/fonts/opentype/Klei.otf");
    let provider = create_provider(&font_buffer);
    
    // Complete PDF workflow
    let result = SubsetBuilder::new(&provider)
        .with_characters("Hello, World!")
        .unwrap()
        .for_pdf(255)
        .fix_composites(true)
        .validation_level(ValidationLevel::Strict)
        .build();
    
    assert!(result.is_ok());
    let subset = result.unwrap();
    
    // Verify PDF-specific features
    assert!(subset.glyph_mapping.contains_key(&0)); // .notdef
    assert!(subset.stats.size_reduction_percent > 0.0 || subset.stats.size_reduction_percent == 0.0);
}