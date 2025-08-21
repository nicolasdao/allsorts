// Phase 2 TDD Tests - Integration

use allsorts::subset::context::FontEncoding;
use allsorts::tables::OpenTypeFont;
use allsorts::binary::read::ReadScope;
use allsorts::subset::pdf::{subset_and_map_for_pdf, PdfFontContext};
use allsorts::subset::builder::subset_for_pdf;


#[test]
fn test_phase1_integration() {
    let font_data = include_bytes!("fonts/svg/gzipped.ttf");
    let scope = ReadScope::new(font_data);
    let font_file = scope.read::<OpenTypeFont>().expect("Failed to read font");
    let provider = font_file.table_provider(0).expect("Failed to get provider");
    let context = PdfFontContext::identity_h();
    
    let result = subset_and_map_for_pdf(&provider, &[0, 1, 2], context).unwrap();
    
    // Verify it uses Phase 1 context system internally
    assert_eq!(result.encoding_used, FontEncoding::Identity { vertical: false });
    
    // Verify CID mapping is consistent with Phase 1
    if result.is_cid_font() {
        assert!(!result.cid_to_gid_map.is_empty());
    }
}

#[test]
#[ignore] // Requires multiple font type providers
fn test_different_font_types() {
    // Test with different providers (CFF, TrueType, etc.)
    // Will require different test providers for each font type
}