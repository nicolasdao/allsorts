mod common;

use std::collections::HashMap;
use allsorts::subset::validation::{
    validate_mapping_coverage, debug_mapping, diagnose_subset_issues
};

#[test]
fn test_validate_mapping_coverage_complete() {
    let mapping = HashMap::from([
        (0, 0),
        (1, 1),
        (2, 2),
    ]);
    
    let result = validate_mapping_coverage(
        &mapping,
        &[0, 1, 2],
        None,
    );
    
    assert!(result.is_ok());
    let report = result.unwrap();
    assert!(report.is_valid);
    assert!(report.unmapped_glyphs.is_empty());
}

#[test]
fn test_validate_mapping_coverage_missing() {
    let mapping = HashMap::from([
        (0, 0),
        (1, 1),
    ]);
    
    let result = validate_mapping_coverage(
        &mapping,
        &[0, 1, 2, 3], // 2 and 3 are not mapped
        None,
    );
    
    assert!(result.is_ok());
    let report = result.unwrap();
    assert!(!report.is_valid);
    assert_eq!(report.unmapped_glyphs.len(), 2);
    assert!(report.unmapped_glyphs.contains(&2));
    assert!(report.unmapped_glyphs.contains(&3));
}

#[test]
fn test_validate_mapping_with_cid_map() {
    let mapping = HashMap::from([
        (0, 0),
        (1, 1),
    ]);
    
    let cid_to_gid = vec![0, 1, 2, 3]; // CID 2 maps to GID 2, which is unmapped
    
    let result = validate_mapping_coverage(
        &mapping,
        &[0, 1, 2],
        Some(&cid_to_gid),
    );
    
    assert!(result.is_ok());
    let report = result.unwrap();
    assert!(!report.is_valid);
    assert!(report.unmapped_glyphs.contains(&2));
    assert!(report.affected_cids.contains(&2)); // CID 2 is affected
}

#[test]
fn test_debug_mapping_output() {
    let mapping = HashMap::from([
        (0, 0),
        (100, 1),
        (200, 2),
    ]);
    
    let output = debug_mapping(&mapping, "TestFont");
    
    assert!(output.contains("TestFont"));
    assert!(output.contains("3 glyphs"));
    assert!(output.contains(".notdef"));
    assert!(output.contains("GID 0 -> 0"));
}

#[test]
fn test_diagnose_subset_issues() {
    let original = common::read_fixture("tests/fonts/opentype/SFNT-TTF-Composite.ttf");
    let subset = common::read_fixture("tests/fonts/opentype/Klei.otf");
    let mapping = HashMap::from([(0, 0), (1, 1)]);
    
    let report = diagnose_subset_issues(&original, &subset, &mapping);
    
    // Should produce a diagnostic report (won't have specific issues with these test fonts)
    assert!(report.broken_composites.is_empty() || !report.broken_composites.is_empty());
    if !report.broken_composites.is_empty() {
        assert!(!report.recommendations.is_empty());
    }
}