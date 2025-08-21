# CJK Font Encoding Support Guide

## Overview

Allsorts now provides comprehensive support for Chinese, Japanese, and Korean (CJK) font encodings, enabling correct CIDToGIDMap generation for PDF embedding and font subsetting operations. This support is essential for handling fonts with predefined CMap encodings beyond Identity-H/V.

## Quick Start

### Basic CJK Font Subsetting

```rust
use allsorts::subset::context::{FontEncoding, CJKLanguage, ChineseVariant};
use allsorts::subset::cjk::{BuiltinCMapProvider, build_cjk_cid_map};
use allsorts::subset::pdf::{PdfFontContext, subset_and_map_for_pdf};
use std::collections::HashMap;

// Detect CJK encoding from PDF
let encoding = FontEncoding::from_pdf_name("GB-EUC-H").unwrap();

// Create CMap provider for CJK encodings
let cmap_provider = Box::new(BuiltinCMapProvider::new());

// Create PDF context with CJK support
let mut context = PdfFontContext::from_pdf_dict("GB-EUC-H", 0)?
    .with_max_cid(8000)
    .with_cmap_provider(cmap_provider);

// Perform subsetting with CJK-aware CID mapping
let result = subset_and_map_for_pdf(&provider, &glyph_ids, context)?;
```

## Supported Encodings

### Chinese Encodings

#### Simplified Chinese
- `GB-EUC-H`, `GB-EUC-V` - GB2312 encoding
- `GBK-EUC-H`, `GBK-EUC-V` - GBK encoding (superset of GB2312)
- `UniGB-UTF16-H`, `UniGB-UTF16-V` - Unicode-based (no CMap required)

#### Traditional Chinese
- `CNS-EUC-H`, `CNS-EUC-V` - CNS encoding
- `B5pc-H`, `B5pc-V` - Big5 encoding
- `ETen-B5-H`, `ETen-B5-V` - ETen Big5 variant
- `UniCNS-UTF16-H`, `UniCNS-UTF16-V` - Unicode-based

#### Hong Kong Chinese
- `HKscs-B5-H`, `HKscs-B5-V` - Hong Kong Supplementary Character Set

### Japanese Encodings
- `90ms-RKSJ-H`, `90ms-RKSJ-V` - Microsoft Shift-JIS
- `83pv-RKSJ-H`, `83pv-RKSJ-V` - Adobe Shift-JIS
- `H`, `V` - Standard JIS horizontal/vertical
- `EUC-H`, `EUC-V` - EUC-JP encoding
- `UniJIS-UTF16-H`, `UniJIS-UTF16-V` - Unicode-based

### Korean Encodings
- `KSCms-UHC-H`, `KSCms-UHC-V` - Unified Hangul Code
- `KSC-EUC-H`, `KSC-EUC-V` - KSC5601 encoding
- `UniKS-UTF16-H`, `UniKS-UTF16-V` - Unicode-based

### Adobe Collections
- `Adobe-GB1-*` - Adobe GB1 collection
- `Adobe-CNS1-*` - Adobe CNS1 collection
- `Adobe-Japan1-*` - Adobe Japan1 collection
- `Adobe-Korea1-*` - Adobe Korea1 collection

## API Reference

### FontEncoding Enum

```rust
pub enum FontEncoding {
    /// Identity mapping (Identity-H/V)
    Identity { vertical: bool },
    
    /// CJK predefined encodings
    CJK {
        language: CJKLanguage,
        encoding_name: String,
        vertical: bool,
        requires_cmap_data: bool,
    },
    
    /// Adobe standard collections
    AdobeCollection {
        registry: String,
        ordering: String,
        supplement: u16,
    },
    
    /// Custom encoding
    Custom(String),
}
```

### CJK Language Variants

```rust
pub enum CJKLanguage {
    Chinese(ChineseVariant),
    Japanese(JapaneseVariant),
    Korean(KoreanVariant),
}

pub enum ChineseVariant {
    Simplified,   // GB/GBK encodings
    Traditional,  // CNS/Big5 encodings
    HongKong,    // HKSCS encodings
}

pub enum JapaneseVariant {
    JIS,         // JIS-based encodings
    ShiftJIS,    // Shift-JIS based (RKSJ)
    Unicode,     // Unicode-based
}

pub enum KoreanVariant {
    KSC,         // KSC5601 based
    UHC,         // Unified Hangul Code
    Unicode,     // Unicode-based
}
```

### CMap Provider Trait

```rust
pub trait CMapProvider {
    /// Get CMap data for encoding
    fn get_cmap_data(&self, encoding_name: &str) -> Option<&[u8]>;
    
    /// Check if CMap is available
    fn has_cmap(&self, encoding_name: &str) -> bool;
}
```

Two implementations are provided:

1. **BuiltinCMapProvider** - Contains embedded CMap data for common encodings
2. **FileCMapProvider** - Loads CMap data from filesystem

### CJK CID Mapping Functions

```rust
/// Build CIDToGIDMap for CJK encodings
pub fn build_cjk_cid_map(
    encoding: &FontEncoding,
    glyph_mapping: &HashMap<u16, u16>,
    max_cid: u16,
    cmap_provider: Option<&dyn CMapProvider>,
) -> Result<Vec<u8>, SubsetError>
```

## Usage Examples

### Example 1: Chinese Font Subsetting

```rust
use allsorts::subset::context::FontEncoding;
use allsorts::subset::cjk::BuiltinCMapProvider;
use allsorts::subset::pdf::PdfFontContext;

// Detect encoding
let encoding = FontEncoding::from_pdf_name("GBK-EUC-H").unwrap();

// Create context with CMap provider
let cmap_provider = Box::new(BuiltinCMapProvider::new());
let context = PdfFontContext {
    encoding,
    max_cid: Some(10000),
    preserve_identity: false,
    is_symbolic: false,
    cid_to_gid_map: None,
    writing_mode: WritingMode::Horizontal,
    cmap_provider: Some(cmap_provider),
};

// Perform subsetting
let result = subset_and_map_for_pdf(&provider, &glyph_ids, context)?;
```

### Example 2: Japanese Vertical Text

```rust
// Japanese vertical text
let encoding = FontEncoding::from_pdf_name("90ms-RKSJ-V").unwrap();
assert!(matches!(encoding, FontEncoding::CJK { vertical: true, .. }));

let context = PdfFontContext::from_pdf_dict("90ms-RKSJ-V", 0)?
    .with_cmap_provider(Box::new(BuiltinCMapProvider::new()));
```

### Example 3: Unicode-Based Korean

```rust
// Unicode-based Korean (no CMap required)
let encoding = FontEncoding::from_pdf_name("UniKS-UTF16-H").unwrap();
assert!(matches!(encoding, FontEncoding::CJK { 
    requires_cmap_data: false, .. 
}));

// No CMap provider needed for Unicode-based encodings
let context = PdfFontContext::from_pdf_dict("UniKS-UTF16-H", 0)?;
```

### Example 4: Using File-Based CMap Provider

```rust
use allsorts::subset::cjk::FileCMapProvider;
use std::path::Path;

// Load CMaps from directory
let cmap_dir = Path::new("/usr/share/fonts/cmap");
let provider = Box::new(FileCMapProvider::new(cmap_dir));

let context = PdfFontContext::from_pdf_dict("GB-EUC-H", 0)?
    .with_cmap_provider(provider);
```

## Performance Considerations

1. **CMap Caching**: The `BuiltinCMapProvider` keeps all CMap data in memory. For large deployments, consider implementing a caching strategy.

2. **Memory Usage**: CIDToGIDMap size is `(max_cid + 1) * 2` bytes. Set `max_cid` appropriately to avoid excessive memory usage.

3. **Unicode-Based Encodings**: These don't require CMap data and are faster to process.

## Error Handling

```rust
use allsorts::subset::SubsetError;

match build_cjk_cid_map(&encoding, &mapping, max_cid, provider) {
    Ok(cid_map) => {
        // Success
    }
    Err(SubsetError::CidGenerationFailed(msg)) => {
        // CMap data missing or invalid
    }
    Err(SubsetError::UnsupportedEncoding(name)) => {
        // Encoding not supported
    }
    Err(e) => {
        // Other errors
    }
}
```

## Testing

The implementation includes comprehensive tests:

```bash
# Run CJK encoding tests
cargo test --test cjk_encodings

# Test specific encoding detection
cargo test test_chinese_gb_encoding_detection
cargo test test_japanese_encoding_detection
cargo test test_korean_encoding_detection

# Test CMap providers
cargo test test_builtin_cmap_provider
cargo test test_file_cmap_provider
```

## Limitations

1. **CMap Data**: Current implementation uses placeholder CMap data. Production use requires actual Adobe CMap files.

2. **Partial Implementation**: Some complex mappings (e.g., full GB18030) are simplified.

3. **Font cmap Table**: Full implementation would need access to the font's cmap table to convert Unicode to original GIDs.

## Future Enhancements

1. **Complete CMap Data**: Embed or load actual Adobe CMap resources
2. **GB18030 Support**: Full support for the complete Chinese character set
3. **Vertical Metrics**: Enhanced support for vertical text layout
4. **CMap Parsing**: Full PostScript CMap format parser
5. **Optimization**: Lazy loading and compression of CMap data

## Migration Guide

### From Identity-Only to CJK Support

Before:
```rust
let context = PdfFontContext::identity_h();
```

After:
```rust
let encoding = FontEncoding::from_pdf_name(encoding_name)?;
let context = PdfFontContext::from_pdf_dict(encoding_name, flags)?
    .with_cmap_provider(Box::new(BuiltinCMapProvider::new()));
```

### Adding CJK to Existing Code

```rust
// Check if encoding is CJK
if matches!(encoding, FontEncoding::CJK { .. }) {
    // Add CMap provider
    context = context.with_cmap_provider(
        Box::new(BuiltinCMapProvider::new())
    );
}
```

## References

- [Adobe CMap Resources](https://github.com/adobe-type-tools/cmap-resources)
- [PDF Reference 1.7](https://www.adobe.com/devnet/pdf/pdf_reference.html) - Section 5.6 (CIDFonts)
- [CJK Unicode FAQ](https://www.unicode.org/faq/han_cjk.html)
- [OpenType CJK Font Guidelines](https://docs.microsoft.com/en-us/typography/opentype/spec/cjk)