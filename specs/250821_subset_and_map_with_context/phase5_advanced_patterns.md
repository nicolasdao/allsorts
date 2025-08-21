# Phase 5: Advanced Pattern Detection

## Executive Summary

Implement advanced pattern detection algorithms to handle edge cases and improve detection accuracy. This phase adds machine learning-inspired techniques, entropy analysis, and adaptive pattern learning to handle previously unseen font patterns.

## 1. What We're Building

### 1.1 Advanced Detection Components

```rust
/// Advanced pattern analyzer with ML-inspired techniques
pub struct AdvancedPatternAnalyzer {
    entropy_analyzer: EntropyAnalyzer,
    cluster_detector: ClusterDetector,
    sequence_analyzer: SequenceAnalyzer,
    pattern_learner: PatternLearner,
}

/// Entropy-based analysis for re-subset detection
pub struct EntropyAnalyzer {
    gap_entropy_threshold: f32,
    distribution_analyzer: DistributionAnalyzer,
}

/// Cluster detection for symbolic and custom fonts
pub struct ClusterDetector {
    dbscan: DBSCAN,
    cluster_profiles: Vec<ClusterProfile>,
}

/// Sequence analysis for CJK and structured fonts
pub struct SequenceAnalyzer {
    markov_chain: MarkovChain<u16>,
    sequence_patterns: Vec<SequencePattern>,
}

/// Adaptive pattern learning
pub struct PatternLearner {
    learned_patterns: HashMap<String, LearnedPattern>,
    confidence_threshold: f32,
    max_patterns: usize,
}
```

### 1.2 Detection Improvements

```rust
pub enum AdvancedPatternType {
    ResubsetFont,          // Previously subset fonts
    CustomSymbolic,        // Corporate/custom symbol fonts
    MixedScript,          // Multi-language fonts
    VariableFont,         // Variable font instances
    LegacyEncoding,       // Old/proprietary encodings
    CompressedMapping,    // Fonts with compressed glyph mappings
}

pub struct AdvancedDetectionResult {
    pub pattern_type: AdvancedPatternType,
    pub confidence: f32,
    pub characteristics: PatternCharacteristics,
    pub recommended_encoding: FontEncoding,
    pub optimization_hints: Vec<OptimizationHint>,
}
```

## 2. Why This Phase

### 2.1 Edge Case Coverage
- **Re-subset Fonts**: 5-10% of PDFs use already-subset fonts
- **Custom Symbols**: Corporate fonts with non-standard mappings
- **Mixed Scripts**: Documents with multiple languages
- **Variable Fonts**: Growing usage in modern PDFs

### 2.2 Technical Excellence
- **Accuracy**: Push detection accuracy to >95%
- **Adaptability**: Learn from new patterns
- **Performance**: Optimize for repeated patterns
- **Robustness**: Handle malformed or unusual fonts

## 3. Technical Implementation

### 3.1 File Structure

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

### 3.2 Implementation Details

#### 3.2.1 Entropy Analysis (`src/subset/advanced/entropy.rs`)

```rust
use std::collections::HashMap;

pub struct EntropyAnalyzer {
    gap_entropy_threshold: f32,
    distribution_analyzer: DistributionAnalyzer,
}

impl EntropyAnalyzer {
    pub fn new() -> Self {
        EntropyAnalyzer {
            gap_entropy_threshold: 3.5,
            distribution_analyzer: DistributionAnalyzer::new(),
        }
    }
    
    /// Analyze entropy to detect re-subset fonts
    pub fn analyze_resubset_probability(
        &self,
        glyph_ids: &[u16],
    ) -> ResubsetAnalysis {
        let mut sorted = glyph_ids.to_vec();
        sorted.sort_unstable();
        sorted.dedup();
        
        // Calculate gap entropy
        let gap_entropy = self.calculate_gap_entropy(&sorted);
        
        // Analyze distribution characteristics
        let distribution = self.distribution_analyzer.analyze(&sorted);
        
        // Check for re-subset indicators
        let indicators = self.find_resubset_indicators(&sorted, &distribution);
        
        // Calculate probability
        let probability = self.calculate_resubset_probability(
            gap_entropy,
            &distribution,
            &indicators,
        );
        
        ResubsetAnalysis {
            probability,
            gap_entropy,
            distribution_type: distribution.distribution_type,
            indicators,
            original_font_estimate: self.estimate_original_font(&sorted, &distribution),
        }
    }
    
    fn calculate_gap_entropy(&self, sorted_glyphs: &[u16]) -> f32 {
        if sorted_glyphs.len() < 2 {
            return 0.0;
        }
        
        // Calculate gaps between consecutive glyphs
        let mut gap_counts: HashMap<u16, usize> = HashMap::new();
        for window in sorted_glyphs.windows(2) {
            let gap = window[1] - window[0];
            *gap_counts.entry(gap).or_insert(0) += 1;
        }
        
        // Calculate Shannon entropy
        let total = sorted_glyphs.len() - 1;
        let mut entropy = 0.0;
        
        for count in gap_counts.values() {
            let p = *count as f32 / total as f32;
            entropy -= p * p.log2();
        }
        
        entropy
    }
    
    fn find_resubset_indicators(
        &self,
        sorted: &[u16],
        distribution: &Distribution,
    ) -> Vec<ResubsetIndicator> {
        let mut indicators = Vec::new();
        
        // Check for non-sequential .notdef
        if sorted[0] == 0 && sorted.len() > 1 && sorted[1] > 1 {
            indicators.push(ResubsetIndicator::NonSequentialNotDef);
        }
        
        // Check for irregular gaps
        if distribution.has_irregular_gaps() {
            indicators.push(ResubsetIndicator::IrregularGaps);
        }
        
        // Check for clustering around specific ranges
        if distribution.has_multiple_clusters() {
            indicators.push(ResubsetIndicator::MultipleClusters);
        }
        
        // Check for missing common glyphs
        let has_basic_latin = sorted.iter().any(|&g| g >= 32 && g <= 126);
        if !has_basic_latin && sorted.len() > 20 {
            indicators.push(ResubsetIndicator::MissingBasicLatin);
        }
        
        indicators
    }
    
    fn calculate_resubset_probability(
        &self,
        gap_entropy: f32,
        distribution: &Distribution,
        indicators: &[ResubsetIndicator],
    ) -> f32 {
        let mut probability = 0.0;
        
        // High entropy suggests re-subset
        if gap_entropy > self.gap_entropy_threshold {
            probability += 0.4;
        } else if gap_entropy > 2.5 {
            probability += 0.2;
        }
        
        // Distribution type
        match distribution.distribution_type {
            DistributionType::Random => probability += 0.3,
            DistributionType::Clustered => probability += 0.2,
            DistributionType::Uniform => probability -= 0.1,
            _ => {}
        }
        
        // Each indicator adds to probability
        probability += indicators.len() as f32 * 0.1;
        
        probability.min(1.0).max(0.0)
    }
    
    fn estimate_original_font(&self, sorted: &[u16], distribution: &Distribution) -> OriginalFontEstimate {
        // Estimate characteristics of original font
        let max_gid = *sorted.last().unwrap_or(&0);
        
        let estimated_type = if max_gid > 10000 {
            FontTypeEstimate::CJK
        } else if distribution.has_symbolic_pattern() {
            FontTypeEstimate::Symbolic
        } else {
            FontTypeEstimate::Latin
        };
        
        let estimated_glyphs = match estimated_type {
            FontTypeEstimate::CJK => 10000..30000,
            FontTypeEstimate::Symbolic => 200..500,
            FontTypeEstimate::Latin => 200..1000,
        };
        
        OriginalFontEstimate {
            font_type: estimated_type,
            estimated_glyph_range: estimated_glyphs,
            subset_ratio: sorted.len() as f32 / estimated_glyphs.start as f32,
        }
    }
}

#[derive(Debug, Clone)]
pub struct ResubsetAnalysis {
    pub probability: f32,
    pub gap_entropy: f32,
    pub distribution_type: DistributionType,
    pub indicators: Vec<ResubsetIndicator>,
    pub original_font_estimate: OriginalFontEstimate,
}

#[derive(Debug, Clone, PartialEq)]
pub enum ResubsetIndicator {
    NonSequentialNotDef,
    IrregularGaps,
    MultipleClusters,
    MissingBasicLatin,
    HighEntropy,
}

#[derive(Debug, Clone)]
pub struct OriginalFontEstimate {
    pub font_type: FontTypeEstimate,
    pub estimated_glyph_range: std::ops::Range<usize>,
    pub subset_ratio: f32,
}

#[derive(Debug, Clone, PartialEq)]
pub enum FontTypeEstimate {
    Latin,
    CJK,
    Symbolic,
}

pub struct DistributionAnalyzer {
    clustering_threshold: f32,
}

impl DistributionAnalyzer {
    pub fn new() -> Self {
        DistributionAnalyzer {
            clustering_threshold: 0.3,
        }
    }
    
    pub fn analyze(&self, sorted: &[u16]) -> Distribution {
        let distribution_type = self.determine_type(sorted);
        let clusters = self.find_clusters(sorted);
        let gap_stats = self.calculate_gap_statistics(sorted);
        
        Distribution {
            distribution_type,
            clusters,
            gap_stats,
        }
    }
    
    fn determine_type(&self, sorted: &[u16]) -> DistributionType {
        // Simplified distribution detection
        let gaps = self.calculate_gaps(sorted);
        let unique_gaps = gaps.iter().collect::<HashSet<_>>().len();
        let gap_ratio = unique_gaps as f32 / gaps.len() as f32;
        
        if gap_ratio > 0.8 {
            DistributionType::Random
        } else if gap_ratio < 0.2 {
            DistributionType::Uniform
        } else {
            DistributionType::Clustered
        }
    }
    
    fn find_clusters(&self, sorted: &[u16]) -> Vec<Cluster> {
        // Simplified clustering
        let mut clusters = Vec::new();
        if sorted.is_empty() {
            return clusters;
        }
        
        let mut current_cluster = Cluster {
            start: sorted[0],
            end: sorted[0],
            count: 1,
        };
        
        for &gid in &sorted[1..] {
            if gid - current_cluster.end <= 10 {
                current_cluster.end = gid;
                current_cluster.count += 1;
            } else {
                clusters.push(current_cluster);
                current_cluster = Cluster {
                    start: gid,
                    end: gid,
                    count: 1,
                };
            }
        }
        
        clusters.push(current_cluster);
        clusters
    }
    
    fn calculate_gaps(&self, sorted: &[u16]) -> Vec<u16> {
        sorted.windows(2)
            .map(|w| w[1] - w[0])
            .collect()
    }
    
    fn calculate_gap_statistics(&self, sorted: &[u16]) -> GapStatistics {
        let gaps = self.calculate_gaps(sorted);
        
        if gaps.is_empty() {
            return GapStatistics::default();
        }
        
        let mean = gaps.iter().sum::<u16>() as f32 / gaps.len() as f32;
        let variance = gaps.iter()
            .map(|&g| (g as f32 - mean).powi(2))
            .sum::<f32>() / gaps.len() as f32;
        let std_dev = variance.sqrt();
        
        GapStatistics {
            mean_gap: mean,
            std_dev,
            max_gap: *gaps.iter().max().unwrap_or(&0),
            min_gap: *gaps.iter().min().unwrap_or(&0),
        }
    }
}

#[derive(Debug, Clone)]
pub struct Distribution {
    pub distribution_type: DistributionType,
    pub clusters: Vec<Cluster>,
    pub gap_stats: GapStatistics,
}

impl Distribution {
    pub fn has_irregular_gaps(&self) -> bool {
        self.gap_stats.std_dev > self.gap_stats.mean_gap * 0.5
    }
    
    pub fn has_multiple_clusters(&self) -> bool {
        self.clusters.len() > 3
    }
    
    pub fn has_symbolic_pattern(&self) -> bool {
        self.clusters.iter().any(|c| c.start >= 256 && c.end <= 512)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub enum DistributionType {
    Uniform,
    Random,
    Clustered,
    Bimodal,
}

#[derive(Debug, Clone)]
pub struct Cluster {
    pub start: u16,
    pub end: u16,
    pub count: usize,
}

#[derive(Debug, Clone, Default)]
pub struct GapStatistics {
    pub mean_gap: f32,
    pub std_dev: f32,
    pub max_gap: u16,
    pub min_gap: u16,
}
```

#### 3.2.2 Cluster Detection (`src/subset/advanced/clustering.rs`)

```rust
/// DBSCAN-inspired clustering for glyph pattern detection
pub struct ClusterDetector {
    epsilon: f32,          // Maximum distance between points in cluster
    min_points: usize,     // Minimum points to form cluster
}

impl ClusterDetector {
    pub fn new() -> Self {
        ClusterDetector {
            epsilon: 50.0,    // GID distance threshold
            min_points: 3,    // Minimum cluster size
        }
    }
    
    pub fn detect_clusters(&self, glyph_ids: &[u16]) -> Vec<GlyphCluster> {
        let mut sorted = glyph_ids.to_vec();
        sorted.sort_unstable();
        sorted.dedup();
        
        let mut clusters = Vec::new();
        let mut visited = vec![false; sorted.len()];
        
        for (i, &gid) in sorted.iter().enumerate() {
            if visited[i] {
                continue;
            }
            
            let neighbors = self.find_neighbors(&sorted, i);
            
            if neighbors.len() >= self.min_points {
                let cluster = self.expand_cluster(&sorted, i, neighbors, &mut visited);
                clusters.push(cluster);
            } else {
                visited[i] = true;  // Mark as noise
            }
        }
        
        // Analyze cluster characteristics
        for cluster in &mut clusters {
            cluster.analyze_pattern();
        }
        
        clusters
    }
    
    fn find_neighbors(&self, sorted: &[u16], index: usize) -> Vec<usize> {
        let center = sorted[index];
        let mut neighbors = Vec::new();
        
        for (i, &gid) in sorted.iter().enumerate() {
            let distance = (gid as i32 - center as i32).abs() as f32;
            if distance <= self.epsilon {
                neighbors.push(i);
            }
        }
        
        neighbors
    }
    
    fn expand_cluster(
        &self,
        sorted: &[u16],
        start_index: usize,
        mut neighbors: Vec<usize>,
        visited: &mut [bool],
    ) -> GlyphCluster {
        let mut cluster_glyphs = vec![sorted[start_index]];
        visited[start_index] = true;
        
        let mut i = 0;
        while i < neighbors.len() {
            let neighbor_idx = neighbors[i];
            
            if !visited[neighbor_idx] {
                visited[neighbor_idx] = true;
                cluster_glyphs.push(sorted[neighbor_idx]);
                
                let new_neighbors = self.find_neighbors(sorted, neighbor_idx);
                if new_neighbors.len() >= self.min_points {
                    for n in new_neighbors {
                        if !neighbors.contains(&n) {
                            neighbors.push(n);
                        }
                    }
                }
            }
            
            i += 1;
        }
        
        GlyphCluster::new(cluster_glyphs)
    }
}

#[derive(Debug, Clone)]
pub struct GlyphCluster {
    pub glyphs: Vec<u16>,
    pub center: u16,
    pub density: f32,
    pub pattern_type: ClusterPatternType,
}

impl GlyphCluster {
    fn new(glyphs: Vec<u16>) -> Self {
        let center = glyphs.iter().sum::<u16>() / glyphs.len() as u16;
        let density = Self::calculate_density(&glyphs);
        
        GlyphCluster {
            glyphs,
            center,
            density,
            pattern_type: ClusterPatternType::Unknown,
        }
    }
    
    fn calculate_density(glyphs: &[u16]) -> f32 {
        if glyphs.len() < 2 {
            return 1.0;
        }
        
        let min = *glyphs.iter().min().unwrap();
        let max = *glyphs.iter().max().unwrap();
        let range = (max - min) as f32 + 1.0;
        
        glyphs.len() as f32 / range
    }
    
    fn analyze_pattern(&mut self) {
        self.pattern_type = if self.is_sequential() {
            ClusterPatternType::Sequential
        } else if self.is_symbolic() {
            ClusterPatternType::Symbolic
        } else if self.is_cjk_like() {
            ClusterPatternType::CJKLike
        } else {
            ClusterPatternType::Sparse
        };
    }
    
    fn is_sequential(&self) -> bool {
        self.density > 0.8
    }
    
    fn is_symbolic(&self) -> bool {
        self.center >= 256 && self.center <= 1000 && self.glyphs.len() < 100
    }
    
    fn is_cjk_like(&self) -> bool {
        self.center > 1000 && self.glyphs.len() > 100
    }
}

#[derive(Debug, Clone, PartialEq)]
pub enum ClusterPatternType {
    Sequential,
    Symbolic,
    CJKLike,
    Sparse,
    Unknown,
}
```

#### 3.2.3 Pattern Learning (`src/subset/advanced/learning.rs`)

```rust
use std::collections::HashMap;
use serde::{Serialize, Deserialize};

/// Adaptive pattern learning system
pub struct PatternLearner {
    learned_patterns: HashMap<String, LearnedPattern>,
    confidence_threshold: f32,
    max_patterns: usize,
    persistence_path: Option<PathBuf>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LearnedPattern {
    pub id: String,
    pub feature_vector: FeatureVector,
    pub encoding: FontEncoding,
    pub confidence: f32,
    pub success_count: usize,
    pub failure_count: usize,
    pub last_seen: SystemTime,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FeatureVector {
    pub glyph_count: usize,
    pub density: f32,
    pub max_gid: u16,
    pub gap_entropy: f32,
    pub cluster_count: usize,
    pub has_cjk: bool,
    pub has_latin: bool,
    pub has_symbolic: bool,
    pub sequential_ratio: f32,
}

impl PatternLearner {
    pub fn new() -> Self {
        PatternLearner {
            learned_patterns: HashMap::new(),
            confidence_threshold: 0.7,
            max_patterns: 1000,
            persistence_path: None,
        }
    }
    
    pub fn with_persistence<P: AsRef<Path>>(mut self, path: P) -> Self {
        self.persistence_path = Some(path.as_ref().to_path_buf());
        self.load_patterns();
        self
    }
    
    /// Learn from a successful detection
    pub fn learn_success(
        &mut self,
        features: FeatureVector,
        encoding: FontEncoding,
    ) {
        let pattern_id = self.generate_pattern_id(&features);
        
        if let Some(pattern) = self.learned_patterns.get_mut(&pattern_id) {
            pattern.success_count += 1;
            pattern.confidence = self.calculate_confidence(pattern);
            pattern.last_seen = SystemTime::now();
        } else if self.learned_patterns.len() < self.max_patterns {
            let pattern = LearnedPattern {
                id: pattern_id.clone(),
                feature_vector: features,
                encoding,
                confidence: 0.6,  // Initial confidence
                success_count: 1,
                failure_count: 0,
                last_seen: SystemTime::now(),
            };
            
            self.learned_patterns.insert(pattern_id, pattern);
        } else {
            // Replace least confident pattern
            self.replace_weakest_pattern(features, encoding);
        }
        
        self.save_patterns();
    }
    
    /// Learn from a failed detection
    pub fn learn_failure(&mut self, features: FeatureVector) {
        let pattern_id = self.generate_pattern_id(&features);
        
        if let Some(pattern) = self.learned_patterns.get_mut(&pattern_id) {
            pattern.failure_count += 1;
            pattern.confidence = self.calculate_confidence(pattern);
            pattern.last_seen = SystemTime::now();
            
            // Remove pattern if confidence too low
            if pattern.confidence < 0.3 {
                self.learned_patterns.remove(&pattern_id);
            }
        }
        
        self.save_patterns();
    }
    
    /// Predict encoding based on learned patterns
    pub fn predict(&self, features: &FeatureVector) -> Option<PredictionResult> {
        let mut best_match: Option<(&LearnedPattern, f32)> = None;
        
        for pattern in self.learned_patterns.values() {
            let similarity = self.calculate_similarity(features, &pattern.feature_vector);
            
            if similarity > 0.8 && pattern.confidence > self.confidence_threshold {
                if best_match.is_none() || similarity > best_match.as_ref().unwrap().1 {
                    best_match = Some((pattern, similarity));
                }
            }
        }
        
        best_match.map(|(pattern, similarity)| PredictionResult {
            encoding: pattern.encoding.clone(),
            confidence: pattern.confidence * similarity,
            pattern_id: pattern.id.clone(),
        })
    }
    
    fn generate_pattern_id(&self, features: &FeatureVector) -> String {
        // Create a hash-like ID from features
        format!(
            "P_{}_{}_{}_{}",
            features.glyph_count / 10 * 10,  // Round to nearest 10
            (features.density * 10.0) as u32,
            features.max_gid / 100 * 100,    // Round to nearest 100
            features.cluster_count
        )
    }
    
    fn calculate_confidence(&self, pattern: &LearnedPattern) -> f32 {
        let total = pattern.success_count + pattern.failure_count;
        if total == 0 {
            return 0.5;
        }
        
        let success_rate = pattern.success_count as f32 / total as f32;
        
        // Apply time decay
        let age = SystemTime::now()
            .duration_since(pattern.last_seen)
            .unwrap_or_default()
            .as_secs() as f32;
        
        let time_factor = 1.0 / (1.0 + age / 86400.0);  // Decay over days
        
        success_rate * time_factor
    }
    
    fn calculate_similarity(&self, a: &FeatureVector, b: &FeatureVector) -> f32 {
        let mut similarity = 0.0;
        let mut weight_sum = 0.0;
        
        // Weighted feature comparison
        let features = [
            (1.0 - (a.glyph_count as f32 - b.glyph_count as f32).abs() / 1000.0, 1.0),
            (1.0 - (a.density - b.density).abs(), 2.0),
            (1.0 - (a.max_gid as f32 - b.max_gid as f32).abs() / 10000.0, 1.0),
            (1.0 - (a.gap_entropy - b.gap_entropy).abs() / 5.0, 2.0),
            (if a.has_cjk == b.has_cjk { 1.0 } else { 0.0 }, 3.0),
            (if a.has_latin == b.has_latin { 1.0 } else { 0.0 }, 1.0),
            (if a.has_symbolic == b.has_symbolic { 1.0 } else { 0.0 }, 2.0),
        ];
        
        for (sim, weight) in features {
            similarity += sim * weight;
            weight_sum += weight;
        }
        
        similarity / weight_sum
    }
    
    fn replace_weakest_pattern(&mut self, features: FeatureVector, encoding: FontEncoding) {
        if let Some(weakest_id) = self.find_weakest_pattern() {
            self.learned_patterns.remove(&weakest_id);
            
            let pattern_id = self.generate_pattern_id(&features);
            let pattern = LearnedPattern {
                id: pattern_id.clone(),
                feature_vector: features,
                encoding,
                confidence: 0.6,
                success_count: 1,
                failure_count: 0,
                last_seen: SystemTime::now(),
            };
            
            self.learned_patterns.insert(pattern_id, pattern);
        }
    }
    
    fn find_weakest_pattern(&self) -> Option<String> {
        self.learned_patterns
            .iter()
            .min_by(|a, b| a.1.confidence.partial_cmp(&b.1.confidence).unwrap())
            .map(|(id, _)| id.clone())
    }
    
    fn load_patterns(&mut self) {
        if let Some(ref path) = self.persistence_path {
            if path.exists() {
                if let Ok(data) = std::fs::read(path) {
                    if let Ok(patterns) = serde_json::from_slice(&data) {
                        self.learned_patterns = patterns;
                    }
                }
            }
        }
    }
    
    fn save_patterns(&self) {
        if let Some(ref path) = self.persistence_path {
            if let Ok(data) = serde_json::to_vec_pretty(&self.learned_patterns) {
                let _ = std::fs::write(path, data);
            }
        }
    }
}

#[derive(Debug, Clone)]
pub struct PredictionResult {
    pub encoding: FontEncoding,
    pub confidence: f32,
    pub pattern_id: String,
}
```

#### 3.2.4 Integration with Main Detection (`src/subset/advanced/mod.rs`)

```rust
use crate::subset::detection::{EncodingDetector, EncodingDetection, DetectionConfidence};

pub struct AdvancedDetector {
    base_detector: EncodingDetector,
    entropy_analyzer: EntropyAnalyzer,
    cluster_detector: ClusterDetector,
    pattern_learner: PatternLearner,
}

impl AdvancedDetector {
    pub fn new(provider: Box<dyn FontTableProvider>) -> Self {
        AdvancedDetector {
            base_detector: EncodingDetector::new(provider),
            entropy_analyzer: EntropyAnalyzer::new(),
            cluster_detector: ClusterDetector::new(),
            pattern_learner: PatternLearner::new(),
        }
    }
    
    pub fn detect_advanced(
        &mut self,
        glyph_ids: &[u16],
        pdf_info: Option<&PdfFontInfo>,
    ) -> AdvancedDetectionResult {
        // Extract features
        let features = self.extract_features(glyph_ids);
        
        // Try learned patterns first
        if let Some(prediction) = self.pattern_learner.predict(&features) {
            if prediction.confidence > 0.8 {
                return AdvancedDetectionResult {
                    encoding: prediction.encoding,
                    confidence: DetectionConfidence::High,
                    reasoning: vec![format!("Matched learned pattern: {}", prediction.pattern_id)],
                    advanced_info: Some(AdvancedInfo {
                        pattern_type: AdvancedPatternType::LearnedPattern,
                        characteristics: features,
                        optimization_hints: vec![],
                    }),
                };
            }
        }
        
        // Check for re-subset
        let resubset_analysis = self.entropy_analyzer.analyze_resubset_probability(glyph_ids);
        if resubset_analysis.probability > 0.7 {
            return self.handle_resubset_font(glyph_ids, resubset_analysis);
        }
        
        // Analyze clusters
        let clusters = self.cluster_detector.detect_clusters(glyph_ids);
        if !clusters.is_empty() {
            if let Some(result) = self.analyze_clusters(&clusters, &features) {
                return result;
            }
        }
        
        // Fall back to base detection
        let base_result = self.base_detector.detect(glyph_ids, pdf_info);
        
        // Learn from result (would need user feedback in real implementation)
        if base_result.confidence >= DetectionConfidence::High {
            self.pattern_learner.learn_success(features, base_result.encoding.clone());
        }
        
        AdvancedDetectionResult {
            encoding: base_result.encoding,
            confidence: base_result.confidence,
            reasoning: base_result.reasoning,
            advanced_info: None,
        }
    }
    
    fn extract_features(&self, glyph_ids: &[u16]) -> FeatureVector {
        let stats = GlyphStatistics::from_glyph_ids(glyph_ids);
        let clusters = self.cluster_detector.detect_clusters(glyph_ids);
        let gap_entropy = self.entropy_analyzer.calculate_gap_entropy(
            &glyph_ids.iter().copied().collect::<BTreeSet<_>>()
                .into_iter().collect::<Vec<_>>()
        );
        
        FeatureVector {
            glyph_count: glyph_ids.len(),
            density: stats.density,
            max_gid: stats.max_gid,
            gap_entropy,
            cluster_count: clusters.len(),
            has_cjk: stats.has_cjk,
            has_latin: stats.has_ascii,
            has_symbolic: clusters.iter().any(|c| c.pattern_type == ClusterPatternType::Symbolic),
            sequential_ratio: stats.sequential_runs.iter()
                .map(|r| r.length)
                .sum::<usize>() as f32 / glyph_ids.len() as f32,
        }
    }
}
```

### 3.3 Usage Examples

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

// Example 3: Entropy analysis
let entropy = detector.analyze_entropy(&glyph_ids);
if entropy > 3.5 {
    println!("High entropy ({:.2}) suggests re-subset font", entropy);
}

// Example 4: Learning from feedback
if user_confirms_encoding(&result.encoding) {
    detector.learn_success(&glyph_ids, &result.encoding);
} else {
    detector.learn_failure(&glyph_ids);
}
```

## 4. Success Criteria

### 4.1 Detection Improvements
- ✅ Re-subset detection accuracy > 85%
- ✅ Symbolic font detection > 90%
- ✅ Mixed script detection > 80%
- ✅ Overall accuracy improvement > 5%

### 4.2 Performance
- Advanced analysis < 5ms
- Pattern learning overhead < 10%
- Memory for learned patterns < 1MB

### 4.3 Adaptability
- Successfully learns from 100+ patterns
- Confidence improves over time
- Persistence across sessions

## 5. Dependencies

- All previous phases (1-4)
- Optional: serde for pattern persistence
- Optional: statistical libraries

## 6. Estimated Timeline

| Task | Duration | Notes |
|------|----------|-------|
| Entropy analysis | 4 hours | Core algorithms |
| Cluster detection | 4 hours | DBSCAN implementation |
| Sequence analysis | 3 hours | Pattern detection |
| Pattern learning | 5 hours | ML-inspired system |
| Integration | 3 hours | Hook into detector |
| Testing | 4 hours | Edge cases |
| Documentation | 2 hours | Advanced guide |
| **Total** | **25 hours** | ~3.5 days |

## 7. Future Possibilities

- Cloud-based pattern sharing
- Neural network integration
- Real-time pattern updates
- Community pattern database
- A/B testing framework

---

*Document Version: 1.0*  
*Created: 2024-08-21*  
*Phase: 5 of 5*  
*Priority: LOW - Nice to have enhancements*