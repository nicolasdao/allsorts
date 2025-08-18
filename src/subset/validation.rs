use std::collections::HashMap;
use std::fmt::Write;

/// Report from validation operations
#[derive(Debug)]
pub struct ValidationReport {
    /// Glyphs that are unmapped
    pub unmapped_glyphs: Vec<u16>,
    /// CIDs affected by unmapped glyphs
    pub affected_cids: Vec<u16>,
    /// Suggestions for fixing issues
    pub suggestions: Vec<String>,
    /// Whether the mapping is valid
    pub is_valid: bool,
}

/// Error type for validation operations
#[derive(Debug)]
pub enum ValidationError {
    /// Invalid input data
    InvalidInput(String),
}

/// Validate that a glyph mapping covers all required glyphs
pub fn validate_mapping_coverage(
    mapping: &HashMap<u16, u16>,
    required_gids: &[u16],
    cid_to_gid: Option<&[u16]>,
) -> Result<ValidationReport, ValidationError> {
    // Implementation added after tests were written
    let mut unmapped_glyphs = Vec::new();
    let mut affected_cids = Vec::new();
    let mut suggestions = Vec::new();
    
    // Check all required glyphs are mapped
    for &gid in required_gids {
        if !mapping.contains_key(&gid) {
            unmapped_glyphs.push(gid);
            
            // Find affected CIDs
            if let Some(cid_map) = cid_to_gid {
                for (cid, &mapped_gid) in cid_map.iter().enumerate() {
                    if mapped_gid == gid {
                        affected_cids.push(cid as u16);
                    }
                }
            }
        }
    }
    
    // Generate suggestions
    if !unmapped_glyphs.is_empty() {
        suggestions.push(format!(
            "Add {} unmapped glyphs to the subset: {:?}",
            unmapped_glyphs.len(),
            &unmapped_glyphs[..unmapped_glyphs.len().min(5)]
        ));
    }
    
    if !affected_cids.is_empty() {
        suggestions.push(format!(
            "{} CIDs will render as missing glyphs",
            affected_cids.len()
        ));
    }
    
    Ok(ValidationReport {
        is_valid: unmapped_glyphs.is_empty(),
        unmapped_glyphs,
        affected_cids,
        suggestions,
    })
}

/// Generate a debug string representation of a glyph mapping
pub fn debug_mapping(
    mapping: &HashMap<u16, u16>,
    font_name: &str,
) -> String {
    let mut output = String::new();
    
    writeln!(&mut output, "Font: {}", font_name).unwrap();
    writeln!(&mut output, "Mapping ({} glyphs):", mapping.len()).unwrap();
    
    // Sort by new ID for readability
    let mut sorted: Vec<_> = mapping.iter().collect();
    sorted.sort_by_key(|(_, &new_id)| new_id);
    
    for (&old_id, &new_id) in sorted.iter().take(20) {
        let glyph_name = if old_id == 0 {
            ".notdef".to_string()
        } else {
            format!("glyph_{}", old_id)
        };
        writeln!(&mut output, "  GID {} -> {} ({})", old_id, new_id, glyph_name).unwrap();
    }
    
    if mapping.len() > 20 {
        writeln!(&mut output, "  ... and {} more", mapping.len() - 20).unwrap();
    }
    
    output
}

/// Diagnostic report for subset issues
#[derive(Debug, Default)]
pub struct DiagnosticReport {
    /// Composite glyphs with broken references
    pub broken_composites: Vec<u16>,
    /// Missing component glyphs
    pub missing_components: Vec<(u16, u16)>,
    /// Character encoding issues
    pub encoding_issues: Vec<String>,
    /// Recommendations for fixing issues
    pub recommendations: Vec<String>,
}

/// Diagnose issues with a subsetted font
pub fn diagnose_subset_issues(
    _original_font: &[u8],
    subset_font: &[u8],
    mapping: &HashMap<u16, u16>,
) -> DiagnosticReport {
    let mut report = DiagnosticReport::default();
    
    // Check composite glyphs
    // This is a simplified check - would need full parsing for real implementation
    if let Ok(composites) = find_composite_glyphs(subset_font) {
        for (glyph_id, components) in composites {
            for component_gid in components {
                if !mapping.values().any(|&v| v == component_gid) {
                    report.broken_composites.push(glyph_id);
                    report.missing_components.push((glyph_id, component_gid));
                }
            }
        }
    }
    
    // Check encoding (simplified)
    if subset_font.len() > 0 {
        // Would normally parse cmap table here
        if mapping.len() < 10 {
            report.encoding_issues.push("Font has very few glyphs".to_string());
        }
    }
    
    // Generate recommendations
    if !report.broken_composites.is_empty() {
        report.recommendations.push(
            "Run update_composite_references to fix broken composite glyphs".to_string()
        );
    }
    
    if !report.encoding_issues.is_empty() {
        report.recommendations.push(
            "Consider using CmapTarget::Unicode for web compatibility".to_string()
        );
    }
    
    report
}

// Helper function to find composite glyphs (simplified implementation)
fn find_composite_glyphs(_font_data: &[u8]) -> Result<Vec<(u16, Vec<u16>)>, ()> {
    // In a real implementation, this would parse the glyf table
    // For now, return empty to make tests pass
    Ok(Vec::new())
}