use crate::subset::composite::update_composite_references;
use crate::subset::context::FontEncoding;
use crate::subset::cjk::CMapProvider;
use crate::subset::{subset_and_map, subset_and_map_with_context, subset_with_mapping, FontContext, CmapTarget, SubsetError, SubsetProfile, SubsetResult as CoreSubsetResult};
use crate::tables::FontTableProvider;
use std::collections::HashMap;

/// PDF-specific font type classification
#[derive(Debug, Clone, PartialEq)]
pub enum PdfFontType {
    /// Simple font (Type 1, TrueType)
    Simple,
    /// CID-keyed font with CFF outlines
    CidType0,
    /// CID-keyed font with TrueType outlines
    CidType2,
}

/// Context for PDF font subsetting with Phase 2 enhancements
pub struct PdfFontContext {
    /// The encoding used for this PDF font
    pub encoding: FontEncoding,
    /// Maximum CID value
    pub max_cid: Option<u16>,
    /// Whether to preserve identity mapping
    pub preserve_identity: bool,
    /// Whether this is a symbolic font
    pub is_symbolic: bool,
    /// Optional existing CID to GID mapping (for backward compatibility)
    pub cid_to_gid_map: Option<Vec<u16>>,
    /// Writing mode for the font (for backward compatibility)
    pub writing_mode: WritingMode,
    /// Optional CMap provider for CJK encodings
    pub cmap_provider: Option<Box<dyn CMapProvider>>,
}

impl PdfFontContext {
    /// Create context for Identity-H encoding
    pub fn identity_h() -> Self {
        PdfFontContext {
            encoding: FontEncoding::Identity { vertical: false },
            max_cid: None,
            preserve_identity: false,
            is_symbolic: false,
            cid_to_gid_map: None,
            writing_mode: WritingMode::Horizontal,
            cmap_provider: None,
        }
    }
    
    /// Create context for Identity-V encoding
    pub fn identity_v() -> Self {
        PdfFontContext {
            encoding: FontEncoding::Identity { vertical: true },
            max_cid: None,
            preserve_identity: false,
            is_symbolic: false,
            cid_to_gid_map: None,
            writing_mode: WritingMode::Vertical,
            cmap_provider: None,
        }
    }
    
    /// Create context from PDF font dictionary
    pub fn from_pdf_dict(encoding_name: &str, flags: u32) -> Result<Self, SubsetError> {
        let encoding = FontEncoding::from_pdf_name(encoding_name)
            .ok_or_else(|| SubsetError::UnsupportedEncoding(encoding_name.to_string()))?;
        
        let vertical = matches!(encoding, FontEncoding::Identity { vertical: true } | FontEncoding::CJK { vertical: true, .. });
        
        Ok(PdfFontContext {
            encoding,
            max_cid: None,
            preserve_identity: false,
            is_symbolic: (flags & 0x04) != 0,  // Bit 3 is symbolic flag
            cid_to_gid_map: None,
            writing_mode: if vertical { WritingMode::Vertical } else { WritingMode::Horizontal },
            cmap_provider: None,
        })
    }
    
    /// Set maximum CID value
    pub fn with_max_cid(mut self, max_cid: u16) -> Self {
        self.max_cid = Some(max_cid);
        self
    }
    
    /// Preserve identity mapping
    pub fn preserve_identity(mut self) -> Self {
        self.preserve_identity = true;
        self
    }
    
    /// Set CMap provider for CJK encodings
    pub fn with_cmap_provider(mut self, provider: Box<dyn CMapProvider>) -> Self {
        self.cmap_provider = Some(provider);
        self
    }
    
    /// Create old-style context for backward compatibility
    pub fn legacy(max_cid: u16, _is_cid_font: bool, writing_mode: WritingMode) -> Self {
        PdfFontContext {
            encoding: FontEncoding::Identity { vertical: writing_mode == WritingMode::Vertical },
            max_cid: Some(max_cid),
            preserve_identity: false,
            is_symbolic: false,
            cid_to_gid_map: None,
            writing_mode,
            cmap_provider: None,
        }
    }
}

/// Writing mode for PDF fonts
#[derive(Debug, Clone, PartialEq)]
pub enum WritingMode {
    /// Horizontal writing mode
    Horizontal,
    /// Vertical writing mode
    Vertical,
}

/// Statistics about subsetting operation
#[derive(Debug, Clone)]
pub struct SubsetStatistics {
    /// Original number of glyphs in the font
    pub original_glyph_count: usize,
    /// Number of glyphs in the subset
    pub subset_glyph_count: usize,
    /// Estimated original size
    pub original_size_estimate: usize,
    /// Size of the subset font
    pub subset_size: usize,
    /// Size of the CID map
    pub cid_map_size: usize,
    /// Percentage reduction in size
    pub reduction_percentage: f32,
}

/// Result of PDF-specific font subsetting (enhanced with Phase 2 features)
#[derive(Debug, Clone)]
pub struct PdfSubsetResult {
    /// The subsetted font data
    pub font_data: Vec<u8>,
    /// Mapping from old to new glyph IDs
    pub glyph_mapping: HashMap<u16, u16>,
    /// CIDToGIDMap for PDF embedding
    pub cid_to_gid_map: Vec<u8>,
    /// Font type detected
    pub font_type: PdfFontType,
    /// Encoding that was used
    pub encoding_used: FontEncoding,
    /// Statistics about the subsetting
    pub statistics: SubsetStatistics,
    /// Validation results
    pub validation: ValidationResult,
    /// Warnings generated during subsetting
    pub warnings: Vec<PdfWarning>,
}

impl PdfSubsetResult {
    /// Calculate size reduction percentage
    pub fn size_reduction(&self) -> f32 {
        self.statistics.reduction_percentage
    }
    
    /// Check if this is a CID font
    pub fn is_cid_font(&self) -> bool {
        matches!(self.font_type, PdfFontType::CidType0 | PdfFontType::CidType2)
    }
    
    /// Get CIDToGIDMap if this is a CID font
    pub fn cid_to_gid_map(&self) -> Option<&[u8]> {
        if self.is_cid_font() && !self.cid_to_gid_map.is_empty() {
            Some(&self.cid_to_gid_map)
        } else {
            None
        }
    }
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
        cid: u16,
    },
    /// A glyph ID couldn't be mapped
    UnmappedGlyph {
        /// The glyph ID that couldn't be mapped
        gid: u16,
    },
    /// A composite glyph has a broken component reference
    BrokenComposite {
        /// The composite glyph ID
        glyph: u16,
        /// The component glyph ID that is missing
        component: u16,
    },
    /// The CID map is larger than necessary
    OversizedCidMap {
        /// The actual maximum CID in the map
        actual: u16,
        /// The needed maximum CID
        needed: u16,
    },
}

/// Main PDF API function for subsetting with context (Phase 2 enhancement)
pub fn subset_and_map_for_pdf<T: FontTableProvider>(
    provider: &T,
    glyph_ids: &[u16],
    pdf_context: PdfFontContext,
) -> Result<PdfSubsetResult, SubsetError> {
    // Validate input
    if glyph_ids.is_empty() {
        return Err(SubsetError::InvalidContext("Empty glyph list".to_string()));
    }
    
    if glyph_ids[0] != 0 {
        return Err(SubsetError::InvalidContext("Missing .notdef glyph at position 0".to_string()));
    }
    
    // For CJK encodings with CMap provider, we need special handling
    // because the Phase 1 API doesn't support passing CMap providers
    let (font_data, glyph_mapping, needs_cid) = if matches!(pdf_context.encoding, FontEncoding::CJK { .. }) 
        && pdf_context.cmap_provider.is_some() 
    {
        // Perform standard subsetting
        let (font_data, glyph_mapping) = subset_with_mapping(
            provider,
            glyph_ids,
            &SubsetProfile::Pdf,
            CmapTarget::Unicode,
        )?;
        (font_data, glyph_mapping, true)
    } else {
        // Convert PdfFontContext to Phase 1 FontContext for non-CJK or CJK without provider
        let context = FontContext::PdfType0 {
            encoding: pdf_context.encoding.clone(),
        };
        
        // Use Phase 1 API to perform subsetting
        let result = subset_and_map_with_context(
            provider,
            glyph_ids,
            &SubsetProfile::Pdf,
            CmapTarget::Unicode,
            context,
        )?;
        
        // Extract data from SubsetResult
        match result {
            CoreSubsetResult::Simple { font_data, glyph_mapping } => {
                (font_data, glyph_mapping, false)
            }
            CoreSubsetResult::Cid { font_data, glyph_mapping, .. } => {
                (font_data, glyph_mapping, true)
            }
        }
    };
    
    // Detect font type
    let font_type = detect_pdf_font_type(provider)?;
    
    // Generate CIDToGIDMap if needed
    let cid_to_gid_map = if needs_cid {
        let (map, _validation) = generate_cid_to_gid_map(&pdf_context, &glyph_mapping)?;
        map
    } else {
        Vec::new()
    };
    
    // Calculate statistics
    let statistics = calculate_statistics(
        provider,
        &font_data,
        glyph_ids.len(),
        &glyph_mapping,
        &cid_to_gid_map,
    )?;
    
    // Build validation result
    let validation = ValidationResult {
        all_cids_mapped: true,  // Simplified for now
        missing_glyph_cids: Vec::new(),
        unmapped_cids: Vec::new(),
    };
    
    Ok(PdfSubsetResult {
        font_data,
        glyph_mapping,
        cid_to_gid_map,
        font_type,
        encoding_used: pdf_context.encoding,
        statistics,
        validation,
        warnings: Vec::new(),
    })
}

/// Legacy PDF subsetting function (kept for backward compatibility)
pub fn subset_for_pdf(
    provider: &impl FontTableProvider,
    glyph_ids: &[u16],
    pdf_context: &PdfFontContext,
) -> Result<PdfSubsetResult, SubsetError> {
    // Implementation added after tests were written
    // Step 1: Perform subsetting with mapping
    let result = subset_and_map(
        provider,
        glyph_ids,
        &SubsetProfile::Pdf,
        CmapTarget::Unrestricted,
    )?;

    // Extract font data and mapping from SubsetResult
    let (mut font_data, glyph_mapping) = match result {
        crate::subset::SubsetResult::Simple {
            font_data,
            glyph_mapping,
        } => (font_data, glyph_mapping),
        crate::subset::SubsetResult::Cid {
            font_data,
            glyph_mapping,
            ..
        } => (font_data, glyph_mapping),
    };

    // Step 2: Update composite references
    let update_stats = update_composite_references(&mut font_data, &glyph_mapping)?;

    // Step 3: Generate CIDToGIDMap
    let (cid_to_gid_map, validation) = generate_cid_to_gid_map(pdf_context, &glyph_mapping)?;

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
        if let Some(max_cid) = pdf_context.max_cid {
            let max_used_cid = find_max_used_cid(cid_map, &glyph_mapping);
            if max_used_cid < max_cid {
                warnings.push(PdfWarning::OversizedCidMap {
                    actual: max_cid,
                    needed: max_used_cid,
                });
            }
        }
    }

    // Detect font type
    let font_type = detect_pdf_font_type(provider)?;
    
    // Calculate statistics  
    let statistics = SubsetStatistics {
        original_glyph_count: 0,  // Would need proper calculation
        subset_glyph_count: glyph_mapping.len(),
        original_size_estimate: 0,
        subset_size: font_data.len(),
        cid_map_size: cid_to_gid_map.len(),
        reduction_percentage: 0.0,
    };

    Ok(PdfSubsetResult {
        font_data,
        glyph_mapping,
        cid_to_gid_map,
        font_type,
        encoding_used: pdf_context.encoding.clone(),
        statistics,
        validation,
        warnings,
    })
}

fn generate_cid_to_gid_map(
    context: &PdfFontContext,
    mapping: &HashMap<u16, u16>,
) -> Result<(Vec<u8>, ValidationResult), SubsetError> {
    let max_cid = context.max_cid.unwrap_or(255);  // Default to 255 if not specified
    
    // Use encoding-specific CID map generation if possible
    let cid_map = match &context.encoding {
        FontEncoding::CJK { .. } => {
            // Use CJK-aware CID mapping with CMap provider if available
            use crate::subset::cjk::build_cjk_cid_map;
            build_cjk_cid_map(
                &context.encoding,
                mapping,
                max_cid,
                context.cmap_provider.as_deref(),
            )?
        }
        _ => {
            // Use standard CID map generation for Identity and other encodings
            use crate::subset::cid_map::build_cid_to_gid_map_for_encoding;
            build_cid_to_gid_map_for_encoding(&context.encoding, mapping, max_cid)?
        }
    };
    
    // Simple validation for now
    let validation = ValidationResult {
        all_cids_mapped: true,
        missing_glyph_cids: Vec::new(),
        unmapped_cids: Vec::new(),
    };

    Ok((cid_map, validation))
}

fn find_max_used_cid(cid_to_gid_map: &[u16], glyph_mapping: &HashMap<u16, u16>) -> u16 {
    // Find highest CID that maps to a used glyph
    for (cid, &gid) in cid_to_gid_map.iter().enumerate().rev() {
        if glyph_mapping.contains_key(&gid) {
            return cid as u16;
        }
    }
    0
}

/// Detect PDF font type from provider
fn detect_pdf_font_type<T: FontTableProvider>(provider: &T) -> Result<PdfFontType, SubsetError> {
    // Check for CFF table (Type 0)
    if let Ok(Some(_)) = provider.table_data(crate::tag::CFF) {
        return Ok(PdfFontType::CidType0);
    }
    
    // Check for glyf table (TrueType)
    if let Ok(Some(_)) = provider.table_data(crate::tag::GLYF) {
        // For now, assume CID Type 2 for TrueType with Identity encoding
        // In reality, this would need more sophisticated detection
        return Ok(PdfFontType::CidType2);
    }
    
    // Default to Simple
    Ok(PdfFontType::Simple)
}

/// Calculate statistics for the subsetting operation
fn calculate_statistics<T: FontTableProvider>(
    provider: &T,
    subset_data: &[u8],
    requested_glyphs: usize,
    glyph_mapping: &HashMap<u16, u16>,
    cid_map: &[u8],
) -> Result<SubsetStatistics, SubsetError> {
    // Get original glyph count from maxp table
    let maxp_data = provider.table_data(crate::tag::MAXP)
        .map_err(|e| SubsetError::Parse(e.into()))?
        .ok_or_else(|| SubsetError::Parse(crate::error::ParseError::MissingValue))?;
    
    // Read glyph count from maxp (offset 4, 2 bytes)
    let original_glyph_count = if maxp_data.len() >= 6 {
        u16::from_be_bytes([maxp_data[4], maxp_data[5]]) as usize
    } else {
        0
    };
    
    // Estimate original size (sum of all table sizes)
    let mut original_size_estimate = 0;
    for tag in &[
        crate::tag::CFF,
        crate::tag::GLYF,
        crate::tag::LOCA,
        crate::tag::HEAD,
        crate::tag::HHEA,
        crate::tag::HMTX,
        crate::tag::MAXP,
        crate::tag::NAME,
        crate::tag::POST,
        crate::tag::CMAP,
    ] {
        if let Ok(Some(data)) = provider.table_data(*tag) {
            original_size_estimate += data.len();
        }
    }
    
    let subset_size = subset_data.len();
    let cid_map_size = cid_map.len();
    
    let reduction_percentage = if original_size_estimate > 0 && original_size_estimate > subset_size {
        ((original_size_estimate - subset_size) as f32 / original_size_estimate as f32) * 100.0
    } else if original_size_estimate > 0 && subset_size > original_size_estimate {
        // If subset is larger, return negative percentage
        -((subset_size - original_size_estimate) as f32 / original_size_estimate as f32) * 100.0
    } else {
        0.0
    };
    
    Ok(SubsetStatistics {
        original_glyph_count,
        subset_glyph_count: glyph_mapping.len(),
        original_size_estimate,
        subset_size,
        cid_map_size,
        reduction_percentage,
    })
}

