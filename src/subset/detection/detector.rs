use std::collections::HashMap;
use crate::subset::context::FontEncoding;
use crate::subset::SubsetError;
use crate::tables::FontTableProvider;
use super::patterns::{PatternMatcher, PatternMatch};
use super::statistics::GlyphStatistics;

/// Result of encoding detection with confidence level and reasoning
#[derive(Debug, Clone)]
pub struct EncodingDetection {
    /// Detected encoding
    pub encoding: FontEncoding,
    /// Confidence level of detection
    pub confidence: DetectionConfidence,
    /// Reasoning steps that led to the detection
    pub reasoning: Vec<String>,
    /// Alternative encoding suggestions
    pub alternatives: Vec<FontEncoding>,
}

/// Confidence level of encoding detection
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum DetectionConfidence {
    /// 100% certain (explicit encoding specified)
    Certain,
    /// 90%+ confidence (strong patterns match)
    High,
    /// 70-90% confidence (some patterns match)
    Medium,
    /// <70% confidence (guessing based on heuristics)
    Low,
}

impl PartialOrd for DetectionConfidence {
    fn partial_cmp(&self, other: &Self) -> Option<std::cmp::Ordering> {
        Some(self.cmp(other))
    }
}

impl Ord for DetectionConfidence {
    fn cmp(&self, other: &Self) -> std::cmp::Ordering {
        // Higher confidence should be "greater" than lower confidence
        // Certain > High > Medium > Low
        let self_rank = match self {
            DetectionConfidence::Certain => 4,
            DetectionConfidence::High => 3,
            DetectionConfidence::Medium => 2,
            DetectionConfidence::Low => 1,
        };
        let other_rank = match other {
            DetectionConfidence::Certain => 4,
            DetectionConfidence::High => 3,
            DetectionConfidence::Medium => 2,
            DetectionConfidence::Low => 1,
        };
        self_rank.cmp(&other_rank)
    }
}

/// Main encoding detector that analyzes fonts and glyphs
pub struct EncodingDetector {
    /// Font table provider for analysis
    provider: Box<dyn FontTableProvider>,
    /// Pattern matcher for glyph analysis
    pattern_matcher: PatternMatcher,
    /// Detection cache to avoid recomputation
    cache: HashMap<CacheKey, EncodingDetection>,
}

/// Cache key for detection results
#[derive(Debug, Clone, Hash, PartialEq, Eq)]
struct CacheKey {
    /// Sorted glyph IDs
    glyph_ids: Vec<u16>,
    /// PDF font info hash (simplified)
    pdf_info_hash: Option<u64>,
}

/// PDF font metadata for detection hints
#[derive(Debug, Clone, Default)]
pub struct PdfFontInfo {
    /// Explicit encoding name from PDF
    pub encoding_name: Option<String>,
    /// Font name from PDF
    pub font_name: Option<String>,
    /// Font descriptor flags
    pub flags: u32,
    /// CIDSystemInfo registry
    pub registry: Option<String>,
    /// CIDSystemInfo ordering
    pub ordering: Option<String>,
    /// CIDSystemInfo supplement
    pub supplement: Option<u16>,
    /// ToUnicode CMap data
    pub to_unicode: Option<Vec<u8>>,
}

impl EncodingDetector {
    /// Create a new encoding detector
    pub fn new(provider: Box<dyn FontTableProvider>) -> Self {
        Self {
            provider,
            pattern_matcher: PatternMatcher::new(),
            cache: HashMap::new(),
        }
    }
    
    /// Detect encoding from glyph IDs and optional PDF info
    pub fn detect(&mut self, glyph_ids: &[u16], pdf_info: Option<&PdfFontInfo>) -> EncodingDetection {
        // Create cache key
        let cache_key = self.create_cache_key(glyph_ids, pdf_info);
        
        // Check cache first
        if let Some(cached_detection) = self.cache.get(&cache_key) {
            return cached_detection.clone();
        }
        
        // Perform detection
        let detection = self.detect_internal(glyph_ids, pdf_info);
        
        // Cache result
        self.cache.insert(cache_key, detection.clone());
        
        detection
    }
    
    /// Internal detection logic
    fn detect_internal(&self, glyph_ids: &[u16], pdf_info: Option<&PdfFontInfo>) -> EncodingDetection {
        let mut reasoning = Vec::new();
        let mut alternatives = Vec::new();
        
        // Handle empty glyph set
        if glyph_ids.is_empty() {
            reasoning.push("Empty glyph set - defaulting to Identity-H".to_string());
            return EncodingDetection {
                encoding: FontEncoding::Identity { vertical: false },
                confidence: DetectionConfidence::Low,
                reasoning,
                alternatives,
            };
        }
        
        // Check for explicit PDF encoding first (highest confidence)
        if let Some(pdf_info) = pdf_info {
            if let Some(encoding_name) = &pdf_info.encoding_name {
                if let Some(encoding) = FontEncoding::from_pdf_name(encoding_name) {
                    reasoning.push(format!("PDF specifies encoding: {}", encoding_name));
                    return EncodingDetection {
                        encoding,
                        confidence: DetectionConfidence::Certain,
                        reasoning,
                        alternatives,
                    };
                } else {
                    reasoning.push(format!("Unknown encoding name in PDF: {}", encoding_name));
                }
            } else {
                reasoning.push("No explicit encoding in PDF font info".to_string());
            }
        }
        
        // Pattern-based detection
        let patterns = self.pattern_matcher.find_patterns(glyph_ids);
        
        if let Some(best_pattern) = patterns.first() {
            let confidence = if best_pattern.match_ratio >= 0.9 {
                DetectionConfidence::High
            } else if best_pattern.match_ratio >= 0.7 {
                DetectionConfidence::Medium
            } else {
                DetectionConfidence::Low
            };
            
            reasoning.push(format!(
                "Pattern '{}' matched with {:.1}% confidence",
                best_pattern.pattern_name,
                best_pattern.match_ratio * 100.0
            ));
            
            // Add alternatives from other patterns
            for pattern in patterns.iter().skip(1).take(3) {
                alternatives.push(pattern.encoding.clone());
            }
            
            return EncodingDetection {
                encoding: best_pattern.encoding.clone(),
                confidence,
                reasoning,
                alternatives,
            };
        }
        
        // Fallback to Identity-H
        reasoning.push("No strong patterns detected - defaulting to Identity-H".to_string());
        EncodingDetection {
            encoding: FontEncoding::Identity { vertical: false },
            confidence: DetectionConfidence::Low,
            reasoning,
            alternatives,
        }
    }
    
    /// Create cache key for glyph IDs and PDF info
    fn create_cache_key(&self, glyph_ids: &[u16], pdf_info: Option<&PdfFontInfo>) -> CacheKey {
        let mut sorted_gids = glyph_ids.to_vec();
        sorted_gids.sort_unstable();
        
        let pdf_info_hash = pdf_info.map(|info| {
            // Simple hash of PDF info fields
            use std::collections::hash_map::DefaultHasher;
            use std::hash::{Hash, Hasher};
            
            let mut hasher = DefaultHasher::new();
            info.encoding_name.hash(&mut hasher);
            info.font_name.hash(&mut hasher);
            info.flags.hash(&mut hasher);
            info.registry.hash(&mut hasher);
            info.ordering.hash(&mut hasher);
            info.supplement.hash(&mut hasher);
            hasher.finish()
        });
        
        CacheKey {
            glyph_ids: sorted_gids,
            pdf_info_hash,
        }
    }
}