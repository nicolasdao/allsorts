use crate::subset::context::{FontEncoding, CJKLanguage, ChineseVariant};
use crate::subset::detection::statistics::GlyphStatistics;

/// Pattern matcher for identifying glyph ID patterns
pub struct PatternMatcher {
    patterns: Vec<PatternDefinition>,
}

/// A pattern definition for matching glyph IDs
#[derive(Debug, Clone)]
struct PatternDefinition {
    name: String,
    encoding: FontEncoding,
    matcher: PatternType,
}

/// Types of pattern matching logic
#[derive(Debug, Clone)]
enum PatternType {
    /// Match Identity-H (sparse, high GIDs)
    IdentityH,
    /// Match CJK dense patterns
    CJKDense,
    /// Match ASCII patterns
    ASCII,
    /// Match Latin extended patterns
    LatinExtended,
    /// Match symbol/private use patterns
    Symbol,
    /// Match sparse high GID patterns
    SparseHighGID,
}

/// Result of pattern matching with confidence score
#[derive(Debug, Clone)]
pub struct PatternMatch {
    /// Name of the matched pattern
    pub pattern_name: String,
    /// Match confidence ratio (0.0-1.0)
    pub match_ratio: f32,
    /// Suggested encoding for this pattern
    pub encoding: FontEncoding,
}

impl PatternMatcher {
    /// Create a new pattern matcher
    pub fn new() -> Self {
        let patterns = vec![
            PatternDefinition {
                name: "Identity-H".to_string(),
                encoding: FontEncoding::Identity { vertical: false },
                matcher: PatternType::IdentityH,
            },
            PatternDefinition {
                name: "CJK-Dense".to_string(),
                encoding: FontEncoding::CJK {
                    language: CJKLanguage::Chinese(ChineseVariant::Simplified),
                    encoding_name: "GB-EUC-H".to_string(),
                    vertical: false,
                    requires_cmap_data: true,
                },
                matcher: PatternType::CJKDense,
            },
            PatternDefinition {
                name: "ASCII-Latin".to_string(),
                encoding: FontEncoding::Identity { vertical: false },
                matcher: PatternType::ASCII,
            },
            PatternDefinition {
                name: "Latin-Extended".to_string(),
                encoding: FontEncoding::Identity { vertical: false },
                matcher: PatternType::LatinExtended,
            },
            PatternDefinition {
                name: "Symbol-Font".to_string(),
                encoding: FontEncoding::Identity { vertical: false },
                matcher: PatternType::Symbol,
            },
            PatternDefinition {
                name: "Sparse-Identity".to_string(),
                encoding: FontEncoding::Identity { vertical: false },
                matcher: PatternType::SparseHighGID,
            },
        ];
        
        Self { patterns }
    }
    
    /// Find patterns in glyph IDs
    pub fn find_patterns(&self, glyph_ids: &[u16]) -> Vec<PatternMatch> {
        if glyph_ids.is_empty() {
            return Vec::new();
        }
        
        let stats = GlyphStatistics::from_glyph_ids(glyph_ids);
        let mut matches = Vec::new();
        
        for pattern in &self.patterns {
            let match_ratio = self.calculate_match_ratio(&pattern.matcher, &stats, glyph_ids);
            
            if match_ratio > 0.0 {
                matches.push(PatternMatch {
                    pattern_name: pattern.name.clone(),
                    match_ratio,
                    encoding: pattern.encoding.clone(),
                });
            }
        }
        
        // Sort by match ratio (highest first)
        matches.sort_by(|a, b| b.match_ratio.partial_cmp(&a.match_ratio).unwrap());
        
        matches
    }
    
    /// Calculate match ratio for a specific pattern type
    fn calculate_match_ratio(&self, pattern_type: &PatternType, stats: &GlyphStatistics, glyph_ids: &[u16]) -> f32 {
        match pattern_type {
            PatternType::IdentityH => {
                // High match for sparse distributions with high GIDs
                let sparseness_score = if stats.is_sparse() { 0.4 } else { 0.0 };
                let high_gid_score = if stats.max_gid > 500 { 0.4 } else { 0.0 };
                let non_sequential_score = if !stats.has_large_sequential_runs() { 0.2 } else { 0.0 };
                
                sparseness_score + high_gid_score + non_sequential_score
            },
            
            PatternType::CJKDense => {
                // High match for dense CJK ranges - requires actual CJK glyphs
                if stats.has_cjk {
                    let cjk_score = 0.5;
                    let density_score = if stats.is_dense() { 0.3 } else { 0.0 };
                    let sequential_score = if stats.has_large_sequential_runs() { 0.2 } else { 0.0 };
                    cjk_score + density_score + sequential_score
                } else {
                    0.0 // No CJK pattern match without CJK glyphs
                }
            },
            
            PatternType::ASCII => {
                // High match for ASCII ranges - require significant coverage and range
                let ascii_coverage = glyph_ids.iter().filter(|&&gid| gid >= 32 && gid <= 126).count();
                let total_range = (stats.max_gid - stats.min_gid + 1) as usize;
                
                // Only match if we have a substantial ASCII character set
                if stats.has_ascii && ascii_coverage >= 20 && total_range >= 50 {
                    let ascii_score = 0.5;
                    let range_score = if stats.min_gid >= 32 && stats.max_gid <= 126 { 0.3 } else { 0.0 };
                    let density_score = if stats.density > 0.5 { 0.2 } else { 0.0 };
                    ascii_score + range_score + density_score
                } else {
                    0.0 // No ASCII pattern match unless substantial
                }
            },
            
            PatternType::LatinExtended => {
                // High match for Latin Extended ranges - require actual Latin Extended glyphs
                let latin_coverage = glyph_ids.iter().filter(|&&gid| gid >= 128 && gid <= 255).count();
                let latin_score = if stats.has_latin_extended && latin_coverage >= 10 { 0.5 } else { 0.0 };
                let range_score = if stats.min_gid >= 128 && stats.max_gid <= 255 { 0.3 } else { 0.0 };
                let density_score = if stats.density > 0.3 { 0.2 } else { 0.0 };
                
                latin_score + range_score + density_score
            },
            
            PatternType::Symbol => {
                // High match for symbol/private use ranges
                let private_use_score = if glyph_ids.iter().any(|&gid| gid >= 0xF020 && gid <= 0xF0FF) { 0.6 } else { 0.0 };
                let symbol_range_score = if glyph_ids.iter().any(|&gid| (gid >= 0x2000 && gid <= 0x2BFF) || (gid >= 0xE000 && gid <= 0xF8FF)) { 0.3 } else { 0.0 };
                let density_score = if stats.density > 0.4 { 0.1 } else { 0.0 };
                
                private_use_score + symbol_range_score + density_score
            },
            
            PatternType::SparseHighGID => {
                // High match for sparse distributions with very high GIDs
                let extreme_sparse_score = if stats.density < 0.05 { 0.4 } else { 0.0 };
                let very_high_gid_score = if stats.max_gid > 1000 { 0.3 } else { 0.0 };
                let large_range_score = if (stats.max_gid - stats.min_gid) > 2000 { 0.3 } else { 0.0 };
                
                extreme_sparse_score + very_high_gid_score + large_range_score
            },
        }
    }
}