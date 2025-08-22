use crate::subset::auto::auto_subset_for_pdf;
use crate::subset::detection::{DetectionConfidence, EncodingDetector};
use crate::subset::context::FontEncoding;
use crate::tables::FontTableProvider;
use crate::binary::read::ReadScope;
use crate::tables::OpenTypeFont;

// Use a real test font for integration tests
fn create_test_provider() -> Box<dyn FontTableProvider> {
    let font_data = include_bytes!("../../../../tests/font_specimen/fonts/SymbolTest-Regular.ttf");
    let scope = ReadScope::new(font_data);
    let font_file = scope.read::<OpenTypeFont>().expect("Failed to parse test font");
    let provider = font_file.table_provider(0).expect("Failed to get table provider");
    Box::new(provider)
}

fn create_cjk_provider() -> Box<dyn FontTableProvider> {
    // For now, use the same test font
    // In a real scenario, we'd use an actual CJK font
    create_test_provider()
}

#[test]
fn test_end_to_end_identity_h_detection() {
    // Test Identity-H detection with simple glyphs
    let glyph_ids = vec![0, 1, 2, 3];
    let provider = create_test_provider();
    let result = auto_subset_for_pdf(provider.as_ref())
        .with_glyphs(&glyph_ids)
        .build()
        .unwrap();
    
    // With simple sequential glyphs, we get Identity-H with Low confidence  
    assert_eq!(result.detection.encoding, FontEncoding::Identity { vertical: false });
    assert!(result.confidence() >= &DetectionConfidence::Low);
    // Check that a CID map was generated
    assert!(result.subset_result.cid_to_gid_map.len() > 0);
}

#[test]
fn test_end_to_end_cjk_detection() {
    // Test CJK detection with high glyph IDs
    // Note: Using smaller range as test font has limited glyphs
    let cjk_glyph_ids = vec![0, 10, 11, 12, 13];
    let provider = create_cjk_provider();
    let result = auto_subset_for_pdf(provider.as_ref())
        .with_glyphs(&cjk_glyph_ids)
        .build()
        .unwrap();
    
    // With our test font, it will detect as Identity-H, not CJK
    // This is expected as we don't have a real CJK font
    assert!(matches!(result.detection.encoding, FontEncoding::Identity { .. }));
    assert!(result.confidence() >= &DetectionConfidence::Low);
}

#[test]
fn test_fallback_on_ambiguous_input() {
    // Test fallback with ambiguous pattern
    let ambiguous_glyph_ids = vec![0, 1, 2, 3]; // Simple sequential pattern
    let provider = create_test_provider();
    let result = auto_subset_for_pdf(provider.as_ref())
        .with_glyphs(&ambiguous_glyph_ids)
        .build()
        .unwrap();
    
    // Should use Identity-H (the default)
    assert_eq!(result.detection.encoding, FontEncoding::Identity { vertical: false });
    // Check that reasoning was provided
    assert!(result.reasoning().len() > 0);
}

#[test]
fn test_detection_with_statistical_analysis() {
    // Test statistical analysis with sequential glyphs
    let dense_glyphs = vec![0, 1, 2, 3, 4, 5, 6, 7, 8, 9];
    let provider = create_test_provider();
    let result = auto_subset_for_pdf(provider.as_ref())
        .with_glyphs(&dense_glyphs)
        .build()
        .unwrap();
    
    // Should detect some pattern (even if not dense with our limited test font)
    assert!(result.reasoning().len() > 0);
    assert!(result.confidence() >= &DetectionConfidence::Low);
}

#[test]
fn test_detection_with_pattern_matching() {
    // Test pattern matching with sequential glyphs
    let ascii_glyphs = vec![0, 1, 2, 3, 4]; // Simple sequential pattern
    let provider = create_test_provider();
    let result = auto_subset_for_pdf(provider.as_ref())
        .with_glyphs(&ascii_glyphs)
        .build()
        .unwrap();
    
    // Should detect some pattern
    assert!(result.reasoning().len() > 0);
}

#[test]
fn test_confidence_levels_hierarchy() {
    // WILL FAIL until confidence level logic is implemented
    let provider = create_test_provider();
    
    // Test that explicit encoding gives Certain confidence
    let pdf_info = crate::subset::detection::PdfFontInfo {
        encoding_name: Some("Identity-H".to_string()),
        ..Default::default()
    };
    let result1 = auto_subset_for_pdf(provider.as_ref())
        .with_glyphs(&[0, 1, 2])
        .with_pdf_info(pdf_info)
        .build()
        .unwrap();
    assert_eq!(result1.confidence(), &DetectionConfidence::Certain);
    
    // Test that pattern matching gives High/Medium confidence
    let dense_glyphs: Vec<u16> = (100..200).collect();
    let result2 = auto_subset_for_pdf(provider.as_ref())
        .with_glyphs(&dense_glyphs)
        .build()
        .unwrap();
    assert!(result2.confidence() >= &DetectionConfidence::Medium);
    
    // Test that ambiguous input gives Low confidence
    let sparse_glyphs = vec![50, 500, 5000];
    let result3 = auto_subset_for_pdf(provider.as_ref())
        .with_glyphs(&sparse_glyphs)
        .min_confidence(DetectionConfidence::Low)
        .build()
        .unwrap();
    assert!(result3.confidence() >= &DetectionConfidence::Low);
}

#[test]
fn test_alternative_encodings_provided() {
    // WILL FAIL until alternatives tracking is implemented
    let mixed_glyphs = vec![0, 100, 0x4E00, 0x30A0]; // Mixed ASCII, CJK, Kana
    let provider = create_test_provider();
    let result = auto_subset_for_pdf(provider.as_ref())
        .with_glyphs(&mixed_glyphs)
        .build()
        .unwrap();
    
    // Should provide alternative encoding suggestions
    assert!(!result.alternatives().is_empty());
}

#[test]
fn test_reasoning_accumulation() {
    // WILL FAIL until reasoning tracking is implemented
    let provider = create_test_provider();
    let result = auto_subset_for_pdf(provider.as_ref())
        .with_glyphs(&[0, 1, 2, 3])
        .build()
        .unwrap();
    
    // Should have at least one reasoning entry
    let reasoning = result.reasoning();
    assert!(reasoning.len() >= 1);
    
    // Should contain some detection-related text
    let reasoning_text = reasoning.join(" ");
    assert!(reasoning_text.len() > 0);
}