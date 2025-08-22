#[cfg(test)]
mod tests {
    use crate::subset::pdf::{generate_cid_to_gid_map, PdfFontContext, WritingMode};
    use crate::subset::FontEncoding;
    use std::collections::HashMap;

    #[test]
    fn test_generate_cid_to_gid_map_respects_max_cid() {
        // Create a glyph mapping similar to what subsetting would produce
        let mut glyph_mapping = HashMap::new();
        glyph_mapping.insert(0, 0);   // .notdef
        glyph_mapping.insert(3, 1);   
        glyph_mapping.insert(15, 2);  
        glyph_mapping.insert(17, 3);  
        glyph_mapping.insert(143, 4); 
        glyph_mapping.insert(159, 5); 
        glyph_mapping.insert(178, 6); 
        glyph_mapping.insert(200, 7); 

        let pdf_context = PdfFontContext {
            encoding: FontEncoding::Identity { vertical: false },
            max_cid: Some(200),
            preserve_identity: false,
            is_symbolic: false,
            cid_to_gid_map: None,
            writing_mode: WritingMode::Horizontal,
            cmap_provider: None,
        };

        let (cid_map, _validation) = generate_cid_to_gid_map(&pdf_context, &glyph_mapping)
            .expect("Should generate CID map successfully");

        // Check size is correct (should be 402 bytes for max_cid=200, not 131072)
        assert_eq!(
            cid_map.len(),
            402,
            "CIDToGIDMap should be 402 bytes for max_cid=200, but got {} bytes",
            cid_map.len()
        );

        // Verify specific mappings
        // CID 143 should map to GID 4
        let offset_143 = 143 * 2;
        let gid_143 = u16::from_be_bytes([cid_map[offset_143], cid_map[offset_143 + 1]]);
        assert_eq!(gid_143, 4, "CID 143 should map to GID 4");

        // CID 200 should map to GID 7
        let offset_200 = 200 * 2;
        let gid_200 = u16::from_be_bytes([cid_map[offset_200], cid_map[offset_200 + 1]]);
        assert_eq!(gid_200, 7, "CID 200 should map to GID 7");
    }

    #[test]
    fn test_generate_cid_to_gid_map_with_none_max_cid() {
        let mut glyph_mapping = HashMap::new();
        glyph_mapping.insert(0, 0);
        glyph_mapping.insert(10, 1);
        glyph_mapping.insert(20, 2);

        let pdf_context = PdfFontContext {
            encoding: FontEncoding::Identity { vertical: false },
            max_cid: None, // Should default to 255
            preserve_identity: false,
            is_symbolic: false,
            cid_to_gid_map: None,
            writing_mode: WritingMode::Horizontal,
            cmap_provider: None,
        };

        let (cid_map, _validation) = generate_cid_to_gid_map(&pdf_context, &glyph_mapping)
            .expect("Should generate CID map successfully");

        // Should be 256 * 2 = 512 bytes (default max_cid=255)
        assert_eq!(
            cid_map.len(),
            512,
            "CIDToGIDMap should be 512 bytes when max_cid is None, but got {} bytes",
            cid_map.len()
        );
    }

    #[test]
    fn test_preserve_identity_flag_effect() {
        // Test 1: preserve_identity = false (normal remapping)
        let mut glyph_mapping = HashMap::new();
        glyph_mapping.insert(0, 0);
        glyph_mapping.insert(50, 1);
        glyph_mapping.insert(100, 2);

        let pdf_context_no_preserve = PdfFontContext {
            encoding: FontEncoding::Identity { vertical: false },
            max_cid: Some(100),
            preserve_identity: false,
            is_symbolic: false,
            cid_to_gid_map: None,
            writing_mode: WritingMode::Horizontal,
            cmap_provider: None,
        };

        let (cid_map, _) = generate_cid_to_gid_map(&pdf_context_no_preserve, &glyph_mapping)
            .expect("Should generate CID map");

        // CID 50 should map to GID 1 (remapped)
        let offset_50 = 50 * 2;
        let gid_50 = u16::from_be_bytes([cid_map[offset_50], cid_map[offset_50 + 1]]);
        assert_eq!(gid_50, 1, "With preserve_identity=false, CID 50 should map to remapped GID 1");

        // Test 2: preserve_identity = true (identity mapping)
        // When preserve_identity is true, the mapping should be identity (no remapping)
        // This would typically be used with all glyphs included
        let mut identity_mapping = HashMap::new();
        for i in 0..=100 {
            identity_mapping.insert(i, i); // Identity mapping
        }

        let pdf_context_preserve = PdfFontContext {
            encoding: FontEncoding::Identity { vertical: false },
            max_cid: Some(100),
            preserve_identity: true,
            is_symbolic: false,
            cid_to_gid_map: None,
            writing_mode: WritingMode::Horizontal,
            cmap_provider: None,
        };

        let (cid_map_preserve, _) = generate_cid_to_gid_map(&pdf_context_preserve, &identity_mapping)
            .expect("Should generate CID map");

        // CID 50 should map to GID 50 (identity)
        let offset_50_preserve = 50 * 2;
        let gid_50_preserve = u16::from_be_bytes([
            cid_map_preserve[offset_50_preserve], 
            cid_map_preserve[offset_50_preserve + 1]
        ]);
        assert_eq!(gid_50_preserve, 50, "With preserve_identity=true and identity mapping, CID 50 should map to GID 50");
    }
}