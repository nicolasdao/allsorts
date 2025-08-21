Read @README.md and the @docs/subsetting.md to understand this project and its current subsetting capabilities and APIs and then read @specs/250821_subset_and_map_with_context/README.md to understand the general context of the current upcoming changes and then implement the new changes below:

# Phase 3: CJK Encoding Support

## Executive Summary

Extend the `FontEncoding` enum to support Chinese, Japanese, and Korean (CJK) encodings using a behavior-based grouping approach. This phase enables correct CIDToGIDMap generation for fonts using predefined CMap encodings beyond Identity-H/V.

## TDD Implementation Steps

This phase follows Test-Driven Development (TDD) methodology as outlined in `specs/TDD.md`. The implementation will proceed through the following phases:

1. **Preparation**: Verify Phase 1 & 2 completion, run existing tests
2. **Analysis & Planning**: Design CJK type system and behavior-based grouping
3. **Define APIs & Signatures**: Create interfaces for CJK encodings and CMap providers
4. **Write Unit Tests (TDD)**: Create comprehensive failing tests for all CJK functionality
5. **Implement Functions Incrementally**: Build minimal implementations to pass tests
6. **Full-Suite Integration**: Verify all tests pass including existing functionality
7. **Documentation & Review**: Update docs and prepare for Phase 4

**Prerequisites**: 
- Phase 1 (Identity encoding) must be complete and tested
- Phase 2 (PdfFontContext API) must be complete and tested
- All existing tests must be passing before starting Phase 3

## TDD Step 1: Preparation

### 1.1 Read Documentation

- Review Phase 1 implementation in `src/subset/context.rs` for Identity encoding
- Review Phase 2 implementation for PdfFontContext API
- Study CJK encoding specifications and Adobe CMap documentation
- Understand existing test patterns in `tests/subset/`

### 1.2 Verify Test Health

```bash
# Run existing tests to ensure clean starting point
cargo test --lib subset
cargo test --lib context
cargo test --doc
```

**Expected Results:**
- All Phase 1 tests passing (Identity encoding)
- All Phase 2 tests passing (PdfFontContext)
- No compilation errors
- Clean test output with no warnings

**If any tests fail:**
1. **STOP** - Do not proceed with Phase 3
2. Fix existing failures first
3. Re-run until suite is green
4. Verify dependencies are properly resolved

## TDD Step 2: Analysis & Planning

### 2.1 High-Level Design

**Goal**: Add support for CJK (Chinese, Japanese, Korean) encodings with behavior-based grouping and CMap data integration.

**Affected Modules:**
- `src/subset/context.rs` - Enhanced FontEncoding enum
- `src/subset/cid_map.rs` - CJK-aware CID mapping functions
- `src/subset/cjk/` - New module for CJK-specific logic
- Tests in `tests/subset/cjk_encodings.rs` (new)

**External Dependencies:**
- CMap data files (embedded or external)
- Unicode mapping tables
- CJK test fonts for validation

### 2.2 Break Down Into Tasks

1. **CJK Type System** - Language variants and encoding enums
2. **CMap Data Integration** - Provider trait and implementations
3. **Chinese Encoding Support** - GB/GBK/CNS mappings
4. **Japanese Encoding Support** - JIS/Shift-JIS/EUC mappings
5. **Korean Encoding Support** - KSC/UHC mappings
6. **CIDToGIDMap Generation** - CJK-aware mapping algorithms
7. **Integration with Phase 2 API** - PdfFontContext extensions

### 2.3 Edge-Case Brainstorm

**Valid Inputs:**
- Standard CJK encoding names (GB-EUC-H, 90ms-RKSJ-V, etc.)
- Unicode-based CJK encodings (UniGB, UniJIS, UniKS)
- Adobe collection names (Adobe-GB1-5, Adobe-Japan1-6)
- Vertical/horizontal variants (-H/-V suffixes)

**Boundary Conditions:**
- Maximum CID values for each encoding
- Empty or minimal CMap data
- Mixed encodings in same document
- Fonts without required glyphs

**Error Scenarios:**
- Unknown encoding names
- Missing CMap data when required
- Corrupted CMap files
- Invalid CID ranges
- Memory constraints with large CMaps

**Performance Constraints:**
- CJK detection < 10µs
- CMap parsing < 100ms for large files
- Memory usage < 1MB for CMap cache

## TDD Step 3: Define APIs & Signatures

| Component / Function | Inputs | Outputs | Error Cases / Behaviors |
|----------------------|--------|---------|------------------------|
| `FontEncoding::from_pdf_name()` | `name: &str` | `Option<FontEncoding>` | Returns None for unsupported encodings |
| `FontEncoding::get_unicode_base()` | `&self` | `Option<u32>` | Returns Unicode base for CJK languages |
| `CMapProvider::get_cmap_data()` | `encoding_name: &str` | `Option<&[u8]>` | Returns None if CMap not found |
| `CMapProvider::has_cmap()` | `encoding_name: &str` | `bool` | Checks availability without loading |
| `build_cjk_cid_map()` | `encoding, glyph_mapping, max_cid, provider` | `Result<Vec<u8>, SubsetError>` | Handles missing CMap data |
| `build_from_cmap_data()` | `cmap_data, glyph_mapping, max_cid` | `Result<Vec<u8>, SubsetError>` | Parses CMap and builds mapping |
| `BuiltinCMapProvider::new()` | None | `Self` | Pre-loads embedded CMap data |
| `FileCMapProvider::new()` | `cmap_dir: &Path` | `Self` | Sets up file-based loading |

**Note**: Only signatures and documentation—no implementation yet.

## TDD Step 4: Write Unit Tests (TDD)

### 4.1 Set Up Test Files

Create comprehensive test files under `tests/subset/cjk_encodings.rs`:

```rust
// tests/subset/cjk_encodings.rs
use allsorts::subset::context::{FontEncoding, CJKLanguage, ChineseVariant, JapaneseVariant, KoreanVariant};
use allsorts::subset::cjk::{CMapProvider, BuiltinCMapProvider, FileCMapProvider};
use allsorts::subset::cjk::{build_cjk_cid_map, build_from_cmap_data};
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
```

### 4.2 Edge Case and Error Tests

```rust
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
```

### 4.3 Integration Tests

```rust
// Integration tests (WILL FAIL until Phase 2 integration complete)

#[test]
fn test_pdf_context_with_cjk_encoding() {
    use allsorts::subset::context::PdfFontContext;
    
    let encoding = FontEncoding::from_pdf_name("GB-EUC-H").unwrap();
    let cmap_provider = Box::new(BuiltinCMapProvider::new());
    
    let context = PdfFontContext {
        encoding,
        max_cid: Some(8000),
        preserve_identity: false,
        is_symbolic: false,
    }.with_cmap_provider(cmap_provider);
    
    // Should be able to create context with CJK encoding
    assert!(matches!(context.encoding, FontEncoding::CJK { .. }));
    assert!(context.cmap_provider.is_some());
}

#[test]
fn test_subset_and_map_with_cjk_encoding() {
    use allsorts::subset::{subset_and_map_for_pdf, create_test_provider};
    
    let provider = create_test_provider(); // From existing test infrastructure
    let glyph_ids = vec![1, 2, 3, 4, 5];
    
    let encoding = FontEncoding::from_pdf_name("90ms-RKSJ-H").unwrap();
    let cmap_provider = Box::new(BuiltinCMapProvider::new());
    
    let context = PdfFontContext {
        encoding,
        max_cid: Some(1000),
        preserve_identity: false,
        is_symbolic: false,
    }.with_cmap_provider(cmap_provider);
    
    let result = subset_and_map_for_pdf(&provider, &glyph_ids, context);
    assert!(result.is_ok());
    
    let subset_result = result.unwrap();
    assert!(!subset_result.cid_to_gid_map.is_empty());
    assert!(subset_result.subset_data.len() > 0);
}
```

**Expected Test Results at This Stage:**
- All tests should compile but FAIL
- Tests highlight the required functionality
- Error messages guide implementation priorities

## TDD Step 5: Implement Functions Incrementally

### 5.1 Implementation Strategy

Implement components in dependency order, writing minimal code to pass one test at a time:

1. **CJK Type System** (enables encoding detection tests)
2. **CMap Provider Trait** (enables CMap provider tests)
3. **Built-in CMap Provider** (enables builtin provider tests) 
4. **File CMap Provider** (enables file provider tests)
5. **CJK CID Mapping** (enables mapping generation tests)
6. **Language-Specific Mappings** (enables Chinese/Japanese/Korean tests)
7. **Phase 2 Integration** (enables full integration tests)

### 5.2 Implementation Order with Test Feedback

**Step 5.1: CJK Type System**
- Run: `cargo test test_chinese_gb_encoding_detection` (WILL FAIL)
- Implement: CJK enums in `src/subset/context.rs`
- Run test again (SHOULD PASS)
- Continue with remaining encoding detection tests

**Step 5.2: CMap Provider Trait**
- Run: `cargo test test_builtin_cmap_provider_creation` (WILL FAIL)
- Implement: CMapProvider trait
- Run test again (SHOULD PASS)

**Step 5.3: Incremental Implementation**
- Continue this pattern for each component
- Never implement more than needed to pass the current failing test
- Refactor only when tests are passing

### 5.3 Minimal Implementation Examples

```rust
// FIRST ITERATION: Just enough to pass basic encoding detection
impl FontEncoding {
    pub fn from_pdf_name(name: &str) -> Option<Self> {
        match name {
            "Identity-H" => Some(FontEncoding::Identity { vertical: false }),
            "Identity-V" => Some(FontEncoding::Identity { vertical: true }),
            "GB-EUC-H" => Some(FontEncoding::CJK {
                language: CJKLanguage::Chinese(ChineseVariant::Simplified),
                encoding_name: name.to_string(),
                vertical: false,
                requires_cmap_data: true,
            }),
            // Add more cases as tests require them
            _ => None,
        }
    }
}
```

## TDD Step 6: Full-Suite Integration

### 6.1 Execute Entire Test Suite

```bash
# Run all tests including existing ones
cargo test

# Run specific CJK tests
cargo test cjk_encodings

# Run with verbose output to see failures
cargo test -- --nocapture
```

### 6.2 Fix Regressions

- Address any failures in existing Phase 1/2 tests
- Ensure new CJK functionality doesn't break existing code
- Verify performance requirements are met

### 6.3 Integration Verification Checklist

- [ ] All existing Identity encoding tests still pass
- [ ] All existing PdfFontContext tests still pass  
- [ ] New CJK encoding detection tests pass
- [ ] CMap provider tests pass
- [ ] CJK CID mapping tests pass
- [ ] Integration tests with PDF context pass
- [ ] Performance benchmarks meet requirements
- [ ] Memory usage within limits

## TDD Step 7: Documentation & Review

### 7.1 Update Documentation

- Add CJK encoding examples to README
- Update API documentation with CJK support
- Add encoding support matrix
- Document CMap provider usage

### 7.2 Code Review Checklist

- [ ] All public APIs documented
- [ ] Error handling comprehensive
- [ ] Performance optimizations applied
- [ ] Memory safety verified
- [ ] Thread safety considered
- [ ] Integration tests comprehensive

## 1. Technical Implementation Details

> **Note**: The following sections provide the complete technical implementation details. 
> These should only be implemented following the TDD approach outlined above.

### 1.1 Enhanced Type System

```rust
/// Language variants for CJK fonts
#[derive(Debug, Clone, PartialEq)]
pub enum CJKLanguage {
    Chinese(ChineseVariant),
    Japanese(JapaneseVariant),
    Korean(KoreanVariant),
}

#[derive(Debug, Clone, PartialEq)]
pub enum ChineseVariant {
    Simplified,   // GB/GBK encodings
    Traditional,  // CNS/Big5 encodings
    HongKong,    // HKSCS encodings
}

#[derive(Debug, Clone, PartialEq)]
pub enum JapaneseVariant {
    JIS,         // JIS-based encodings
    ShiftJIS,    // Shift-JIS based (RKSJ)
    Unicode,     // Unicode-based
}

#[derive(Debug, Clone, PartialEq)]
pub enum KoreanVariant {
    KSC,         // KSC5601 based
    UHC,         // Unified Hangul Code
    Unicode,     // Unicode-based
}

/// Enhanced FontEncoding with CJK support
#[derive(Debug, Clone, PartialEq)]
pub enum FontEncoding {
    /// Identity mapping (from Phase 1)
    Identity { vertical: bool },
    
    /// CJK predefined encodings (NEW)
    CJK {
        language: CJKLanguage,
        encoding_name: String,  // Original encoding name
        vertical: bool,
        requires_cmap_data: bool,  // If true, need external CMap
    },
    
    /// Adobe standard collections (NEW)
    AdobeCollection {
        registry: String,    // "Adobe"
        ordering: String,    // "GB1", "CNS1", "Japan1", "Korea1"
        supplement: u16,     // Version
    },
    
    /// Custom encoding
    Custom(String),
}
```

### 1.2 CMap Data Integration

```rust
/// CMap data provider trait
pub trait CMapProvider {
    /// Get CMap data for encoding
    fn get_cmap_data(&self, encoding_name: &str) -> Option<&[u8]>;
    
    /// Check if CMap is available
    fn has_cmap(&self, encoding_name: &str) -> bool;
}

/// Built-in CMap provider with common encodings
pub struct BuiltinCMapProvider {
    cmaps: HashMap<String, Vec<u8>>,
}

/// External CMap provider (loads from files)
pub struct FileCMapProvider {
    cmap_dir: PathBuf,
}
```

## 2. Why This Phase

### 2.1 Market Coverage
- **Chinese Market**: ~1.4 billion potential users
- **Japanese Market**: ~125 million users
- **Korean Market**: ~80 million users
- **Legacy Documents**: Millions of existing PDFs use these encodings

### 2.2 Technical Necessity
- Identity encoding doesn't work for these fonts
- Predefined CMaps have complex CID→GID mappings
- Required for PDF/A compliance in some regions
- Essential for government and enterprise documents

## 3. Technical Implementation

### 3.1 File Structure

```
src/subset/
├── context.rs               (enhance with CJK types)
├── cid_map.rs              (add CJK mapping functions)
├── cjk/
│   ├── mod.rs              (CJK module root)
│   ├── chinese.rs          (Chinese encoding logic)
│   ├── japanese.rs         (Japanese encoding logic)
│   ├── korean.rs           (Korean encoding logic)
│   └── cmap_data.rs        (CMap data handling)
└── resources/
    └── cmaps/              (embedded CMap data)
        ├── gb_euc_h.cmap
        ├── uni_jis_utf16_h.cmap
        └── ksc_ms_uhc_h.cmap
```

### 3.2 Implementation Details

#### 3.2.1 Enhanced FontEncoding (`src/subset/context.rs`)

```rust
impl FontEncoding {
    /// Parse encoding name with full CJK support
    pub fn from_pdf_name(name: &str) -> Option<Self> {
        // Phase 1 encodings
        match name {
            "Identity-H" => return Some(FontEncoding::Identity { vertical: false }),
            "Identity-V" => return Some(FontEncoding::Identity { vertical: true }),
            _ => {}
        }
        
        // Phase 3: CJK encodings
        let vertical = name.ends_with("-V");
        
        // Chinese encodings
        if name.starts_with("GB") || name.starts_with("GBK") {
            return Some(FontEncoding::CJK {
                language: CJKLanguage::Chinese(ChineseVariant::Simplified),
                encoding_name: name.to_string(),
                vertical,
                requires_cmap_data: !name.starts_with("UniGB"),
            });
        }
        
        if name.starts_with("CNS") || name.starts_with("B5") || 
           name.starts_with("ETen") || name.starts_with("HK") {
            return Some(FontEncoding::CJK {
                language: CJKLanguage::Chinese(ChineseVariant::Traditional),
                encoding_name: name.to_string(),
                vertical,
                requires_cmap_data: !name.starts_with("UniCNS"),
            });
        }
        
        // Japanese encodings
        if name.contains("RKSJ") || name.contains("JIS") || 
           name == "H" || name == "V" || name.starts_with("EUC") {
            let variant = if name.contains("RKSJ") {
                JapaneseVariant::ShiftJIS
            } else if name.starts_with("UniJIS") {
                JapaneseVariant::Unicode
            } else {
                JapaneseVariant::JIS
            };
            
            return Some(FontEncoding::CJK {
                language: CJKLanguage::Japanese(variant),
                encoding_name: name.to_string(),
                vertical,
                requires_cmap_data: !name.starts_with("Uni"),
            });
        }
        
        // Korean encodings
        if name.starts_with("KSC") || name.starts_with("UniKS") {
            let variant = if name.contains("UHC") {
                KoreanVariant::UHC
            } else if name.starts_with("UniKS") {
                KoreanVariant::Unicode
            } else {
                KoreanVariant::KSC
            };
            
            return Some(FontEncoding::CJK {
                language: CJKLanguage::Korean(variant),
                encoding_name: name.to_string(),
                vertical,
                requires_cmap_data: !name.starts_with("UniKS"),
            });
        }
        
        // Adobe collections
        if name.starts_with("Adobe-") {
            return Self::parse_adobe_collection(name);
        }
        
        None
    }
    
    /// Get the Unicode base for CJK encodings
    pub fn get_unicode_base(&self) -> Option<u32> {
        match self {
            FontEncoding::CJK { language, .. } => {
                match language {
                    CJKLanguage::Chinese(_) => Some(0x4E00),  // CJK Unified start
                    CJKLanguage::Japanese(_) => Some(0x3040),  // Hiragana start
                    CJKLanguage::Korean(_) => Some(0xAC00),   // Hangul start
                }
            }
            _ => None,
        }
    }
}
```

#### 3.2.2 CJK-Aware CID Mapping (`src/subset/cjk/mod.rs`)

```rust
use crate::subset::context::FontEncoding;
use std::collections::HashMap;

/// Build CIDToGIDMap for CJK encodings
pub fn build_cjk_cid_map(
    encoding: &FontEncoding,
    glyph_mapping: &HashMap<u16, u16>,
    max_cid: u16,
    cmap_provider: Option<&dyn CMapProvider>,
) -> Result<Vec<u8>, SubsetError> {
    match encoding {
        FontEncoding::CJK { encoding_name, requires_cmap_data, .. } => {
            if *requires_cmap_data {
                // Need external CMap data
                let provider = cmap_provider
                    .ok_or_else(|| SubsetError::CidGenerationFailed(
                        format!("CMap data required for {}", encoding_name)
                    ))?;
                    
                let cmap_data = provider.get_cmap_data(encoding_name)
                    .ok_or_else(|| SubsetError::CidGenerationFailed(
                        format!("CMap {} not found", encoding_name)
                    ))?;
                    
                build_from_cmap_data(cmap_data, glyph_mapping, max_cid)
            } else {
                // Unicode-based CJK encoding
                build_unicode_cjk_map(encoding, glyph_mapping, max_cid)
            }
        }
        _ => Err(SubsetError::InvalidContext("Not a CJK encoding".to_string())),
    }
}

/// Build map from CMap data
fn build_from_cmap_data(
    cmap_data: &[u8],
    glyph_mapping: &HashMap<u16, u16>,
    max_cid: u16,
) -> Result<Vec<u8>, SubsetError> {
    // Parse CMap to get CID->Unicode mappings
    let cid_to_unicode = parse_cmap(cmap_data)?;
    
    // Create CIDToGIDMap
    let mut map = vec![0u8; ((max_cid as usize) + 1) * 2];
    
    for (cid, unicode) in cid_to_unicode {
        // Map Unicode to original GID (would need font's cmap)
        // Then map original GID to new GID
        // This is simplified - real implementation needs font's cmap table
        if let Some(&new_gid) = glyph_mapping.get(&cid) {
            let offset = (cid as usize) * 2;
            map[offset..offset + 2].copy_from_slice(&new_gid.to_be_bytes());
        }
    }
    
    Ok(map)
}

/// Build map for Unicode-based CJK encodings
fn build_unicode_cjk_map(
    encoding: &FontEncoding,
    glyph_mapping: &HashMap<u16, u16>,
    max_cid: u16,
) -> Result<Vec<u8>, SubsetError> {
    let mut map = vec![0u8; ((max_cid as usize) + 1) * 2];
    
    // For Unicode-based encodings, we can compute the mapping
    match encoding {
        FontEncoding::CJK { language, encoding_name, .. } => {
            match language {
                CJKLanguage::Chinese(_) if encoding_name.starts_with("UniGB") => {
                    // UniGB uses Unicode code points as CIDs
                    build_unicode_direct_map(&mut map, glyph_mapping, max_cid);
                }
                CJKLanguage::Japanese(_) if encoding_name.starts_with("UniJIS") => {
                    // UniJIS maps JIS to Unicode
                    build_jis_unicode_map(&mut map, glyph_mapping, max_cid)?;
                }
                CJKLanguage::Korean(_) if encoding_name.starts_with("UniKS") => {
                    // UniKS maps KSC to Unicode
                    build_ksc_unicode_map(&mut map, glyph_mapping, max_cid)?;
                }
                _ => {
                    return Err(SubsetError::UnsupportedEncoding(
                        encoding_name.to_string()
                    ));
                }
            }
        }
        _ => unreachable!(),
    }
    
    Ok(map)
}
```

#### 3.2.3 Chinese Encoding Support (`src/subset/cjk/chinese.rs`)

```rust
/// Handle Simplified Chinese encodings
pub fn build_gb_map(
    encoding_name: &str,
    glyph_mapping: &HashMap<u16, u16>,
    max_cid: u16,
) -> Result<Vec<u8>, SubsetError> {
    let mut map = vec![0u8; ((max_cid as usize) + 1) * 2];
    
    match encoding_name {
        "GB-EUC-H" | "GB-EUC-V" => {
            // GB2312 encoding
            for cid in 0..=max_cid {
                let gid = gb2312_to_gid(cid)?;
                if let Some(&new_gid) = glyph_mapping.get(&gid) {
                    let offset = (cid as usize) * 2;
                    map[offset..offset + 2].copy_from_slice(&new_gid.to_be_bytes());
                }
            }
        }
        "GBK-EUC-H" | "GBK-EUC-V" => {
            // GBK encoding (superset of GB2312)
            for cid in 0..=max_cid {
                let gid = gbk_to_gid(cid)?;
                if let Some(&new_gid) = glyph_mapping.get(&gid) {
                    let offset = (cid as usize) * 2;
                    map[offset..offset + 2].copy_from_slice(&new_gid.to_be_bytes());
                }
            }
        }
        _ => return Err(SubsetError::UnsupportedEncoding(encoding_name.to_string())),
    }
    
    Ok(map)
}

/// Convert GB2312 code to glyph ID
fn gb2312_to_gid(gb_code: u16) -> Result<u16, SubsetError> {
    // GB2312 has a specific mapping table
    // Area 1-9: Symbols and punctuation
    // Area 16-55: Level 1 Hanzi (3755 chars)
    // Area 56-87: Level 2 Hanzi (3008 chars)
    
    let area = (gb_code >> 8) & 0xFF;
    let position = gb_code & 0xFF;
    
    if area >= 0xA1 && area <= 0xA9 {
        // Symbol area
        Ok(((area - 0xA1) * 94 + (position - 0xA1)) as u16)
    } else if area >= 0xB0 && area <= 0xF7 {
        // Hanzi area
        let base = 846;  // After symbols
        Ok((base + (area - 0xB0) * 94 + (position - 0xA1)) as u16)
    } else {
        Ok(0)  // Map to .notdef
    }
}
```

#### 3.2.4 Japanese Encoding Support (`src/subset/cjk/japanese.rs`)

```rust
/// Handle Japanese encodings
pub fn build_japanese_map(
    encoding_name: &str,
    glyph_mapping: &HashMap<u16, u16>,
    max_cid: u16,
) -> Result<Vec<u8>, SubsetError> {
    let mut map = vec![0u8; ((max_cid as usize) + 1) * 2];
    
    match encoding_name {
        "90ms-RKSJ-H" | "90ms-RKSJ-V" => {
            // Microsoft Shift-JIS
            for cid in 0..=max_cid {
                let gid = shift_jis_to_gid(cid)?;
                if let Some(&new_gid) = glyph_mapping.get(&gid) {
                    let offset = (cid as usize) * 2;
                    map[offset..offset + 2].copy_from_slice(&new_gid.to_be_bytes());
                }
            }
        }
        "UniJIS-UTF16-H" | "UniJIS-UTF16-V" => {
            // Unicode to JIS mapping
            build_jis_unicode_map(&mut map, glyph_mapping, max_cid)?;
        }
        "H" | "V" => {
            // Standard JIS encoding
            for cid in 0..=max_cid {
                let gid = jis_to_gid(cid)?;
                if let Some(&new_gid) = glyph_mapping.get(&gid) {
                    let offset = (cid as usize) * 2;
                    map[offset..offset + 2].copy_from_slice(&new_gid.to_be_bytes());
                }
            }
        }
        _ => return Err(SubsetError::UnsupportedEncoding(encoding_name.to_string())),
    }
    
    Ok(map)
}

/// Build JIS to Unicode mapping
pub fn build_jis_unicode_map(
    map: &mut [u8],
    glyph_mapping: &HashMap<u16, u16>,
    max_cid: u16,
) -> Result<(), SubsetError> {
    // JIS X 0208 mapping
    // Row 1-8: Symbols
    // Row 9-15: Hiragana/Katakana
    // Row 16-47: Level 1 Kanji
    // Row 48-84: Level 2 Kanji
    
    for cid in 0..=max_cid.min(8836) {  // JIS has 8836 characters
        let unicode = jis_to_unicode(cid)?;
        // Would need font's cmap to convert Unicode to GID
        // Then use glyph_mapping to get new GID
        // Simplified here
        if let Some(&new_gid) = glyph_mapping.get(&cid) {
            let offset = (cid as usize) * 2;
            map[offset..offset + 2].copy_from_slice(&new_gid.to_be_bytes());
        }
    }
    
    Ok(())
}
```

#### 3.2.5 Korean Encoding Support (`src/subset/cjk/korean.rs`)

```rust
/// Handle Korean encodings
pub fn build_korean_map(
    encoding_name: &str,
    glyph_mapping: &HashMap<u16, u16>,
    max_cid: u16,
) -> Result<Vec<u8>, SubsetError> {
    let mut map = vec![0u8; ((max_cid as usize) + 1) * 2];
    
    match encoding_name {
        "KSCms-UHC-H" | "KSCms-UHC-V" => {
            // Unified Hangul Code
            for cid in 0..=max_cid {
                let gid = uhc_to_gid(cid)?;
                if let Some(&new_gid) = glyph_mapping.get(&gid) {
                    let offset = (cid as usize) * 2;
                    map[offset..offset + 2].copy_from_slice(&new_gid.to_be_bytes());
                }
            }
        }
        "UniKS-UTF16-H" | "UniKS-UTF16-V" => {
            // Unicode-based Korean
            build_ksc_unicode_map(&mut map, glyph_mapping, max_cid)?;
        }
        _ => return Err(SubsetError::UnsupportedEncoding(encoding_name.to_string())),
    }
    
    Ok(map)
}

/// Build KSC to Unicode mapping
pub fn build_ksc_unicode_map(
    map: &mut [u8],
    glyph_mapping: &HashMap<u16, u16>,
    max_cid: u16,
) -> Result<(), SubsetError> {
    // KSC5601 structure:
    // - Hangul syllables: 2350 chars
    // - Hanja (Chinese chars): 4888 chars
    // - Special symbols
    
    for cid in 0..=max_cid {
        let unicode = if cid < 2350 {
            // Hangul syllable
            0xAC00 + cid as u32  // Hangul Syllables block
        } else if cid < 7238 {
            // Hanja
            ksc_hanja_to_unicode(cid - 2350)?
        } else {
            // Symbols
            ksc_symbol_to_unicode(cid - 7238)?
        };
        
        // Simplified - would need font's cmap
        if let Some(&new_gid) = glyph_mapping.get(&cid) {
            let offset = (cid as usize) * 2;
            map[offset..offset + 2].copy_from_slice(&new_gid.to_be_bytes());
        }
    }
    
    Ok(())
}
```

#### 3.2.6 CMap Data Provider (`src/subset/cjk/cmap_data.rs`)

```rust
use std::collections::HashMap;
use std::path::{Path, PathBuf};

/// Built-in CMap provider with embedded data
pub struct BuiltinCMapProvider {
    cmaps: HashMap<String, Vec<u8>>,
}

impl BuiltinCMapProvider {
    pub fn new() -> Self {
        let mut cmaps = HashMap::new();
        
        // Embed common CMaps
        cmaps.insert(
            "GB-EUC-H".to_string(),
            include_bytes!("../resources/cmaps/gb_euc_h.cmap").to_vec(),
        );
        cmaps.insert(
            "90ms-RKSJ-H".to_string(),
            include_bytes!("../resources/cmaps/90ms_rksj_h.cmap").to_vec(),
        );
        cmaps.insert(
            "KSCms-UHC-H".to_string(),
            include_bytes!("../resources/cmaps/kscms_uhc_h.cmap").to_vec(),
        );
        
        BuiltinCMapProvider { cmaps }
    }
}

impl CMapProvider for BuiltinCMapProvider {
    fn get_cmap_data(&self, encoding_name: &str) -> Option<&[u8]> {
        self.cmaps.get(encoding_name).map(|v| v.as_slice())
    }
    
    fn has_cmap(&self, encoding_name: &str) -> bool {
        self.cmaps.contains_key(encoding_name)
    }
}

/// File-based CMap provider
pub struct FileCMapProvider {
    cmap_dir: PathBuf,
}

impl FileCMapProvider {
    pub fn new<P: AsRef<Path>>(cmap_dir: P) -> Self {
        FileCMapProvider {
            cmap_dir: cmap_dir.as_ref().to_path_buf(),
        }
    }
}

impl CMapProvider for FileCMapProvider {
    fn get_cmap_data(&self, encoding_name: &str) -> Option<&[u8]> {
        let file_name = format!("{}.cmap", encoding_name.to_lowercase().replace("-", "_"));
        let path = self.cmap_dir.join(file_name);
        
        // Would need to cache loaded files
        // Simplified here
        None
    }
    
    fn has_cmap(&self, encoding_name: &str) -> bool {
        let file_name = format!("{}.cmap", encoding_name.to_lowercase().replace("-", "_"));
        self.cmap_dir.join(file_name).exists()
    }
}
```

### 3.3 Integration with Phase 2 API

```rust
// Update PdfFontContext to support CMap provider
impl PdfFontContext {
    pub fn with_cmap_provider(mut self, provider: Box<dyn CMapProvider>) -> Self {
        self.cmap_provider = Some(provider);
        self
    }
}

// Update subset_and_map_for_pdf to use CMap provider
pub fn subset_and_map_for_pdf(
    provider: &impl FontTableProvider,
    glyph_ids: &[u16],
    pdf_context: PdfFontContext,
) -> Result<PdfSubsetResult, SubsetError> {
    // ... existing code ...
    
    let cid_to_gid_map = match &pdf_context.encoding {
        FontEncoding::Identity { .. } => {
            build_identity_cid_map(&glyph_mapping, max_cid)
        }
        FontEncoding::CJK { .. } => {
            build_cjk_cid_map(
                &pdf_context.encoding,
                &glyph_mapping,
                max_cid,
                pdf_context.cmap_provider.as_deref(),
            )?
        }
        _ => Vec::new(),
    };
    
    // ... rest of function
}
```

### 3.4 TDD Testing Strategy

**Test-First Approach**: All tests are written BEFORE implementation (see TDD Step 4 above).

**Test Categories**:
1. **Encoding Detection Tests** - Verify FontEncoding::from_pdf_name() for all CJK variants
2. **CMap Provider Tests** - Test both builtin and file-based providers
3. **CID Mapping Tests** - Verify correct CIDToGIDMap generation
4. **Edge Case Tests** - Handle malformed inputs, memory constraints, errors
5. **Integration Tests** - Full PDF context integration
6. **Performance Tests** - Verify timing and memory requirements

**TDD Implementation Pattern**:
```bash
# For each component:
1. Run failing test: cargo test test_name (WILL FAIL)
2. Write minimal implementation
3. Run test again: cargo test test_name (SHOULD PASS)
4. Refactor if needed (while keeping tests green)
5. Move to next test
```

**Comprehensive Test Coverage**: See TDD Step 4 above for the complete test suite including:
- 20+ encoding detection tests covering Chinese, Japanese, Korean variants
- CMap provider tests for both builtin and file-based providers
- Error handling tests for missing CMap data and corrupted files
- Memory constraint and performance tests
- Full integration tests with Phase 2 API

**Test Execution Order**:
1. Basic encoding detection (enables type system development)
2. CMap provider functionality (enables data access)
3. CID mapping generation (enables core functionality)
4. Error handling and edge cases (ensures robustness)
5. Integration with existing API (ensures compatibility)

## 4. Success Criteria

### 4.1 Functional Requirements
- ✅ Support for major Chinese encodings (GB, GBK, CNS, Big5)
- ✅ Support for major Japanese encodings (JIS, Shift-JIS, EUC)
- ✅ Support for major Korean encodings (KSC, UHC)
- ✅ CMap data integration for non-Unicode encodings

### 4.2 Performance Requirements
- CJK detection < 10µs
- CMap parsing < 100ms for large CMaps
- Memory usage < 1MB for CMap cache

### 4.3 Quality Metrics
- Test coverage > 85% for CJK code
- Support for 95% of common CJK encodings
- Correct rendering in PDF viewers

## 5. Dependencies

### 5.1 On Previous Phases
- Phase 1: Core context API
- Phase 2: PdfFontContext structure

### 5.2 External Resources
- Adobe CMap resources (for testing)
- CJK test fonts
- Unicode mapping tables

## 6. Estimated Timeline

| Task | Duration | Notes |
|------|----------|-------|
| CJK type system | 3 hours | Enums and structures |
| Chinese support | 4 hours | GB/GBK/CNS mappings |
| Japanese support | 4 hours | JIS/SJIS mappings |
| Korean support | 3 hours | KSC/UHC mappings |
| CMap integration | 4 hours | Provider system |
| Testing | 4 hours | All encodings |
| Documentation | 2 hours | CJK guide |
| **Total** | **24 hours** | ~3 days |

## 7. Risks & Mitigations

| Risk | Impact | Mitigation |
|------|--------|------------|
| CMap data size | High | Lazy loading, compression |
| Mapping complexity | High | Thorough testing with real fonts |
| Performance impact | Medium | Caching, optimized lookups |
| Incomplete coverage | Low | Start with most common encodings |

## Phase 3 TDD Checklist

### Pre-Implementation (TDD Steps 1-4)

#### Step 1: Preparation
- [ ] Phase 1 (Identity encoding) tests are all passing
- [ ] Phase 2 (PdfFontContext API) tests are all passing  
- [ ] No compilation errors in existing codebase
- [ ] Dependencies properly resolved
- [ ] CJK encoding documentation reviewed
- [ ] Adobe CMap specifications studied

#### Step 2: Analysis & Planning
- [ ] High-level design documented
- [ ] Affected modules identified
- [ ] Task breakdown completed (7 major components)
- [ ] Edge cases enumerated (valid inputs, boundaries, errors)
- [ ] Performance constraints defined (<10µs detection, <100ms parsing, <1MB cache)

#### Step 3: APIs & Signatures Defined
- [ ] FontEncoding enum design complete
- [ ] CMapProvider trait signature defined
- [ ] CJK language variant enums specified
- [ ] CID mapping function signatures planned
- [ ] Integration points with Phase 2 identified
- [ ] Error handling approach documented

#### Step 4: Comprehensive Tests Written (ALL FAILING)
- [ ] 20+ encoding detection tests for Chinese variants (GB, GBK, CNS, Big5, etc.)
- [ ] 15+ encoding detection tests for Japanese variants (JIS, Shift-JIS, Unicode)
- [ ] 10+ encoding detection tests for Korean variants (KSC, UHC, Unicode)
- [ ] Unicode base calculation tests for all CJK languages
- [ ] BuiltinCMapProvider creation and data access tests
- [ ] FileCMapProvider creation and file handling tests
- [ ] CJK CID map generation tests with various inputs
- [ ] Error handling tests (missing CMap, corrupted data, invalid ranges)
- [ ] Memory constraint tests (large max_cid values)
- [ ] Adobe collection parsing tests
- [ ] Edge case tests (malformed names, case sensitivity)
- [ ] Integration tests with PdfFontContext
- [ ] Full subset_and_map_for_pdf integration tests

### Implementation (TDD Step 5)

#### Component 1: CJK Type System
- [ ] CJKLanguage enum implemented
- [ ] ChineseVariant enum implemented (Simplified, Traditional, HongKong)
- [ ] JapaneseVariant enum implemented (JIS, ShiftJIS, Unicode)
- [ ] KoreanVariant enum implemented (KSC, UHC, Unicode)
- [ ] FontEncoding::CJK variant added
- [ ] FontEncoding::AdobeCollection variant added
- [ ] Basic encoding detection tests passing

#### Component 2: CMap Provider System
- [ ] CMapProvider trait implemented
- [ ] BuiltinCMapProvider struct implemented
- [ ] FileCMapProvider struct implemented
- [ ] CMap data access methods working
- [ ] Provider tests passing

#### Component 3: Enhanced FontEncoding
- [ ] from_pdf_name() supports all CJK encodings
- [ ] get_unicode_base() implemented for CJK languages
- [ ] Adobe collection parsing implemented
- [ ] Case sensitivity properly handled
- [ ] All encoding detection tests passing

#### Component 4: CJK CID Mapping
- [ ] build_cjk_cid_map() function implemented
- [ ] build_from_cmap_data() function implemented
- [ ] CMap parsing logic implemented
- [ ] Unicode-based mapping implemented
- [ ] Error handling for missing/corrupted CMap data
- [ ] All CID mapping tests passing

#### Component 5: Language-Specific Support
- [ ] Chinese encoding support (GB/GBK/CNS mappings)
- [ ] Japanese encoding support (JIS/Shift-JIS mappings)
- [ ] Korean encoding support (KSC/UHC mappings)
- [ ] Language-specific tests passing

#### Component 6: Performance & Memory
- [ ] CJK detection under 10µs
- [ ] CMap parsing under 100ms for large files
- [ ] Memory usage under 1MB for CMap cache
- [ ] No memory leaks detected
- [ ] Performance tests passing

#### Component 7: Phase 2 Integration
- [ ] PdfFontContext.with_cmap_provider() implemented
- [ ] subset_and_map_for_pdf() supports CJK encodings
- [ ] CMap provider properly passed through
- [ ] Integration tests passing

### Post-Implementation (TDD Steps 6-7)

#### Step 6: Full-Suite Integration
- [ ] All existing Phase 1 tests still passing
- [ ] All existing Phase 2 tests still passing
- [ ] All new CJK tests passing (100+ tests)
- [ ] No regressions detected
- [ ] Performance benchmarks met
- [ ] Memory usage within limits
- [ ] Thread safety verified (if applicable)

#### Step 7: Documentation & Review
- [ ] API documentation updated with CJK examples
- [ ] Encoding support matrix documented
- [ ] CMap provider usage guide written
- [ ] Code comments added to complex logic
- [ ] Error handling documented
- [ ] Performance characteristics documented
- [ ] Integration examples provided

### Quality Gates

#### Code Quality
- [ ] All functions have unit tests
- [ ] Error paths tested
- [ ] Edge cases covered
- [ ] Code coverage > 85% for CJK module
- [ ] No TODO comments in production code
- [ ] No unwrap() calls in error paths

#### Performance Verification
- [ ] Encoding detection: < 10µs per call
- [ ] CMap parsing: < 100ms for largest CMap
- [ ] Memory usage: < 1MB for CMap cache
- [ ] No performance regression in existing functionality

#### Integration Verification
- [ ] Works with existing Identity encoding
- [ ] Compatible with Phase 2 PdfFontContext API
- [ ] Ready for Phase 4 integration
- [ ] No breaking changes to public API

#### Documentation Completeness
- [ ] All public APIs documented
- [ ] Usage examples provided
- [ ] Error scenarios documented
- [ ] Performance characteristics noted
- [ ] CMap data requirements explained

### Phase 3 Completion Criteria

**Ready for Phase 4 when:**
- [ ] All checklist items above completed
- [ ] Full test suite passing (existing + new)
- [ ] Performance requirements met
- [ ] Documentation complete
- [ ] Code review approved
- [ ] No critical issues outstanding

---

*Document Version: 1.0*  
*Created: 2024-08-21*  
*Updated for TDD: 2024-08-21*  
*Phase: 3 of 5*  
*Priority: MEDIUM - Extends to international markets*