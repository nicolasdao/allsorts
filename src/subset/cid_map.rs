//! CIDToGIDMap generation for font subsetting

use std::collections::HashMap;
use crate::subset::context::{FontContext, FontEncoding};
use crate::subset::SubsetError;


/// Build a CIDToGIDMap for Identity encodings
/// 
/// For Identity-H/V encodings, CID values equal the original GID values.
/// This function creates a map from CIDs to the new (remapped) GID values.
///
/// # Arguments
/// * `glyph_mapping` - HashMap of original GID to new GID
/// * `max_cid` - Maximum CID value to include in the map
///
/// # Returns
/// Binary CIDToGIDMap in big-endian format (2 bytes per entry)
pub fn build_identity_cid_map(
    glyph_mapping: &HashMap<u16, u16>,
    max_cid: u16,
) -> Vec<u8> {
    let map_size = ((max_cid as usize) + 1) * 2;
    let mut cid_to_gid_map = vec![0u8; map_size];
    
    // For Identity encoding: CID == original GID
    // So we map: CID -> glyph_mapping[CID]
    for cid in 0..=max_cid {
        let new_gid = glyph_mapping.get(&cid).copied().unwrap_or(0);
        let offset = (cid as usize) * 2;
        cid_to_gid_map[offset..offset + 2].copy_from_slice(&new_gid.to_be_bytes());
    }
    
    cid_to_gid_map
}

/// Build CIDToGIDMap based on encoding type
pub fn build_cid_to_gid_map_for_encoding(
    encoding: &FontEncoding,
    glyph_mapping: &HashMap<u16, u16>,
    max_cid: u16,
) -> Result<Vec<u8>, SubsetError> {
    match encoding {
        FontEncoding::Identity { .. } => {
            Ok(build_identity_cid_map(glyph_mapping, max_cid))
        }
        FontEncoding::CJK { .. } => {
            // CJK encodings require special handling with CMap data
            // This should be handled by the caller (e.g., pdf.rs) which has access to CMap provider
            // For contexts without CMap provider, we'll use a simplified approach
            use crate::subset::cjk::build_cjk_cid_map;
            
            // Use None for CMap provider - the CJK module will handle encodings that don't require it
            // Encodings that require CMap data will return an error
            build_cjk_cid_map(encoding, glyph_mapping, max_cid, None)
        }
        FontEncoding::AdobeCollection { .. } => {
            // Adobe collections will be handled similarly to CJK
            Err(SubsetError::UnsupportedEncoding("Adobe collections not yet implemented".to_string()))
        }
        FontEncoding::Custom(_) => {
            // Custom encodings are not supported for CID mapping
            Err(SubsetError::UnsupportedEncoding("Custom encodings not supported".to_string()))
        }
    }
}

/// Determine the maximum CID value based on context
pub fn determine_max_cid(
    context: &FontContext,
    glyph_ids: &[u16],
) -> u16 {
    match context {
        FontContext::PdfType0 { encoding } => {
            match encoding {
                FontEncoding::Identity { .. } => {
                    // For Identity, max CID equals max original GID
                    glyph_ids.iter().copied().max().unwrap_or(0)
                }
                FontEncoding::CJK { .. } | FontEncoding::AdobeCollection { .. } => {
                    // For CJK and Adobe collections, we'll need to consult CMap data
                    // For now, use a conservative estimate
                    glyph_ids.iter().copied().max().unwrap_or(0).max(8000)
                }
                FontEncoding::Custom(_) => {
                    // Conservative: use the highest glyph ID
                    glyph_ids.iter().copied().max().unwrap_or(0)
                }
            }
        }
        FontContext::Unknown => {
            // Conservative: use the highest glyph ID
            glyph_ids.iter().copied().max().unwrap_or(0)
        }
    }
}

#[cfg(test)]
mod tests {
    use super::{build_identity_cid_map, build_cid_to_gid_map_for_encoding};
    use crate::subset::context::FontEncoding;
    use std::collections::HashMap;

    #[test]
    fn test_identity_cid_map_respects_max_cid() {
        // Create a glyph mapping: old GID -> new GID
        let mut glyph_mapping = HashMap::new();
        glyph_mapping.insert(0, 0);   // .notdef
        glyph_mapping.insert(3, 1);   
        glyph_mapping.insert(15, 2);  
        glyph_mapping.insert(17, 3);  
        glyph_mapping.insert(143, 4); 
        glyph_mapping.insert(159, 5); 
        glyph_mapping.insert(178, 6); 
        glyph_mapping.insert(200, 7); 

        let max_cid = 200;
        
        // Build CIDToGIDMap
        let cid_map = build_identity_cid_map(&glyph_mapping, max_cid);
        
        // Check size is correct
        assert_eq!(
            cid_map.len(),
            (max_cid as usize + 1) * 2,
            "CIDToGIDMap should be {} bytes for max_cid={}, but got {} bytes",
            (max_cid as usize + 1) * 2,
            max_cid,
            cid_map.len()
        );
        
        // Check specific mappings
        // CID 0 -> GID 0
        assert_eq!(u16::from_be_bytes([cid_map[0], cid_map[1]]), 0);
        // CID 3 -> GID 1
        assert_eq!(u16::from_be_bytes([cid_map[6], cid_map[7]]), 1);
        // CID 15 -> GID 2
        assert_eq!(u16::from_be_bytes([cid_map[30], cid_map[31]]), 2);
        // CID 143 -> GID 4
        assert_eq!(u16::from_be_bytes([cid_map[286], cid_map[287]]), 4);
        // CID 200 -> GID 7
        assert_eq!(u16::from_be_bytes([cid_map[400], cid_map[401]]), 7);
        
        // Check unmapped CID returns 0
        // CID 100 (not in mapping) -> GID 0
        assert_eq!(u16::from_be_bytes([cid_map[200], cid_map[201]]), 0);
    }

    #[test]
    fn test_cid_map_not_oversized() {
        // Test that CIDToGIDMap is not 65536 entries
        let mut glyph_mapping = HashMap::new();
        glyph_mapping.insert(0, 0);
        glyph_mapping.insert(10, 1);
        glyph_mapping.insert(20, 2);

        let max_cid = 50;
        let cid_map = build_identity_cid_map(&glyph_mapping, max_cid);
        
        // Should be 51 * 2 = 102 bytes, not 131072 bytes
        assert_eq!(cid_map.len(), 102);
        assert_ne!(cid_map.len(), 131072, "CIDToGIDMap should not be 131072 bytes");
    }

    #[test]
    fn test_encoding_specific_cid_map() {
        let mut glyph_mapping = HashMap::new();
        glyph_mapping.insert(0, 0);
        glyph_mapping.insert(143, 1);
        glyph_mapping.insert(159, 2);
        glyph_mapping.insert(178, 3);

        let encoding = FontEncoding::Identity { vertical: false };
        let max_cid = 200;

        let cid_map = build_cid_to_gid_map_for_encoding(
            &encoding,
            &glyph_mapping,
            max_cid,
        ).expect("Should build CID map successfully");

        // Check size
        assert_eq!(cid_map.len(), (max_cid as usize + 1) * 2);
        
        // Check mappings for Identity encoding (CID = original GID)
        // CID 143 should map to new GID 1
        let offset_143 = 143 * 2;
        assert_eq!(
            u16::from_be_bytes([cid_map[offset_143], cid_map[offset_143 + 1]]),
            1,
            "CID 143 should map to GID 1"
        );
    }

    #[test]
    fn test_zero_mappings_percentage() {
        // Create sparse mapping to test zero percentage
        let mut glyph_mapping = HashMap::new();
        glyph_mapping.insert(0, 0);
        for i in 1..=20 {
            glyph_mapping.insert(i * 10, i as u16);
        }

        let max_cid = 200;
        let cid_map = build_identity_cid_map(&glyph_mapping, max_cid);

        // Count zeros (excluding CID 0)
        let mut zero_count = 0;
        for cid in 1..=max_cid {
            let offset = (cid as usize) * 2;
            let gid = u16::from_be_bytes([cid_map[offset], cid_map[offset + 1]]);
            if gid == 0 {
                zero_count += 1;
            }
        }

        // We mapped 20 glyphs out of 200, so ~180 should be zero
        let zero_percentage = (zero_count as f32 / max_cid as f32) * 100.0;
        assert!(
            zero_percentage > 85.0 && zero_percentage < 95.0,
            "Expected ~90% zeros, got {:.1}%",
            zero_percentage
        );
    }
}