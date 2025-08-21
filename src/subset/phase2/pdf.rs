//! PDF-specific font subsetting features for Phase 2

use crate::subset::context::{FontContext, FontEncoding};
use crate::subset::{subset_and_map_with_context, CmapTarget, SubsetError, SubsetProfile, SubsetResult};
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

/// Context for PDF font subsetting
#[derive(Debug, Clone, PartialEq)]
pub struct PdfFontContext {
    /// The encoding used for this PDF font
    pub encoding: FontEncoding,
    /// Maximum CID value
    pub max_cid: Option<u16>,
    /// Whether to preserve identity mapping
    pub preserve_identity: bool,
    /// Whether this is a symbolic font
    pub is_symbolic: bool,
}

impl PdfFontContext {
    /// Create context for Identity-H encoding
    pub fn identity_h() -> Self {
        PdfFontContext {
            encoding: FontEncoding::Identity { vertical: false },
            max_cid: None,
            preserve_identity: false,
            is_symbolic: false,
        }
    }
    
    /// Create context for Identity-V encoding
    pub fn identity_v() -> Self {
        PdfFontContext {
            encoding: FontEncoding::Identity { vertical: true },
            max_cid: None,
            preserve_identity: false,
            is_symbolic: false,
        }
    }
    
    /// Create context from PDF font dictionary
    pub fn from_pdf_dict(encoding_name: &str, flags: u32) -> Result<Self, SubsetError> {
        let encoding = FontEncoding::from_pdf_name(encoding_name)
            .ok_or_else(|| SubsetError::UnsupportedEncoding(encoding_name.to_string()))?;
        
        Ok(PdfFontContext {
            encoding,
            max_cid: None,
            preserve_identity: false,
            is_symbolic: (flags & 0x04) != 0,  // Bit 3 is symbolic flag
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

/// Result of PDF-specific subsetting
#[derive(Debug, Clone)]
pub struct PdfSubsetResult {
    /// The subset font data
    pub font_data: Vec<u8>,
    /// Mapping from old to new glyph IDs
    pub glyph_mapping: HashMap<u16, u16>,
    /// CIDToGIDMap data
    pub cid_to_gid_map: Vec<u8>,
    /// Font type detected
    pub font_type: PdfFontType,
    /// Encoding that was used
    pub encoding_used: FontEncoding,
    /// Statistics about the subsetting
    pub statistics: SubsetStatistics,
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

/// Main PDF API function for subsetting with context
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
    
    // Convert PdfFontContext to Phase 1 FontContext
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
    let (font_data, glyph_mapping, cid_to_gid_map) = match result {
        SubsetResult::Simple { font_data, glyph_mapping } => {
            (font_data, glyph_mapping, Vec::new())
        }
        SubsetResult::Cid { font_data, glyph_mapping, cid_to_gid_map } => {
            (font_data, glyph_mapping, cid_to_gid_map)
        }
    };
    
    // Detect font type
    let font_type = detect_pdf_font_type(provider)?;
    
    // Calculate statistics
    let statistics = calculate_statistics(
        provider,
        &font_data,
        glyph_ids.len(),
        &glyph_mapping,
        &cid_to_gid_map,
    )?;
    
    Ok(PdfSubsetResult {
        font_data,
        glyph_mapping,
        cid_to_gid_map,
        font_type,
        encoding_used: pdf_context.encoding,
        statistics,
    })
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

// Enhanced error handling
impl SubsetError {
    /// Create an unsupported encoding error
    pub fn unsupported_encoding(encoding: impl Into<String>) -> Self {
        SubsetError::UnsupportedEncoding(encoding.into())
    }
    
    /// Create an invalid context error
    pub fn invalid_context(msg: impl Into<String>) -> Self {
        SubsetError::InvalidContext(msg.into())
    }
}

// Enhanced error helper methods are defined above
// Display implementation will be updated in subset.rs