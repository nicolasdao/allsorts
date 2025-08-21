use crate::binary::read::ReadScope;
use crate::error::ParseError;
use crate::subset::pdf::{PdfFontContext, WritingMode};
use crate::subset::result::SubsetResult;
use crate::subset::{CmapTarget, SubsetError, SubsetProfile};
use crate::tables::{FontTableProvider, MaxpTable};
use crate::tag;
use std::collections::HashSet;

// Helper struct to wrap provider references for APIs expecting owned providers
struct CloneableProvider<'a> {
    provider: &'a dyn FontTableProvider,
}

impl<'a> FontTableProvider for CloneableProvider<'a> {
    fn read_table_data(&self, tag: u32) -> Result<std::borrow::Cow<'_, [u8]>, ParseError> {
        self.provider.read_table_data(tag)
    }

    fn table_data(&self, tag: u32) -> Result<Option<std::borrow::Cow<'_, [u8]>>, ParseError> {
        self.provider.table_data(tag)
    }

    fn has_table(&self, tag: u32) -> bool {
        self.provider.has_table(tag)
    }

    fn table_tags(&self) -> Option<Vec<u32>> {
        self.provider.table_tags()
    }
}

/// Builder for configuring font subsetting operations
pub struct SubsetBuilder<'a> {
    provider: &'a dyn FontTableProvider,
    glyph_ids: Vec<u16>,
    cmap_target: CmapTarget,
    profile: SubsetProfile,
    fix_composites: bool,
    validation_level: ValidationLevel,
    pdf_context: Option<PdfFontContext>,
}

/// Validation strictness levels
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum ValidationLevel {
    /// No validation
    None,
    /// Basic validation (check .notdef)
    Basic,
    /// Standard validation (check glyph IDs exist)
    Standard,
    /// Strict validation (comprehensive checks)
    Strict,
}

impl<'a> SubsetBuilder<'a> {
    /// Create a new builder with a font provider
    pub fn new(provider: &'a dyn FontTableProvider) -> Self {
        Self {
            provider,
            glyph_ids: vec![0], // Always include .notdef
            cmap_target: CmapTarget::Unrestricted,
            profile: SubsetProfile::Minimal,
            fix_composites: false,
            validation_level: ValidationLevel::Standard,
            pdf_context: None,
        }
    }

    /// Add glyphs by their IDs
    pub fn with_glyphs(mut self, glyph_ids: &[u16]) -> Self {
        // Ensure .notdef is first
        if !glyph_ids.is_empty() && glyph_ids[0] != 0 {
            self.glyph_ids = vec![0];
            self.glyph_ids.extend_from_slice(glyph_ids);
        } else {
            self.glyph_ids = glyph_ids.to_vec();
        }

        // Remove duplicates while preserving order
        let mut seen = HashSet::new();
        self.glyph_ids.retain(|&gid| seen.insert(gid));

        self
    }

    /// Add glyphs for specific text (alias for with_characters, Phase 2 compatibility)
    pub fn with_text(self, text: &str) -> Result<Self, SubsetError> {
        self.with_characters(text)
    }
    
    /// Add glyphs for specific characters
    pub fn with_characters(mut self, chars: &str) -> Result<Self, SubsetError> {
        // Map characters to glyph IDs using cmap
        use crate::font::MatchingPresentation;
        use crate::Font;

        let provider_clone = Box::new(CloneableProvider {
            provider: self.provider,
        });
        let mut font = Font::new(provider_clone).map_err(|e| SubsetError::Parse(e))?;

        for ch in chars.chars() {
            let (glyph_id, _) =
                font.lookup_glyph_index(ch, MatchingPresentation::NotRequired, None);
            if glyph_id != 0 && !self.glyph_ids.contains(&glyph_id) {
                self.glyph_ids.push(glyph_id);
            }
        }

        Ok(self)
    }

    /// Configure for PDF embedding
    pub fn for_pdf(mut self, max_cid: u16) -> Self {
        self.profile = SubsetProfile::Pdf;
        self.pdf_context = Some(PdfFontContext::legacy(
            max_cid,
            true,  // is_cid_font
            WritingMode::Horizontal,
        ));
        self.fix_composites = true; // Auto-enable for PDF
        self
    }

    /// Set CID-to-GID mapping for PDF
    pub fn with_cid_map(mut self, map: &[u16]) -> Self {
        if let Some(ref mut ctx) = self.pdf_context {
            ctx.cid_to_gid_map = Some(map.to_vec());
        }
        self
    }

    /// Enable/disable composite glyph fixing
    pub fn fix_composites(mut self, enabled: bool) -> Self {
        self.fix_composites = enabled;
        self
    }

    /// Set validation level
    pub fn validation_level(mut self, level: ValidationLevel) -> Self {
        self.validation_level = level;
        self
    }

    /// Set subsetting profile
    pub fn with_profile(mut self, profile: SubsetProfile) -> Self {
        self.profile = profile;
        self
    }

    /// Set cmap target format
    pub fn with_cmap_target(mut self, target: CmapTarget) -> Self {
        self.cmap_target = target;
        self
    }
    
    /// Set Identity-H encoding (Phase 2 enhancement)
    pub fn identity_h(mut self) -> Self {
        use crate::subset::context::FontEncoding;
        if self.pdf_context.is_none() {
            self.pdf_context = Some(PdfFontContext::identity_h());
        } else if let Some(ref mut ctx) = self.pdf_context {
            ctx.encoding = FontEncoding::Identity { vertical: false };
            ctx.writing_mode = WritingMode::Horizontal;
        }
        self
    }
    
    /// Set Identity-V encoding (Phase 2 enhancement)
    pub fn identity_v(mut self) -> Self {
        use crate::subset::context::FontEncoding;
        if self.pdf_context.is_none() {
            self.pdf_context = Some(PdfFontContext::identity_v());
        } else if let Some(ref mut ctx) = self.pdf_context {
            ctx.encoding = FontEncoding::Identity { vertical: true };
            ctx.writing_mode = WritingMode::Vertical;
        }
        self
    }
    
    /// Set maximum CID value (Phase 2 enhancement)
    pub fn with_max_cid(mut self, max_cid: u16) -> Self {
        if let Some(ref mut ctx) = self.pdf_context {
            ctx.max_cid = Some(max_cid);
        }
        self
    }
    
    /// Preserve identity mapping (Phase 2 enhancement)
    pub fn preserve_identity(mut self) -> Self {
        if let Some(ref mut ctx) = self.pdf_context {
            ctx.preserve_identity = true;
        }
        self
    }
    
    /// Mark as symbolic font (Phase 2 enhancement)
    pub fn symbolic(mut self) -> Self {
        if let Some(ref mut ctx) = self.pdf_context {
            ctx.is_symbolic = true;
        }
        self
    }

    /// Build and execute the subsetting operation
    pub fn build(self) -> Result<SubsetResult, SubsetError> {
        // Validate input based on level
        match self.validation_level {
            ValidationLevel::None => {}
            ValidationLevel::Basic => {
                if self.glyph_ids.is_empty() || self.glyph_ids[0] != 0 {
                    return Err(SubsetError::NotDef);
                }
            }
            ValidationLevel::Standard => {
                validate_glyph_ids(&self.glyph_ids, self.provider)?;
            }
            ValidationLevel::Strict => {
                strict_validate_glyph_ids(&self.glyph_ids, self.provider)?;
            }
        }

        // Perform subsetting
        let mut result = if let Some(ref pdf_ctx) = self.pdf_context {
            // PDF-specific subsetting
            use crate::subset::pdf::subset_for_pdf;
            use crate::subset::result::{FontFormat, FontInfo, SubsetStats};

            let provider_clone = &CloneableProvider {
                provider: self.provider,
            };
            let pdf_result = subset_for_pdf(provider_clone, &self.glyph_ids, &pdf_ctx)?;

            // Convert PdfSubsetResult to SubsetResult
            // Get font info
            let original_size = self
                .provider
                .read_table_data(tag::HEAD)
                .map(|data| data.len())
                .unwrap_or(0);

            let subset_size = pdf_result.font_data.len();

            SubsetResult {
                data: pdf_result.font_data,
                glyph_mapping: pdf_result.glyph_mapping.clone(),
                reverse_mapping: pdf_result
                    .glyph_mapping
                    .iter()
                    .map(|(&k, &v)| (v, k))
                    .collect(),
                added_glyphs: Vec::new(), // Would need more tracking
                missing_glyphs: Vec::new(),
                original_info: FontInfo {
                    glyph_count: get_glyph_count(self.provider).unwrap_or(0),
                    max_gid: get_glyph_count(self.provider)
                        .unwrap_or(0)
                        .saturating_sub(1),
                    size: original_size,
                    format: FontFormat::TrueType, // Would need detection
                    has_composites: false,        // Would need detection
                },
                subset_info: FontInfo {
                    glyph_count: pdf_result.glyph_mapping.len() as u16,
                    max_gid: (pdf_result.glyph_mapping.len() as u16).saturating_sub(1),
                    size: subset_size,
                    format: FontFormat::TrueType,
                    has_composites: false,
                },
                stats: SubsetStats {
                    size_reduction_bytes: (original_size as i64 - subset_size as i64),
                    size_reduction_percent: if original_size > 0 && original_size > subset_size {
                        ((original_size - subset_size) as f32 / original_size as f32) * 100.0
                    } else {
                        0.0
                    },
                    composite_glyphs: 0, // Would need counting
                    simple_glyphs: pdf_result.glyph_mapping.len(),
                    removed_tables: Vec::new(),
                },
            }
        } else {
            // Standard subsetting
            use crate::subset::result::subset_detailed;
            let provider_clone = &CloneableProvider {
                provider: self.provider,
            };
            subset_detailed(
                provider_clone,
                &self.glyph_ids,
                &self.profile,
                self.cmap_target,
            )?
        };

        // Fix composites if requested (and not already done by PDF subsetting)
        if self.fix_composites && self.pdf_context.is_none() {
            use crate::subset::composite::update_composite_references;
            update_composite_references(&mut result.data, &result.glyph_mapping)?;
        }

        Ok(result)
    }
}

// Validation helper functions
fn validate_glyph_ids(
    glyph_ids: &[u16],
    provider: &dyn FontTableProvider,
) -> Result<(), SubsetError> {
    // Check .notdef is first
    if glyph_ids.is_empty() || glyph_ids[0] != 0 {
        return Err(SubsetError::NotDef);
    }

    // Check all IDs are valid
    let maxp_data = provider
        .read_table_data(tag::MAXP)
        .map_err(|e| SubsetError::Parse(e))?;
    let maxp = ReadScope::new(&maxp_data)
        .read::<MaxpTable>()
        .map_err(|e| SubsetError::Parse(e))?;

    for &gid in glyph_ids {
        if gid >= maxp.num_glyphs {
            return Err(SubsetError::TooManyGlyphs);
        }
    }

    Ok(())
}

fn strict_validate_glyph_ids(
    glyph_ids: &[u16],
    provider: &dyn FontTableProvider,
) -> Result<(), SubsetError> {
    // Basic validation first
    validate_glyph_ids(glyph_ids, provider)?;

    // Additional strict checks
    // Check for duplicate glyph IDs
    let mut seen = HashSet::new();
    for &gid in glyph_ids {
        if !seen.insert(gid) {
            return Err(SubsetError::TooManyGlyphs); // Using as generic validation error
        }
    }

    // Would add more checks here:
    // - Verify glyphs actually exist in glyf/CFF table
    // - Check composite dependencies are included

    Ok(())
}

fn get_glyph_count(provider: &dyn FontTableProvider) -> Result<u16, ParseError> {
    let maxp_data = provider.read_table_data(tag::MAXP)?;
    let maxp = ReadScope::new(&maxp_data).read::<MaxpTable>()?;
    Ok(maxp.num_glyphs)
}

/// Convenience function to create a PDF subset builder (Phase 2 enhancement)
pub fn subset_for_pdf(provider: &dyn FontTableProvider) -> SubsetBuilder<'_> {
    SubsetBuilder::new(provider)
}

/// PDF-specific builder wrapper for compatibility with Phase 2 tests
pub type PdfSubsetBuilder<'a> = SubsetBuilder<'a>;
