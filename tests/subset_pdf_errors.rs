// Phase 2 TDD Tests - Error Handling

use allsorts::subset::SubsetError;
use allsorts::tables::OpenTypeFont;
use allsorts::binary::read::ReadScope;
use allsorts::subset::phase2::{subset_and_map_for_pdf, PdfFontContext};


#[test]
fn test_enhanced_error_messages() {
    let error = SubsetError::UnsupportedEncoding("CustomEncoding".to_string());
    let msg = error.to_string();
    
    assert!(msg.contains("not supported"));
    assert!(msg.contains("CustomEncoding"));
    assert!(msg.contains("Identity-H"));  // Should suggest alternatives
}

#[test]
fn test_invalid_context_error() {
    let font_data = include_bytes!("fonts/svg/gzipped.ttf");
    let scope = ReadScope::new(font_data);
    let font_file = scope.read::<OpenTypeFont>().expect("Failed to read font");
    let provider = font_file.table_provider(0).expect("Failed to get provider");
    let context = PdfFontContext::identity_h();
    
    let result = subset_and_map_for_pdf(&provider, &[5, 10], context);
    
    match result {
        Err(SubsetError::InvalidContext(msg)) => {
            assert!(msg.contains(".notdef"));
        }
        _ => panic!("Expected InvalidContext error for missing .notdef"),
    }
}

#[test]
#[ignore] // Font type detection error test requires special setup
fn test_font_type_detection_error() {
    // This test requires a malformed provider that fails detection
    // Skipping for now as it needs special test setup
}