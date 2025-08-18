use std::collections::HashMap;
use crate::tables::FontTableProvider;
use crate::subset::{SubsetProfile, CmapTarget, subset_and_map, SubsetError};
use crate::subset::composite::update_composite_references;

/// Context for PDF font subsetting
#[derive(Debug, Clone)]
pub struct PdfFontContext {
    /// Optional existing CID to GID mapping
    pub cid_to_gid_map: Option<Vec<u16>>,
    /// Maximum CID value
    pub max_cid: u16,
    /// Whether this is a CID font
    pub is_cid_font: bool,
    /// Writing mode for the font
    pub writing_mode: WritingMode,
}

/// Writing mode for PDF fonts
#[derive(Debug, Clone, PartialEq)]
pub enum WritingMode {
    /// Horizontal writing mode
    Horizontal,
    /// Vertical writing mode
    Vertical,
}

/// Result of PDF-specific font subsetting
#[derive(Debug, Clone)]
pub struct PdfSubsetResult {
    /// The subsetted font data
    pub font_data: Vec<u8>,
    /// Mapping from old to new glyph IDs
    pub glyph_mapping: HashMap<u16, u16>,
    /// CIDToGIDMap for PDF embedding
    pub cid_to_gid_map: Vec<u8>,
    /// Validation results
    pub validation: ValidationResult,
    /// Warnings generated during subsetting
    pub warnings: Vec<PdfWarning>,
}

/// Validation results for PDF subsetting
#[derive(Debug, Clone)]
pub struct ValidationResult {
    /// Whether all CIDs are properly mapped
    pub all_cids_mapped: bool,
    /// CIDs that reference missing glyphs
    pub missing_glyph_cids: Vec<u16>,
    /// CIDs that couldn't be mapped
    pub unmapped_cids: Vec<u16>,
}

/// Warnings that can occur during PDF subsetting
#[derive(Debug, Clone)]
pub enum PdfWarning {
    /// A CID references a missing glyph
    MissingGlyph { 
        /// The CID that references a missing glyph
        cid: u16 
    },
    /// A glyph ID couldn't be mapped
    UnmappedGlyph { 
        /// The glyph ID that couldn't be mapped
        gid: u16 
    },
    /// A composite glyph has a broken component reference
    BrokenComposite { 
        /// The composite glyph ID
        glyph: u16, 
        /// The component glyph ID that is missing
        component: u16 
    },
    /// The CID map is larger than necessary
    OversizedCidMap { 
        /// The actual maximum CID in the map
        actual: u16, 
        /// The needed maximum CID
        needed: u16 
    },
}

/// Perform font subsetting optimized for PDF embedding
pub fn subset_for_pdf(
    provider: &impl FontTableProvider,
    glyph_ids: &[u16],
    pdf_context: &PdfFontContext,
) -> Result<PdfSubsetResult, SubsetError> {
    // Implementation added after tests were written
    // Step 1: Perform subsetting with mapping
    let (mut font_data, glyph_mapping) = subset_and_map(
        provider,
        glyph_ids,
        &SubsetProfile::Pdf,
        CmapTarget::Unrestricted,
    )?;
    
    // Step 2: Update composite references
    let update_stats = update_composite_references(&mut font_data, &glyph_mapping)?;
    
    // Step 3: Generate CIDToGIDMap
    let (cid_to_gid_map, validation) = generate_cid_to_gid_map(
        pdf_context,
        &glyph_mapping,
    )?;
    
    // Step 4: Collect warnings
    let mut warnings = Vec::new();
    
    // Check for missing glyphs
    for &cid in &validation.missing_glyph_cids {
        warnings.push(PdfWarning::MissingGlyph { cid });
    }
    
    // Check for unmapped references
    for &gid in &update_stats.unmapped_references {
        warnings.push(PdfWarning::UnmappedGlyph { gid });
    }
    
    // Check for oversized CID map
    if let Some(ref cid_map) = pdf_context.cid_to_gid_map {
        let max_used_cid = find_max_used_cid(cid_map, &glyph_mapping);
        if max_used_cid < pdf_context.max_cid {
            warnings.push(PdfWarning::OversizedCidMap {
                actual: pdf_context.max_cid,
                needed: max_used_cid,
            });
        }
    }
    
    Ok(PdfSubsetResult {
        font_data,
        glyph_mapping,
        cid_to_gid_map,
        validation,
        warnings,
    })
}

fn generate_cid_to_gid_map(
    context: &PdfFontContext,
    mapping: &HashMap<u16, u16>,
) -> Result<(Vec<u8>, ValidationResult), SubsetError> {
    let mut cid_map = Vec::with_capacity((context.max_cid as usize + 1) * 2);
    let missing_glyph_cids = Vec::new();
    let mut unmapped_cids = Vec::new();
    
    for cid in 0..=context.max_cid {
        // Get original GID for this CID
        let old_gid = if let Some(ref existing_map) = context.cid_to_gid_map {
            existing_map.get(cid as usize).copied().unwrap_or(cid)
        } else {
            // Identity mapping if no existing map
            cid
        };
        
        // Map to new GID
        let new_gid = mapping.get(&old_gid).copied().unwrap_or(0);
        
        // Track validation issues only for explicitly mapped CIDs
        if context.cid_to_gid_map.is_some() && old_gid != 0 && !mapping.contains_key(&old_gid) {
            unmapped_cids.push(cid);
        }
        
        // Write as big-endian for PDF
        cid_map.extend_from_slice(&new_gid.to_be_bytes());
    }
    
    let validation = ValidationResult {
        all_cids_mapped: missing_glyph_cids.is_empty() && unmapped_cids.is_empty(),
        missing_glyph_cids,
        unmapped_cids,
    };
    
    Ok((cid_map, validation))
}

fn find_max_used_cid(
    cid_to_gid_map: &[u16],
    glyph_mapping: &HashMap<u16, u16>,
) -> u16 {
    // Find highest CID that maps to a used glyph
    for (cid, &gid) in cid_to_gid_map.iter().enumerate().rev() {
        if glyph_mapping.contains_key(&gid) {
            return cid as u16;
        }
    }
    0
}