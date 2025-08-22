use crate::subset::auto::auto_subset_for_pdf;
use crate::subset::detection::{DetectionConfidence, PdfFontInfo, EncodingDetector};
use crate::subset::context::FontEncoding;
use crate::tables::FontTableProvider;
use crate::binary::read::ReadScope;
use crate::tables::OpenTypeFont;

// Use a real test font for proper testing
fn create_test_provider() -> Box<dyn FontTableProvider> {
    // Use a test font that should parse correctly
    let font_data = include_bytes!("../../../../tests/font_specimen/fonts/SymbolTest-Regular.ttf");
    let scope = ReadScope::new(font_data);
    let font_file = scope.read::<OpenTypeFont>().expect("Failed to parse test font");
    let provider = font_file.table_provider(0).expect("Failed to get table provider");
    Box::new(provider)
}

fn create_cjk_provider() -> Box<dyn FontTableProvider> {
    // For now, use the same font
    create_test_provider()
}

#[test]
fn test_auto_builder_basic_usage() {
    // Test with valid glyph IDs that should exist in most fonts
    let provider = create_test_provider();
    let result = auto_subset_for_pdf(provider.as_ref())
        .with_glyphs(&[0, 1, 2])  // Use simple glyphs that should exist
        .min_confidence(DetectionConfidence::Low)
        .build()
        .unwrap();
    
    assert!(result.subset_result.cid_to_gid_map.len() > 0);
    assert!(!result.reasoning().is_empty());
    assert!(result.confidence() >= &DetectionConfidence::Low);
}

#[test]
fn test_auto_builder_with_pdf_info() {
    // WILL FAIL until PDF info integration is implemented
    let pdf_info = PdfFontInfo {
        encoding_name: Some("Identity-H".to_string()),
        font_name: Some("TestFont".to_string()),
        flags: 0x04, // Symbolic
        ..Default::default()
    };
    
    let provider = create_test_provider();
    let result = auto_subset_for_pdf(provider.as_ref())
        .with_glyphs(&[0, 1, 2])
        .with_pdf_info(pdf_info)
        .build()
        .unwrap();
    
    assert_eq!(result.detection.confidence, DetectionConfidence::Certain);
    assert!(result.reasoning().iter().any(|r| r.contains("PDF specifies")));
}

#[test]
fn test_auto_builder_confidence_threshold() {
    // WILL FAIL until confidence checking is implemented
    let provider = create_test_provider();
    let result = auto_subset_for_pdf(provider.as_ref())
        .with_glyphs(&[1, 2, 3]) // Ambiguous pattern
        .min_confidence(DetectionConfidence::High)
        .build();
    
    // Should fail if detection confidence is below High
    assert!(result.is_err());
    if let Err(e) = result {
        assert!(e.to_string().contains("below minimum threshold"));
    }
}

#[test]
fn test_auto_builder_override_encoding() {
    // Test encoding override with Identity-V (vertical)
    let override_encoding = FontEncoding::Identity { vertical: true };
    
    let provider = create_test_provider();
    let result = auto_subset_for_pdf(provider.as_ref())
        .with_glyphs(&[0, 1, 2])
        .override_encoding(override_encoding.clone())
        .build()
        .unwrap();
    
    assert_eq!(result.detection.encoding, override_encoding);
    assert_eq!(result.detection.confidence, DetectionConfidence::Certain);
    assert!(result.reasoning().iter().any(|r| r.contains("Manual override")));
}

#[test]
fn test_auto_builder_glyph_deduplication() {
    // Test glyph deduplication 
    let provider = create_test_provider();
    let result = auto_subset_for_pdf(provider.as_ref())
        .with_glyphs(&[0, 1, 2])
        .with_glyphs(&[1, 2, 3]) // Overlapping glyphs
        .build()
        .unwrap();
    
    // Should have deduplicated glyphs: [0, 1, 2, 3]
    // The CID map should contain entries for at least these 4 glyphs
    let glyph_count = result.subset_result.glyph_mapping.len();
    assert_eq!(glyph_count, 4);
}

#[test]
fn test_auto_builder_empty_glyphs() {
    // Test empty glyph handling
    let provider = create_test_provider();
    let result = auto_subset_for_pdf(provider.as_ref())
        .with_glyphs(&[]) // No glyphs specified
        .build();
    
    // Should add at least .notdef (glyph 0)
    assert!(result.is_ok());
    let result = result.unwrap();
    assert!(result.subset_result.glyph_mapping.len() >= 1);
}

#[test]
fn test_auto_builder_with_detector() {
    // WILL FAIL until custom detector support is implemented
    let provider = create_test_provider();
    let detector = EncodingDetector::new(create_test_provider()); // Create a new provider for detector
    
    let result = auto_subset_for_pdf(provider.as_ref())
        .with_glyphs(&[0, 1, 2])
        .with_detector(detector)
        .build()
        .unwrap();
    
    assert!(!result.reasoning().is_empty());
}

#[test]
fn test_auto_builder_incremental_glyphs() {
    // Test incremental glyph addition
    let provider = create_test_provider();
    let builder = auto_subset_for_pdf(provider.as_ref())
        .with_glyphs(&[0, 1])
        .with_glyphs(&[2, 3])
        .with_glyphs(&[4, 5]);
    
    let result = builder.build().unwrap();
    
    // Should have all glyphs: [0, 1, 2, 3, 4, 5]
    assert_eq!(result.subset_result.glyph_mapping.len(), 6);
}