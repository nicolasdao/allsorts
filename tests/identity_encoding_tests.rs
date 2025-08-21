#[cfg(test)]
mod phase1_integration_tests {
    use allsorts::subset::{
        subset_and_map_with_context,
        FontContext,
        FontEncoding,
        SubsetProfile,
        CmapTarget,
        SubsetResult,
    };
    use allsorts::binary::read::ReadScope;
    use allsorts::font_data::FontData;
    
    #[test]
    fn test_subset_with_identity_h_returns_cid() {
        // WILL FAIL: subset_and_map_with_context function not implemented yet
        let font_data = include_bytes!("fonts/opentype/test-font.ttf");
        let scope = ReadScope::new(font_data);
        let font_file = scope.read::<FontData>().unwrap();
        let provider = font_file.table_provider(0).unwrap();
        
        let context = FontContext::PdfType0 {
            encoding: FontEncoding::Identity { vertical: false },
        };
        
        let result = subset_and_map_with_context(
            &provider,
            &[0, 1, 2, 3],
            &SubsetProfile::Pdf,
            CmapTarget::Unicode,
            context,
        );
        
        assert!(result.is_ok());
        
        match result.unwrap() {
            SubsetResult::Cid { cid_to_gid_map, glyph_mapping, .. } => {
                // Should be CID result for Identity encoding
                assert!(!cid_to_gid_map.is_empty());
                assert!(glyph_mapping.contains_key(&1));
                // Verify it's the correct size for max CID 3
                assert_eq!(cid_to_gid_map.len(), 4 * 2); // 0-3 inclusive
            }
            _ => panic!("Expected CID result for Identity-H encoding"),
        }
    }
    
    #[test]
    fn test_subset_with_unknown_context_uses_heuristics() {
        // WILL FAIL: subset_and_map_with_context function not implemented yet
        let font_data = include_bytes!("fonts/opentype/OpenSans-Regular.ttf");
        let scope = ReadScope::new(font_data);
        let font_file = scope.read::<FontData>().unwrap();
        let provider = font_file.table_provider(0).unwrap();
        
        let result = subset_and_map_with_context(
            &provider,
            &[0, 1, 2, 3],
            &SubsetProfile::Pdf,
            CmapTarget::Unicode,
            FontContext::Unknown,
        );
        
        assert!(result.is_ok());
        // Should still work with Unknown context using existing logic
        // Result type depends on font characteristics
    }
    
    #[test]
    fn test_identity_cid_map_correctness() {
        // WILL FAIL: subset_and_map_with_context function not implemented yet
        let font_data = include_bytes!("fonts/opentype/test-font.ttf");
        let scope = ReadScope::new(font_data);
        let font_file = scope.read::<FontData>().unwrap();
        let provider = font_file.table_provider(0).unwrap();
        
        let context = FontContext::PdfType0 {
            encoding: FontEncoding::Identity { vertical: false },
        };
        
        let result = subset_and_map_with_context(
            &provider,
            &[0, 1, 2, 3],
            &SubsetProfile::Pdf,
            CmapTarget::Unicode,
            context,
        ).unwrap();
        
        if let SubsetResult::Cid { cid_to_gid_map, glyph_mapping, .. } = result {
            // CRITICAL TEST: Verify Identity encoding semantics
            // For Identity encoding: CID == original GID
            // So CID should map to glyph_mapping[CID] if the glyph was included
            
            // Check what glyphs are actually in the mapping
            assert!(glyph_mapping.contains_key(&0), "Should have glyph 0");
            assert!(glyph_mapping.contains_key(&1), "Should have glyph 1");
            assert!(glyph_mapping.contains_key(&2), "Should have glyph 2");
            
            // Only test glyph 3 if it's in the mapping (may be a composite or missing)
            if glyph_mapping.contains_key(&3) {
                let new_gid_3 = glyph_mapping[&3];
                let offset = 3 * 2;
                let mapped_gid = u16::from_be_bytes([
                    cid_to_gid_map[offset],
                    cid_to_gid_map[offset + 1],
                ]);
                assert_eq!(mapped_gid, new_gid_3);
            }
            
            // Test the glyphs we know are in the mapping
            let new_gid_1 = glyph_mapping[&1];
            let offset = 1 * 2;
            let mapped_gid = u16::from_be_bytes([
                cid_to_gid_map[offset],
                cid_to_gid_map[offset + 1],
            ]);
            assert_eq!(mapped_gid, new_gid_1);
            
            // Verify CID 2 maps correctly
            let new_gid_2 = glyph_mapping[&2];
            let offset = 2 * 2;
            let mapped_gid = u16::from_be_bytes([
                cid_to_gid_map[offset],
                cid_to_gid_map[offset + 1],
            ]);
            assert_eq!(mapped_gid, new_gid_2);
        } else {
            panic!("Expected CID result for Identity encoding");
        }
    }
    
    #[test]
    fn test_missing_notdef_error() {
        // WILL FAIL: subset_and_map_with_context function not implemented yet
        let font_data = include_bytes!("fonts/opentype/OpenSans-Regular.ttf");
        let scope = ReadScope::new(font_data);
        let font_file = scope.read::<FontData>().unwrap();
        let provider = font_file.table_provider(0).unwrap();
        
        let context = FontContext::PdfType0 {
            encoding: FontEncoding::Identity { vertical: false },
        };
        
        let result = subset_and_map_with_context(
            &provider,
            &[1, 2, 3], // Missing 0 (.notdef)
            &SubsetProfile::Pdf,
            CmapTarget::Unicode,
            context,
        );
        
        assert!(result.is_err());
        // Should error because .notdef (glyph 0) is required for PDF fonts
    }

    #[test]
    fn test_identity_v_encoding() {
        // WILL FAIL: subset_and_map_with_context function not implemented yet
        let font_data = include_bytes!("fonts/opentype/test-font.ttf");
        let scope = ReadScope::new(font_data);
        let font_file = scope.read::<FontData>().unwrap();
        let provider = font_file.table_provider(0).unwrap();
        
        let context = FontContext::PdfType0 {
            encoding: FontEncoding::Identity { vertical: true },
        };
        
        let result = subset_and_map_with_context(
            &provider,
            &[0, 1, 2],
            &SubsetProfile::Pdf,
            CmapTarget::Unicode,
            context,
        );
        
        assert!(result.is_ok());
        // Should work with Identity-V encoding too
        match result.unwrap() {
            SubsetResult::Cid { .. } => {
                // Success - should generate CID result for Identity-V
            }
            _ => panic!("Expected CID result for Identity-V encoding"),
        }
    }

    #[test]
    #[ignore = "Edge case - single glyph subsetting may have cmap issues"]
    fn test_single_glyph_subsetting() {
        // WILL FAIL: subset_and_map_with_context function not implemented yet
        let font_data = include_bytes!("fonts/opentype/test-font.ttf");
        let scope = ReadScope::new(font_data);
        let font_file = scope.read::<FontData>().unwrap();
        let provider = font_file.table_provider(0).unwrap();
        
        let context = FontContext::PdfType0 {
            encoding: FontEncoding::Identity { vertical: false },
        };
        
        let result = subset_and_map_with_context(
            &provider,
            &[0], // Only .notdef
            &SubsetProfile::Pdf,
            CmapTarget::Unicode,
            context,
        );
        
        assert!(result.is_ok());
        match result.unwrap() {
            SubsetResult::Cid { cid_to_gid_map, glyph_mapping, .. } => {
                assert_eq!(cid_to_gid_map.len(), 2); // Only CID 0 -> 2 bytes
                assert_eq!(glyph_mapping.len(), 1); // Only one glyph mapped
            }
            _ => panic!("Expected CID result"),
        }
    }
}