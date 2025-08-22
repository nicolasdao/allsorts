use std::collections::HashMap;

/// Statistical analysis of glyph distributions
pub struct GlyphStatistics {
    /// Total number of unique glyphs
    pub total_glyphs: usize,
    /// Minimum glyph ID
    pub min_gid: u16,
    /// Maximum glyph ID
    pub max_gid: u16,
    /// Density of glyphs in the range
    pub density: f32,
    /// Has ASCII range glyphs
    pub has_ascii: bool,
    /// Has Latin Extended glyphs
    pub has_latin_extended: bool,
    /// Has CJK ideograph glyphs
    pub has_cjk: bool,
    /// Has Japanese Kana glyphs
    pub has_kana: bool,
    /// Has Korean Hangul glyphs
    pub has_hangul: bool,
    /// Sequential runs of glyph IDs
    pub sequential_runs: Vec<SequentialRun>,
    /// Distribution of gaps between glyphs
    pub gap_distribution: HashMap<u16, usize>,
}

/// A sequential run of glyph IDs
#[derive(Debug, Clone)]
pub struct SequentialRun {
    /// Starting glyph ID of the run
    pub start: u16,
    /// Length of the sequential run
    pub length: usize,
}

impl GlyphStatistics {
    /// Create statistics from glyph IDs
    pub fn from_glyph_ids(glyph_ids: &[u16]) -> Self {
        if glyph_ids.is_empty() {
            return Self {
                total_glyphs: 0,
                min_gid: 0,
                max_gid: 0,
                density: 0.0,
                has_ascii: false,
                has_latin_extended: false,
                has_cjk: false,
                has_kana: false,
                has_hangul: false,
                sequential_runs: Vec::new(),
                gap_distribution: HashMap::new(),
            };
        }
        
        let mut sorted_gids = glyph_ids.to_vec();
        sorted_gids.sort_unstable();
        sorted_gids.dedup();
        
        let total_glyphs = sorted_gids.len();
        let min_gid = sorted_gids[0];
        let max_gid = sorted_gids[total_glyphs - 1];
        
        // Calculate density: unique glyphs / range
        let range = if max_gid == min_gid {
            1
        } else {
            (max_gid - min_gid + 1) as usize
        };
        let density = total_glyphs as f32 / range as f32;
        
        // Detect character ranges
        let has_ascii = sorted_gids.iter().any(|&gid| gid >= 32 && gid <= 126);
        let has_latin_extended = sorted_gids.iter().any(|&gid| gid >= 128 && gid <= 255);
        
        // CJK ranges
        let has_cjk_ideographs = sorted_gids.iter().any(|&gid| gid >= 0x4E00 && gid <= 0x9FFF);
        let has_kana = sorted_gids.iter().any(|&gid| 
            (gid >= 0x3040 && gid <= 0x309F) || // Hiragana
            (gid >= 0x30A0 && gid <= 0x30FF)    // Katakana
        );
        let has_hangul = sorted_gids.iter().any(|&gid| gid >= 0xAC00 && gid <= 0xD7AF);
        let has_cjk = has_cjk_ideographs || has_kana || has_hangul;
        
        // Find sequential runs
        let mut sequential_runs = Vec::new();
        if !sorted_gids.is_empty() {
            let mut run_start = sorted_gids[0];
            let mut run_length = 1;
            
            for i in 1..sorted_gids.len() {
                if sorted_gids[i] == sorted_gids[i - 1] + 1 {
                    run_length += 1;
                } else {
                    if run_length >= 2 {
                        sequential_runs.push(SequentialRun {
                            start: run_start,
                            length: run_length,
                        });
                    }
                    run_start = sorted_gids[i];
                    run_length = 1;
                }
            }
            
            // Don't forget the last run
            if run_length >= 2 {
                sequential_runs.push(SequentialRun {
                    start: run_start,
                    length: run_length,
                });
            }
        }
        
        // Calculate gap distribution
        let mut gap_distribution = HashMap::new();
        for i in 1..sorted_gids.len() {
            let gap = sorted_gids[i] - sorted_gids[i - 1] - 1;
            if gap > 0 {
                *gap_distribution.entry(gap).or_insert(0) += 1;
            }
        }
        
        Self {
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
    
    /// Check if statistics indicate CJK glyphs
    pub fn has_cjk_range(&self) -> bool {
        self.has_cjk
    }
    
    /// Check if statistics indicate Kana glyphs
    pub fn has_kana_range(&self) -> bool {
        self.has_kana
    }
    
    /// Check if statistics indicate Hangul glyphs
    pub fn has_hangul_range(&self) -> bool {
        self.has_hangul
    }
    
    /// Check if glyph distribution is dense
    pub fn is_dense(&self) -> bool {
        self.density >= 0.7 // 70% or higher density considered dense
    }
    
    /// Check if glyph distribution is sparse
    pub fn is_sparse(&self) -> bool {
        self.density <= 0.1 // 10% or lower density considered sparse
    }
    
    /// Check if there are large sequential runs
    pub fn has_large_sequential_runs(&self) -> bool {
        self.sequential_runs.iter().any(|run| run.length >= 50)
    }
}