// tests/subset/cjk_encodings.rs
use allsorts::subset::context::{FontEncoding, CJKLanguage, ChineseVariant, JapaneseVariant, KoreanVariant};
use allsorts::subset::cjk::{CMapProvider, BuiltinCMapProvider, FileCMapProvider};
use allsorts::subset::cjk::{build_cjk_cid_map, build_from_cmap_data};
use allsorts::subset::SubsetError;
use std::collections::HashMap;

// WILL FAIL - CJK types don't exist yet
#[test]
fn test_chinese_gb_encoding_detection() {
    let encoding = FontEncoding::from_pdf_name("GB-EUC-H");
    assert_eq!(
        encoding,
        Some(FontEncoding::CJK {
            language: CJKLanguage::Chinese(ChineseVariant::Simplified),
            encoding_name: "GB-EUC-H".to_string(),
            vertical: false,
            requires_cmap_data: true,
        })
    );
}

// WILL FAIL - CJK types don't exist yet
#[test]
fn test_chinese_gbk_encoding_detection() {
    let encoding = FontEncoding::from_pdf_name("GBK-EUC-V");
    assert_eq!(
        encoding,
        Some(FontEncoding::CJK {
            language: CJKLanguage::Chinese(ChineseVariant::Simplified),
            encoding_name: "GBK-EUC-V".to_string(),
            vertical: true,
            requires_cmap_data: true,
        })
    );
}

// WILL FAIL - CJK types don't exist yet
#[test]
fn test_chinese_traditional_encoding_detection() {
    let test_cases = vec![
        ("CNS-EUC-H", ChineseVariant::Traditional),
        ("B5pc-H", ChineseVariant::Traditional),
        ("ETen-B5-V", ChineseVariant::Traditional),
        ("HKscs-B5-H", ChineseVariant::HongKong),
    ];
    
    for (name, expected_variant) in test_cases {
        let encoding = FontEncoding::from_pdf_name(name);
        assert!(matches!(encoding, Some(FontEncoding::CJK { 
            language: CJKLanguage::Chinese(variant), .. 
        }) if variant == expected_variant), "Failed for {}", name);
    }
}

// WILL FAIL - CJK types don't exist yet
#[test]
fn test_japanese_encoding_detection() {
    let test_cases = vec![
        ("90ms-RKSJ-H", JapaneseVariant::ShiftJIS, false),
        ("90ms-RKSJ-V", JapaneseVariant::ShiftJIS, true),
        ("UniJIS-UTF16-H", JapaneseVariant::Unicode, false),
        ("H", JapaneseVariant::JIS, false),
        ("V", JapaneseVariant::JIS, true),
        ("EUC-H", JapaneseVariant::JIS, false),
    ];
    
    for (name, expected_variant, expected_vertical) in test_cases {
        let encoding = FontEncoding::from_pdf_name(name);
        assert!(matches!(encoding, Some(FontEncoding::CJK { 
            language: CJKLanguage::Japanese(variant), 
            vertical, 
            .. 
        }) if variant == expected_variant && vertical == expected_vertical), 
        "Failed for {}", name);
    }
}

// WILL FAIL - CJK types don't exist yet
#[test]
fn test_korean_encoding_detection() {
    let test_cases = vec![
        ("KSCms-UHC-H", KoreanVariant::UHC, false),
        ("KSCms-UHC-V", KoreanVariant::UHC, true),
        ("UniKS-UTF16-H", KoreanVariant::Unicode, false),
        ("KSC-EUC-H", KoreanVariant::KSC, false),
    ];
    
    for (name, expected_variant, expected_vertical) in test_cases {
        let encoding = FontEncoding::from_pdf_name(name);
        assert!(matches!(encoding, Some(FontEncoding::CJK { 
            language: CJKLanguage::Korean(variant), 
            vertical, 
            .. 
        }) if variant == expected_variant && vertical == expected_vertical), 
        "Failed for {}", name);
    }
}

// WILL FAIL - Unicode base method doesn't exist yet
#[test]
fn test_unicode_base_for_cjk_languages() {
    let chinese = FontEncoding::CJK {
        language: CJKLanguage::Chinese(ChineseVariant::Simplified),
        encoding_name: "GB-EUC-H".to_string(),
        vertical: false,
        requires_cmap_data: true,
    };
    assert_eq!(chinese.get_unicode_base(), Some(0x4E00)); // CJK Unified start
    
    let japanese = FontEncoding::CJK {
        language: CJKLanguage::Japanese(JapaneseVariant::JIS),
        encoding_name: "H".to_string(),
        vertical: false,
        requires_cmap_data: true,
    };
    assert_eq!(japanese.get_unicode_base(), Some(0x3040)); // Hiragana start
    
    let korean = FontEncoding::CJK {
        language: CJKLanguage::Korean(KoreanVariant::KSC),
        encoding_name: "KSC-EUC-H".to_string(),
        vertical: false,
        requires_cmap_data: true,
    };
    assert_eq!(korean.get_unicode_base(), Some(0xAC00)); // Hangul start
}

// WILL FAIL - CMapProvider trait doesn't exist yet
#[test]
fn test_builtin_cmap_provider_creation() {
    let provider = BuiltinCMapProvider::new();
    
    // Should have common CMaps
    assert!(provider.has_cmap("GB-EUC-H"));
    assert!(provider.has_cmap("90ms-RKSJ-H"));
    assert!(provider.has_cmap("KSCms-UHC-H"));
    
    // Should not have unknown CMaps
    assert!(!provider.has_cmap("UnknownEncoding"));
}

// WILL FAIL - CMapProvider trait doesn't exist yet
#[test]
fn test_builtin_cmap_provider_data_access() {
    let provider = BuiltinCMapProvider::new();
    
    let gb_data = provider.get_cmap_data("GB-EUC-H");
    assert!(gb_data.is_some());
    assert!(!gb_data.unwrap().is_empty());
    
    let unknown_data = provider.get_cmap_data("UnknownEncoding");
    assert!(unknown_data.is_none());
}

// WILL FAIL - FileCMapProvider doesn't exist yet
#[test]
fn test_file_cmap_provider_creation() {
    let temp_dir = std::env::temp_dir();
    let provider = FileCMapProvider::new(&temp_dir);
    
    // Should handle non-existent files gracefully
    assert!(!provider.has_cmap("NonExistentCMap"));
    assert!(provider.get_cmap_data("NonExistentCMap").is_none());
}

// WILL FAIL - build_cjk_cid_map doesn't exist yet
#[test]
fn test_cjk_cid_map_generation_with_builtin_cmap() {
    let encoding = FontEncoding::CJK {
        language: CJKLanguage::Chinese(ChineseVariant::Simplified),
        encoding_name: "GB-EUC-H".to_string(),
        vertical: false,
        requires_cmap_data: true,
    };
    
    let glyph_mapping: HashMap<u16, u16> = [(1, 100), (2, 101), (3, 102)].iter().cloned().collect();
    let provider = BuiltinCMapProvider::new();
    
    let result = build_cjk_cid_map(&encoding, &glyph_mapping, 1000, Some(&provider));
    assert!(result.is_ok());
    
    let map = result.unwrap();
    assert_eq!(map.len(), 1001 * 2); // (max_cid + 1) * 2 bytes per entry
}

// WILL FAIL - build_cjk_cid_map doesn't exist yet
#[test]
fn test_cjk_cid_map_generation_missing_cmap_data() {
    let encoding = FontEncoding::CJK {
        language: CJKLanguage::Chinese(ChineseVariant::Simplified),
        encoding_name: "GB-EUC-H".to_string(),
        vertical: false,
        requires_cmap_data: true,
    };
    
    let glyph_mapping: HashMap<u16, u16> = HashMap::new();
    
    // No CMap provider - should fail
    let result = build_cjk_cid_map(&encoding, &glyph_mapping, 100, None);
    assert!(result.is_err());
    assert!(matches!(result.unwrap_err(), SubsetError::CidGenerationFailed(_)));
}

// WILL FAIL - Unicode-based CJK mapping doesn't exist yet
#[test]
fn test_unicode_based_cjk_encoding() {
    let encoding = FontEncoding::CJK {
        language: CJKLanguage::Chinese(ChineseVariant::Simplified),
        encoding_name: "UniGB-UTF16-H".to_string(),
        vertical: false,
        requires_cmap_data: false, // Unicode-based, no CMap needed
    };
    
    let glyph_mapping: HashMap<u16, u16> = [(1, 100), (2, 101)].iter().cloned().collect();
    
    // Should work without CMap provider
    let result = build_cjk_cid_map(&encoding, &glyph_mapping, 100, None);
    assert!(result.is_ok());
}

// WILL FAIL - Adobe collection parsing doesn't exist yet
#[test]
fn test_adobe_collection_parsing() {
    let test_cases = vec![
        "Adobe-GB1-5",
        "Adobe-CNS1-6", 
        "Adobe-Japan1-6",
        "Adobe-Korea1-2",
    ];
    
    for name in test_cases {
        let encoding = FontEncoding::from_pdf_name(name);
        assert!(matches!(encoding, Some(FontEncoding::AdobeCollection { .. })), 
                "Failed to parse {}", name);
    }
}

// WILL FAIL - Error handling doesn't exist yet
#[test]
fn test_unsupported_encoding_names() {
    let unsupported = vec![
        "UnknownEncoding-H",
        "InvalidCJK-V",
        "NotACJKEncoding",
        "",
        "H-GB", // Invalid order
    ];
    
    for name in unsupported {
        let encoding = FontEncoding::from_pdf_name(name);
        assert_eq!(encoding, None, "Should not support: {}", name);
    }
}

// Additional tests for edge cases (WILL FAIL)

#[test]
fn test_cjk_encoding_edge_cases() {
    // Test case sensitivity
    let encoding = FontEncoding::from_pdf_name("gb-euc-h"); // lowercase
    assert_eq!(encoding, None); // Should be case-sensitive
    
    // Test malformed names
    let encoding = FontEncoding::from_pdf_name("GB-EUC-"); // incomplete
    assert_eq!(encoding, None);
    
    // Test maximum CID handling
    let encoding = FontEncoding::CJK {
        language: CJKLanguage::Chinese(ChineseVariant::Simplified),
        encoding_name: "GB-EUC-H".to_string(),
        vertical: false,
        requires_cmap_data: true,
    };
    
    let glyph_mapping = HashMap::new();
    let provider = BuiltinCMapProvider::new();
    
    // Test with very large max_cid
    let result = build_cjk_cid_map(&encoding, &glyph_mapping, u16::MAX, Some(&provider));
    assert!(result.is_ok());
}

#[test]
fn test_cmap_data_corruption_handling() {
    // Test with invalid CMap data
    let corrupted_data = b"invalid cmap data";
    let glyph_mapping = HashMap::new();
    
    let result = build_from_cmap_data(corrupted_data, &glyph_mapping, 100);
    assert!(result.is_err());
    assert!(matches!(result.unwrap_err(), SubsetError::CidGenerationFailed(_)));
}

#[test]
fn test_memory_constraints() {
    // Test that we don't allocate excessive memory
    let encoding = FontEncoding::CJK {
        language: CJKLanguage::Chinese(ChineseVariant::Simplified),
        encoding_name: "GB-EUC-H".to_string(),
        vertical: false,
        requires_cmap_data: true,
    };
    
    let glyph_mapping = HashMap::new();
    let provider = BuiltinCMapProvider::new();
    
    let result = build_cjk_cid_map(&encoding, &glyph_mapping, 10000, Some(&provider));
    assert!(result.is_ok());
    
    let map = result.unwrap();
    // Should allocate exactly (max_cid + 1) * 2 bytes
    assert_eq!(map.len(), 10001 * 2);
}

// Integration tests (WILL FAIL until Phase 2 integration complete)

#[test]
fn test_pdf_context_with_cjk_encoding() {
    use allsorts::subset::pdf::{PdfFontContext, WritingMode};
    
    let encoding = FontEncoding::from_pdf_name("GB-EUC-H").unwrap();
    let cmap_provider = Box::new(BuiltinCMapProvider::new());
    
    // Note: We need to ADD cmap_provider field to PdfFontContext (doesn't exist yet)
    let context = PdfFontContext {
        encoding,
        max_cid: Some(8000),
        preserve_identity: false,
        is_symbolic: false,
        cid_to_gid_map: None,
        writing_mode: WritingMode::Horizontal,
        cmap_provider: Some(cmap_provider),
    };
    
    // Should be able to create context with CJK encoding
    assert!(matches!(context.encoding, FontEncoding::CJK { .. }));
    assert!(context.cmap_provider.is_some());
}

#[test]
fn test_subset_and_map_with_cjk_encoding() {
    use allsorts::subset::pdf::{subset_and_map_for_pdf, PdfFontContext, WritingMode};
    use allsorts::binary::read::ReadScope;
    use allsorts::tables::{FontTableProvider, OpenTypeFont};
    use std::fs;
    
    // Read fixture font - using a path that should exist
    let buffer = fs::read("tests/fixtures/opentype/Klei.otf").unwrap_or_else(|_| vec![]);
    if buffer.is_empty() {
        // Skip test if fixture not available
        return;
    }
    
    let scope = ReadScope::new(&buffer);
    let font_file = scope.read::<OpenTypeFont>().unwrap();
    let provider = font_file.table_provider(0).unwrap();
    let glyph_ids = vec![1, 2, 3, 4, 5];
    
    let encoding = FontEncoding::from_pdf_name("90ms-RKSJ-H").unwrap();
    let cmap_provider = Box::new(BuiltinCMapProvider::new());
    
    let context = PdfFontContext {
        encoding,
        max_cid: Some(1000),
        preserve_identity: false,
        is_symbolic: false,
        cid_to_gid_map: None,
        writing_mode: WritingMode::Horizontal,
        cmap_provider: Some(cmap_provider),
    };
    
    let result = subset_and_map_for_pdf(&provider, &glyph_ids, context);
    assert!(result.is_ok());
    
    let subset_result = result.unwrap();
    assert!(!subset_result.cid_to_gid_map.is_empty());
    assert!(subset_result.font_data.len() > 0);
}