Read @README.md and the @docs/subsetting.md to understand this project and its current subsetting capabilities and APIs and then read @specs/250821_subset_and_map_with_context/README.md to understand the general context of the current upcoming changes and then implement the new changes below:

# Phase 4: Detection & Auto-Configuration (TDD Implementation)

## Executive Summary

Implement intelligent encoding detection and auto-configuration to reduce manual configuration burden. This phase adds the Detection Helper system that can automatically determine the correct encoding from PDF context and font characteristics.

**TDD Approach**: This phase follows Test-Driven Development methodology, writing failing tests first, then implementing minimal code to pass tests, ensuring high code quality and comprehensive test coverage.

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

## 2. TDD Implementation Steps

### 2.1 Preparation Phase
**Dependencies**: Phases 1-3 must be complete before starting Phase 4.

1. **Read Documentation**
   - Review Phase 1-3 implementations for `FontEncoding`, `PdfFontContext`, and CJK support
   - Understand existing test patterns and conventions
   - Verify integration points with core subsetting API

2. **Verify Test Health**
   ```bash
   cargo test --package allsorts-subset
   ```
   - All existing tests must pass before implementing Phase 4
   - Fix any failures in core subsetting functionality
   - Ensure CJK encoding tests from Phase 3 are green

### 2.2 Analysis & Planning
**High-Level Design**: Add intelligent encoding detection with confidence levels, pattern matching, statistical analysis, and auto-configuration builder.

**Components to Build**:
- `EncodingDetector` - Main detection engine
- `PatternMatcher` - Glyph pattern recognition
- `GlyphStatistics` - Statistical analysis
- `AutoSubsetBuilder` - Convenience API with auto-detection
- Caching system for performance

**Edge Cases to Handle**:
- Empty glyph sets
- Conflicting detection signals
- Unknown encoding names
- Very large glyph sets (>10k glyphs)
- Mixed-script fonts
- Malformed PDF font info
- Cache memory limits

### 2.3 Why This Phase

#### 2.3.1 User Experience Benefits
- **Reduces Errors**: Automatic detection prevents wrong encoding selection
- **Simplifies API**: Users don't need deep PDF/font knowledge
- **Faster Integration**: Less configuration required
- **Better Defaults**: Smart fallbacks for edge cases

#### 2.3.2 Technical Benefits
- **Caching**: Avoid repeated detection for same patterns
- **Confidence Levels**: Users know when to verify
- **Debugging**: Reasoning explains decisions
- **Flexibility**: Override auto-detection when needed

## 3. Define APIs & Signatures

Define interfaces without implementation. These signatures will guide our TDD process.

| Component | Key Methods | Inputs | Outputs | Error Cases |
|-----------|-------------|--------|---------|-------------|
| `EncodingDetector` | `detect()` | `&[u16]`, `Option<&PdfFontInfo>` | `EncodingDetection` | Invalid glyph IDs |
| `PatternMatcher` | `find_patterns()` | `&[u16]` | `Vec<PatternMatch>` | Empty glyph set |
| `GlyphStatistics` | `from_glyph_ids()` | `&[u16]` | `GlyphStatistics` | Empty input |
| `AutoSubsetBuilder` | `build()` | Various builder params | `Result<AutoSubsetResult, SubsetError>` | Low confidence, invalid glyphs |

### 3.1 Core Data Structures

```rust
/// Encoding detection result with confidence - SIGNATURE ONLY
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

/// Auto-detection helper - SIGNATURE ONLY
pub struct EncodingDetector {
    // Implementation details to be determined by tests
}

/// PDF font information for detection - SIGNATURE ONLY
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

## 4. Write Unit Tests (TDD) - FAILING TESTS FIRST

**IMPORTANT**: All tests below should be written BEFORE any implementation. They WILL FAIL initially - this is expected and correct TDD practice.

### 4.1 Test File Structure

Create test files under `src/subset/detection/tests/`:
- `detector_tests.rs` - Main detector tests
- `pattern_tests.rs` - Pattern matching tests  
- `statistics_tests.rs` - Statistical analysis tests
- `auto_builder_tests.rs` - Auto-builder tests
- `cache_tests.rs` - Caching behavior tests

### 4.2 Core Detection Tests

```rust
// WILL FAIL - detector_tests.rs
use super::*;

#[test]
fn test_explicit_pdf_encoding_detection() {
    // WILL FAIL until EncodingDetector::detect() is implemented
    let pdf_info = PdfFontInfo {
        encoding_name: Some("Identity-H".to_string()),
        font_name: None,
        flags: 0,
        registry: None,
        ordering: None,
        supplement: None,
        to_unicode: None,
    };
    
    let mut detector = EncodingDetector::new(create_test_provider());
    let detection = detector.detect(&[0, 1, 2], Some(&pdf_info));
    
    assert_eq!(detection.encoding, FontEncoding::Identity { vertical: false });
    assert_eq!(detection.confidence, DetectionConfidence::Certain);
    assert!(detection.reasoning.iter().any(|r| r.contains("PDF specifies encoding")));
}

#[test] 
fn test_unknown_pdf_encoding_fallback() {
    // WILL FAIL until fallback logic is implemented
    let pdf_info = PdfFontInfo {
        encoding_name: Some("UnknownEncoding".to_string()),
        ..Default::default()
    };
    
    let mut detector = EncodingDetector::new(create_test_provider());
    let detection = detector.detect(&[143, 159, 178], Some(&pdf_info));
    
    assert_ne!(detection.confidence, DetectionConfidence::Certain);
    assert!(detection.reasoning.iter().any(|r| r.contains("Unknown encoding name")));
}

#[test]
fn test_empty_glyph_set_handling() {
    // WILL FAIL until edge case handling is implemented
    let mut detector = EncodingDetector::new(create_test_provider());
    let detection = detector.detect(&[], None);
    
    assert_eq!(detection.confidence, DetectionConfidence::Low);
    assert_eq!(detection.encoding, FontEncoding::Identity { vertical: false });
}

#[test]
fn test_detection_caching() {
    // WILL FAIL until caching is implemented
    let glyph_ids = vec![0, 143, 159, 178];
    let mut detector = EncodingDetector::new(create_test_provider());
    
    let detection1 = detector.detect(&glyph_ids, None);
    let detection2 = detector.detect(&glyph_ids, None);
    
    // Should be identical (cached)
    assert_eq!(detection1.encoding, detection2.encoding);
    assert_eq!(detection1.confidence, detection2.confidence);
}
```

### 4.3 Pattern Matching Tests

```rust
// WILL FAIL - pattern_tests.rs
#[test]
fn test_identity_h_pattern_detection() {
    // WILL FAIL until PatternMatcher::find_patterns() is implemented
    let pattern_matcher = PatternMatcher::new();
    let glyph_ids = vec![0, 143, 159, 178]; // Sparse, high IDs
    
    let patterns = pattern_matcher.find_patterns(&glyph_ids);
    
    assert!(!patterns.is_empty());
    assert!(patterns[0].pattern_name.contains("Identity"));
    assert!(patterns[0].match_ratio > 0.5);
}

#[test]
fn test_cjk_dense_pattern_detection() {
    // WILL FAIL until CJK pattern logic is implemented
    let pattern_matcher = PatternMatcher::new();
    let glyph_ids: Vec<u16> = (1000..2000).collect(); // Dense CJK range
    
    let patterns = pattern_matcher.find_patterns(&glyph_ids);
    
    assert!(!patterns.is_empty());
    assert!(patterns[0].pattern_name.contains("CJK"));
    assert!(matches!(patterns[0].encoding, FontEncoding::CJK { .. }));
}

#[test]
fn test_no_pattern_match() {
    // WILL FAIL until no-match handling is implemented
    let pattern_matcher = PatternMatcher::new();
    let glyph_ids = vec![50, 51, 52]; // Should not match known patterns
    
    let patterns = pattern_matcher.find_patterns(&glyph_ids);
    
    // Should either be empty or have very low match ratios
    assert!(patterns.is_empty() || patterns[0].match_ratio < 0.3);
}

#[test]
fn test_multiple_pattern_ranking() {
    // WILL FAIL until pattern ranking is implemented
    let pattern_matcher = PatternMatcher::new();
    let glyph_ids = vec![0, 143, 300, 400, 1500]; // Mixed patterns
    
    let patterns = pattern_matcher.find_patterns(&glyph_ids);
    
    // Results should be sorted by match ratio (highest first)
    for window in patterns.windows(2) {
        assert!(window[0].match_ratio >= window[1].match_ratio);
    }
}
```

### 4.4 Statistical Analysis Tests

```rust
// WILL FAIL - statistics_tests.rs
#[test]
fn test_glyph_statistics_basic_calculations() {
    // WILL FAIL until GlyphStatistics::from_glyph_ids() is implemented
    let glyph_ids = vec![0, 5, 10, 15, 20];
    let stats = GlyphStatistics::from_glyph_ids(&glyph_ids);
    
    assert_eq!(stats.total_glyphs, 5);
    assert_eq!(stats.min_gid, 0);
    assert_eq!(stats.max_gid, 20);
    assert_eq!(stats.density, 5.0 / 21.0); // 5 glyphs in range 0-20
}

#[test]
fn test_cjk_range_detection() {
    // WILL FAIL until CJK range detection is implemented
    let glyph_ids = vec![0x4E00, 0x4E01, 0x9FFF]; // CJK Unified Ideographs
    let stats = GlyphStatistics::from_glyph_ids(&glyph_ids);
    
    assert!(stats.has_cjk);
    assert!(stats.has_cjk_range());
    assert!(!stats.has_kana);
    assert!(!stats.has_hangul);
}

#[test]
fn test_kana_detection() {
    // WILL FAIL until Kana detection is implemented
    let glyph_ids = vec![0x3042, 0x30A2]; // Hiragana あ and Katakana ア
    let stats = GlyphStatistics::from_glyph_ids(&glyph_ids);
    
    assert!(stats.has_kana);
    assert!(stats.has_kana_range());
    assert!(stats.has_cjk_range()); // Kana counts as CJK
}

#[test]
fn test_hangul_detection() {
    // WILL FAIL until Hangul detection is implemented
    let glyph_ids = vec![0xAC00, 0xD7AF]; // Hangul syllables
    let stats = GlyphStatistics::from_glyph_ids(&glyph_ids);
    
    assert!(stats.has_hangul);
    assert!(stats.has_hangul_range());
    assert!(stats.has_cjk_range()); // Hangul counts as CJK
}

#[test]
fn test_density_calculations() {
    // WILL FAIL until density calculations are implemented
    let dense_ids: Vec<u16> = (100..200).collect(); // 100 consecutive
    let dense_stats = GlyphStatistics::from_glyph_ids(&dense_ids);
    assert!(dense_stats.is_dense());
    assert!(!dense_stats.is_sparse());
    
    let sparse_ids = vec![0, 100, 500, 1000]; // 4 glyphs in range 0-1000
    let sparse_stats = GlyphStatistics::from_glyph_ids(&sparse_ids);
    assert!(sparse_stats.is_sparse());
    assert!(!sparse_stats.is_dense());
}

#[test]
fn test_sequential_run_detection() {
    // WILL FAIL until sequential run detection is implemented
    let glyph_ids = vec![100, 101, 102, 103, 104, 200, 201, 202];
    let stats = GlyphStatistics::from_glyph_ids(&glyph_ids);
    
    assert_eq!(stats.sequential_runs.len(), 2);
    assert_eq!(stats.sequential_runs[0].start, 100);
    assert_eq!(stats.sequential_runs[0].length, 5);
    assert_eq!(stats.sequential_runs[1].start, 200);
    assert_eq!(stats.sequential_runs[1].length, 3);
}

#[test]
fn test_large_sequential_runs() {
    // WILL FAIL until large run detection is implemented
    let large_run: Vec<u16> = (1000..1200).collect(); // 200 consecutive
    let stats = GlyphStatistics::from_glyph_ids(&large_run);
    
    assert!(stats.has_large_sequential_runs());
}
```

### 4.5 Auto-Builder Tests

```rust
// WILL FAIL - auto_builder_tests.rs
#[test]
fn test_auto_builder_basic_usage() {
    // WILL FAIL until AutoSubsetBuilder is implemented
    let result = auto_subset_for_pdf(&create_test_provider())
        .with_glyphs(&[0, 143, 159])
        .min_confidence(DetectionConfidence::Low)
        .build()
        .unwrap();
    
    assert!(result.subset_result.cid_to_gid_map.len() > 0);
    assert!(!result.reasoning().is_empty());
    assert!(result.confidence() >= &DetectionConfidence::Low);
}

#[test]
fn test_auto_builder_with_pdf_info() {
    // WILL FAIL until PDF info integration is implemented
    let pdf_info = PdfFontInfo {
        encoding_name: Some("Identity-H".to_string()),
        font_name: Some("TestFont".to_string()),
        flags: 0x04, // Symbolic
        ..Default::default()
    };
    
    let result = auto_subset_for_pdf(&create_test_provider())
        .with_glyphs(&[0, 1, 2])
        .with_pdf_info(pdf_info)
        .build()
        .unwrap();
    
    assert_eq!(result.detection.confidence, DetectionConfidence::Certain);
    assert!(result.reasoning().iter().any(|r| r.contains("PDF specifies")));
}

#[test]
fn test_auto_builder_confidence_threshold() {
    // WILL FAIL until confidence checking is implemented
    let result = auto_subset_for_pdf(&create_test_provider())
        .with_glyphs(&[1, 2, 3]) // Ambiguous pattern
        .min_confidence(DetectionConfidence::High)
        .build();
    
    // Should fail if detection confidence is below High
    assert!(result.is_err());
    assert!(result.unwrap_err().to_string().contains("confidence too low"));
}

#[test]
fn test_auto_builder_override_encoding() {
    // WILL FAIL until encoding override is implemented
    let override_encoding = FontEncoding::CJK {
        language: CJKLanguage::Japanese(JapaneseVariant::Unicode),
        encoding_name: "UniJIS-UTF16-H".to_string(),
        vertical: false,
        requires_cmap_data: false,
    };
    
    let result = auto_subset_for_pdf(&create_test_provider())
        .with_glyphs(&[0, 1, 2])
        .override_encoding(override_encoding.clone())
        .build()
        .unwrap();
    
    assert_eq!(result.detection.encoding, override_encoding);
    assert_eq!(result.detection.confidence, DetectionConfidence::Certain);
    assert!(result.reasoning().iter().any(|r| r.contains("Manual override")));
}

#[test]
fn test_auto_builder_glyph_deduplication() {
    // WILL FAIL until glyph deduplication is implemented
    let result = auto_subset_for_pdf(&create_test_provider())
        .with_glyphs(&[0, 1, 2])
        .with_glyphs(&[1, 2, 3]) // Overlapping glyphs
        .build()
        .unwrap();
    
    let expected_glyphs = [0, 1, 2, 3];
    assert_eq!(result.subset_result.cid_to_gid_map.len(), expected_glyphs.len());
}
```

### 4.6 Caching Behavior Tests

```rust
// WILL FAIL - cache_tests.rs
#[test]
fn test_cache_hit_for_identical_glyphs() {
    // WILL FAIL until caching is implemented
    let mut detector = EncodingDetector::new(create_test_provider());
    let glyph_ids = vec![0, 143, 159, 178];
    
    let start = std::time::Instant::now();
    let _detection1 = detector.detect(&glyph_ids, None);
    let first_duration = start.elapsed();
    
    let start = std::time::Instant::now();
    let _detection2 = detector.detect(&glyph_ids, None);
    let second_duration = start.elapsed();
    
    // Second call should be faster (cached)
    assert!(second_duration < first_duration);
}

#[test]
fn test_cache_miss_for_different_glyphs() {
    // WILL FAIL until cache key logic is implemented
    let mut detector = EncodingDetector::new(create_test_provider());
    
    let detection1 = detector.detect(&[0, 1, 2], None);
    let detection2 = detector.detect(&[3, 4, 5], None);
    
    // Different inputs should potentially give different results
    // (though they might be the same if both fall back to Identity-H)
    assert!(detection1.reasoning != detection2.reasoning);
}

#[test]
fn test_cache_with_different_pdf_info() {
    // WILL FAIL until PDF info affects caching is implemented
    let mut detector = EncodingDetector::new(create_test_provider());
    let glyph_ids = vec![0, 1, 2];
    
    let pdf_info1 = PdfFontInfo {
        encoding_name: Some("Identity-H".to_string()),
        ..Default::default()
    };
    
    let pdf_info2 = PdfFontInfo {
        encoding_name: Some("Identity-V".to_string()),
        ..Default::default()
    };
    
    let detection1 = detector.detect(&glyph_ids, Some(&pdf_info1));
    let detection2 = detector.detect(&glyph_ids, Some(&pdf_info2));
    
    // Should give different results despite same glyph IDs
    assert_ne!(detection1.encoding, detection2.encoding);
}
```

### 4.7 Integration Tests

```rust
// WILL FAIL - integration_tests.rs
#[test]
fn test_end_to_end_identity_h_detection() {
    // WILL FAIL until full pipeline is implemented
    let glyph_ids = vec![0, 143, 159, 178];
    let result = auto_subset_for_pdf(&create_test_provider())
        .with_glyphs(&glyph_ids)
        .build()
        .unwrap();
    
    assert_eq!(result.detection.encoding, FontEncoding::Identity { vertical: false });
    assert!(result.confidence() >= &DetectionConfidence::Medium);
    assert!(result.subset_result.is_cid_font());
}

#[test]
fn test_end_to_end_cjk_detection() {
    // WILL FAIL until CJK detection pipeline is implemented
    let cjk_glyph_ids: Vec<u16> = (0x4E00..0x4E50).collect();
    let result = auto_subset_for_pdf(&create_cjk_provider())
        .with_glyphs(&cjk_glyph_ids)
        .build()
        .unwrap();
    
    assert!(matches!(result.detection.encoding, FontEncoding::CJK { .. }));
    assert!(result.confidence() >= &DetectionConfidence::High);
}

#[test]
fn test_fallback_on_ambiguous_input() {
    // WILL FAIL until fallback logic is implemented
    let ambiguous_glyph_ids = vec![1, 2, 3]; // No clear pattern
    let result = auto_subset_for_pdf(&create_test_provider())
        .with_glyphs(&ambiguous_glyph_ids)
        .build()
        .unwrap();
    
    assert_eq!(result.detection.encoding, FontEncoding::Identity { vertical: false });
    assert!(result.reasoning().iter().any(|r| r.contains("falling back")));
}
```

## 5. Implement Functions Incrementally (TDD)

**CRITICAL**: Implementation should only begin AFTER all tests above are written and verified to fail correctly.

### 5.1 Implementation Strategy

For each component, follow this TDD cycle:
1. Pick one failing test
2. Write minimal code to make that test pass
3. Run the specific test: `cargo test test_name`
4. Refactor if needed
5. Move to next failing test

### 5.2 File Structure

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

### 5.3 Implementation Order (TDD Sequence)

**Step 1**: Start with basic data structures to make tests compile
**Step 2**: Implement `GlyphStatistics` (simplest component)
**Step 3**: Implement `PatternMatcher` 
**Step 4**: Implement core `EncodingDetector`
**Step 5**: Implement `AutoSubsetBuilder`
**Step 6**: Add caching and performance optimizations

### 5.4 Implementation Details

#### 5.4.1 Step 1: Basic Data Structures

First, make the tests compile by adding minimal struct definitions:

```rust
// src/subset/detection/mod.rs - START WITH THIS
pub struct EncodingDetector {
    // TODO: Add fields as tests demand them
}

impl EncodingDetector {
    pub fn new(_provider: Box<dyn FontTableProvider>) -> Self {
        todo!("Implement after tests are written")
    }
    
    pub fn detect(&mut self, _glyph_ids: &[u16], _pdf_info: Option<&PdfFontInfo>) -> EncodingDetection {
        todo!("Implement incrementally")
    }
}
```

#### 5.4.2 Main Detector Implementation (`src/subset/detection/detector.rs`)

**TDD Note**: Only implement pieces as the tests require them.

```rust
// Full implementation - only add after TDD cycle completes
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

#### 5.4.3 Pattern Matching Implementation (`src/subset/detection/patterns.rs`)\n\n**TDD Note**: Implement incrementally to satisfy pattern tests.

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

#### 5.4.4 Statistical Analysis Implementation (`src/subset/detection/statistics.rs`)\n\n**TDD Note**: Start here as it's the simplest component with clear test cases.

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

#### 5.4.5 Auto-Configuration Builder Implementation (`src/subset/auto/builder.rs`)\n\n**TDD Note**: Implement last as it depends on all other components.

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

## 6. Full-Suite Integration\n\nAfter implementing all components using TDD:\n\n### 6.1 Run Complete Test Suite\n\n```bash\n# Run all Phase 4 tests\ncargo test --package allsorts-subset detection\ncargo test --package allsorts-subset auto_builder\n\n# Run full integration tests\ncargo test --package allsorts-subset integration\n```\n\n### 6.2 Verify No Regressions\n\n```bash\n# Ensure all existing tests still pass\ncargo test --package allsorts-subset\n\n# Run tests for dependent phases\ncargo test --package allsorts-subset context  # Phase 2\ncargo test --package allsorts-subset cjk      # Phase 3\n```\n\n### 6.3 Performance Verification\n\n```bash\n# Run benchmark tests if available\ncargo bench --package allsorts-subset detection\n\n# Profile memory usage\ncargo test --package allsorts-subset cache_tests --release\n```\n\n## 7. Documentation & Review\n\n### 7.1 Usage Examples\n\n**Note**: Only write these AFTER implementation is complete and tests pass.

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

### 7.2 Update Documentation\n\n```rust\n// Update module documentation\n//! # Detection and Auto-Configuration\n//!\n//! This module provides intelligent encoding detection for PDF font subsetting.\n//! It analyzes glyph patterns, font structure, and PDF metadata to automatically\n//! determine the best encoding configuration.\n//!\n//! ## Usage\n//!\n//! ```rust\n//! use allsorts_subset::auto_subset_for_pdf;\n//!\n//! let result = auto_subset_for_pdf(&font_provider)\n//!     .with_glyphs(&glyph_ids)\n//!     .build()?;\n//!\n//! println!(\"Detected: {:?}\", result.detection.encoding);\n//! ```\n```\n\n### 7.3 Code Comments\n\nEnsure all public interfaces have comprehensive documentation:\n\n```rust\n/// Detects the most appropriate encoding for a set of glyph IDs.\n/// \n/// This function analyzes multiple signals:\n/// - Explicit PDF encoding information\n/// - Glyph ID patterns and distributions\n/// - Font table structure (CFF vs TrueType)\n/// - Statistical characteristics\n/// - CJK script detection\n///\n/// # Arguments\n/// * `glyph_ids` - The glyph IDs to analyze\n/// * `pdf_info` - Optional PDF font metadata\n///\n/// # Returns\n/// An `EncodingDetection` with the suggested encoding, confidence level,\n/// reasoning for the decision, and alternative encodings considered.\npub fn detect(&mut self, glyph_ids: &[u16], pdf_info: Option<&PdfFontInfo>) -> EncodingDetection\n```

## 8. Success Criteria & Verification

### 8.1 TDD Completion Verification

**All tests must pass before considering Phase 4 complete:**

```bash\n# Verify all detection tests pass\ncargo test test_explicit_pdf_encoding_detection -- --exact\ncargo test test_unknown_pdf_encoding_fallback -- --exact\ncargo test test_empty_glyph_set_handling -- --exact\ncargo test test_detection_caching -- --exact\n\n# Verify all pattern tests pass\ncargo test test_identity_h_pattern_detection -- --exact\ncargo test test_cjk_dense_pattern_detection -- --exact\ncargo test test_no_pattern_match -- --exact\ncargo test test_multiple_pattern_ranking -- --exact\n\n# Verify all statistics tests pass\ncargo test test_glyph_statistics_basic_calculations -- --exact\ncargo test test_cjk_range_detection -- --exact\ncargo test test_kana_detection -- --exact\ncargo test test_hangul_detection -- --exact\ncargo test test_density_calculations -- --exact\ncargo test test_sequential_run_detection -- --exact\ncargo test test_large_sequential_runs -- --exact\n\n# Verify all auto-builder tests pass\ncargo test test_auto_builder_basic_usage -- --exact\ncargo test test_auto_builder_with_pdf_info -- --exact\ncargo test test_auto_builder_confidence_threshold -- --exact\ncargo test test_auto_builder_override_encoding -- --exact\ncargo test test_auto_builder_glyph_deduplication -- --exact\n\n# Verify all caching tests pass\ncargo test test_cache_hit_for_identical_glyphs -- --exact\ncargo test test_cache_miss_for_different_glyphs -- --exact\ncargo test test_cache_with_different_pdf_info -- --exact\n\n# Verify all integration tests pass\ncargo test test_end_to_end_identity_h_detection -- --exact\ncargo test test_end_to_end_cjk_detection -- --exact\ncargo test test_fallback_on_ambiguous_input -- --exact\n```\n\n### 8.2 Detection Accuracy Targets\n- ✅ 100% accuracy for explicit PDF encodings\n- ✅ >90% accuracy for Identity-H/V patterns\n- ✅ >80% accuracy for CJK fonts\n- ✅ >70% accuracy for symbolic fonts\n\n### 8.3 Performance Targets\n- Detection time < 1ms for typical fonts\n- Cache hit rate > 80% for repeated patterns\n- Memory usage < 100KB for cache\n\n### 8.4 Usability Targets\n- Zero-configuration works for 80% of cases\n- Clear reasoning for debugging\n- Easy override mechanism

## 9. Dependencies

**CRITICAL**: These phases must be fully complete and tested before starting Phase 4:

- ✅ Phase 1: Core encoding types (`FontEncoding`, `CJKLanguage`, etc.)
- ✅ Phase 2: PDF context API (`PdfFontContext`, `subset_and_map_for_pdf`)
- ✅ Phase 3: CJK encoding support (all CJK variants implemented)

**Verification Command:**
```bash
cargo test --package allsorts-subset context cjk
```

## 10. TDD Implementation Timeline

| TDD Phase | Task | Duration | Verification |
|-----------|------|----------|--------------|
| **Preparation** | Read docs, verify test health | 0.5 hours | `cargo test` passes |
| **Planning** | Design analysis, edge cases | 1 hour | Documentation complete |
| **API Design** | Define signatures, interfaces | 1 hour | Tests compile (but fail) |
| **Write Tests** | All failing tests written | 4 hours | All tests fail as expected |
| **Basic Structs** | Make tests compile | 1 hour | Tests compile and fail |
| **Statistics** | Implement `GlyphStatistics` | 2 hours | Statistics tests pass |
| **Patterns** | Implement `PatternMatcher` | 2.5 hours | Pattern tests pass |
| **Detector Core** | Main detection logic | 3 hours | Core detection tests pass |
| **Auto-Builder** | Convenience API | 2 hours | Builder tests pass |
| **Caching** | Performance optimization | 1.5 hours | Cache tests pass |
| **Integration** | Full pipeline testing | 2 hours | Integration tests pass |
| **Documentation** | Comments, examples, docs | 1.5 hours | Doc tests pass |
| **Review** | Code review, final testing | 1 hour | All criteria met |
| **Total** | **TDD Implementation** | **22 hours** | **~3 days** |

## 11. TDD Checklist for Phase 4

### 11.1 Pre-Implementation Checklist

- [ ] **Preparation Complete**
  - [ ] Reviewed Phase 1-3 implementations
  - [ ] All existing tests pass: `cargo test --package allsorts-subset`
  - [ ] Understood integration points with core API

- [ ] **Analysis & Planning Complete**
  - [ ] Component responsibilities defined
  - [ ] Edge cases documented
  - [ ] File structure planned
  - [ ] Integration strategy clear

- [ ] **API Signatures Defined**
  - [ ] `EncodingDetector` interface designed
  - [ ] `PatternMatcher` interface designed
  - [ ] `GlyphStatistics` interface designed
  - [ ] `AutoSubsetBuilder` interface designed
  - [ ] Error handling strategy defined

### 11.2 TDD Implementation Checklist

- [ ] **Test Files Created**
  - [ ] `detector_tests.rs` with all core tests
  - [ ] `pattern_tests.rs` with pattern matching tests
  - [ ] `statistics_tests.rs` with statistical analysis tests
  - [ ] `auto_builder_tests.rs` with builder tests
  - [ ] `cache_tests.rs` with caching tests
  - [ ] `integration_tests.rs` with end-to-end tests

- [ ] **All Tests Written (FAILING)**
  - [ ] All 30+ test cases implemented
  - [ ] Tests compile but fail as expected
  - [ ] Test coverage includes all edge cases
  - [ ] Tests are isolated and focused

- [ ] **Incremental Implementation**
  - [ ] Basic data structures implemented (tests compile)
  - [ ] `GlyphStatistics` fully implemented (statistics tests pass)
  - [ ] `PatternMatcher` fully implemented (pattern tests pass)
  - [ ] `EncodingDetector` core implemented (detection tests pass)
  - [ ] `AutoSubsetBuilder` implemented (builder tests pass)
  - [ ] Caching implemented (cache tests pass)
  - [ ] Integration complete (integration tests pass)

### 11.3 Quality Verification Checklist

- [ ] **Test Suite Health**
  - [ ] All Phase 4 tests pass: `cargo test detection pattern statistics auto_builder cache integration`
  - [ ] No regressions: `cargo test --package allsorts-subset`
  - [ ] Performance tests pass
  - [ ] Memory usage within limits

- [ ] **Detection Accuracy Verification**
  - [ ] Explicit PDF encodings: 100% accuracy verified
  - [ ] Identity-H/V patterns: >90% accuracy verified
  - [ ] CJK fonts: >80% accuracy verified
  - [ ] Symbolic fonts: >70% accuracy verified

- [ ] **Code Quality**
  - [ ] All public APIs documented with examples
  - [ ] Error handling comprehensive
  - [ ] Performance requirements met
  - [ ] Memory usage requirements met

### 11.4 Integration Verification Checklist

- [ ] **Phase Integration**
  - [ ] Works with Phase 1 encoding types
  - [ ] Integrates with Phase 2 PDF context
  - [ ] Supports Phase 3 CJK encodings
  - [ ] Auto-detection improves user experience

- [ ] **API Integration**
  - [ ] `auto_subset_for_pdf()` function works
  - [ ] Builder pattern intuitive
  - [ ] Error messages helpful
  - [ ] Override mechanisms work

### 11.5 Documentation & Review Checklist

- [ ] **Documentation Complete**
  - [ ] Module-level documentation written
  - [ ] All public functions documented
  - [ ] Usage examples provided
  - [ ] Integration guide updated

- [ ] **Final Review**
  - [ ] Code review completed
  - [ ] Performance benchmarks satisfied
  - [ ] Memory profiling completed
  - [ ] Ready for Phase 5 dependencies

## 12. Future Enhancements

This TDD-implemented phase enables:
- Phase 5: Advanced pattern learning with ML
- User-provided pattern training datasets
- Cloud-based encoding database integration
- Continuous learning from detection results

---

*Document Version: 2.0 (TDD)*  
*Created: 2024-08-21*  
*Updated: 2024-08-21 (TDD Methodology)*  
*Phase: 4 of 5*  
*Methodology: Test-Driven Development*  
*Priority: MEDIUM - Improves usability through intelligent auto-detection*