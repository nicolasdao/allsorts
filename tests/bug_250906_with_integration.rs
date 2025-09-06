//! Test to reproduce the allsorts `Parse(BadIndex)` error using the integration module
//! 
//! This test recreates the issue where trying to subset CID fonts with 65,280 glyphs
//! fails because allsorts cannot handle requests for glyphs that don't exist.

// Include the allsorts_integration module
mod allsorts_integration {
    use std::collections::HashMap;
    use allsorts::subset::{subset_and_map, SubsetProfile, CmapTarget, SubsetResult};
    use allsorts::tables::FontTableProvider;

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
    ) -> Result<(Vec<u8>, HashMap<u16, u16>, Option<Vec<u8>>), String> {
        println!("Using new subset_and_map API with {} glyphs", glyph_ids.len());
        
        // Use the new API that provides glyph mapping
        match subset_and_map(provider, glyph_ids, &config.profile, config.cmap_target) {
            Ok(result) => {
                // Handle the SubsetResult enum
                let (font_data, mapping, cid_to_gid_map) = match result {
                    SubsetResult::Simple { font_data, glyph_mapping } => {
                        println!("Successfully subsetted standard font with mapping: {} glyphs -> {} bytes", 
                              glyph_ids.len(), font_data.len());
                        (font_data, glyph_mapping, None)
                    }
                    SubsetResult::Cid { font_data, glyph_mapping, cid_to_gid_map } => {
                        println!("Successfully subsetted CID font with mapping: {} glyphs -> {} bytes (CIDToGIDMap: {} bytes)", 
                              glyph_ids.len(), font_data.len(), cid_to_gid_map.len());
                        (font_data, glyph_mapping, Some(cid_to_gid_map))
                    }
                };
                
                // Log some mapping details for debugging
                if !mapping.is_empty() {
                    println!("Glyph mapping: {} entries", mapping.len());
                    for (old_gid, new_gid) in mapping.iter().take(5) {
                        println!("  GID {} -> {}", old_gid, new_gid);
                    }
                    if mapping.len() > 5 {
                        println!("  ... and {} more", mapping.len() - 5);
                    }
                }
                
                Ok((font_data, mapping, cid_to_gid_map))
            }
            Err(e) => {
                println!("subset_and_map failed: {:?}", e);
                Err(format!("Failed to subset font with mapping: {:?}", e))
            }
        }
    }

    /// Get recommended configuration based on font type and settings
    #[allow(dead_code)]
    pub fn get_recommended_config(
        font_type: &str,
        _preserve_identity: bool,
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
}

#[cfg(test)]
mod allsorts_badindex_tests {
    use super::allsorts_integration::{subset_with_mapping, get_recommended_config, AllsortsConfig};
    use allsorts::binary::read::ReadScope;
    use allsorts::font_data::FontData;
    use allsorts::subset::{SubsetProfile, CmapTarget};
    
    /// Test using the integration module with a real font file
    #[test]
    fn test_subset_cid_font_with_65280_glyphs_using_integration() {
        // Load a real font for testing
        let font_bytes = include_bytes!("../tests/fonts/opentype/Klei.otf");
        
        // Parse the font
        let scope = ReadScope::new(font_bytes);
        let font_file = scope.read::<FontData<'_>>().expect("Failed to parse font");
        let provider = font_file.table_provider(0).expect("Failed to get table provider");
        
        // This is the problematic request: all glyphs from 0 to 65279 (65,280 total)
        let glyph_ids_vec: Vec<u16> = (0..=65279).collect();
        
        println!("🧪 Testing subset_and_map with {} glyphs (0..=65279) using integration module", 
                 glyph_ids_vec.len());
        
        // Configure for CID font subsetting using the integration module
        let config = AllsortsConfig {
            use_subset_and_map: true,
            profile: SubsetProfile::Pdf,
            cmap_target: CmapTarget::Unicode, // CID fonts use Unicode cmap
        };
        
        // Use the integration module's wrapper function
        let result = subset_with_mapping(&provider, &glyph_ids_vec, &config);
        
        match result {
            Ok((data, mapping, cid_map)) => {
                println!("✅ SUCCESS: subset_and_map succeeded with {} glyphs via integration module", glyph_ids_vec.len());
                println!("   Result: {} bytes, {} mapped glyphs", data.len(), mapping.len());
                if let Some(map) = cid_map {
                    println!("   CIDToGIDMap: {} bytes", map.len());
                }
                assert!(data.len() > 0, "Should have non-empty font data");
                assert!(mapping.len() <= 770, "Should only map existing glyphs");
            }
            Err(e) => {
                let error_string = format!("{:?}", e);
                if error_string.contains("Parse") && error_string.contains("BadIndex") {
                    panic!("❌ BUG STILL EXISTS: Parse(BadIndex) error when requesting 65,280 glyphs!\nError: {}", error_string);
                } else {
                    panic!("Unexpected error: {}", error_string);
                }
            }
        }
    }
    
    /// Test with a reasonable glyph count using the integration module
    #[test] 
    fn test_subset_cid_font_with_reasonable_count_using_integration() {
        let font_bytes = include_bytes!("../tests/fonts/opentype/Klei.otf");
        let scope = ReadScope::new(font_bytes);
        let font_file = scope.read::<FontData<'_>>().expect("Failed to parse font");
        let provider = font_file.table_provider(0).expect("Failed to get table provider");
        
        // Request a reasonable number of glyphs
        let glyph_ids_vec: Vec<u16> = (0..=255).collect();
        
        println!("🧪 Testing subset_and_map with {} glyphs using integration module", 
                 glyph_ids_vec.len());
        
        let config = AllsortsConfig {
            use_subset_and_map: true,
            profile: SubsetProfile::Pdf,
            cmap_target: CmapTarget::Unicode,
        };
        
        let result = subset_with_mapping(&provider, &glyph_ids_vec, &config);
        
        match result {
            Ok((data, mapping, _cid_map)) => {
                println!("✅ SUCCESS: subset_and_map worked with reasonable glyph count");
                println!("   Result: {} bytes, {} mapped glyphs", data.len(), mapping.len());
                assert!(data.len() > 0, "Should have non-empty font data");
            }
            Err(e) => {
                panic!("Should work with reasonable glyph count, got error: {:?}", e);
            }
        }
    }
    
    /// Test using get_recommended_config from the integration module
    #[test]
    fn test_with_recommended_config_for_cid_fonts() {
        let font_bytes = include_bytes!("../tests/fonts/opentype/Klei.otf");
        let scope = ReadScope::new(font_bytes);
        let font_file = scope.read::<FontData<'_>>().expect("Failed to parse font");
        let provider = font_file.table_provider(0).expect("Failed to get table provider");
        
        // Get recommended config for CID fonts
        let config = get_recommended_config("CIDFontType2", true, false);
        
        // Test with excessive glyphs using recommended config
        let glyph_ids_vec: Vec<u16> = vec![0, 100, 500, 1000, 5000, 10000, 50000];
        
        println!("🧪 Testing with recommended CID config for {} glyphs", glyph_ids_vec.len());
        
        let result = subset_with_mapping(&provider, &glyph_ids_vec, &config);
        
        match result {
            Ok((data, mapping, _cid_map)) => {
                println!("✅ SUCCESS with recommended config");
                println!("   Result: {} bytes, {} mapped glyphs", data.len(), mapping.len());
                assert!(data.len() > 0, "Should have non-empty font data");
                // Should only map glyphs that exist
                assert!(mapping.len() <= glyph_ids_vec.len(), "Should not map more than requested");
            }
            Err(e) => {
                panic!("Should handle excessive glyphs with recommended config, got: {:?}", e);
            }
        }
    }
}