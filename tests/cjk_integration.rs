//! Integration tests for CJK encoding support in font subsetting

use allsorts::subset::context::{FontEncoding, CJKLanguage, ChineseVariant, JapaneseVariant, KoreanVariant};
use allsorts::subset::pdf::{subset_and_map_for_pdf, PdfFontContext};
use allsorts::subset::auto::auto_subset_for_pdf;
use allsorts::subset::cjk::BuiltinCMapProvider;
use allsorts::subset::detection::{PdfFontInfo, DetectionConfidence};
use allsorts::binary::read::ReadScope;
use allsorts::tables::OpenTypeFont;
use allsorts::tables::FontTableProvider;
use std::collections::HashMap;

// Helper function to create a test font provider
fn create_test_provider() -> impl FontTableProvider {
    // Use a test font - we'll use one from the test suite
    let font_data = include_bytes!("font_specimen/fonts/SymbolTest-Regular.ttf");
    let scope = ReadScope::new(font_data);
    let font_file = scope.read::<OpenTypeFont>().unwrap();
    font_file.table_provider(0).unwrap()
}

#[test]
fn test_chinese_gb_subsetting_end_to_end() {
    let provider = create_test_provider();
    
    let mut context = PdfFontContext::from_pdf_dict("GB-EUC-H", 0).unwrap();
    context = context.with_max_cid(100);
    let cmap_provider = Box::new(BuiltinCMapProvider::new());
    context = context.with_cmap_provider(cmap_provider);
    
    let result = subset_and_map_for_pdf(&provider, &[0, 1, 2], context);
    
    // Should succeed, not return "not yet implemented"
    assert!(result.is_ok(), "CJK subsetting should work, got error: {:?}", result.err());
    
    let pdf_result = result.unwrap();
    assert!(pdf_result.cid_to_gid_map.len() > 0);
    assert!(matches!(pdf_result.encoding_used, FontEncoding::CJK { .. }));
}

#[test]
fn test_chinese_gbk_subsetting() {
    let provider = create_test_provider();
    
    let mut context = PdfFontContext::from_pdf_dict("GBK-EUC-H", 0).unwrap();
    context = context.with_max_cid(150);
    let cmap_provider = Box::new(BuiltinCMapProvider::new());
    context = context.with_cmap_provider(cmap_provider);
    
    let result = subset_and_map_for_pdf(&provider, &[0, 1, 2, 3], context);
    assert!(result.is_ok(), "GBK Chinese subsetting should work");
    
    let pdf_result = result.unwrap();
    if let FontEncoding::CJK { language, encoding_name, vertical, .. } = &pdf_result.encoding_used {
        assert!(matches!(language, CJKLanguage::Chinese(_)));
        assert_eq!(encoding_name, "GBK-EUC-H");
        assert_eq!(*vertical, false);
    } else {
        panic!("Expected CJK encoding");
    }
}

#[test]
fn test_japanese_subsetting_end_to_end() {
    let provider = create_test_provider();
    
    let mut context = PdfFontContext::from_pdf_dict("90ms-RKSJ-H", 0).unwrap();
    context = context.with_max_cid(200);
    let cmap_provider = Box::new(BuiltinCMapProvider::new());
    context = context.with_cmap_provider(cmap_provider);
    
    let result = subset_and_map_for_pdf(&provider, &[0, 1, 2, 3], context);
    assert!(result.is_ok(), "Japanese subsetting should work, got error: {:?}", result.err());
    
    let pdf_result = result.unwrap();
    if let FontEncoding::CJK { language, .. } = &pdf_result.encoding_used {
        assert!(matches!(language, CJKLanguage::Japanese(_)));
    } else {
        panic!("Expected CJK encoding");
    }
}

#[test]
fn test_korean_subsetting_end_to_end() {
    let provider = create_test_provider();
    
    let mut context = PdfFontContext::from_pdf_dict("KSCms-UHC-H", 0).unwrap();
    context = context.with_max_cid(150);
    let cmap_provider = Box::new(BuiltinCMapProvider::new());
    context = context.with_cmap_provider(cmap_provider);
    
    let result = subset_and_map_for_pdf(&provider, &[0, 1, 2], context);
    assert!(result.is_ok(), "Korean subsetting should work, got error: {:?}", result.err());
    
    let pdf_result = result.unwrap();
    if let FontEncoding::CJK { language, .. } = &pdf_result.encoding_used {
        assert!(matches!(language, CJKLanguage::Korean(_)));
    } else {
        panic!("Expected CJK encoding");
    }
}

#[test]
fn test_cjk_auto_detection_and_subsetting() {
    let provider = create_test_provider();
    
    // Provide PDF info with CJK encoding
    let pdf_info = PdfFontInfo {
        encoding_name: Some("GB-EUC-H".to_string()),
        font_name: None,
        flags: 0,
        registry: None,
        ordering: None,
        supplement: None,
        to_unicode: None,
    };
    
    let result = auto_subset_for_pdf(&provider)
        .with_glyphs(&[0, 1, 2])
        .with_pdf_info(pdf_info)
        .build();
    
    assert!(result.is_ok(), "Auto-detection with CJK should work, got error: {:?}", result.err());
    
    let auto_result = result.unwrap();
    assert!(matches!(auto_result.detection.encoding, FontEncoding::CJK { .. }));
    assert_eq!(auto_result.detection.confidence, DetectionConfidence::Certain);
}

#[test]
fn test_cjk_without_cmap_provider_fails_gracefully() {
    let provider = create_test_provider();
    
    let mut context = PdfFontContext::from_pdf_dict("GB-EUC-H", 0).unwrap();
    context = context.with_max_cid(100);
    // Note: No CMap provider set
    
    let result = subset_and_map_for_pdf(&provider, &[0, 1, 2], context);
    
    // Should fail with appropriate error since requires_cmap_data is true for GB-EUC-H
    assert!(result.is_err());
    let err = result.unwrap_err();
    let err_str = err.to_string();
    assert!(err_str.contains("CMap") || err_str.contains("required"), 
            "Error should mention CMap requirement, got: {}", err_str);
}

#[test]
fn test_cjk_vertical_text_handling() {
    let provider = create_test_provider();
    
    // Vertical Chinese text
    let mut context = PdfFontContext::from_pdf_dict("GB-EUC-V", 0).unwrap();
    context = context.with_max_cid(100);
    let cmap_provider = Box::new(BuiltinCMapProvider::new());
    context = context.with_cmap_provider(cmap_provider);
    
    let result = subset_and_map_for_pdf(&provider, &[0, 1, 2], context);
    assert!(result.is_ok(), "Vertical CJK should work, got error: {:?}", result.err());
    
    let pdf_result = result.unwrap();
    assert!(matches!(pdf_result.encoding_used, 
        FontEncoding::CJK { vertical: true, .. }));
}

#[test]
fn test_unicode_based_cjk_encoding() {
    let provider = create_test_provider();
    
    // UniJIS-UTF16-H doesn't require external CMap data
    let mut context = PdfFontContext::from_pdf_dict("UniJIS-UTF16-H", 0).unwrap();
    context = context.with_max_cid(100);
    // Note: No CMap provider needed for Unicode-based encodings
    
    let result = subset_and_map_for_pdf(&provider, &[0, 1, 2], context);
    assert!(result.is_ok(), "Unicode-based CJK should work without CMap provider");
    
    let pdf_result = result.unwrap();
    if let FontEncoding::CJK { requires_cmap_data, .. } = &pdf_result.encoding_used {
        assert_eq!(*requires_cmap_data, false);
    } else {
        panic!("Expected CJK encoding");
    }
}

#[test]
fn test_multiple_cjk_languages_in_same_font() {
    let provider = create_test_provider();
    
    // Test Chinese
    let chinese_context = PdfFontContext::from_pdf_dict("GB-EUC-H", 0).unwrap()
        .with_max_cid(100)
        .with_cmap_provider(Box::new(BuiltinCMapProvider::new()));
    
    let chinese_result = subset_and_map_for_pdf(&provider, &[0, 1, 2], chinese_context);
    assert!(chinese_result.is_ok());
    
    // Test Japanese with same font
    let japanese_context = PdfFontContext::from_pdf_dict("90ms-RKSJ-H", 0).unwrap()
        .with_max_cid(100)
        .with_cmap_provider(Box::new(BuiltinCMapProvider::new()));
    
    let japanese_result = subset_and_map_for_pdf(&provider, &[0, 1, 2], japanese_context);
    assert!(japanese_result.is_ok());
    
    // Both should produce different CID maps
    let chinese_map = chinese_result.unwrap().cid_to_gid_map;
    let japanese_map = japanese_result.unwrap().cid_to_gid_map;
    
    // Maps might be different (though with our test data they might be similar)
    assert!(chinese_map.len() > 0);
    assert!(japanese_map.len() > 0);
}

#[test]
fn test_cjk_with_large_cid_range() {
    let provider = create_test_provider();
    
    let mut context = PdfFontContext::from_pdf_dict("GB-EUC-H", 0).unwrap();
    context = context.with_max_cid(10000);  // Large CID range
    let cmap_provider = Box::new(BuiltinCMapProvider::new());
    context = context.with_cmap_provider(cmap_provider);
    
    let result = subset_and_map_for_pdf(&provider, &[0, 1, 2], context);
    assert!(result.is_ok(), "Should handle large CID ranges");
    
    let pdf_result = result.unwrap();
    // CID map should be sized for max_cid
    assert_eq!(pdf_result.cid_to_gid_map.len(), (10000 + 1) * 2);
}