// Phase 2 TDD Tests - Statistics

use allsorts::tables::OpenTypeFont;
use allsorts::binary::read::ReadScope;
use allsorts::subset::pdf::{subset_and_map_for_pdf, PdfFontContext};


#[test]
fn test_statistics_calculation() {
    let font_data = include_bytes!("fonts/svg/gzipped.ttf");
    let scope = ReadScope::new(font_data);
    let font_file = scope.read::<OpenTypeFont>().expect("Failed to read font");
    let provider = font_file.table_provider(0).expect("Failed to get provider");
    let context = PdfFontContext::identity_h();
    
    let result = subset_and_map_for_pdf(&provider, &[0, 1, 2], context).unwrap();
    let stats = &result.statistics;
    
    assert!(stats.original_glyph_count > 0);
    assert!(stats.subset_glyph_count >= 1); // At least .notdef
    assert!(stats.original_size_estimate > 0);
    assert!(stats.subset_size > 0);
    // Reduction percentage can be negative if subset is larger
    assert!(stats.reduction_percentage >= -200.0);
    assert!(stats.reduction_percentage <= 100.0);
}

#[test]
fn test_statistics_with_cid_map() {
    let font_data = include_bytes!("fonts/svg/gzipped.ttf");
    let scope = ReadScope::new(font_data);
    let font_file = scope.read::<OpenTypeFont>().expect("Failed to read font");
    let provider = font_file.table_provider(0).expect("Failed to get provider");
    let context = PdfFontContext::identity_h();
    
    let result = subset_and_map_for_pdf(&provider, &[0, 1], context).unwrap();
    let stats = &result.statistics;
    
    if result.is_cid_font() {
        assert!(stats.cid_map_size > 0);
    } else {
        assert_eq!(stats.cid_map_size, 0);
    }
}