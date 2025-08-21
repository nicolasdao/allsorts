//! Chinese encoding support for font subsetting

use crate::subset::SubsetError;
use std::collections::HashMap;

/// Build JIS to Unicode mapping (placeholder for Chinese module)
/// Note: This is actually for Japanese, but included here temporarily
pub fn build_jis_unicode_map(
    map: &mut [u8],
    glyph_mapping: &HashMap<u16, u16>,
    max_cid: u16,
) -> Result<(), SubsetError> {
    // JIS X 0208 mapping
    // Row 1-8: Symbols
    // Row 9-15: Hiragana/Katakana
    // Row 16-47: Level 1 Kanji
    // Row 48-84: Level 2 Kanji
    
    for cid in 0..=max_cid.min(8836) {  // JIS has 8836 characters
        // In a real implementation:
        // let unicode = jis_to_unicode(cid)?;
        // Would need font's cmap to convert Unicode to GID
        // Then use glyph_mapping to get new GID
        
        // Simplified here
        if let Some(&new_gid) = glyph_mapping.get(&cid) {
            let offset = (cid as usize) * 2;
            map[offset..offset + 2].copy_from_slice(&new_gid.to_be_bytes());
        }
    }
    
    Ok(())
}

/// Handle Simplified Chinese encodings
pub fn build_gb_map(
    encoding_name: &str,
    glyph_mapping: &HashMap<u16, u16>,
    max_cid: u16,
) -> Result<Vec<u8>, SubsetError> {
    let mut map = vec![0u8; ((max_cid as usize) + 1) * 2];
    
    match encoding_name {
        "GB-EUC-H" | "GB-EUC-V" => {
            // GB2312 encoding
            for cid in 0..=max_cid {
                let gid = gb2312_to_gid(cid)?;
                if let Some(&new_gid) = glyph_mapping.get(&gid) {
                    let offset = (cid as usize) * 2;
                    map[offset..offset + 2].copy_from_slice(&new_gid.to_be_bytes());
                }
            }
        }
        "GBK-EUC-H" | "GBK-EUC-V" => {
            // GBK encoding (superset of GB2312)
            for cid in 0..=max_cid {
                let gid = gbk_to_gid(cid)?;
                if let Some(&new_gid) = glyph_mapping.get(&gid) {
                    let offset = (cid as usize) * 2;
                    map[offset..offset + 2].copy_from_slice(&new_gid.to_be_bytes());
                }
            }
        }
        _ => return Err(SubsetError::UnsupportedEncoding(encoding_name.to_string())),
    }
    
    Ok(map)
}

/// Convert GB2312 code to glyph ID
fn gb2312_to_gid(gb_code: u16) -> Result<u16, SubsetError> {
    // GB2312 has a specific mapping table
    // Area 1-9: Symbols and punctuation
    // Area 16-55: Level 1 Hanzi (3755 chars)
    // Area 56-87: Level 2 Hanzi (3008 chars)
    
    let area = (gb_code >> 8) & 0xFF;
    let position = gb_code & 0xFF;
    
    if area >= 0xA1 && area <= 0xA9 {
        // Symbol area
        Ok(((area - 0xA1) * 94 + (position - 0xA1)) as u16)
    } else if area >= 0xB0 && area <= 0xF7 {
        // Hanzi area
        let base = 846;  // After symbols
        Ok((base + (area - 0xB0) * 94 + (position - 0xA1)) as u16)
    } else {
        Ok(0)  // Map to .notdef
    }
}

/// Convert GBK code to glyph ID
fn gbk_to_gid(gbk_code: u16) -> Result<u16, SubsetError> {
    // GBK is a superset of GB2312
    // For simplicity, we'll use the same mapping as GB2312
    // In reality, GBK has additional characters
    gb2312_to_gid(gbk_code)
}