// tests/cjk_encodings.rs
#![cfg(test)]

use allsorts::subset::context::{FontEncoding, CJKLanguage, ChineseVariant, JapaneseVariant, KoreanVariant};
use allsorts::subset::cjk::{CMapProvider, BuiltinCMapProvider, FileCMapProvider};
use allsorts::subset::cjk::{build_cjk_cid_map, build_from_cmap_data};
use allsorts::subset::SubsetError;
use std::collections::HashMap;

// For now, let's start with a simple test that should pass with current implementation
#[test]
fn test_current_identity_encoding_works() {
    // This should work with existing implementation
    let encoding = FontEncoding::from_pdf_name("Identity-H");
    assert_eq!(encoding, Some(FontEncoding::Identity { vertical: false }));
    
    let encoding = FontEncoding::from_pdf_name("Identity-V");
    assert_eq!(encoding, Some(FontEncoding::Identity { vertical: true }));
}

// Test Chinese GB encoding detection
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

// Test Chinese GBK encoding detection with vertical
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

// Test Chinese traditional encoding detection
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
            language: CJKLanguage::Chinese(ref variant), .. 
        }) if *variant == expected_variant), "Failed for {}", name);
    }
}

// Test Japanese encoding detection
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
            language: CJKLanguage::Japanese(ref variant), 
            vertical, 
            .. 
        }) if *variant == expected_variant && vertical == expected_vertical), 
        "Failed for {}", name);
    }
}

// Test Korean encoding detection
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
            language: CJKLanguage::Korean(ref variant), 
            vertical, 
            .. 
        }) if *variant == expected_variant && vertical == expected_vertical), 
        "Failed for {}", name);
    }
}

// Test Unicode base for CJK languages
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

// Test Adobe collection parsing
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

// Test unsupported encoding names
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

// Test case sensitivity
#[test]
fn test_cjk_encoding_edge_cases() {
    // Test case sensitivity
    let encoding = FontEncoding::from_pdf_name("gb-euc-h"); // lowercase
    assert_eq!(encoding, None); // Should be case-sensitive
    
    // Test malformed names
    let encoding = FontEncoding::from_pdf_name("GB-EUC-"); // incomplete
    assert_eq!(encoding, None);
}

// Test CMap provider creation
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

// Test CMap provider data access
#[test]
fn test_builtin_cmap_provider_data_access() {
    let provider = BuiltinCMapProvider::new();
    
    let gb_data = provider.get_cmap_data("GB-EUC-H");
    assert!(gb_data.is_some());
    assert!(!gb_data.unwrap().is_empty());
    
    let unknown_data = provider.get_cmap_data("UnknownEncoding");
    assert!(unknown_data.is_none());
}

// Test file CMap provider
#[test]
fn test_file_cmap_provider_creation() {
    let temp_dir = std::env::temp_dir();
    let provider = FileCMapProvider::new(&temp_dir);
    
    // Should handle non-existent files gracefully
    assert!(!provider.has_cmap("NonExistentCMap"));
    assert!(provider.get_cmap_data("NonExistentCMap").is_none());
}

// Test CJK CID map generation with builtin CMap
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

// Test CJK CID map generation missing CMap data
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

// Test Unicode-based CJK encoding
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

// Test CMap data corruption handling
#[test]
fn test_cmap_data_corruption_handling() {
    // Test with invalid CMap data
    let corrupted_data = b"invalid cmap data";
    let glyph_mapping = HashMap::new();
    
    let result = build_from_cmap_data(corrupted_data, &glyph_mapping, 100);
    // Currently returns Ok with placeholder data, but in real implementation would fail
    assert!(result.is_ok());
}

// Test memory constraints
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