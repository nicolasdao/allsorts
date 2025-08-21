# Phase 4: Detection & Auto-Configuration

## Executive Summary

Implement intelligent encoding detection and auto-configuration to reduce manual configuration burden. This phase adds the Detection Helper system that can automatically determine the correct encoding from PDF context and font characteristics.

## 1. What We're Building

### 1.1 Core Components

```rust
/// Encoding detection result with confidence
#[derive(Debug, Clone)]
pub struct EncodingDetection {
    pub encoding: FontEncoding,
    pub confidence: DetectionConfidence,
    pub reasoning: Vec<String>,
    pub alternatives: Vec<FontEncoding>,
}

#[derive(Debug, Clone, PartialEq, Ord, PartialOrd, Eq)]
pub enum DetectionConfidence {
    Certain,      // 100% sure (e.g., explicit encoding name)
    High,         // 90%+ (strong patterns match)
    Medium,       // 70-90% (some patterns match)
    Low,          // <70% (guessing based on heuristics)
}

/// Auto-detection helper
pub struct EncodingDetector {
    font_provider: Box<dyn FontTableProvider>,
    cmap_provider: Option<Box<dyn CMapProvider>>,
    detection_cache: HashMap<Vec<u16>, EncodingDetection>,
}

/// PDF font information for detection
#[derive(Debug, Clone)]
pub struct PdfFontInfo {
    pub encoding_name: Option<String>,
    pub font_name: Option<String>,
    pub flags: u32,
    pub registry: Option<String>,
    pub ordering: Option<String>,
    pub supplement: Option<u16>,
    pub to_unicode: Option<Vec<u8>>,
}
```

### 1.2 Detection Pipeline

```rust
impl EncodingDetector {
    /// Detect encoding from multiple sources
    pub fn detect(
        &mut self,
        glyph_ids: &[u16],
        pdf_info: Option<&PdfFontInfo>,
    ) -> EncodingDetection {
        // 1. Check cache
        // 2. Try explicit encoding name
        // 3. Analyze font structure
        // 4. Pattern matching on glyph IDs
        // 5. Statistical analysis
        // 6. Fallback to Identity-H
    }
}
```

## 2. Why This Phase

### 2.1 User Experience
- **Reduces Errors**: Automatic detection prevents wrong encoding selection
- **Simplifies API**: Users don't need deep PDF/font knowledge
- **Faster Integration**: Less configuration required
- **Better Defaults**: Smart fallbacks for edge cases

### 2.2 Technical Benefits
- **Caching**: Avoid repeated detection for same patterns
- **Confidence Levels**: Users know when to verify
- **Debugging**: Reasoning explains decisions
- **Flexibility**: Override auto-detection when needed

## 3. Technical Implementation

### 3.1 File Structure

```
src/subset/
├── detection/
│   ├── mod.rs              (detection module root)
│   ├── detector.rs         (main detector implementation)
│   ├── patterns.rs         (pattern matching)
│   ├── statistics.rs       (statistical analysis)
│   └── heuristics.rs       (encoding heuristics)
└── auto/
    ├── mod.rs              (auto-configuration)
    └── builder.rs          (auto-builder)
```

### 3.2 Implementation Details

#### 3.2.1 Main Detector (`src/subset/detection/detector.rs`)

```rust
use crate::subset::context::{FontEncoding, CJKLanguage};
use crate::subset::detection::patterns::{PatternMatcher, PatternType};
use crate::subset::detection::statistics::GlyphStatistics;
use std::collections::HashMap;

pub struct EncodingDetector {
    font_provider: Box<dyn FontTableProvider>,
    cmap_provider: Option<Box<dyn CMapProvider>>,
    detection_cache: HashMap<Vec<u16>, EncodingDetection>,
    pattern_matcher: PatternMatcher,
}

impl EncodingDetector {
    pub fn new(font_provider: Box<dyn FontTableProvider>) -> Self {
        EncodingDetector {
            font_provider,
            cmap_provider: None,
            detection_cache: HashMap::new(),
            pattern_matcher: PatternMatcher::new(),
        }
    }
    
    pub fn with_cmap_provider(mut self, provider: Box<dyn CMapProvider>) -> Self {
        self.cmap_provider = Some(provider);
        self
    }
    
    /// Main detection pipeline
    pub fn detect(
        &mut self,
        glyph_ids: &[u16],
        pdf_info: Option<&PdfFontInfo>,
    ) -> EncodingDetection {
        // Check cache
        if let Some(cached) = self.detection_cache.get(glyph_ids) {
            return cached.clone();
        }
        
        let mut reasoning = Vec::new();
        let mut alternatives = Vec::new();
        
        // Step 1: Try explicit encoding from PDF
        if let Some(info) = pdf_info {
            if let Some(ref encoding_name) = info.encoding_name {
                if let Some(encoding) = FontEncoding::from_pdf_name(encoding_name) {
                    reasoning.push(format!("PDF specifies encoding: {}", encoding_name));
                    
                    let detection = EncodingDetection {
                        encoding,
                        confidence: DetectionConfidence::Certain,
                        reasoning,
                        alternatives,
                    };
                    
                    self.detection_cache.insert(glyph_ids.to_vec(), detection.clone());
                    return detection;
                }
                
                reasoning.push(format!("Unknown encoding name: {}", encoding_name));
            }
        }
        
        // Step 2: Analyze font structure
        let font_type = self.analyze_font_structure(&mut reasoning);
        
        // Step 3: Pattern matching
        let patterns = self.pattern_matcher.find_patterns(glyph_ids);
        let (pattern_encoding, pattern_confidence) = self.evaluate_patterns(
            &patterns,
            font_type,
            &mut reasoning,
        );
        
        // Step 4: Statistical analysis
        let stats = GlyphStatistics::from_glyph_ids(glyph_ids);
        let (stat_encoding, stat_confidence) = self.evaluate_statistics(
            &stats,
            &mut reasoning,
        );
        
        // Step 5: Check for CJK characteristics
        if let Some(cjk_encoding) = self.detect_cjk_encoding(
            glyph_ids,
            pdf_info,
            &stats,
            &mut reasoning,
        ) {
            alternatives.push(pattern_encoding.clone());
            alternatives.push(stat_encoding.clone());
            
            let detection = EncodingDetection {
                encoding: cjk_encoding,
                confidence: DetectionConfidence::High,
                reasoning,
                alternatives,
            };
            
            self.detection_cache.insert(glyph_ids.to_vec(), detection.clone());
            return detection;
        }
        
        // Step 6: Choose best detection
        let (encoding, confidence) = if pattern_confidence >= stat_confidence {
            reasoning.push("Using pattern-based detection".to_string());
            alternatives.push(stat_encoding);
            (pattern_encoding, pattern_confidence)
        } else {
            reasoning.push("Using statistics-based detection".to_string());
            alternatives.push(pattern_encoding);
            (stat_encoding, stat_confidence)
        };
        
        // Step 7: Fallback if confidence too low
        let (final_encoding, final_confidence) = if confidence == DetectionConfidence::Low {
            reasoning.push("Low confidence - falling back to Identity-H".to_string());
            alternatives.insert(0, encoding);
            (FontEncoding::Identity { vertical: false }, DetectionConfidence::Low)
        } else {
            (encoding, confidence)
        };
        
        let detection = EncodingDetection {
            encoding: final_encoding,
            confidence: final_confidence,
            reasoning,
            alternatives,
        };
        
        self.detection_cache.insert(glyph_ids.to_vec(), detection.clone());
        detection
    }
    
    fn analyze_font_structure(&self, reasoning: &mut Vec<String>) -> FontType {
        use crate::tag;
        
        if self.font_provider.has_table(tag::CFF) {
            reasoning.push("Font has CFF table".to_string());
            
            // Check if it's CID-keyed CFF
            if let Ok(is_cid) = is_cff_cid_font(&self.font_provider) {
                if is_cid {
                    reasoning.push("CFF is CID-keyed".to_string());
                    return FontType::CidCFF;
                }
            }
            return FontType::SimpleCFF;
        }
        
        reasoning.push("Font is TrueType".to_string());
        FontType::TrueType
    }
    
    fn detect_cjk_encoding(
        &self,
        glyph_ids: &[u16],
        pdf_info: Option<&PdfFontInfo>,
        stats: &GlyphStatistics,
        reasoning: &mut Vec<String>,
    ) -> Option<FontEncoding> {
        // Check for CJK patterns
        if stats.has_cjk_range() {
            reasoning.push("Detected CJK glyph ranges".to_string());
            
            // Try to determine specific CJK language
            if stats.has_hangul_range() {
                reasoning.push("Detected Hangul syllables - Korean font".to_string());
                return Some(FontEncoding::CJK {
                    language: CJKLanguage::Korean(KoreanVariant::Unicode),
                    encoding_name: "UniKS-UTF16-H".to_string(),
                    vertical: false,
                    requires_cmap_data: false,
                });
            }
            
            if stats.has_kana_range() {
                reasoning.push("Detected Kana characters - Japanese font".to_string());
                return Some(FontEncoding::CJK {
                    language: CJKLanguage::Japanese(JapaneseVariant::Unicode),
                    encoding_name: "UniJIS-UTF16-H".to_string(),
                    vertical: false,
                    requires_cmap_data: false,
                });
            }
            
            // Default to Chinese if has CJK but not specific
            reasoning.push("CJK without specific markers - assuming Chinese".to_string());
            return Some(FontEncoding::CJK {
                language: CJKLanguage::Chinese(ChineseVariant::Simplified),
                encoding_name: "UniGB-UCS2-H".to_string(),
                vertical: false,
                requires_cmap_data: false,
            });
        }
        
        None
    }
}

#[derive(Debug, Clone, PartialEq)]
enum FontType {
    TrueType,
    SimpleCFF,
    CidCFF,
}
```

#### 3.2.2 Pattern Matching (`src/subset/detection/patterns.rs`)

```rust
use std::collections::HashSet;

pub struct PatternMatcher {
    known_patterns: Vec<Pattern>,
}

#[derive(Debug, Clone)]
struct Pattern {
    name: String,
    glyph_ids: HashSet<u16>,
    min_match_ratio: f32,
    encoding_hint: FontEncoding,
}

impl PatternMatcher {
    pub fn new() -> Self {
        let mut patterns = Vec::new();
        
        // Identity-H pattern (sparse high IDs)
        patterns.push(Pattern {
            name: "Identity-H Sparse".to_string(),
            glyph_ids: [143, 159, 178].iter().copied().collect(),
            min_match_ratio: 0.6,
            encoding_hint: FontEncoding::Identity { vertical: false },
        });
        
        // CJK dense pattern
        patterns.push(Pattern {
            name: "CJK Dense".to_string(),
            glyph_ids: (1000..2000).collect(),
            min_match_ratio: 0.3,
            encoding_hint: FontEncoding::CJK {
                language: CJKLanguage::Chinese(ChineseVariant::Simplified),
                encoding_name: "GB-EUC-H".to_string(),
                vertical: false,
                requires_cmap_data: true,
            },
        });
        
        // Symbol font pattern
        patterns.push(Pattern {
            name: "Symbol Font".to_string(),
            glyph_ids: (256..512).collect(),
            min_match_ratio: 0.4,
            encoding_hint: FontEncoding::Identity { vertical: false },
        });
        
        PatternMatcher { known_patterns: patterns }
    }
    
    pub fn find_patterns(&self, glyph_ids: &[u16]) -> Vec<PatternMatch> {
        let glyph_set: HashSet<u16> = glyph_ids.iter().copied().collect();
        let mut matches = Vec::new();
        
        for pattern in &self.known_patterns {
            let intersection = pattern.glyph_ids.intersection(&glyph_set).count();
            let match_ratio = intersection as f32 / pattern.glyph_ids.len() as f32;
            
            if match_ratio >= pattern.min_match_ratio {
                matches.push(PatternMatch {
                    pattern_name: pattern.name.clone(),
                    match_ratio,
                    encoding: pattern.encoding_hint.clone(),
                });
            }
        }
        
        matches.sort_by(|a, b| b.match_ratio.partial_cmp(&a.match_ratio).unwrap());
        matches
    }
}

#[derive(Debug, Clone)]
pub struct PatternMatch {
    pub pattern_name: String,
    pub match_ratio: f32,
    pub encoding: FontEncoding,
}

impl EncodingDetector {
    fn evaluate_patterns(
        &self,
        patterns: &[PatternMatch],
        font_type: FontType,
        reasoning: &mut Vec<String>,
    ) -> (FontEncoding, DetectionConfidence) {
        if patterns.is_empty() {
            reasoning.push("No patterns matched".to_string());
            return (
                FontEncoding::Identity { vertical: false },
                DetectionConfidence::Low,
            );
        }
        
        let best = &patterns[0];
        reasoning.push(format!(
            "Matched pattern '{}' with {:.0}% confidence",
            best.pattern_name,
            best.match_ratio * 100.0
        ));
        
        let confidence = if best.match_ratio >= 0.8 {
            DetectionConfidence::High
        } else if best.match_ratio >= 0.5 {
            DetectionConfidence::Medium
        } else {
            DetectionConfidence::Low
        };
        
        (best.encoding.clone(), confidence)
    }
}
```

#### 3.2.3 Statistical Analysis (`src/subset/detection/statistics.rs`)

```rust
pub struct GlyphStatistics {
    pub total_glyphs: usize,
    pub min_gid: u16,
    pub max_gid: u16,
    pub density: f32,
    pub has_ascii: bool,
    pub has_latin_extended: bool,
    pub has_cjk: bool,
    pub has_kana: bool,
    pub has_hangul: bool,
    pub sequential_runs: Vec<SequentialRun>,
    pub gap_distribution: HashMap<u16, usize>,
}

#[derive(Debug, Clone)]
pub struct SequentialRun {
    pub start: u16,
    pub length: usize,
}

impl GlyphStatistics {
    pub fn from_glyph_ids(glyph_ids: &[u16]) -> Self {
        let mut sorted = glyph_ids.to_vec();
        sorted.sort_unstable();
        sorted.dedup();
        
        let total_glyphs = sorted.len();
        let min_gid = *sorted.first().unwrap_or(&0);
        let max_gid = *sorted.last().unwrap_or(&0);
        
        // Calculate density
        let range = (max_gid - min_gid) as usize + 1;
        let density = if range > 0 {
            total_glyphs as f32 / range as f32
        } else {
            0.0
        };
        
        // Check ranges
        let has_ascii = sorted.iter().any(|&g| g > 0 && g < 128);
        let has_latin_extended = sorted.iter().any(|&g| g >= 128 && g < 256);
        let has_cjk = sorted.iter().any(|&g| g >= 0x4E00 && g <= 0x9FFF);
        let has_kana = sorted.iter().any(|&g| {
            (g >= 0x3040 && g <= 0x309F) || // Hiragana
            (g >= 0x30A0 && g <= 0x30FF)    // Katakana
        });
        let has_hangul = sorted.iter().any(|&g| g >= 0xAC00 && g <= 0xD7AF);
        
        // Find sequential runs
        let sequential_runs = Self::find_sequential_runs(&sorted);
        
        // Calculate gap distribution
        let gap_distribution = Self::calculate_gaps(&sorted);
        
        GlyphStatistics {
            total_glyphs,
            min_gid,
            max_gid,
            density,
            has_ascii,
            has_latin_extended,
            has_cjk,
            has_kana,
            has_hangul,
            sequential_runs,
            gap_distribution,
        }
    }
    
    fn find_sequential_runs(sorted: &[u16]) -> Vec<SequentialRun> {
        let mut runs = Vec::new();
        if sorted.is_empty() {
            return runs;
        }
        
        let mut start = sorted[0];
        let mut length = 1;
        
        for window in sorted.windows(2) {
            if window[1] == window[0] + 1 {
                length += 1;
            } else {
                if length >= 3 {
                    runs.push(SequentialRun { start, length });
                }
                start = window[1];
                length = 1;
            }
        }
        
        if length >= 3 {
            runs.push(SequentialRun { start, length });
        }
        
        runs
    }
    
    fn calculate_gaps(sorted: &[u16]) -> HashMap<u16, usize> {
        let mut gaps = HashMap::new();
        
        for window in sorted.windows(2) {
            let gap = window[1] - window[0] - 1;
            if gap > 0 {
                *gaps.entry(gap).or_insert(0) += 1;
            }
        }
        
        gaps
    }
    
    pub fn has_cjk_range(&self) -> bool {
        self.has_cjk || self.has_kana || self.has_hangul
    }
    
    pub fn has_kana_range(&self) -> bool {
        self.has_kana
    }
    
    pub fn has_hangul_range(&self) -> bool {
        self.has_hangul
    }
    
    pub fn is_dense(&self) -> bool {
        self.density > 0.7
    }
    
    pub fn is_sparse(&self) -> bool {
        self.density < 0.3
    }
    
    pub fn has_large_sequential_runs(&self) -> bool {
        self.sequential_runs.iter().any(|r| r.length > 100)
    }
}

impl EncodingDetector {
    fn evaluate_statistics(
        &self,
        stats: &GlyphStatistics,
        reasoning: &mut Vec<String>,
    ) -> (FontEncoding, DetectionConfidence) {
        // Dense CJK pattern
        if stats.total_glyphs > 500 && stats.is_dense() && stats.has_large_sequential_runs() {
            reasoning.push(format!(
                "Dense font with {} glyphs and {:.0}% density",
                stats.total_glyphs,
                stats.density * 100.0
            ));
            
            if stats.has_cjk_range() {
                return (
                    FontEncoding::Identity { vertical: false },
                    DetectionConfidence::High,
                );
            }
        }
        
        // Sparse pattern
        if stats.is_sparse() && stats.max_gid > 256 {
            reasoning.push(format!(
                "Sparse font with max GID {} and {:.0}% density",
                stats.max_gid,
                stats.density * 100.0
            ));
            
            return (
                FontEncoding::Identity { vertical: false },
                DetectionConfidence::Medium,
            );
        }
        
        // Default
        reasoning.push("No clear statistical pattern".to_string());
        (
            FontEncoding::Identity { vertical: false },
            DetectionConfidence::Low,
        )
    }
}
```

#### 3.2.4 Auto-Configuration Builder (`src/subset/auto/builder.rs`)

```rust
use crate::subset::detection::{EncodingDetector, PdfFontInfo};
use crate::subset::pdf::{PdfFontContext, PdfSubsetResult};

/// Auto-configuring subset builder
pub struct AutoSubsetBuilder<'a> {
    provider: &'a dyn FontTableProvider,
    glyph_ids: Vec<u16>,
    pdf_info: Option<PdfFontInfo>,
    detector: Option<EncodingDetector>,
    override_encoding: Option<FontEncoding>,
    min_confidence: DetectionConfidence,
}

impl<'a> AutoSubsetBuilder<'a> {
    pub fn new(provider: &'a dyn FontTableProvider) -> Self {
        AutoSubsetBuilder {
            provider,
            glyph_ids: vec![0],
            pdf_info: None,
            detector: None,
            override_encoding: None,
            min_confidence: DetectionConfidence::Medium,
        }
    }
    
    /// Add glyphs to subset
    pub fn with_glyphs(mut self, glyph_ids: &[u16]) -> Self {
        for &gid in glyph_ids {
            if gid != 0 && !self.glyph_ids.contains(&gid) {
                self.glyph_ids.push(gid);
            }
        }
        self
    }
    
    /// Provide PDF font information for better detection
    pub fn with_pdf_info(mut self, info: PdfFontInfo) -> Self {
        self.pdf_info = Some(info);
        self
    }
    
    /// Use custom detector
    pub fn with_detector(mut self, detector: EncodingDetector) -> Self {
        self.detector = Some(detector);
        self
    }
    
    /// Override auto-detection
    pub fn override_encoding(mut self, encoding: FontEncoding) -> Self {
        self.override_encoding = Some(encoding);
        self
    }
    
    /// Set minimum confidence for auto-detection
    pub fn min_confidence(mut self, confidence: DetectionConfidence) -> Self {
        self.min_confidence = confidence;
        self
    }
    
    /// Build with auto-detection
    pub fn build(self) -> Result<AutoSubsetResult, SubsetError> {
        // Use override if provided
        let encoding = if let Some(enc) = self.override_encoding {
            EncodingDetection {
                encoding: enc.clone(),
                confidence: DetectionConfidence::Certain,
                reasoning: vec!["Manual override".to_string()],
                alternatives: vec![],
            }
        } else {
            // Auto-detect
            let mut detector = self.detector.unwrap_or_else(|| {
                EncodingDetector::new(Box::new(self.provider.clone()))
            });
            
            detector.detect(&self.glyph_ids, self.pdf_info.as_ref())
        };
        
        // Check confidence threshold
        if encoding.confidence < self.min_confidence {
            return Err(SubsetError::InvalidContext(format!(
                "Detection confidence too low: {:?} < {:?}",
                encoding.confidence,
                self.min_confidence
            )));
        }
        
        // Create PDF context from detection
        let pdf_context = PdfFontContext {
            encoding: encoding.encoding.clone(),
            max_cid: None,
            preserve_identity: false,
            is_symbolic: self.pdf_info
                .as_ref()
                .map(|i| i.flags & 0x04 != 0)
                .unwrap_or(false),
        };
        
        // Perform subsetting
        let subset_result = subset_and_map_for_pdf(
            self.provider,
            &self.glyph_ids,
            pdf_context,
        )?;
        
        Ok(AutoSubsetResult {
            subset_result,
            detection: encoding,
        })
    }
}

/// Result with detection information
pub struct AutoSubsetResult {
    pub subset_result: PdfSubsetResult,
    pub detection: EncodingDetection,
}

impl AutoSubsetResult {
    /// Get detection confidence
    pub fn confidence(&self) -> &DetectionConfidence {
        &self.detection.confidence
    }
    
    /// Get detection reasoning
    pub fn reasoning(&self) -> &[String] {
        &self.detection.reasoning
    }
    
    /// Get alternative encodings
    pub fn alternatives(&self) -> &[FontEncoding] {
        &self.detection.alternatives
    }
}

/// Convenience function for auto-detection
pub fn auto_subset_for_pdf(provider: &dyn FontTableProvider) -> AutoSubsetBuilder {
    AutoSubsetBuilder::new(provider)
}
```

### 3.3 Usage Examples

```rust
// Example 1: Fully automatic
let result = auto_subset_for_pdf(&provider)
    .with_glyphs(&glyph_ids)
    .build()?;

println!("Detected: {:?} with {:?} confidence", 
         result.detection.encoding,
         result.confidence());

// Example 2: With PDF info for better detection
let pdf_info = PdfFontInfo {
    encoding_name: Some("Identity-H".to_string()),
    font_name: Some("NotoSansCJK-Regular".to_string()),
    flags: 0x04,  // Symbolic
    registry: Some("Adobe".to_string()),
    ordering: Some("GB1".to_string()),
    supplement: Some(5),
    to_unicode: None,
};

let result = auto_subset_for_pdf(&provider)
    .with_glyphs(&glyph_ids)
    .with_pdf_info(pdf_info)
    .build()?;

// Example 3: With confidence threshold
let result = auto_subset_for_pdf(&provider)
    .with_glyphs(&glyph_ids)
    .min_confidence(DetectionConfidence::High)
    .build()
    .unwrap_or_else(|_| {
        // Fall back to manual if confidence too low
        subset_for_pdf(&provider)
            .with_glyphs(&glyph_ids)
            .identity_h()
            .build()
    })?;

// Example 4: Debug detection
let result = auto_subset_for_pdf(&provider)
    .with_glyphs(&glyph_ids)
    .build()?;

if result.confidence() <= &DetectionConfidence::Medium {
    eprintln!("Detection reasoning:");
    for reason in result.reasoning() {
        eprintln!("  - {}", reason);
    }
    eprintln!("Alternatives considered:");
    for alt in result.alternatives() {
        eprintln!("  - {:?}", alt);
    }
}
```

### 3.4 Testing Strategy

```rust
#[test]
fn test_identity_detection() {
    let glyph_ids = vec![0, 143, 159, 178];
    let mut detector = EncodingDetector::new(create_test_provider());
    
    let detection = detector.detect(&glyph_ids, None);
    
    assert_eq!(detection.encoding, FontEncoding::Identity { vertical: false });
    assert!(detection.confidence >= DetectionConfidence::Medium);
    assert!(detection.reasoning.iter().any(|r| r.contains("pattern")));
}

#[test]
fn test_cjk_detection() {
    let glyph_ids: Vec<u16> = (0..5000).collect();
    let mut detector = EncodingDetector::new(create_cjk_provider());
    
    let detection = detector.detect(&glyph_ids, None);
    
    assert!(matches!(detection.encoding, FontEncoding::CJK { .. }));
    assert_eq!(detection.confidence, DetectionConfidence::High);
}

#[test]
fn test_pdf_info_override() {
    let pdf_info = PdfFontInfo {
        encoding_name: Some("90ms-RKSJ-H".to_string()),
        ..Default::default()
    };
    
    let mut detector = EncodingDetector::new(create_test_provider());
    let detection = detector.detect(&[0, 1, 2], Some(&pdf_info));
    
    assert_eq!(detection.confidence, DetectionConfidence::Certain);
    assert!(matches!(detection.encoding, 
                    FontEncoding::CJK { language: CJKLanguage::Japanese(_), .. }));
}

#[test]
fn test_auto_builder() {
    let result = auto_subset_for_pdf(&create_test_provider())
        .with_glyphs(&[0, 143, 159])
        .min_confidence(DetectionConfidence::Low)
        .build()
        .unwrap();
    
    assert!(result.subset_result.is_cid_font());
    assert!(!result.reasoning().is_empty());
}
```

## 4. Success Criteria

### 4.1 Detection Accuracy
- ✅ 100% accuracy for explicit PDF encodings
- ✅ >90% accuracy for Identity-H/V patterns
- ✅ >80% accuracy for CJK fonts
- ✅ >70% accuracy for symbolic fonts

### 4.2 Performance
- Detection time < 1ms for typical fonts
- Cache hit rate > 80% for repeated patterns
- Memory usage < 100KB for cache

### 4.3 Usability
- Zero-configuration works for 80% of cases
- Clear reasoning for debugging
- Easy override mechanism

## 5. Dependencies

- Phase 1: Core encoding types
- Phase 2: PDF context API
- Phase 3: CJK encoding support

## 6. Estimated Timeline

| Task | Duration | Notes |
|------|----------|-------|
| Detector core | 4 hours | Main detection logic |
| Pattern matching | 3 hours | Known patterns |
| Statistical analysis | 3 hours | Glyph statistics |
| CJK detection | 2 hours | Language-specific |
| Auto-builder | 3 hours | Convenience API |
| Caching | 2 hours | Performance |
| Testing | 3 hours | All scenarios |
| Documentation | 2 hours | Usage guide |
| **Total** | **22 hours** | ~3 days |

## 7. Future Enhancements

This phase enables:
- Phase 5: Advanced pattern learning
- Machine learning integration
- User-provided pattern training
- Cloud-based encoding database

---

*Document Version: 1.0*  
*Created: 2024-08-21*  
*Phase: 4 of 5*  
*Priority: MEDIUM - Improves usability*