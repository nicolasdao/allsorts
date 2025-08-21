//! Korean encoding support for font subsetting

use crate::subset::SubsetError;
use std::collections::HashMap;

/// Handle Korean encodings
pub fn build_korean_map(
    encoding_name: &str,
    glyph_mapping: &HashMap<u16, u16>,
    max_cid: u16,
) -> Result<Vec<u8>, SubsetError> {
    let mut map = vec![0u8; ((max_cid as usize) + 1) * 2];
    
    match encoding_name {
        "KSCms-UHC-H" | "KSCms-UHC-V" => {
            // Unified Hangul Code
            for cid in 0..=max_cid {
                let gid = uhc_to_gid(cid)?;
                if let Some(&new_gid) = glyph_mapping.get(&gid) {
                    let offset = (cid as usize) * 2;
                    map[offset..offset + 2].copy_from_slice(&new_gid.to_be_bytes());
                }
            }
        }
        "UniKS-UTF16-H" | "UniKS-UTF16-V" => {
            // Unicode-based Korean
            build_ksc_unicode_map(&mut map, glyph_mapping, max_cid)?;
        }
        _ => return Err(SubsetError::UnsupportedEncoding(encoding_name.to_string())),
    }
    
    Ok(map)
}

/// Build KSC to Unicode mapping
pub fn build_ksc_unicode_map(
    map: &mut [u8],
    glyph_mapping: &HashMap<u16, u16>,
    max_cid: u16,
) -> Result<(), SubsetError> {
    // KSC5601 structure:
    // - Hangul syllables: 2350 chars
    // - Hanja (Chinese chars): 4888 chars
    // - Special symbols
    
    for cid in 0..=max_cid {
        let unicode = if cid < 2350 {
            // Hangul syllable
            0xAC00 + cid as u32  // Hangul Syllables block
        } else if cid < 7238 {
            // Hanja
            ksc_hanja_to_unicode(cid - 2350)?
        } else {
            // Symbols
            ksc_symbol_to_unicode(cid - 7238)?
        };
        
        // Simplified - would need font's cmap
        if let Some(&new_gid) = glyph_mapping.get(&cid) {
            let offset = (cid as usize) * 2;
            map[offset..offset + 2].copy_from_slice(&new_gid.to_be_bytes());
        }
    }
    
    Ok(())
}

/// Convert UHC code to glyph ID
fn uhc_to_gid(uhc_code: u16) -> Result<u16, SubsetError> {
    // Simplified UHC mapping
    Ok(uhc_code)  // Placeholder
}

/// Convert KSC Hanja to Unicode
fn ksc_hanja_to_unicode(hanja_idx: u16) -> Result<u32, SubsetError> {
    // Simplified mapping - would use KSC5601 tables
    Ok(0x4E00 + hanja_idx as u32)  // Map to CJK Unified Ideographs
}

/// Convert KSC symbol to Unicode
fn ksc_symbol_to_unicode(symbol_idx: u16) -> Result<u32, SubsetError> {
    // Simplified mapping
    Ok(0x3000 + symbol_idx as u32)  // Map to CJK symbols range
}