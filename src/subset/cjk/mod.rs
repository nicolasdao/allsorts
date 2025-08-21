//! CJK font encoding support and CMap data handling

use crate::subset::context::{FontEncoding, CJKLanguage};
use crate::subset::SubsetError;
use std::collections::HashMap;
use std::path::{Path, PathBuf};

pub mod chinese;
pub mod japanese;
pub mod korean;
pub mod cmap_data;

/// CMap data provider trait
pub trait CMapProvider {
    /// Get CMap data for encoding
    fn get_cmap_data(&self, encoding_name: &str) -> Option<&[u8]>;
    
    /// Check if CMap is available
    fn has_cmap(&self, encoding_name: &str) -> bool;
}

/// Built-in CMap provider with common encodings
pub struct BuiltinCMapProvider {
    cmaps: HashMap<String, Vec<u8>>,
}

impl BuiltinCMapProvider {
    /// Create a new builtin CMap provider
    pub fn new() -> Self {
        let mut cmaps = HashMap::new();
        
        // For now, we'll use placeholder data
        // In a real implementation, these would be actual CMap files
        // embedded as resources or loaded from files
        
        // Chinese CMaps
        cmaps.insert("GB-EUC-H".to_string(), b"<placeholder GB-EUC-H CMap data>".to_vec());
        cmaps.insert("GB-EUC-V".to_string(), b"<placeholder GB-EUC-V CMap data>".to_vec());
        cmaps.insert("GBK-EUC-H".to_string(), b"<placeholder GBK-EUC-H CMap data>".to_vec());
        cmaps.insert("GBK-EUC-V".to_string(), b"<placeholder GBK-EUC-V CMap data>".to_vec());
        
        // Japanese CMaps  
        cmaps.insert("90ms-RKSJ-H".to_string(), b"<placeholder 90ms-RKSJ-H CMap data>".to_vec());
        cmaps.insert("90ms-RKSJ-V".to_string(), b"<placeholder 90ms-RKSJ-V CMap data>".to_vec());
        cmaps.insert("EUC-H".to_string(), b"<placeholder EUC-H CMap data>".to_vec());
        cmaps.insert("EUC-V".to_string(), b"<placeholder EUC-V CMap data>".to_vec());
        cmaps.insert("H".to_string(), b"<placeholder H CMap data>".to_vec());
        cmaps.insert("V".to_string(), b"<placeholder V CMap data>".to_vec());
        
        // Korean CMaps
        cmaps.insert("KSCms-UHC-H".to_string(), b"<placeholder KSCms-UHC-H CMap data>".to_vec());
        cmaps.insert("KSCms-UHC-V".to_string(), b"<placeholder KSCms-UHC-V CMap data>".to_vec());
        cmaps.insert("KSC-EUC-H".to_string(), b"<placeholder KSC-EUC-H CMap data>".to_vec());
        cmaps.insert("KSC-EUC-V".to_string(), b"<placeholder KSC-EUC-V CMap data>".to_vec());
        
        BuiltinCMapProvider { cmaps }
    }
}

impl CMapProvider for BuiltinCMapProvider {
    fn get_cmap_data(&self, encoding_name: &str) -> Option<&[u8]> {
        self.cmaps.get(encoding_name).map(|v| v.as_slice())
    }
    
    fn has_cmap(&self, encoding_name: &str) -> bool {
        self.cmaps.contains_key(encoding_name)
    }
}

/// External CMap provider (loads from files)
pub struct FileCMapProvider {
    cmap_dir: PathBuf,
    cache: HashMap<String, Vec<u8>>,
}

impl FileCMapProvider {
    /// Create a new file-based CMap provider
    pub fn new<P: AsRef<Path>>(cmap_dir: P) -> Self {
        FileCMapProvider {
            cmap_dir: cmap_dir.as_ref().to_path_buf(),
            cache: HashMap::new(),
        }
    }
    
    fn load_cmap(&mut self, encoding_name: &str) -> Option<Vec<u8>> {
        // Convert encoding name to filename
        // e.g., "GB-EUC-H" -> "gb_euc_h.cmap"
        let file_name = format!("{}.cmap", encoding_name.to_lowercase().replace('-', "_"));
        let path = self.cmap_dir.join(file_name);
        
        // Try to read the file
        std::fs::read(path).ok()
    }
}

impl CMapProvider for FileCMapProvider {
    fn get_cmap_data(&self, encoding_name: &str) -> Option<&[u8]> {
        // For simplicity, we're not caching in this implementation
        // In a real implementation, you'd want to cache loaded files
        None
    }
    
    fn has_cmap(&self, encoding_name: &str) -> bool {
        let file_name = format!("{}.cmap", encoding_name.to_lowercase().replace('-', "_"));
        self.cmap_dir.join(file_name).exists()
    }
}

/// Build CIDToGIDMap for CJK encodings
pub fn build_cjk_cid_map(
    encoding: &FontEncoding,
    glyph_mapping: &HashMap<u16, u16>,
    max_cid: u16,
    cmap_provider: Option<&dyn CMapProvider>,
) -> Result<Vec<u8>, SubsetError> {
    match encoding {
        FontEncoding::CJK { encoding_name, requires_cmap_data, .. } => {
            if *requires_cmap_data {
                // Need external CMap data
                let provider = cmap_provider
                    .ok_or_else(|| SubsetError::CidGenerationFailed(
                        format!("CMap data required for {}", encoding_name)
                    ))?;
                    
                let cmap_data = provider.get_cmap_data(encoding_name)
                    .ok_or_else(|| SubsetError::CidGenerationFailed(
                        format!("CMap {} not found", encoding_name)
                    ))?;
                    
                build_from_cmap_data(cmap_data, glyph_mapping, max_cid)
            } else {
                // Unicode-based CJK encoding
                build_unicode_cjk_map(encoding, glyph_mapping, max_cid)
            }
        }
        _ => Err(SubsetError::InvalidContext("Not a CJK encoding".to_string())),
    }
}

/// Build map from CMap data
pub fn build_from_cmap_data(
    cmap_data: &[u8],
    glyph_mapping: &HashMap<u16, u16>,
    max_cid: u16,
) -> Result<Vec<u8>, SubsetError> {
    // This is a simplified implementation
    // In reality, we'd need to parse the CMap format
    
    // For now, just create a simple identity-like mapping as a placeholder
    let mut map = vec![0u8; ((max_cid as usize) + 1) * 2];
    
    for cid in 0..=max_cid {
        // In a real implementation, we'd:
        // 1. Parse the CMap to get CID->Unicode mappings
        // 2. Use the font's cmap to convert Unicode->GID
        // 3. Apply the glyph_mapping to get the new GID
        
        // For now, just use the glyph mapping directly if available
        let new_gid = glyph_mapping.get(&cid).copied().unwrap_or(0);
        let offset = (cid as usize) * 2;
        map[offset..offset + 2].copy_from_slice(&new_gid.to_be_bytes());
    }
    
    Ok(map)
}

/// Build map for Unicode-based CJK encodings
fn build_unicode_cjk_map(
    encoding: &FontEncoding,
    glyph_mapping: &HashMap<u16, u16>,
    max_cid: u16,
) -> Result<Vec<u8>, SubsetError> {
    let mut map = vec![0u8; ((max_cid as usize) + 1) * 2];
    
    // For Unicode-based encodings, we can compute the mapping
    match encoding {
        FontEncoding::CJK { language, encoding_name, .. } => {
            match language {
                CJKLanguage::Chinese(_) if encoding_name.starts_with("UniGB") => {
                    // UniGB uses Unicode code points as CIDs
                    build_unicode_direct_map(&mut map, glyph_mapping, max_cid);
                }
                CJKLanguage::Japanese(_) if encoding_name.starts_with("UniJIS") => {
                    // UniJIS maps JIS to Unicode
                    chinese::build_jis_unicode_map(&mut map, glyph_mapping, max_cid)?;
                }
                CJKLanguage::Korean(_) if encoding_name.starts_with("UniKS") => {
                    // UniKS maps KSC to Unicode
                    korean::build_ksc_unicode_map(&mut map, glyph_mapping, max_cid)?;
                }
                _ => {
                    return Err(SubsetError::UnsupportedEncoding(
                        encoding_name.to_string()
                    ));
                }
            }
        }
        _ => unreachable!(),
    }
    
    Ok(map)
}

/// Build a direct Unicode mapping
fn build_unicode_direct_map(
    map: &mut [u8],
    glyph_mapping: &HashMap<u16, u16>,
    max_cid: u16,
) {
    for cid in 0..=max_cid {
        // For Unicode-based encodings, CID often equals Unicode code point
        // We'd need the font's cmap to convert Unicode to original GID
        // Then apply glyph_mapping to get new GID
        
        // Simplified: just use glyph_mapping if available
        let new_gid = glyph_mapping.get(&cid).copied().unwrap_or(0);
        let offset = (cid as usize) * 2;
        map[offset..offset + 2].copy_from_slice(&new_gid.to_be_bytes());
    }
}