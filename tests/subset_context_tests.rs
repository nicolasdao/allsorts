#[cfg(test)]
mod phase1_context_tests {
    use allsorts::subset::context::{FontEncoding, FontContext};
    use allsorts::subset::cid_map::build_identity_cid_map;
    use std::collections::HashMap;

    // BASIC ENUM CONSTRUCTION TESTS
    #[test]
    fn test_font_encoding_identity_h() {
        // WILL FAIL: FontEncoding not implemented yet
        let encoding = FontEncoding::Identity { vertical: false };
        match encoding {
            FontEncoding::Identity { vertical } => assert!(!vertical),
        }
    }

    #[test]
    fn test_font_encoding_identity_v() {
        // WILL FAIL: FontEncoding not implemented yet
        let encoding = FontEncoding::Identity { vertical: true };
        match encoding {
            FontEncoding::Identity { vertical } => assert!(vertical),
        }
    }

    // ENCODING PARSING TESTS
    #[test]
    fn test_font_encoding_from_pdf_name_identity_h() {
        // WILL FAIL: from_pdf_name method not implemented yet
        let encoding = FontEncoding::from_pdf_name("Identity-H");
        assert_eq!(encoding, Some(FontEncoding::Identity { vertical: false }));
    }

    #[test]
    fn test_font_encoding_from_pdf_name_identity_v() {
        // WILL FAIL: from_pdf_name method not implemented yet
        let encoding = FontEncoding::from_pdf_name("Identity-V");
        assert_eq!(encoding, Some(FontEncoding::Identity { vertical: true }));
    }

    #[test]
    fn test_font_encoding_from_pdf_name_unsupported() {
        // WILL FAIL: from_pdf_name method not implemented yet
        // Phase 1: Other encodings return None
        let encoding = FontEncoding::from_pdf_name("GB-EUC-H");
        assert_eq!(encoding, None);
    }

    #[test]
    fn test_font_encoding_from_pdf_name_case_sensitive() {
        // WILL FAIL: from_pdf_name method not implemented yet
        let encoding = FontEncoding::from_pdf_name("identity-h");
        assert_eq!(encoding, None); // Should be case-sensitive
    }

    // CID REQUIREMENT TESTS
    #[test]
    fn test_font_encoding_requires_cid() {
        // WILL FAIL: requires_cid method not implemented yet
        let encoding = FontEncoding::Identity { vertical: false };
        assert!(encoding.requires_cid());
    }

    #[test]
    fn test_font_encoding_requires_cid_vertical() {
        // WILL FAIL: requires_cid method not implemented yet
        let encoding = FontEncoding::Identity { vertical: true };
        assert!(encoding.requires_cid());
    }

    // CONTEXT TESTS
    #[test]
    fn test_font_context_default() {
        // WILL FAIL: Default trait not implemented yet
        let context = FontContext::default();
        assert_eq!(context, FontContext::Unknown);
    }

    #[test]
    fn test_font_context_pdf_type0() {
        // WILL FAIL: FontContext enum not implemented yet
        let context = FontContext::PdfType0 {
            encoding: FontEncoding::Identity { vertical: false },
        };
        match context {
            FontContext::PdfType0 { encoding } => {
                assert_eq!(encoding, FontEncoding::Identity { vertical: false });
            }
            _ => panic!("Expected PdfType0 context"),
        }
    }

    // CID MAP GENERATION TESTS
    #[test]
    fn test_build_identity_cid_map_empty() {
        // WILL FAIL: build_identity_cid_map function not implemented yet
        let mapping = HashMap::new();
        let cid_map = build_identity_cid_map(&mapping, 0);
        assert_eq!(cid_map.len(), 2); // One entry (CID 0) = 2 bytes
        // CID 0 should map to GID 0 (default for unmapped)
        assert_eq!(u16::from_be_bytes([cid_map[0], cid_map[1]]), 0);
    }

    #[test]
    fn test_build_identity_cid_map_basic() {
        // WILL FAIL: build_identity_cid_map function not implemented yet
        let mut mapping = HashMap::new();
        mapping.insert(0, 0);  // .notdef
        mapping.insert(42, 1); // GID 42 -> new GID 1
        mapping.insert(100, 2); // GID 100 -> new GID 2
        
        let cid_map = build_identity_cid_map(&mapping, 100);
        
        // Should have 101 entries * 2 bytes = 202 bytes (CID 0-100)
        assert_eq!(cid_map.len(), 202);
        
        // CID 0 should map to new GID 0
        assert_eq!(u16::from_be_bytes([cid_map[0], cid_map[1]]), 0);
        
        // CID 42 should map to new GID 1
        assert_eq!(u16::from_be_bytes([cid_map[84], cid_map[85]]), 1);
        
        // CID 100 should map to new GID 2
        assert_eq!(u16::from_be_bytes([cid_map[200], cid_map[201]]), 2);
        
        // Unmapped CID 50 should map to 0 (default)
        assert_eq!(u16::from_be_bytes([cid_map[100], cid_map[101]]), 0);
    }

    #[test]
    fn test_build_identity_cid_map_large() {
        // WILL FAIL: build_identity_cid_map function not implemented yet
        let mut mapping = HashMap::new();
        for i in 0..1000u16 {
            mapping.insert(i * 2, i); // Even GIDs map to sequential new GIDs
        }
        
        let cid_map = build_identity_cid_map(&mapping, 2000);
        
        // Should have 2001 entries * 2 bytes = 4002 bytes (CID 0-2000)
        assert_eq!(cid_map.len(), 4002);
        
        // Check some mappings (CID == original GID for Identity encoding)
        assert_eq!(u16::from_be_bytes([cid_map[0], cid_map[1]]), 0);   // CID 0 -> new GID 0
        assert_eq!(u16::from_be_bytes([cid_map[200], cid_map[201]]), 50); // CID 100 -> new GID 50 (100 is even, maps to 50)
        assert_eq!(u16::from_be_bytes([cid_map[400], cid_map[401]]), 100); // CID 200 -> new GID 100 (200 is even, maps to 100)
        
        // Unmapped CID should map to 0
        assert_eq!(u16::from_be_bytes([cid_map[2], cid_map[3]]), 0);  // CID 1 (odd) -> GID 0
    }

    #[test]
    fn test_build_identity_cid_map_max_cid_boundary() {
        // WILL FAIL: build_identity_cid_map function not implemented yet
        let mut mapping = HashMap::new();
        mapping.insert(65535, 1); // Max u16 value
        
        let cid_map = build_identity_cid_map(&mapping, 65535);
        
        // Should handle max CID value
        let offset = 65535 * 2;
        assert_eq!(u16::from_be_bytes([cid_map[offset], cid_map[offset + 1]]), 1);
    }

    // MAX CID DETERMINATION TESTS
    #[test]
    fn test_determine_max_cid() {
        // WILL FAIL: determine_max_cid function not implemented yet
        use allsorts::subset::cid_map::determine_max_cid;
        
        let context = FontContext::PdfType0 {
            encoding: FontEncoding::Identity { vertical: false },
        };
        
        let glyph_ids = vec![0, 42, 100, 200];
        let max_cid = determine_max_cid(&context, &glyph_ids);
        assert_eq!(max_cid, 200);
    }

    #[test]
    fn test_determine_max_cid_empty_list() {
        // WILL FAIL: determine_max_cid function not implemented yet
        use allsorts::subset::cid_map::determine_max_cid;
        
        let context = FontContext::PdfType0 {
            encoding: FontEncoding::Identity { vertical: false },
        };
        
        let glyph_ids = vec![];
        let max_cid = determine_max_cid(&context, &glyph_ids);
        assert_eq!(max_cid, 0);
    }

    #[test]
    fn test_determine_max_cid_unknown_context() {
        // WILL FAIL: determine_max_cid function not implemented yet
        use allsorts::subset::cid_map::determine_max_cid;
        
        let context = FontContext::Unknown;
        let glyph_ids = vec![0, 42, 100];
        let max_cid = determine_max_cid(&context, &glyph_ids);
        assert_eq!(max_cid, 100); // Should use conservative approach
    }
}