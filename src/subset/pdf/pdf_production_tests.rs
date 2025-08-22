#[cfg(test)]
mod production_tests {
    use crate::subset::pdf::{subset_for_pdf, PdfFontContext, WritingMode};
    use crate::subset::FontEncoding;
    use std::collections::HashMap;

    /// Test with the exact production data from the bug report
    #[test]
    fn test_calibri_bold_production_case() {
        // Case 1: Calibri-Bold - 19 actually used glyphs
        let calibri_bold_gids = vec![
            0,   // .notdef (always required)
            3,   // Actual character glyph
            15,  
            17,  
            25,  
            36,  
            43,  
            71,  
            79,  
            80,  
            88,  
            100, 
            103, 
            104, 
            144, 
            191, 
            193, 
            194, 
            199, 
            200  // Max GID for this subset
        ];

        // These are the exact parameters that trigger the bug
        let pdf_context = PdfFontContext {
            encoding: FontEncoding::Identity { vertical: false }, // Identity-H
            max_cid: Some(255),  // User uses 255 as max for all fonts
            preserve_identity: false,  // This should now work correctly
            is_symbolic: false,  // CID fonts are not symbolic
            cid_to_gid_map: None,  // Let API generate it
            writing_mode: WritingMode::Horizontal,
            cmap_provider: None,  // Not needed for Identity-H
        };

        // Create a simulated glyph mapping (what subset_and_map would produce)
        let mut glyph_mapping = HashMap::new();
        for (new_id, &old_id) in calibri_bold_gids.iter().enumerate() {
            glyph_mapping.insert(old_id, new_id as u16);
        }

        // Test the CIDToGIDMap generation directly
        let (cid_to_gid_map, _validation) = 
            crate::subset::pdf::generate_cid_to_gid_map(&pdf_context, &glyph_mapping)
                .expect("Should generate CID map successfully");

        // CRITICAL TEST 1: Map should be ~512 bytes (256 * 2), NOT 131,072 bytes
        assert_eq!(
            cid_to_gid_map.len(),
            512,  // (255 + 1) * 2 bytes
            "BUG: CIDToGIDMap is {} bytes, should be 512 bytes for max_cid=255",
            cid_to_gid_map.len()
        );

        // CRITICAL TEST 2: Verify specific CID mappings are correct
        // For Identity-H: CID equals original GID
        // So CID 3 should map to new GID 1 (second in our list)
        let cid_3_offset = 3 * 2;
        let gid_for_cid_3 = u16::from_be_bytes([
            cid_to_gid_map[cid_3_offset],
            cid_to_gid_map[cid_3_offset + 1]
        ]);
        assert_eq!(gid_for_cid_3, 1, "CID 3 should map to GID 1");

        // CID 200 should map to new GID 19
        let cid_200_offset = 200 * 2;
        let gid_for_cid_200 = u16::from_be_bytes([
            cid_to_gid_map[cid_200_offset],
            cid_to_gid_map[cid_200_offset + 1]
        ]);
        assert_eq!(gid_for_cid_200, 19, "CID 200 should map to GID 19");

        // CRITICAL TEST 3: Count zero mappings - should be reasonable
        let mut zero_count = 0;
        let mut non_zero_count = 0;
        
        for cid in 1..=255u16 {
            let offset = (cid as usize) * 2;
            let gid = u16::from_be_bytes([
                cid_to_gid_map[offset],
                cid_to_gid_map[offset + 1]
            ]);
            
            if gid == 0 {
                zero_count += 1;
            } else {
                non_zero_count += 1;
            }
        }
        
        let zero_percentage = (zero_count as f32 / 255.0) * 100.0;
        
        // We have 19 non-zero glyphs, so ~236 should be zero
        // That's about 92.5% zeros
        assert!(
            zero_percentage > 90.0 && zero_percentage < 95.0,
            "Expected ~92.5% zeros for 19 glyphs out of 255, got {:.1}%",
            zero_percentage
        );
        
        // Should have at least 19 non-zero mappings (our actual glyphs)
        assert!(
            non_zero_count >= 19,
            "Should have at least 19 non-zero mappings, got {}",
            non_zero_count
        );
    }

    #[test]
    fn test_calibri_regular_production_case() {
        // Case 2: Calibri Regular - 29 actually used glyphs
        let calibri_gids = vec![
            0,   // .notdef
            3, 23, 25, 34, 36, 71, 78, 79, 80, 
            87, 88, 102, 103, 104, 155, 159, 
            178, 191, 193, 194, 199, 200, 201, 223
        ];

        let pdf_context = PdfFontContext {
            encoding: FontEncoding::Identity { vertical: false },
            max_cid: Some(255),
            preserve_identity: false,
            is_symbolic: false,
            cid_to_gid_map: None,
            writing_mode: WritingMode::Horizontal,
            cmap_provider: None,
        };

        // Create mapping
        let mut glyph_mapping = HashMap::new();
        for (new_id, &old_id) in calibri_gids.iter().enumerate() {
            glyph_mapping.insert(old_id, new_id as u16);
        }

        let (cid_to_gid_map, _) = 
            crate::subset::pdf::generate_cid_to_gid_map(&pdf_context, &glyph_mapping)
                .expect("Should generate CID map");

        // Should be 512 bytes
        assert_eq!(cid_to_gid_map.len(), 512);

        // Verify some specific mappings
        // CID 223 (last in list) should map to GID 24
        let cid_223_offset = 223 * 2;
        let gid_for_cid_223 = u16::from_be_bytes([
            cid_to_gid_map[cid_223_offset],
            cid_to_gid_map[cid_223_offset + 1]
        ]);
        assert_eq!(gid_for_cid_223, 24, "CID 223 should map to GID 24");
    }

    #[test]
    fn test_edge_case_single_glyph() {
        // Case 3: Arial-BoldMT - Only 1 glyph used (edge case)
        let arial_bold_gids = vec![0]; // Only .notdef

        let pdf_context = PdfFontContext {
            encoding: FontEncoding::Identity { vertical: false },
            max_cid: Some(255),
            preserve_identity: false,
            is_symbolic: false,
            cid_to_gid_map: None,
            writing_mode: WritingMode::Horizontal,
            cmap_provider: None,
        };

        let mut glyph_mapping = HashMap::new();
        glyph_mapping.insert(0, 0);

        let (cid_to_gid_map, _) = 
            crate::subset::pdf::generate_cid_to_gid_map(&pdf_context, &glyph_mapping)
                .expect("Should generate CID map");

        // Should still be 512 bytes
        assert_eq!(cid_to_gid_map.len(), 512);

        // CID 0 should map to GID 0
        let gid_for_cid_0 = u16::from_be_bytes([
            cid_to_gid_map[0],
            cid_to_gid_map[1]
        ]);
        assert_eq!(gid_for_cid_0, 0);

        // All other CIDs should map to 0 (missing)
        for cid in 1..=255u16 {
            let offset = (cid as usize) * 2;
            let gid = u16::from_be_bytes([
                cid_to_gid_map[offset],
                cid_to_gid_map[offset + 1]
            ]);
            assert_eq!(gid, 0, "CID {} should map to 0 (missing)", cid);
        }
    }

    #[test]
    fn test_preserve_identity_workaround() {
        // Test the workaround case where preserve_identity=true
        let all_gids: Vec<u16> = (0..=255).collect();

        let pdf_context = PdfFontContext {
            encoding: FontEncoding::Identity { vertical: false },
            max_cid: Some(255),
            preserve_identity: true,  // Workaround enabled
            is_symbolic: false,
            cid_to_gid_map: None,
            writing_mode: WritingMode::Horizontal,
            cmap_provider: None,
        };

        // With preserve_identity=true, we expect identity mapping
        let mut identity_mapping = HashMap::new();
        for &gid in &all_gids {
            identity_mapping.insert(gid, gid);
        }

        let (cid_to_gid_map, _) = 
            crate::subset::pdf::generate_cid_to_gid_map(&pdf_context, &identity_mapping)
                .expect("Should generate CID map");

        // Should be 512 bytes
        assert_eq!(cid_to_gid_map.len(), 512);

        // Every CID should map to itself (identity)
        for cid in 0..=255u16 {
            let offset = (cid as usize) * 2;
            let gid = u16::from_be_bytes([
                cid_to_gid_map[offset],
                cid_to_gid_map[offset + 1]
            ]);
            assert_eq!(gid, cid, "With preserve_identity=true, CID {} should map to GID {}", cid, cid);
        }
    }

    #[test]
    fn test_verify_fix_with_smaller_max_cid() {
        // User's validation test case
        let glyph_ids = vec![0, 3, 15, 17, 25, 36, 43, 71, 79, 80];

        let pdf_context = PdfFontContext {
            encoding: FontEncoding::Identity { vertical: false },
            max_cid: Some(100),  // Smaller max_cid
            preserve_identity: false,
            is_symbolic: false,
            cid_to_gid_map: None,
            writing_mode: WritingMode::Horizontal,
            cmap_provider: None,
        };

        let mut glyph_mapping = HashMap::new();
        for (new_id, &old_id) in glyph_ids.iter().enumerate() {
            glyph_mapping.insert(old_id, new_id as u16);
        }

        let (cid_to_gid_map, _) = 
            crate::subset::pdf::generate_cid_to_gid_map(&pdf_context, &glyph_mapping)
                .expect("Should generate CID map");

        // After fix: CIDToGIDMap should be ~202 bytes (101 CIDs * 2)
        assert_eq!(
            cid_to_gid_map.len(),
            202,
            "CIDToGIDMap should be 202 bytes for max_cid=100, got {}",
            cid_to_gid_map.len()
        );

        // Check specific CID mappings
        let cid_3_offset = 3 * 2;
        let gid_for_cid_3 = u16::from_be_bytes([
            cid_to_gid_map[cid_3_offset],
            cid_to_gid_map[cid_3_offset + 1]
        ]);
        assert_eq!(gid_for_cid_3, 1, "CID 3 should map to GID 1");

        // CID 80 should map to GID 9
        let cid_80_offset = 80 * 2;
        let gid_for_cid_80 = u16::from_be_bytes([
            cid_to_gid_map[cid_80_offset],
            cid_to_gid_map[cid_80_offset + 1]
        ]);
        assert_eq!(gid_for_cid_80, 9, "CID 80 should map to GID 9");
    }

    #[test]
    fn test_production_bug_symptoms() {
        // This test verifies that the bug symptoms reported by the user are fixed
        
        // Simulate the broken case the user reported
        let glyph_ids = vec![
            0, 3, 15, 17, 25, 36, 43, 71, 79, 80, 
            88, 100, 103, 104, 144, 191, 193, 194, 199, 200
        ];

        let pdf_context = PdfFontContext {
            encoding: FontEncoding::Identity { vertical: false },
            max_cid: Some(255),
            preserve_identity: false,
            is_symbolic: false,
            cid_to_gid_map: None,
            writing_mode: WritingMode::Horizontal,
            cmap_provider: None,
        };

        let mut glyph_mapping = HashMap::new();
        for (new_id, &old_id) in glyph_ids.iter().enumerate() {
            glyph_mapping.insert(old_id, new_id as u16);
        }

        let (cid_to_gid_map, _) = 
            crate::subset::pdf::generate_cid_to_gid_map(&pdf_context, &glyph_mapping)
                .expect("Should generate CID map");

        // User reported symptoms:
        // 1. CIDToGIDMap was 131,072 bytes instead of ~512
        assert_ne!(
            cid_to_gid_map.len(),
            131072,
            "BUG NOT FIXED: CIDToGIDMap is still 131KB!"
        );
        assert_eq!(
            cid_to_gid_map.len(),
            512,
            "CIDToGIDMap should be 512 bytes"
        );

        // 2. ~95% of CIDs mapped to GID 0 (should be ~92% for this case)
        let mut zero_count = 0;
        for cid in 1..=255u16 {
            let offset = (cid as usize) * 2;
            let gid = u16::from_be_bytes([
                cid_to_gid_map[offset],
                cid_to_gid_map[offset + 1]
            ]);
            if gid == 0 {
                zero_count += 1;
            }
        }
        
        let zero_percentage = (zero_count as f32 / 255.0) * 100.0;
        
        // With 20 glyphs out of 256, we expect ~92% zeros, not 95%+
        assert!(
            zero_percentage < 95.0,
            "Too many zeros: {:.1}% (user reported 95% as bug symptom)",
            zero_percentage
        );

        // 3. Verify that used CIDs don't map to 0
        for &original_gid in &glyph_ids[1..] { // Skip .notdef
            if original_gid <= 255 {
                let offset = (original_gid as usize) * 2;
                let mapped_gid = u16::from_be_bytes([
                    cid_to_gid_map[offset],
                    cid_to_gid_map[offset + 1]
                ]);
                assert_ne!(
                    mapped_gid, 0,
                    "Used CID {} should not map to 0 (would render as □)",
                    original_gid
                );
            }
        }
    }
}