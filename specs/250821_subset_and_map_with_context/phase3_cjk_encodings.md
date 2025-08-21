# Phase 3: CJK Encoding Support

## Executive Summary

Extend the `FontEncoding` enum to support Chinese, Japanese, and Korean (CJK) encodings using a behavior-based grouping approach. This phase enables correct CIDToGIDMap generation for fonts using predefined CMap encodings beyond Identity-H/V.

## 1. What We're Building

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

### 3.4 Testing Strategy

```rust
#[test]
fn test_chinese_encoding_detection() {
    assert_eq!(
        FontEncoding::from_pdf_name("GB-EUC-H"),
        Some(FontEncoding::CJK {
            language: CJKLanguage::Chinese(ChineseVariant::Simplified),
            encoding_name: "GB-EUC-H".to_string(),
            vertical: false,
            requires_cmap_data: true,
        })
    );
}

#[test]
fn test_japanese_encoding_detection() {
    assert_eq!(
        FontEncoding::from_pdf_name("90ms-RKSJ-V"),
        Some(FontEncoding::CJK {
            language: CJKLanguage::Japanese(JapaneseVariant::ShiftJIS),
            encoding_name: "90ms-RKSJ-V".to_string(),
            vertical: true,
            requires_cmap_data: true,
        })
    );
}

#[test]
fn test_korean_encoding_detection() {
    assert_eq!(
        FontEncoding::from_pdf_name("UniKS-UTF16-H"),
        Some(FontEncoding::CJK {
            language: CJKLanguage::Korean(KoreanVariant::Unicode),
            encoding_name: "UniKS-UTF16-H".to_string(),
            vertical: false,
            requires_cmap_data: false,
        })
    );
}

#[test]
fn test_cjk_with_builtin_cmap() {
    let provider = create_test_provider();
    let cmap_provider = Box::new(BuiltinCMapProvider::new());
    
    let context = PdfFontContext {
        encoding: FontEncoding::from_pdf_name("GB-EUC-H").unwrap(),
        max_cid: Some(8000),
        preserve_identity: false,
        is_symbolic: false,
    }.with_cmap_provider(cmap_provider);
    
    let result = subset_and_map_for_pdf(&provider, &glyph_ids, context)?;
    assert!(!result.cid_to_gid_map.is_empty());
}
```

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

---

*Document Version: 1.0*  
*Created: 2024-08-21*  
*Phase: 3 of 5*  
*Priority: MEDIUM - Extends to international markets*