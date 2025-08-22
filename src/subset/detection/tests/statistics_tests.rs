use crate::subset::detection::statistics::{GlyphStatistics, SequentialRun};

#[test]
fn test_glyph_statistics_basic_calculations() {
    // WILL FAIL until GlyphStatistics::from_glyph_ids() is implemented
    let glyph_ids = vec![0, 5, 10, 15, 20];
    let stats = GlyphStatistics::from_glyph_ids(&glyph_ids);
    
    assert_eq!(stats.total_glyphs, 5);
    assert_eq!(stats.min_gid, 0);
    assert_eq!(stats.max_gid, 20);
    let expected_density = 5.0 / 21.0; // 5 glyphs in range 0-20
    assert!((stats.density - expected_density).abs() < 0.001);
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

#[test]
fn test_ascii_range_detection() {
    // WILL FAIL until ASCII detection is implemented
    let glyph_ids = vec![65, 66, 67, 97, 98, 99]; // ABC and abc
    let stats = GlyphStatistics::from_glyph_ids(&glyph_ids);
    
    assert!(stats.has_ascii);
    assert!(!stats.has_latin_extended);
    assert!(!stats.has_cjk);
}

#[test]
fn test_latin_extended_detection() {
    // WILL FAIL until Latin Extended detection is implemented
    let glyph_ids = vec![192, 193, 224, 225]; // À Á à á
    let stats = GlyphStatistics::from_glyph_ids(&glyph_ids);
    
    assert!(stats.has_latin_extended);
    assert!(!stats.has_ascii); // These are beyond ASCII range
}

#[test]
fn test_gap_distribution() {
    // WILL FAIL until gap distribution is implemented
    let glyph_ids = vec![0, 2, 5, 10, 20]; // Gaps: 1, 2, 4, 9
    let stats = GlyphStatistics::from_glyph_ids(&glyph_ids);
    
    assert_eq!(stats.gap_distribution.get(&1), Some(&1)); // Gap of 1 appears once
    assert_eq!(stats.gap_distribution.get(&2), Some(&1)); // Gap of 2 appears once
    assert_eq!(stats.gap_distribution.get(&4), Some(&1)); // Gap of 4 appears once
    assert_eq!(stats.gap_distribution.get(&9), Some(&1)); // Gap of 9 appears once
}

#[test]
fn test_empty_glyphs_statistics() {
    // WILL FAIL until edge case handling is implemented
    let stats = GlyphStatistics::from_glyph_ids(&[]);
    
    assert_eq!(stats.total_glyphs, 0);
    assert_eq!(stats.min_gid, 0);
    assert_eq!(stats.max_gid, 0);
    assert_eq!(stats.density, 0.0);
    assert!(!stats.has_ascii);
    assert!(!stats.has_cjk);
    assert!(stats.sequential_runs.is_empty());
}

#[test]
fn test_single_glyph_statistics() {
    // WILL FAIL until single glyph handling is implemented
    let stats = GlyphStatistics::from_glyph_ids(&[42]);
    
    assert_eq!(stats.total_glyphs, 1);
    assert_eq!(stats.min_gid, 42);
    assert_eq!(stats.max_gid, 42);
    assert_eq!(stats.density, 1.0); // One glyph in a range of one
}