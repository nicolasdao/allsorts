Read @README.md and the @docs/subsetting.md to understand this project and its current subsetting capabilities and APIs and then read @specs/250821_subset_and_map_with_context/README.md to understand the general context of the current upcoming changes and then implement the new changes below:

# Phase 5: Advanced Pattern Detection (TDD Implementation)

## Executive Summary

Implement advanced pattern detection algorithms to handle edge cases and improve detection accuracy. This phase adds machine learning-inspired techniques, entropy analysis, and adaptive pattern learning to handle previously unseen font patterns.

**Prerequisites:** Phases 1-4 must be complete with all tests passing.

## TDD Implementation Steps

This phase follows Test-Driven Development methodology:

1. **Preparation:** Verify test health and review dependencies
2. **Analysis & Planning:** Design advanced pattern detection architecture
3. **Define APIs & Signatures:** Draft interfaces without implementation
4. **Write Unit Tests (TDD):** Create comprehensive failing tests first
5. **Implement Functions Incrementally:** Minimal implementation to pass tests
6. **Full-Suite Integration:** Run all tests and fix regressions
7. **Documentation & Review:** Update docs and conduct peer review

---

## 1. Preparation

### 1.1 Verify Dependencies

**Prerequisites Check:**
```bash
# Verify Phases 1-4 are complete
cargo test subset::detection::basic
cargo test subset::detection::context  
cargo test subset::optimization
cargo test subset::validation
```

All tests must pass before proceeding.

### 1.2 Review Documentation

- Read Phase 1-4 specifications for context
- Review `src/subset/detection/mod.rs` architecture
- Understand existing encoding detection flow

---

## 2. Analysis & Planning

### 2.1 High-Level Design

**Goal:** Add advanced pattern detection with ML-inspired techniques for edge cases:
- Re-subset font detection using entropy analysis
- Symbolic font clustering using DBSCAN-inspired algorithms  
- Adaptive pattern learning with persistence
- Mixed script and variable font detection

**Affected Modules:**
- `src/subset/advanced/` (new module)
- `src/subset/detection/mod.rs` (integration)
- Test suite expansion

### 2.2 Break Down Into Tasks

| Component | Purpose | Dependencies |
|-----------|---------|--------------|
| EntropyAnalyzer | Re-subset detection via gap entropy | GlyphStatistics |
| ClusterDetector | DBSCAN clustering for patterns | None |
| SequenceAnalyzer | CJK/structured font patterns | StateMachine |
| PatternLearner | Adaptive learning with persistence | serde |
| AdvancedDetector | Integration with base detection | All above |

### 2.3 Edge Cases & Constraints

**Re-subset Detection:**
- Fonts already subset multiple times
- Non-sequential glyph IDs
- High entropy gaps between glyphs
- Mixed original font types

**Symbolic Fonts:**
- Corporate/custom symbol mappings
- Non-standard Unicode ranges
- Clustered glyph patterns
- Small glyph counts

**Performance Constraints:**
- Advanced analysis < 5ms
- Pattern learning overhead < 10%
- Memory usage < 1MB for patterns

---

## 3. Define APIs & Signatures

### 3.1 Core Interfaces

| Component | Method | Inputs | Outputs | Error Cases |
|-----------|--------|--------|---------|-------------|
| `EntropyAnalyzer::analyze_resubset_probability` | `&[u16]` | `ResubsetAnalysis` | Invalid glyph data |
| `ClusterDetector::detect_clusters` | `&[u16]` | `Vec<GlyphCluster>` | Empty input |
| `PatternLearner::predict` | `&FeatureVector` | `Option<PredictionResult>` | No patterns learned |
| `PatternLearner::learn_success` | `FeatureVector, FontEncoding` | `()` | Persistence failure |
| `AdvancedDetector::detect_advanced` | `&[u16], Option<&PdfFontInfo>` | `AdvancedDetectionResult` | All detection failures |

### 3.2 Data Structures

```rust
// Core analysis structures - signatures only
pub struct AdvancedPatternAnalyzer;
pub struct EntropyAnalyzer;
pub struct ClusterDetector;
pub struct SequenceAnalyzer;
pub struct PatternLearner;

// Results and analysis
pub struct ResubsetAnalysis;
pub struct GlyphCluster;
pub struct FeatureVector;
pub struct LearnedPattern;
pub struct AdvancedDetectionResult;

// Pattern types
pub enum AdvancedPatternType;
pub enum ClusterPatternType;
pub enum ResubsetIndicator;
```

---

## 4. Write Unit Tests (TDD)

### 4.1 Test File Structure

```
tests/subset/advanced/
├── entropy_tests.rs           # Entropy analysis tests
├── clustering_tests.rs        # Cluster detection tests  
├── learning_tests.rs          # Pattern learning tests
├── integration_tests.rs       # Advanced detector tests
└── fixtures/
    ├── resubset_fonts.json    # Test data for re-subset fonts
    ├── symbolic_fonts.json    # Test data for symbolic fonts
    └── learned_patterns.json  # Test pattern data
```

### 4.2 Entropy Analysis Tests (WILL FAIL)

```rust
// tests/subset/advanced/entropy_tests.rs

use crate::subset::advanced::entropy::{EntropyAnalyzer, ResubsetIndicator};

#[test]
fn test_high_entropy_indicates_resubset() {
    // WILL FAIL - EntropyAnalyzer not implemented yet
    let analyzer = EntropyAnalyzer::new();
    
    // Re-subset font pattern: irregular gaps, non-sequential
    let resubset_glyphs = vec![0, 15, 23, 45, 78, 156, 234, 456, 789, 1234];
    
    let analysis = analyzer.analyze_resubset_probability(&resubset_glyphs);
    
    assert!(analysis.probability > 0.7, "Should detect high re-subset probability");
    assert!(analysis.gap_entropy > 3.5, "Should have high entropy");
    assert!(analysis.indicators.contains(&ResubsetIndicator::IrregularGaps));
}

#[test]
fn test_low_entropy_indicates_original_font() {
    // WILL FAIL - EntropyAnalyzer not implemented yet
    let analyzer = EntropyAnalyzer::new();
    
    // Original font pattern: sequential, low gaps
    let original_glyphs = (0..100).collect::<Vec<u16>>();
    
    let analysis = analyzer.analyze_resubset_probability(&original_glyphs);
    
    assert!(analysis.probability < 0.3, "Should detect low re-subset probability");
    assert!(analysis.gap_entropy < 2.0, "Should have low entropy");
}

#[test]
fn test_missing_basic_latin_indicator() {
    // WILL FAIL - EntropyAnalyzer not implemented yet
    let analyzer = EntropyAnalyzer::new();
    
    // CJK-only font: no ASCII range
    let cjk_glyphs = (0x4E00..0x4E20).collect::<Vec<u16>>();
    
    let analysis = analyzer.analyze_resubset_probability(&cjk_glyphs);
    
    assert!(analysis.indicators.contains(&ResubsetIndicator::MissingBasicLatin));
}

#[test]
fn test_non_sequential_notdef_indicator() {
    // WILL FAIL - EntropyAnalyzer not implemented yet
    let analyzer = EntropyAnalyzer::new();
    
    // .notdef at 0, then jump to high GIDs
    let glyphs = vec![0, 500, 501, 502, 503];
    
    let analysis = analyzer.analyze_resubset_probability(&glyphs);
    
    assert!(analysis.indicators.contains(&ResubsetIndicator::NonSequentialNotDef));
}

#[test]
fn test_entropy_calculation_empty_input() {
    // WILL FAIL - EntropyAnalyzer not implemented yet
    let analyzer = EntropyAnalyzer::new();
    
    let analysis = analyzer.analyze_resubset_probability(&[]);
    
    assert_eq!(analysis.gap_entropy, 0.0);
    assert_eq!(analysis.probability, 0.0);
}

#[test]
fn test_entropy_calculation_single_glyph() {
    // WILL FAIL - EntropyAnalyzer not implemented yet
    let analyzer = EntropyAnalyzer::new();
    
    let analysis = analyzer.analyze_resubset_probability(&[42]);
    
    assert_eq!(analysis.gap_entropy, 0.0);
    assert!(analysis.probability < 0.5);
}

#[test]
fn test_original_font_type_estimation() {
    // WILL FAIL - EntropyAnalyzer not implemented yet
    let analyzer = EntropyAnalyzer::new();
    
    // Test CJK estimation
    let cjk_glyphs = vec![0, 1, 2, 10000, 10001, 15000];
    let analysis = analyzer.analyze_resubset_probability(&cjk_glyphs);
    assert_eq!(analysis.original_font_estimate.font_type, FontTypeEstimate::CJK);
    
    // Test symbolic estimation  
    let symbol_glyphs = vec![0, 256, 300, 400, 450];
    let analysis = analyzer.analyze_resubset_probability(&symbol_glyphs);
    assert_eq!(analysis.original_font_estimate.font_type, FontTypeEstimate::Symbolic);
    
    // Test Latin estimation
    let latin_glyphs = vec![0, 32, 65, 90, 122, 200];
    let analysis = analyzer.analyze_resubset_probability(&latin_glyphs);
    assert_eq!(analysis.original_font_estimate.font_type, FontTypeEstimate::Latin);
}
```

### 4.3 Cluster Detection Tests (WILL FAIL)

```rust
// tests/subset/advanced/clustering_tests.rs

use crate::subset::advanced::clustering::{ClusterDetector, ClusterPatternType};

#[test]
fn test_sequential_cluster_detection() {
    // WILL FAIL - ClusterDetector not implemented yet
    let detector = ClusterDetector::new();
    
    // Sequential Latin cluster
    let glyphs = (32..127).collect::<Vec<u16>>();
    
    let clusters = detector.detect_clusters(&glyphs);
    
    assert_eq!(clusters.len(), 1);
    assert_eq!(clusters[0].pattern_type, ClusterPatternType::Sequential);
    assert!(clusters[0].density > 0.8);
}

#[test]
fn test_symbolic_cluster_detection() {
    // WILL FAIL - ClusterDetector not implemented yet
    let detector = ClusterDetector::new();
    
    // Symbolic font cluster in private use area
    let glyphs = vec![256, 258, 260, 262, 264, 300, 302, 304];
    
    let clusters = detector.detect_clusters(&glyphs);
    
    assert_eq!(clusters.len(), 1);
    assert_eq!(clusters[0].pattern_type, ClusterPatternType::Symbolic);
    assert!(clusters[0].center >= 256 && clusters[0].center <= 1000);
}

#[test]
fn test_cjk_cluster_detection() {
    // WILL FAIL - ClusterDetector not implemented yet
    let detector = ClusterDetector::new();
    
    // Large CJK cluster
    let glyphs = (0x4E00..0x4F00).collect::<Vec<u16>>();
    
    let clusters = detector.detect_clusters(&glyphs);
    
    assert_eq!(clusters.len(), 1);
    assert_eq!(clusters[0].pattern_type, ClusterPatternType::CJKLike);
    assert!(clusters[0].glyphs.len() > 100);
}

#[test]
fn test_multiple_clusters() {
    // WILL FAIL - ClusterDetector not implemented yet
    let detector = ClusterDetector::new();
    
    // Mixed clusters: ASCII + symbols + CJK
    let mut glyphs = (32..127).collect::<Vec<u16>>();  // ASCII
    glyphs.extend(300..350);                           // Symbols
    glyphs.extend(0x4E00..0x4E20);                    // CJK
    
    let clusters = detector.detect_clusters(&glyphs);
    
    assert!(clusters.len() >= 2);
    
    let has_sequential = clusters.iter().any(|c| c.pattern_type == ClusterPatternType::Sequential);
    let has_cjk = clusters.iter().any(|c| c.pattern_type == ClusterPatternType::CJKLike);
    
    assert!(has_sequential);
    assert!(has_cjk);
}

#[test]
fn test_sparse_cluster_detection() {
    // WILL FAIL - ClusterDetector not implemented yet
    let detector = ClusterDetector::new();
    
    // Sparse, irregular pattern
    let glyphs = vec![0, 50, 200, 500, 1000, 2000];
    
    let clusters = detector.detect_clusters(&glyphs);
    
    // Should either form sparse clusters or be treated as noise
    if !clusters.is_empty() {
        assert!(clusters.iter().any(|c| c.pattern_type == ClusterPatternType::Sparse));
    }
}

#[test]
fn test_empty_input_clustering() {
    // WILL FAIL - ClusterDetector not implemented yet
    let detector = ClusterDetector::new();
    
    let clusters = detector.detect_clusters(&[]);
    
    assert!(clusters.is_empty());
}

#[test]
fn test_single_glyph_clustering() {
    // WILL FAIL - ClusterDetector not implemented yet
    let detector = ClusterDetector::new();
    
    let clusters = detector.detect_clusters(&[42]);
    
    // Single point should not form cluster (min_points = 3)
    assert!(clusters.is_empty());
}

#[test]
fn test_cluster_density_calculation() {
    // WILL FAIL - ClusterDetector not implemented yet
    let detector = ClusterDetector::new();
    
    // Dense cluster: consecutive glyphs
    let dense_glyphs = (100..110).collect::<Vec<u16>>();
    let dense_clusters = detector.detect_clusters(&dense_glyphs);
    
    // Sparse cluster: spread out glyphs
    let sparse_glyphs = vec![100, 102, 104, 106, 108, 150, 152, 154];
    let sparse_clusters = detector.detect_clusters(&sparse_glyphs);
    
    if !dense_clusters.is_empty() && !sparse_clusters.is_empty() {
        assert!(dense_clusters[0].density > sparse_clusters[0].density);
    }
}
```

### 4.4 Pattern Learning Tests (WILL FAIL)

```rust
// tests/subset/advanced/learning_tests.rs

use crate::subset::advanced::learning::{PatternLearner, FeatureVector};
use crate::subset::detection::FontEncoding;
use std::path::Path;

#[test]
fn test_pattern_learning_success() {
    // WILL FAIL - PatternLearner not implemented yet
    let mut learner = PatternLearner::new();
    
    let features = FeatureVector {
        glyph_count: 50,
        density: 0.8,
        max_gid: 200,
        gap_entropy: 2.5,
        cluster_count: 1,
        has_cjk: false,
        has_latin: true,
        has_symbolic: false,
        sequential_ratio: 0.9,
    };
    
    let encoding = FontEncoding::WinAnsiEncoding;
    
    learner.learn_success(features.clone(), encoding.clone());
    
    let prediction = learner.predict(&features);
    assert!(prediction.is_some());
    
    let result = prediction.unwrap();
    assert_eq!(result.encoding, encoding);
    assert!(result.confidence > 0.5);
}

#[test]
fn test_pattern_learning_failure() {
    // WILL FAIL - PatternLearner not implemented yet
    let mut learner = PatternLearner::new();
    
    let features = FeatureVector {
        glyph_count: 50,
        density: 0.8,
        max_gid: 200,
        gap_entropy: 2.5,
        cluster_count: 1,
        has_cjk: false,
        has_latin: true,
        has_symbolic: false,
        sequential_ratio: 0.9,
    };
    
    // Learn success first
    learner.learn_success(features.clone(), FontEncoding::WinAnsiEncoding);
    let initial_prediction = learner.predict(&features).unwrap();
    let initial_confidence = initial_prediction.confidence;
    
    // Then learn failure
    learner.learn_failure(features.clone());
    
    let updated_prediction = learner.predict(&features);
    if let Some(result) = updated_prediction {
        assert!(result.confidence < initial_confidence);
    }
}

#[test]
fn test_pattern_similarity_matching() {
    // WILL FAIL - PatternLearner not implemented yet
    let mut learner = PatternLearner::new();
    
    let base_features = FeatureVector {
        glyph_count: 100,
        density: 0.7,
        max_gid: 500,
        gap_entropy: 3.0,
        cluster_count: 2,
        has_cjk: false,
        has_latin: true,
        has_symbolic: true,
        sequential_ratio: 0.6,
    };
    
    learner.learn_success(base_features.clone(), FontEncoding::MacRomanEncoding);
    
    // Similar features should match
    let similar_features = FeatureVector {
        glyph_count: 105,  // Close to 100
        density: 0.72,     // Close to 0.7
        max_gid: 520,      // Close to 500
        gap_entropy: 3.1,  // Close to 3.0
        cluster_count: 2,  // Exact match
        has_cjk: false,    // Exact match
        has_latin: true,   // Exact match
        has_symbolic: true, // Exact match
        sequential_ratio: 0.62, // Close to 0.6
    };
    
    let prediction = learner.predict(&similar_features);
    assert!(prediction.is_some());
    assert!(prediction.unwrap().confidence > 0.7);
    
    // Very different features should not match
    let different_features = FeatureVector {
        glyph_count: 10,
        density: 0.1,
        max_gid: 50,
        gap_entropy: 1.0,
        cluster_count: 0,
        has_cjk: true,
        has_latin: false,
        has_symbolic: false,
        sequential_ratio: 0.1,
    };
    
    let no_prediction = learner.predict(&different_features);
    assert!(no_prediction.is_none() || no_prediction.unwrap().confidence < 0.5);
}

#[test]
fn test_pattern_persistence() {
    // WILL FAIL - PatternLearner not implemented yet
    let temp_file = "/tmp/test_patterns.json";
    
    {
        let mut learner = PatternLearner::new().with_persistence(temp_file);
        
        let features = FeatureVector {
            glyph_count: 75,
            density: 0.9,
            max_gid: 300,
            gap_entropy: 2.0,
            cluster_count: 1,
            has_cjk: false,
            has_latin: true,
            has_symbolic: false,
            sequential_ratio: 0.95,
        };
        
        learner.learn_success(features, FontEncoding::StandardEncoding);
    }
    
    // Load in new instance
    {
        let learner = PatternLearner::new().with_persistence(temp_file);
        
        let query_features = FeatureVector {
            glyph_count: 76,
            density: 0.88,
            max_gid: 305,
            gap_entropy: 2.1,
            cluster_count: 1,
            has_cjk: false,
            has_latin: true,
            has_symbolic: false,
            sequential_ratio: 0.93,
        };
        
        let prediction = learner.predict(&query_features);
        assert!(prediction.is_some());
        assert_eq!(prediction.unwrap().encoding, FontEncoding::StandardEncoding);
    }
    
    // Cleanup
    let _ = std::fs::remove_file(temp_file);
}

#[test]
fn test_pattern_confidence_decay() {
    // WILL FAIL - PatternLearner not implemented yet
    let mut learner = PatternLearner::new();
    
    let features = FeatureVector {
        glyph_count: 60,
        density: 0.6,
        max_gid: 400,
        gap_entropy: 2.8,
        cluster_count: 3,
        has_cjk: true,
        has_latin: false,
        has_symbolic: false,
        sequential_ratio: 0.4,
    };
    
    // Learn pattern
    learner.learn_success(features.clone(), FontEncoding::Identity);
    
    let initial = learner.predict(&features).unwrap();
    let initial_confidence = initial.confidence;
    
    // Simulate time passage (would need mocking in real implementation)
    // For now, just test that confidence calculation works
    assert!(initial_confidence > 0.5);
    
    // Multiple failures should reduce confidence
    for _ in 0..5 {
        learner.learn_failure(features.clone());
    }
    
    let after_failures = learner.predict(&features);
    assert!(after_failures.is_none() || after_failures.unwrap().confidence < initial_confidence);
}

#[test]
fn test_max_patterns_limit() {
    // WILL FAIL - PatternLearner not implemented yet
    let mut learner = PatternLearner::new();
    
    // Add patterns up to limit (1000)
    for i in 0..1005 {
        let features = FeatureVector {
            glyph_count: i,
            density: 0.5,
            max_gid: (i * 10) as u16,
            gap_entropy: 2.0,
            cluster_count: 1,
            has_cjk: false,
            has_latin: true,
            has_symbolic: false,
            sequential_ratio: 0.7,
        };
        
        learner.learn_success(features, FontEncoding::WinAnsiEncoding);
    }
    
    // Should not exceed max_patterns
    assert!(learner.learned_patterns.len() <= 1000);
}
```

### 4.5 Advanced Detection Integration Tests (WILL FAIL)

```rust
// tests/subset/advanced/integration_tests.rs

use crate::subset::advanced::{AdvancedDetector, AdvancedPatternType};
use crate::subset::detection::{DetectionConfidence, MockFontTableProvider};

#[test]
fn test_resubset_font_detection() {
    // WILL FAIL - AdvancedDetector not implemented yet
    let provider = Box::new(MockFontTableProvider::new());
    let mut detector = AdvancedDetector::new(provider);
    
    // Re-subset pattern: high entropy, irregular gaps
    let resubset_glyphs = vec![0, 15, 23, 45, 78, 156, 234, 456, 789, 1234];
    
    let result = detector.detect_advanced(&resubset_glyphs, None);
    
    if let Some(advanced_info) = result.advanced_info {
        assert_eq!(advanced_info.pattern_type, AdvancedPatternType::ResubsetFont);
    }
    assert!(result.confidence >= DetectionConfidence::Medium);
    assert!(result.reasoning.iter().any(|r| r.contains("re-subset") || r.contains("entropy")));
}

#[test]
fn test_symbolic_font_detection() {
    // WILL FAIL - AdvancedDetector not implemented yet
    let provider = Box::new(MockFontTableProvider::new());
    let mut detector = AdvancedDetector::new(provider);
    
    // Symbolic font pattern: clustered in symbol range
    let symbolic_glyphs = vec![0, 256, 258, 260, 300, 302, 304, 350, 352, 400];
    
    let result = detector.detect_advanced(&symbolic_glyphs, None);
    
    if let Some(advanced_info) = result.advanced_info {
        assert_eq!(advanced_info.pattern_type, AdvancedPatternType::CustomSymbolic);
    }
    assert!(result.confidence >= DetectionConfidence::Medium);
}

#[test]
fn test_mixed_script_detection() {
    // WILL FAIL - AdvancedDetector not implemented yet
    let provider = Box::new(MockFontTableProvider::new());
    let mut detector = AdvancedDetector::new(provider);
    
    // Mixed script: Latin + CJK
    let mut mixed_glyphs = (32..127).collect::<Vec<u16>>();  // ASCII
    mixed_glyphs.extend(0x4E00..0x4E20);                    // CJK
    
    let result = detector.detect_advanced(&mixed_glyphs, None);
    
    if let Some(advanced_info) = result.advanced_info {
        assert_eq!(advanced_info.pattern_type, AdvancedPatternType::MixedScript);
    }
    assert!(result.confidence >= DetectionConfidence::Medium);
}

#[test]
fn test_learned_pattern_matching() {
    // WILL FAIL - AdvancedDetector not implemented yet
    let provider = Box::new(MockFontTableProvider::new());
    let mut detector = AdvancedDetector::new(provider);
    
    // First, learn a pattern
    let training_glyphs = (32..100).collect::<Vec<u16>>();
    let training_result = detector.detect_advanced(&training_glyphs, None);
    
    // Simulate successful feedback
    detector.learn_success(&training_glyphs, &training_result.encoding);
    
    // Now test similar pattern
    let test_glyphs = (35..105).collect::<Vec<u16>>();
    let test_result = detector.detect_advanced(&test_glyphs, None);
    
    assert!(test_result.confidence >= DetectionConfidence::High);
    assert!(test_result.reasoning.iter().any(|r| r.contains("learned pattern")));
}

#[test]
fn test_fallback_to_base_detection() {
    // WILL FAIL - AdvancedDetector not implemented yet
    let provider = Box::new(MockFontTableProvider::new());
    let mut detector = AdvancedDetector::new(provider);
    
    // Standard pattern that advanced detection shouldn't claim
    let standard_glyphs = (32..127).collect::<Vec<u16>>();
    
    let result = detector.detect_advanced(&standard_glyphs, None);
    
    // Should fall back to base detection
    assert!(result.advanced_info.is_none() || 
            result.advanced_info.unwrap().pattern_type != AdvancedPatternType::ResubsetFont);
    assert!(result.confidence >= DetectionConfidence::Medium);
}

#[test]
fn test_performance_constraint() {
    // WILL FAIL - AdvancedDetector not implemented yet
    let provider = Box::new(MockFontTableProvider::new());
    let mut detector = AdvancedDetector::new(provider);
    
    // Large glyph set for performance testing
    let large_glyphs = (0..5000).collect::<Vec<u16>>();
    
    let start = std::time::Instant::now();
    let _result = detector.detect_advanced(&large_glyphs, None);
    let duration = start.elapsed();
    
    // Should complete in < 5ms
    assert!(duration.as_millis() < 5, "Advanced detection took too long: {:?}", duration);
}

#[test]
fn test_empty_input_handling() {
    // WILL FAIL - AdvancedDetector not implemented yet
    let provider = Box::new(MockFontTableProvider::new());
    let mut detector = AdvancedDetector::new(provider);
    
    let result = detector.detect_advanced(&[], None);
    
    // Should handle gracefully
    assert_eq!(result.confidence, DetectionConfidence::Low);
    assert!(result.advanced_info.is_none());
}
```

### 4.6 Feature Extraction Tests (WILL FAIL)

```rust
// tests/subset/advanced/feature_tests.rs

use crate::subset::advanced::{AdvancedDetector, FeatureVector};
use crate::subset::detection::MockFontTableProvider;

#[test]
fn test_feature_extraction_basic() {
    // WILL FAIL - AdvancedDetector not implemented yet
    let provider = Box::new(MockFontTableProvider::new());
    let detector = AdvancedDetector::new(provider);
    
    let glyphs = (32..127).collect::<Vec<u16>>();
    let features = detector.extract_features(&glyphs);
    
    assert_eq!(features.glyph_count, 95);
    assert!(features.has_latin);
    assert!(!features.has_cjk);
    assert!(!features.has_symbolic);
    assert!(features.sequential_ratio > 0.9);
    assert!(features.density > 0.8);
}

#[test] 
fn test_feature_extraction_cjk() {
    // WILL FAIL - AdvancedDetector not implemented yet
    let provider = Box::new(MockFontTableProvider::new());
    let detector = AdvancedDetector::new(provider);
    
    let glyphs = (0x4E00..0x4E50).collect::<Vec<u16>>();
    let features = detector.extract_features(&glyphs);
    
    assert_eq!(features.glyph_count, 80);
    assert!(!features.has_latin);
    assert!(features.has_cjk);
    assert!(!features.has_symbolic);
    assert!(features.max_gid >= 0x4E00);
}

#[test]
fn test_feature_extraction_symbolic() {
    // WILL FAIL - AdvancedDetector not implemented yet
    let provider = Box::new(MockFontTableProvider::new());
    let detector = AdvancedDetector::new(provider);
    
    let glyphs = vec![0, 256, 258, 260, 300, 302, 400];
    let features = detector.extract_features(&glyphs);
    
    assert_eq!(features.glyph_count, 7);
    assert!(!features.has_latin);
    assert!(!features.has_cjk);
    assert!(features.has_symbolic);
    assert!(features.density < 0.5);
}

#[test]
fn test_feature_extraction_mixed() {
    // WILL FAIL - AdvancedDetector not implemented yet
    let provider = Box::new(MockFontTableProvider::new());
    let detector = AdvancedDetector::new(provider);
    
    let mut glyphs = (32..95).collect::<Vec<u16>>();  // Latin
    glyphs.extend(256..280);                          // Symbols
    glyphs.extend(0x4E00..0x4E10);                   // CJK
    
    let features = detector.extract_features(&glyphs);
    
    assert!(features.has_latin);
    assert!(features.has_cjk);
    assert!(features.has_symbolic);
    assert!(features.cluster_count >= 2);
}
```

---

## 5. Implement Functions Incrementally

### 5.1 Implementation Order

**Phase 5A: Core Data Structures**
1. Define all structs and enums (signatures only)
2. Implement basic constructors
3. Run basic compilation tests

**Phase 5B: Entropy Analysis**
1. Implement `EntropyAnalyzer::new()`
2. Implement `calculate_gap_entropy()` 
3. Implement `analyze_resubset_probability()`
4. Run entropy tests until passing

**Phase 5C: Cluster Detection**
1. Implement `ClusterDetector::new()`
2. Implement `detect_clusters()` with basic DBSCAN
3. Implement cluster analysis
4. Run clustering tests until passing

**Phase 5D: Pattern Learning**
1. Implement `PatternLearner::new()`
2. Implement learning and prediction logic
3. Add persistence functionality
4. Run learning tests until passing

**Phase 5E: Integration**
1. Implement `AdvancedDetector::new()`
2. Implement `detect_advanced()`
3. Connect all components
4. Run integration tests until passing

---

## 6. Full-Suite Integration

### 6.1 Execute Entire Test Suite

After implementing each component incrementally:

```bash
# Run all Phase 5 tests
cargo test subset::advanced

# Run full subset detection suite  
cargo test subset

# Run entire test suite to check for regressions
cargo test
```

### 6.2 Fix Regressions

Address any test failures in:
- Existing basic detection (Phase 1)
- Context-aware detection (Phase 2)  
- Optimization (Phase 3)
- Validation (Phase 4)
- New advanced detection (Phase 5)

### 6.3 Performance Validation

```bash
# Run performance benchmarks
cargo bench subset_detection

# Profile memory usage
cargo run --example memory_profile

# Validate constraints:
# - Advanced analysis < 5ms
# - Pattern learning overhead < 10%
# - Memory for patterns < 1MB
```

---

## 7. Documentation & Review

### 7.1 Update Documentation

**API Documentation:**
```rust
/// Advanced pattern detection for edge cases and improved accuracy.
/// 
/// This module provides machine learning-inspired techniques for:
/// - Re-subset font detection using entropy analysis
/// - Symbolic font clustering with DBSCAN algorithms
/// - Adaptive pattern learning with persistence
/// - Mixed script and variable font detection
/// 
/// # Examples
/// 
/// ```rust
/// use allsorts::subset::advanced::AdvancedDetector;
/// 
/// let mut detector = AdvancedDetector::new(provider);
/// let result = detector.detect_advanced(&glyph_ids, None);
/// 
/// if result.is_resubset() {
///     println!("Detected re-subset font");
/// }
/// ```
pub mod advanced;
```

**User Guide Updates:**
- Add advanced detection section to README
- Document new pattern learning features
- Add troubleshooting for edge cases

**Changelog Entry:**
```markdown
## [Version] - 2024-08-21

### Added
- Advanced pattern detection for edge cases (Phase 5)
- Entropy-based re-subset font detection
- DBSCAN-inspired clustering for symbolic fonts  
- Adaptive pattern learning with persistence
- Mixed script and variable font detection
- 95%+ detection accuracy for complex cases
```

### 7.2 Code Comments

Ensure comprehensive documentation for:
- Public APIs with examples
- Complex algorithms (entropy, clustering)
- ML-inspired learning mechanisms
- Performance-critical sections
- Edge case handling

### 7.3 Peer Review Checklist

**Code Quality:**
- [ ] All tests passing
- [ ] Performance constraints met
- [ ] Memory usage within limits
- [ ] Error handling comprehensive
- [ ] Code follows project conventions

**Algorithm Correctness:**
- [ ] Entropy calculations mathematically sound
- [ ] DBSCAN clustering properly implemented
- [ ] Pattern learning convergence validated
- [ ] Feature extraction representative

**Integration:**
- [ ] Clean integration with base detection
- [ ] Proper fallback mechanisms
- [ ] Consistent API design
- [ ] Thread safety considered

---

## 8. Implementation Details (Reference)

### 8.1 File Structure

```
src/subset/advanced/
├── mod.rs                  (advanced module root)
├── entropy.rs              (entropy analysis)
├── clustering.rs           (cluster detection)
├── sequences.rs            (sequence analysis)
├── learning.rs             (pattern learning)
├── patterns/
│   ├── resubset.rs        (re-subset detection)
│   ├── symbolic.rs        (symbolic font patterns)
│   ├── mixed.rs           (mixed script detection)
│   └── variable.rs        (variable font detection)
└── optimization.rs         (optimization hints)
```

### 8.2 Advanced Detection Components

```rust
/// Advanced pattern analyzer with ML-inspired techniques
pub struct AdvancedPatternAnalyzer {
    entropy_analyzer: EntropyAnalyzer,
    cluster_detector: ClusterDetector,
    sequence_analyzer: SequenceAnalyzer,
    pattern_learner: PatternLearner,
}

pub enum AdvancedPatternType {
    ResubsetFont,          // Previously subset fonts
    CustomSymbolic,        // Corporate/custom symbol fonts
    MixedScript,          // Multi-language fonts
    VariableFont,         // Variable font instances
    LegacyEncoding,       // Old/proprietary encodings
    CompressedMapping,    // Fonts with compressed glyph mappings
    LearnedPattern,       // Pattern from ML learning
}

pub struct AdvancedDetectionResult {
    pub encoding: FontEncoding,
    pub confidence: DetectionConfidence,
    pub reasoning: Vec<String>,
    pub advanced_info: Option<AdvancedInfo>,
}

pub struct AdvancedInfo {
    pub pattern_type: AdvancedPatternType,
    pub characteristics: FeatureVector,
    pub optimization_hints: Vec<OptimizationHint>,
}
```

### 8.3 Usage Examples

```rust
// Example 1: Advanced detection with learning
let mut detector = AdvancedDetector::new(provider)
    .with_pattern_learning("patterns.json");

let result = detector.detect_advanced(&glyph_ids, None);

if result.is_resubset() {
    println!("Detected re-subset font with {:.0}% probability", 
             result.resubset_probability() * 100.0);
    println!("Estimated original: {:?}", result.original_font_estimate());
}

// Example 2: Cluster-based detection
let clusters = detector.analyze_clusters(&glyph_ids);
for cluster in clusters {
    println!("Cluster at GID {}: {} glyphs, pattern: {:?}",
             cluster.center, cluster.glyphs.len(), cluster.pattern_type);
}

// Example 3: Learning from feedback
if user_confirms_encoding(&result.encoding) {
    detector.learn_success(&glyph_ids, &result.encoding);
} else {
    detector.learn_failure(&glyph_ids);
}
```

### 8.4 Why This Phase

**Edge Case Coverage:**
- **Re-subset Fonts**: 5-10% of PDFs use already-subset fonts
- **Custom Symbols**: Corporate fonts with non-standard mappings
- **Mixed Scripts**: Documents with multiple languages
- **Variable Fonts**: Growing usage in modern PDFs

**Technical Excellence:**
- **Accuracy**: Push detection accuracy to >95%
- **Adaptability**: Learn from new patterns
- **Performance**: Optimize for repeated patterns
- **Robustness**: Handle malformed or unusual fonts

---

## 9. TDD Checklist for Phase 5

### 9.1 Preparation Phase
- [ ] All Phase 1-4 tests passing
- [ ] Dependencies reviewed and understood
- [ ] Architecture documentation read
- [ ] Test environment prepared

### 9.2 Planning Phase
- [ ] High-level design documented
- [ ] Components broken down into tasks
- [ ] Edge cases identified and catalogued
- [ ] Performance constraints defined
- [ ] API interfaces designed

### 9.3 Test Creation Phase
- [ ] Test file structure created
- [ ] Entropy analysis tests written (FAILING)
- [ ] Cluster detection tests written (FAILING)
- [ ] Pattern learning tests written (FAILING)
- [ ] Integration tests written (FAILING)
- [ ] Feature extraction tests written (FAILING)
- [ ] Performance tests written (FAILING)
- [ ] Edge case tests written (FAILING)

### 9.4 Implementation Phase
- [ ] Core data structures implemented
- [ ] EntropyAnalyzer implemented and tests passing
- [ ] ClusterDetector implemented and tests passing
- [ ] PatternLearner implemented and tests passing
- [ ] AdvancedDetector implemented and tests passing
- [ ] Integration with base detection working
- [ ] All component tests passing

### 9.5 Integration Phase
- [ ] All Phase 5 tests passing
- [ ] Full subset detection suite passing
- [ ] No regressions in existing phases
- [ ] Performance constraints met (<5ms analysis)
- [ ] Memory constraints met (<1MB patterns)
- [ ] Learning overhead acceptable (<10%)

### 9.6 Quality Assurance
- [ ] Code coverage >90% for new components
- [ ] All edge cases tested and handled
- [ ] Error handling comprehensive
- [ ] Thread safety verified
- [ ] API consistency maintained

### 9.7 Documentation Phase
- [ ] Public APIs documented with examples
- [ ] Complex algorithms explained
- [ ] User guide updated
- [ ] Changelog entry written
- [ ] Code comments comprehensive

### 9.8 Review Phase
- [ ] Peer review completed
- [ ] Review feedback addressed
- [ ] Performance benchmarks run
- [ ] Final testing completed
- [ ] Ready for merge

### 9.9 Success Criteria Verification

**Detection Improvements:**
- [ ] Re-subset detection accuracy > 85%
- [ ] Symbolic font detection > 90%
- [ ] Mixed script detection > 80%
- [ ] Overall accuracy improvement > 5%

**Performance:**
- [ ] Advanced analysis < 5ms
- [ ] Pattern learning overhead < 10%
- [ ] Memory for learned patterns < 1MB

**Adaptability:**
- [ ] Successfully learns from 100+ patterns
- [ ] Confidence improves over time
- [ ] Persistence across sessions works

### 9.10 Dependencies
- [ ] All previous phases (1-4) complete
- [ ] Optional: serde for pattern persistence
- [ ] Optional: statistical libraries

---

**Estimated Timeline: 25 hours (~3.5 days)**

| Task | Duration | TDD Steps |
|------|----------|----------|
| Planning & Design | 3 hours | Steps 1-2 |
| Test Creation | 6 hours | Step 3 |
| Core Implementation | 12 hours | Step 4 |
| Integration & Testing | 3 hours | Step 5 |
| Documentation | 1 hour | Steps 6-7 |

---

*Document Version: 2.0 (TDD)*  
*Updated: 2024-08-21*  
*Phase: 5 of 5*  
*Priority: LOW - Nice to have enhancements*  
*Methodology: Test-Driven Development*

## Appendix: Advanced Implementation Details

### A.1 Key Implementation Notes

This appendix contains reference implementation details to support the TDD process. The full implementations will be built incrementally following the test-driven approach outlined above.

**Entropy Analysis:** Shannon entropy calculation on glyph ID gaps to detect re-subset patterns.

**Cluster Detection:** DBSCAN-inspired algorithm for identifying glyph patterns and font types.

**Pattern Learning:** Adaptive learning system with feature vectors and confidence-based matching.

**Performance Considerations:** All advanced analysis optimized for <5ms execution time.

The comprehensive implementation details were moved to this appendix to maintain focus on the TDD methodology in the main document.