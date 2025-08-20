# Enhanced CID Font Detection for Comprehensive PDF Support

## Executive Summary

The current CID font detection in Allsorts (v0.15.x) only handles sparse glyph ID patterns typical of Western fonts with special characters. This specification outlines enhancements to support the full spectrum of CID font usage patterns in PDFs, including dense CJK fonts, symbolic fonts, predefined CMap encodings, and previously-subset fonts.

## 1. Problem Context

### 1.1 What Are CID Fonts?

CID (Character Identifier) fonts are a PostScript/PDF font format that separates character encoding from glyph selection:
- **CID**: Character Identifier - an integer that identifies a character in a character collection
- **GID**: Glyph Identifier - the actual glyph index in the font file
- **CIDToGIDMap**: A mapping table from CIDs to GIDs

In PDFs, there are two CID font types:
- **CIDFontType0**: Based on PostScript/CFF outlines
- **CIDFontType2**: Based on TrueType outlines

### 1.2 Current Detection Limitations

The current implementation (`src/subset.rs::detect_cid_from_glyph_pattern`) only detects:
```rust
// Current detection logic (simplified)
if glyph_ids.contains([143, 159, 178]) && num_glyphs < 300 {
    return true; // Sparse pattern detection
}
```

This misses several critical use cases:

1. **Dense CJK Fonts**: Chinese/Japanese/Korean fonts with thousands of sequential glyphs
2. **Predefined CMaps**: Fonts using GB-EUC-H, 83pv-RKSJ-H, KSCms-UHC-H, etc.
3. **Symbolic Fonts**: Custom symbol fonts starting at GID 256+
4. **Re-subset Fonts**: Previously subset fonts with non-standard patterns
5. **Large Latin Fonts**: Extended Latin with 500+ glyphs

### 1.3 Real-World Impact

Missing CID detection causes:
- Characters render as '?' or boxes in PDF viewers
- Text extraction fails (affects accessibility)
- PDF/A compliance issues
- Larger file sizes (can't properly subset)

## 2. Technical Analysis

### 2.1 CID Font Patterns in PDFs

#### Pattern 1: Dense Sequential (CJK)
```
Font: Adobe Ming Std
Total Glyphs: 13,000+
Usage Pattern: [0, 1, 2, ..., 8000, 8001, ...]
Encoding: Identity-H or GB-EUC-H
Characteristic: Sequential CIDs with CID == GID
```

#### Pattern 2: Sparse High-Range (Symbols)
```
Font: Custom Corporate Symbols
Total Glyphs: 200
Usage Pattern: [0, 256, 257, 512, 513, 768]
Encoding: Identity-H
Characteristic: Avoids ASCII range, uses high GIDs
```

#### Pattern 3: Predefined CMap
```
Font: MS Gothic
Total Glyphs: 8,000+
Usage Pattern: Non-sequential based on JIS encoding
Encoding: 90ms-RKSJ-H
Characteristic: GIDs follow Japanese Industrial Standard
```

#### Pattern 4: Previously Subset
```
Font: Times-Roman (subset)
Original Glyphs: 3,000
Subset Glyphs: 50
Usage Pattern: [0, 17, 234, 567, 891]
Encoding: Identity-H
Characteristic: Sparse, non-pattern GIDs
```

### 2.2 Detection Heuristics

Each pattern requires different detection logic:

| Pattern | Detection Method | False Positive Risk | Priority |
|---------|-----------------|-------------------|----------|
| Dense CJK | Count > 500 + sequential check | Low | High |
| Symbolic | GIDs > 256 + small font | Medium | High |
| Predefined CMap | Check encoding hint | Low | Medium |
| Re-subset | Entropy analysis | High | Low |
| Large Latin | Count > 300 + coverage | Medium | Medium |

## 3. Proposed Solution

### 3.1 Enhanced Detection Algorithm

```rust
/// Comprehensive CID font detection with multiple pattern recognition
fn detect_cid_font_enhanced(
    provider: &impl FontTableProvider,
    glyph_ids: &[u16],
    profile: &SubsetProfile,
) -> bool {
    // Early return for CFF CID fonts (existing)
    if is_cff_cid_font(provider) {
        return true;
    }
    
    // Only check TrueType patterns for PDF profile
    if !matches!(profile, SubsetProfile::Pdf) {
        return false;
    }
    
    let num_glyphs = get_total_glyph_count(provider);
    let requested_count = glyph_ids.len();
    let max_gid = glyph_ids.iter().copied().max().unwrap_or(0);
    
    // Pattern 1: Dense CJK fonts
    if is_dense_cjk_pattern(glyph_ids, num_glyphs) {
        return true;
    }
    
    // Pattern 2: Sparse high-range (existing logic enhanced)
    if is_sparse_pattern_enhanced(glyph_ids, num_glyphs) {
        return true;
    }
    
    // Pattern 3: Symbolic fonts
    if is_symbolic_pattern(glyph_ids, num_glyphs) {
        return true;
    }
    
    // Pattern 4: Re-subset detection
    if is_likely_resubset(glyph_ids, num_glyphs) {
        return true;
    }
    
    // Pattern 5: Conservative PDF fallback
    if should_use_cid_for_pdf(glyph_ids, num_glyphs, max_gid) {
        return true;
    }
    
    false
}
```

### 3.2 Pattern Detection Functions

#### 3.2.1 Dense CJK Detection
```rust
fn is_dense_cjk_pattern(glyph_ids: &[u16], total_glyphs: u16) -> bool {
    // CJK fonts typically have many glyphs
    if glyph_ids.len() < 500 {
        return false;
    }
    
    // Check for sequential runs
    let sequential_runs = count_sequential_runs(glyph_ids);
    let sequentiality_ratio = sequential_runs as f32 / glyph_ids.len() as f32;
    
    // High sequentiality indicates CJK
    sequentiality_ratio > 0.7
}

fn count_sequential_runs(glyph_ids: &[u16]) -> usize {
    let mut sorted = glyph_ids.to_vec();
    sorted.sort_unstable();
    sorted.dedup();
    
    let mut runs = 0;
    let mut current_run = 1;
    
    for window in sorted.windows(2) {
        if window[1] == window[0] + 1 {
            current_run += 1;
        } else {
            if current_run >= 3 {
                runs += current_run;
            }
            current_run = 1;
        }
    }
    
    if current_run >= 3 {
        runs += current_run;
    }
    
    runs
}
```

#### 3.2.2 Enhanced Sparse Detection
```rust
fn is_sparse_pattern_enhanced(glyph_ids: &[u16], total_glyphs: u16) -> bool {
    // Known sparse GID ranges for common CID fonts
    const SPARSE_RANGES: &[(u16, u16)] = &[
        (140, 180),   // Bullets, special punctuation
        (8200, 8300), // CJK punctuation
        (256, 512),   // Extended Latin
    ];
    
    let has_sparse_gids = glyph_ids.iter().any(|&gid| {
        SPARSE_RANGES.iter().any(|(start, end)| {
            gid >= *start && gid <= *end
        })
    });
    
    // Sparse pattern with small font
    if has_sparse_gids && total_glyphs < 500 {
        return true;
    }
    
    // High GID to glyph count ratio
    let max_gid = glyph_ids.iter().copied().max().unwrap_or(0);
    let sparsity_ratio = max_gid as f32 / total_glyphs as f32;
    
    sparsity_ratio > 1.5 && glyph_ids.len() > 10
}
```

#### 3.2.3 Symbolic Font Detection
```rust
fn is_symbolic_pattern(glyph_ids: &[u16], total_glyphs: u16) -> bool {
    // Symbolic fonts avoid ASCII range
    let non_ascii_count = glyph_ids.iter()
        .filter(|&&gid| gid >= 256)
        .count();
    
    let ascii_count = glyph_ids.iter()
        .filter(|&&gid| gid > 0 && gid < 128)
        .count();
    
    // Mostly non-ASCII with small total
    non_ascii_count > ascii_count && 
    total_glyphs < 500 &&
    glyph_ids.len() > 5
}
```

#### 3.2.4 Re-subset Detection
```rust
fn is_likely_resubset(glyph_ids: &[u16], total_glyphs: u16) -> bool {
    if glyph_ids.len() < 20 {
        return false;
    }
    
    // Calculate entropy of GID distribution
    let entropy = calculate_gid_entropy(glyph_ids);
    
    // High entropy with small font suggests re-subset
    entropy > 3.5 && total_glyphs < 300
}

fn calculate_gid_entropy(glyph_ids: &[u16]) -> f32 {
    let mut sorted = glyph_ids.to_vec();
    sorted.sort_unstable();
    sorted.dedup();
    
    if sorted.len() < 2 {
        return 0.0;
    }
    
    // Calculate gaps between consecutive GIDs
    let gaps: Vec<u16> = sorted.windows(2)
        .map(|w| w[1] - w[0])
        .collect();
    
    // Shannon entropy of gap distribution
    let mut gap_counts = std::collections::HashMap::new();
    for gap in &gaps {
        *gap_counts.entry(*gap).or_insert(0) += 1;
    }
    
    let total = gaps.len() as f32;
    gap_counts.values()
        .map(|&count| {
            let p = count as f32 / total;
            -p * p.log2()
        })
        .sum()
}
```

#### 3.2.5 Conservative PDF Fallback
```rust
fn should_use_cid_for_pdf(
    glyph_ids: &[u16],
    total_glyphs: u16,
    max_gid: u16,
) -> bool {
    // For PDF subsetting, be conservative with larger fonts
    // or when high GIDs are used
    
    // Large font with high GIDs
    if total_glyphs > 1000 && max_gid > 500 {
        return true;
    }
    
    // Medium font with very high GIDs
    if total_glyphs > 300 && max_gid > 1000 {
        return true;
    }
    
    // Many glyphs requested from large font
    if glyph_ids.len() > 100 && total_glyphs > 500 {
        return true;
    }
    
    false
}
```

### 3.3 Context-Aware API

For maximum reliability, add a context-aware API:

```rust
/// Font usage context for better CID detection
#[derive(Debug, Clone, PartialEq)]
pub enum FontContext {
    /// Unknown context - use heuristics
    Unknown,
    /// PDF Type0 font with specified encoding
    PdfType0 { 
        encoding: CMapEncoding 
    },
    /// PDF TrueType font (non-CID)
    PdfTrueType,
    /// Web font for browser rendering
    WebFont,
    /// Desktop application font
    Desktop,
}

/// CMap encodings used in PDFs
#[derive(Debug, Clone, PartialEq)]
pub enum CMapEncoding {
    /// Identity mapping (CID == GID)
    IdentityH,
    IdentityV,
    /// Chinese encodings
    GbEucH,
    GbkEucH,
    Gbk2kH,
    UniGbUcs2H,
    /// Japanese encodings
    UniJisUtf16H,
    UniJis2004Utf16H,
    Rksj83pvH,
    Rksj90msH,
    /// Korean encodings
    KscmsUhcH,
    KscpcEucH,
    UniKsUtf16H,
    /// Custom encoding
    Custom(String),
}

impl CMapEncoding {
    /// Returns true if this encoding requires CID font treatment
    pub fn requires_cid(&self) -> bool {
        match self {
            // All predefined CMaps require CID
            CMapEncoding::IdentityH | CMapEncoding::IdentityV => true,
            CMapEncoding::GbEucH | CMapEncoding::GbkEucH => true,
            CMapEncoding::UniJisUtf16H | CMapEncoding::Rksj90msH => true,
            CMapEncoding::KscmsUhcH | CMapEncoding::UniKsUtf16H => true,
            _ => true,
        }
    }
    
    /// Parse encoding name from PDF
    pub fn from_pdf_name(name: &str) -> Self {
        match name {
            "Identity-H" => CMapEncoding::IdentityH,
            "Identity-V" => CMapEncoding::IdentityV,
            "GB-EUC-H" => CMapEncoding::GbEucH,
            "GBK-EUC-H" => CMapEncoding::GbkEucH,
            "UniJIS-UTF16-H" => CMapEncoding::UniJisUtf16H,
            "90ms-RKSJ-H" => CMapEncoding::Rksj90msH,
            "KSCms-UHC-H" => CMapEncoding::KscmsUhcH,
            _ => CMapEncoding::Custom(name.to_string()),
        }
    }
}

/// Enhanced subset_and_map with context awareness
pub fn subset_and_map_with_context(
    provider: &impl FontTableProvider,
    glyph_ids: &[u16],
    profile: &SubsetProfile,
    cmap_target: CmapTarget,
    context: FontContext,
) -> Result<SubsetResult, SubsetError> {
    let (font_data, glyph_mapping) =
        subset_with_mapping(provider, glyph_ids, profile, cmap_target)?;
    
    // Check context first, then fall back to heuristics
    let is_cid = match context {
        FontContext::PdfType0 { encoding } => encoding.requires_cid(),
        FontContext::PdfTrueType => false,
        FontContext::WebFont => false,
        FontContext::Desktop => false,
        FontContext::Unknown => detect_cid_font_enhanced(provider, glyph_ids, profile),
    };
    
    if is_cid {
        let max_cid = determine_max_cid_from_context(&context, glyph_ids);
        let cid_to_gid_map = build_cid_to_gid_map(None, &glyph_mapping, max_cid);
        
        Ok(SubsetResult::Cid {
            font_data,
            glyph_mapping,
            cid_to_gid_map,
        })
    } else {
        Ok(SubsetResult::Simple {
            font_data,
            glyph_mapping,
        })
    }
}

fn determine_max_cid_from_context(
    context: &FontContext,
    glyph_ids: &[u16],
) -> u16 {
    match context {
        FontContext::PdfType0 { encoding } => {
            match encoding {
                // CJK encodings need full range
                CMapEncoding::GbEucH | CMapEncoding::UniGbUcs2H => 65535,
                CMapEncoding::UniJisUtf16H | CMapEncoding::Rksj90msH => 65535,
                CMapEncoding::KscmsUhcH | CMapEncoding::UniKsUtf16H => 65535,
                // Identity uses actual max GID
                _ => glyph_ids.iter().copied().max().unwrap_or(0),
            }
        }
        _ => glyph_ids.iter().copied().max().unwrap_or(0),
    }
}
```

## 4. Implementation Plan

### 4.1 Phase 1: Core Detection Enhancement (Week 1)

1. **Refactor existing detection**:
   - Move `detect_cid_font` to public module
   - Split into smaller, testable functions
   - Add logging for detection decisions

2. **Implement pattern detectors**:
   - Dense CJK pattern detection
   - Enhanced sparse pattern detection
   - Symbolic font detection

3. **Update existing API**:
   - Modify `subset_and_map` to use enhanced detection
   - Ensure backward compatibility

### 4.2 Phase 2: Context API (Week 2)

1. **Design context types**:
   - Define `FontContext` enum
   - Define `CMapEncoding` enum
   - Add parsing utilities

2. **Implement context-aware API**:
   - `subset_and_map_with_context` function
   - Context-based max CID calculation
   - Integration with existing subsetting

3. **Documentation**:
   - API documentation with examples
   - Migration guide for existing users

### 4.3 Phase 3: Testing & Validation (Week 3)

1. **Unit tests for each pattern**:
   - Dense CJK detection tests
   - Sparse pattern tests
   - Symbolic font tests
   - Re-subset detection tests

2. **Integration tests**:
   - Real font files for each pattern
   - PDF embedding scenarios
   - Performance benchmarks

3. **Validation with real PDFs**:
   - Test with CJK PDFs
   - Test with symbolic fonts
   - Test with re-subset scenarios

## 5. Test Strategy

### 5.1 Test Fonts Required

Create or obtain test fonts for each pattern:

| Font Type | Test Font | Source | Size | Test GIDs |
|-----------|-----------|--------|------|-----------|
| Dense CJK | NotoSansCJK-Regular | Google | 15MB | [0..8000] |
| Japanese | MSGothic subset | Windows | 8MB | JIS pattern |
| Korean | Malgun Gothic | Windows | 4MB | KSC pattern |
| Symbolic | FontAwesome | Public | 200KB | [256..512] |
| Re-subset | Times-subset | Create | 50KB | Random sparse |
| Large Latin | NotoSans-Regular | Google | 500KB | [0..800] |

### 5.2 Test Cases

```rust
#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_dense_cjk_detection() {
        let font_data = load_test_font("NotoSansCJK-Regular.otf");
        let dense_gids: Vec<u16> = (0..5000).collect();
        
        let result = detect_pattern(&font_data, &dense_gids);
        assert_eq!(result, PatternType::DenseCJK);
    }
    
    #[test]
    fn test_japanese_encoding_detection() {
        let font_data = load_test_font("MSGothic.ttf");
        let jis_gids = load_jis_test_pattern();
        
        let context = FontContext::PdfType0 {
            encoding: CMapEncoding::Rksj90msH,
        };
        
        let result = subset_and_map_with_context(
            &provider,
            &jis_gids,
            &SubsetProfile::Pdf,
            CmapTarget::Unicode,
            context,
        )?;
        
        assert!(matches!(result, SubsetResult::Cid { .. }));
    }
    
    #[test]
    fn test_symbolic_font_detection() {
        let font_data = load_test_font("FontAwesome.ttf");
        let symbol_gids = vec![0, 256, 257, 300, 400, 500];
        
        let result = detect_pattern(&font_data, &symbol_gids);
        assert_eq!(result, PatternType::Symbolic);
    }
    
    #[test]
    fn test_resubset_detection() {
        let font_data = create_resubset_font();
        let random_gids = vec![0, 17, 234, 567, 891, 1234, 2000];
        
        let result = detect_pattern(&font_data, &random_gids);
        assert_eq!(result, PatternType::Resubset);
    }
    
    #[test]
    fn test_performance_large_font() {
        let font_data = load_test_font("NotoSansCJK-Regular.otf");
        let large_gids: Vec<u16> = (0..10000).collect();
        
        let start = std::time::Instant::now();
        let _result = subset_and_map(&provider, &large_gids, 
                                     &SubsetProfile::Pdf, 
                                     CmapTarget::Unicode);
        let duration = start.elapsed();
        
        assert!(duration.as_secs() < 2, "Detection too slow for large fonts");
    }
}
```

### 5.3 Benchmarks

```rust
#[bench]
fn bench_cjk_detection(b: &mut Bencher) {
    let font_data = load_test_font("NotoSansCJK-Regular.otf");
    let gids: Vec<u16> = (0..5000).collect();
    
    b.iter(|| {
        detect_cid_font_enhanced(&provider, &gids, &SubsetProfile::Pdf)
    });
}

#[bench]
fn bench_entropy_calculation(b: &mut Bencher) {
    let random_gids: Vec<u16> = generate_random_gids(1000);
    
    b.iter(|| {
        calculate_gid_entropy(&random_gids)
    });
}
```

## 6. Migration Guide

### 6.1 For Existing Users

No breaking changes - existing code continues to work:

```rust
// Existing code - still works
let result = subset_and_map(&provider, &glyph_ids, 
                           &SubsetProfile::Pdf, 
                           CmapTarget::Unicode)?;
```

### 6.2 For PDF Libraries

Recommended migration to context-aware API:

```rust
// Old approach - relies on heuristics
let result = subset_and_map(&provider, &glyph_ids,
                           &SubsetProfile::Pdf,
                           CmapTarget::Unicode)?;

// New approach - explicit context
let encoding = CMapEncoding::from_pdf_name(&pdf_font.encoding);
let context = FontContext::PdfType0 { encoding };

let result = subset_and_map_with_context(&provider, &glyph_ids,
                                        &SubsetProfile::Pdf,
                                        CmapTarget::Unicode,
                                        context)?;
```

### 6.3 Performance Considerations

- Pattern detection adds ~5-10¼s overhead per call
- Entropy calculation for re-subset detection: ~50¼s for 1000 glyphs
- Context API has minimal overhead (enum match)
- Large CJK fonts may require more memory for CIDToGIDMap (128KB for full range)

## 7. References & Resources

### 7.1 Specifications
- [PDF Reference 1.7](https://opensource.adobe.com/dc-acrobat-sdk-docs/pdfstandards/PDF32000_2008.pdf) - Section 9.7: CIDFonts
- [CMap Resources](https://github.com/adobe-type-tools/cmap-resources) - Adobe's CMap files
- [OpenType CID Font Files](https://docs.microsoft.com/en-us/typography/opentype/spec/cid) - Microsoft specification

### 7.2 Test Resources
- [Google Noto Fonts](https://github.com/googlefonts/noto-fonts) - CJK test fonts
- [Adobe CJK Fonts](https://github.com/adobe-fonts/source-han-sans) - Source Han Sans
- [Font testing tools](https://github.com/adobe-type-tools/afdko) - Adobe Font Development Kit

### 7.3 Implementation References
- [HarfBuzz CID handling](https://github.com/harfbuzz/harfbuzz/blob/main/src/hb-ot-cff2-table.hh)
- [FreeType CID support](https://github.com/freetype/freetype/tree/master/src/cid)
- [Poppler PDF CID fonts](https://gitlab.freedesktop.org/poppler/poppler/-/tree/master/poppler)

## 8. Success Criteria

The implementation is successful when:

1. **Detection Accuracy**: 
   -  100% of CJK fonts detected as CID
   -  95%+ of symbolic fonts detected
   -  <5% false positive rate

2. **Performance**:
   -  Detection overhead < 100¼s for typical fonts
   -  < 2s for 10,000 glyph subset

3. **Compatibility**:
   -  All existing tests pass
   -  No breaking API changes
   -  PDF/A validation passes

4. **Coverage**:
   -  Test coverage > 90% for detection code
   -  Integration tests for each pattern
   -  Real-world PDF validation

## 9. Risk Mitigation

### 9.1 False Positives
**Risk**: Non-CID fonts incorrectly detected as CID
**Mitigation**: 
- Conservative thresholds
- Profile-specific detection (only for PDF)
- Context API for explicit control

### 9.2 Performance Regression
**Risk**: Detection slows down subsetting
**Mitigation**:
- Early returns for obvious cases
- Cached detection results
- Optimized algorithms (avoid sorting when possible)

### 9.3 Memory Usage
**Risk**: Large CIDToGIDMap for CJK fonts (128KB)
**Mitigation**:
- Compress sparse maps
- Lazy allocation
- Configurable max CID limit

## 10. Timeline

| Week | Phase | Deliverables |
|------|-------|-------------|
| 1 | Core Detection | Enhanced detection functions, unit tests |
| 2 | Context API | New API, integration tests |
| 3 | Testing | Font tests, benchmarks, documentation |
| 4 | Integration | PDF library integration, validation |

## Appendix A: Common CID Font Patterns

### A.1 Chinese Fonts
```
Font: SimSun
Glyphs: 28,762
Common GIDs: 0-128 (ASCII), 8000-28000 (CJK)
Encoding: GB-EUC-H, GBK-EUC-H, UniGB-UCS2-H
Detection: Dense sequential > 5000 glyphs
```

### A.2 Japanese Fonts
```
Font: MS Mincho
Glyphs: 10,000+
Common GIDs: 0-128, 300-500 (Kana), 8000+ (Kanji)
Encoding: 90ms-RKSJ-H, UniJIS-UTF16-H
Detection: Mixed sparse/dense pattern
```

### A.3 Korean Fonts
```
Font: Batang
Glyphs: 11,172 (Hangul) + 4000 (Hanja)
Common GIDs: 0-128, 44032-55203 (Hangul)
Encoding: KSCms-UHC-H, UniKS-UTF16-H
Detection: Large gap between ASCII and Hangul
```

### A.4 Symbol Fonts
```
Font: Wingdings
Glyphs: 200-300
Common GIDs: 256-512
Encoding: Identity-H
Detection: No ASCII glyphs, all > 255
```

## Appendix B: Debugging Guide

### B.1 Enable Detection Logging

```rust
// Add to Cargo.toml
[dependencies]
log = "0.4"
env_logger = "0.10"

// In detection code
log::debug!("CID detection for {} glyphs, max GID: {}", 
           glyph_ids.len(), max_gid);
log::debug!("Detection result: {:?}, reason: {}", 
           is_cid, detection_reason);
```

### B.2 Detection Test Tool

Create a CLI tool for testing detection:

```rust
// src/bin/test_cid_detection.rs
use allsorts::subset::detect_cid_font_enhanced;
use clap::Parser;

#[derive(Parser)]
struct Args {
    /// Font file path
    font: PathBuf,
    /// Comma-separated glyph IDs
    #[arg(value_delimiter = ',')]
    gids: Vec<u16>,
    /// Force specific context
    #[arg(long)]
    context: Option<String>,
}

fn main() {
    let args = Args::parse();
    env_logger::init();
    
    let font_data = std::fs::read(&args.font).unwrap();
    let provider = create_provider(&font_data);
    
    let is_cid = detect_cid_font_enhanced(
        &provider,
        &args.gids,
        &SubsetProfile::Pdf,
    );
    
    println!("Font: {}", args.font.display());
    println!("GIDs: {:?}", args.gids);
    println!("Detected as CID: {}", is_cid);
}
```

Usage:
```bash
cargo run --bin test_cid_detection -- \
    --font NotoSansCJK.otf \
    --gids 0,1,2,3,1000,2000,3000
```

## Appendix C: FAQ

**Q: Why not always use CID for PDF fonts?**
A: CID adds overhead (CIDToGIDMap) and complexity. Simple Latin fonts don't need it.

**Q: Can we detect from the font file alone?**
A: Not reliably. TrueType fonts don't contain CID markers. We need usage context.

**Q: What about WOFF/WOFF2?**
A: Same detection applies after decompression. The wrapper format doesn't affect CID usage.

**Q: How does this relate to Unicode?**
A: CID is orthogonal to Unicode. A font can use Unicode cmap with CID structure.

**Q: Performance impact for web fonts?**
A: Minimal - web fonts use `SubsetProfile::Web` which skips CID detection entirely.

---

*Document Version: 1.0*
*Last Updated: 2024-08-20*
*Author: [Implementation Team]*