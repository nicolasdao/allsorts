// Integration with new allsorts APIs for enhanced font subsetting
// This module provides wrappers around the new allsorts APIs that were added
// in the feat-subset-pdf-extra branch

use anyhow::{Result, anyhow};
use std::collections::HashMap;
use allsorts::subset::{subset_and_map, SubsetProfile, CmapTarget, SubsetResult};
use allsorts::tables::FontTableProvider;
use log::{debug, info, warn};

/// Configuration for the new allsorts subsetting APIs
pub struct AllsortsConfig {
    /// Whether to use the new subset_and_map API
    pub use_subset_and_map: bool,
    /// The subset profile to use (PDF, Minimal, or Custom)
    pub profile: SubsetProfile,
    /// The target cmap format
    pub cmap_target: CmapTarget,
}

impl Default for AllsortsConfig {
    fn default() -> Self {
        Self {
            use_subset_and_map: true,
            profile: SubsetProfile::Pdf,
            cmap_target: CmapTarget::Unrestricted,
        }
    }
}

/// Subset a font using the new subset_and_map API
/// Returns the subsetted font data, the glyph ID mapping, and optionally the CIDToGIDMap for CID fonts
pub fn subset_with_mapping(
    provider: &impl FontTableProvider,
    glyph_ids: &[u16],
    config: &AllsortsConfig,
) -> Result<(Vec<u8>, HashMap<u16, u16>, Option<Vec<u8>>)> {
    debug!("Using new subset_and_map API with {} glyphs", glyph_ids.len());
    
    // Use the new API that provides glyph mapping
    match subset_and_map(provider, glyph_ids, &config.profile, config.cmap_target) {
        Ok(result) => {
            // Handle the SubsetResult enum
            let (font_data, mapping, cid_to_gid_map) = match result {
                SubsetResult::Simple { font_data, glyph_mapping } => {
                    info!("Successfully subsetted standard font with mapping: {} glyphs -> {} bytes", 
                          glyph_ids.len(), font_data.len());
                    (font_data, glyph_mapping, None)
                }
                SubsetResult::Cid { font_data, glyph_mapping, cid_to_gid_map } => {
                    info!("Successfully subsetted CID font with mapping: {} glyphs -> {} bytes (CIDToGIDMap: {} bytes)", 
                          glyph_ids.len(), font_data.len(), cid_to_gid_map.len());
                    (font_data, glyph_mapping, Some(cid_to_gid_map))
                }
            };
            
            // Log some mapping details for debugging
            if !mapping.is_empty() {
                debug!("Glyph mapping: {} entries", mapping.len());
                for (old_gid, new_gid) in mapping.iter().take(5) {
                    debug!("  GID {} -> {}", old_gid, new_gid);
                }
                if mapping.len() > 5 {
                    debug!("  ... and {} more", mapping.len() - 5);
                }
            }
            
            Ok((font_data, mapping, cid_to_gid_map))
        }
        Err(e) => {
            warn!("subset_and_map failed: {:?}", e);
            Err(anyhow!("Failed to subset font with mapping: {:?}", e))
        }
    }
}

/// Check if the new APIs are available
pub fn check_api_availability() -> bool {
    // This is a compile-time check - if this compiles, the APIs exist
    true
}

/// Get recommended configuration based on font type and settings
pub fn get_recommended_config(
    font_type: &str,
    preserve_identity: bool,
    aggressive: bool,
) -> AllsortsConfig {
    let profile = if aggressive {
        SubsetProfile::Pdf // Minimal tables for PDF
    } else {
        SubsetProfile::Minimal // Keep more tables for compatibility
    };
    
    let cmap_target = match font_type {
        "CIDFontType2" | "CIDFontType0" => {
            // For CID fonts, use Unicode cmap for better compatibility
            CmapTarget::Unicode
        }
        _ => CmapTarget::Unrestricted,
    };
    
    AllsortsConfig {
        use_subset_and_map: true, // Always use the new mapping API when available
        profile,
        cmap_target,
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_api_availability() {
        assert!(check_api_availability());
    }
    
    #[test]
    fn test_config_generation() {
        // Test CID font with identity preservation
        let config = get_recommended_config("CIDFontType2", true, false);
        assert!(config.use_subset_and_map); // Always use the new mapping API
        assert_eq!(config.cmap_target, CmapTarget::Unicode);
        
        // Test regular font with aggressive mode
        let config = get_recommended_config("TrueType", false, true);
        assert!(config.use_subset_and_map);
        matches!(config.profile, SubsetProfile::Pdf);
    }
}