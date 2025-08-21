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
            // For now, CJK encodings will be handled in a future phase
            // This is a placeholder that returns an error
            Err(SubsetError::UnsupportedEncoding("CJK encodings not yet implemented".to_string()))
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