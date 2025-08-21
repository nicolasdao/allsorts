//! Builder pattern for PDF font subsetting

use crate::subset::context::FontEncoding;
use crate::subset::phase2::pdf::{PdfFontContext, PdfSubsetResult, subset_and_map_for_pdf};
use crate::subset::SubsetError;
use crate::tables::FontTableProvider;
use std::collections::HashSet;

/// Builder for PDF font subsetting
pub struct PdfSubsetBuilder<'a, T: FontTableProvider> {
    provider: &'a T,
    glyph_ids: Vec<u16>,
    encoding: Option<FontEncoding>,
    max_cid: Option<u16>,
    preserve_identity: bool,
    is_symbolic: bool,
}

impl<'a, T: FontTableProvider> PdfSubsetBuilder<'a, T> {
    /// Create a new builder with a font provider
    pub fn new(provider: &'a T) -> Self {
        PdfSubsetBuilder {
            provider,
            glyph_ids: vec![0],  // Always start with .notdef
            encoding: None,
            max_cid: None,
            preserve_identity: false,
            is_symbolic: false,
        }
    }
    
    /// Add glyph IDs to subset
    pub fn with_glyphs(mut self, glyph_ids: &[u16]) -> Self {
        // Add glyphs, ensuring .notdef stays first
        let mut glyph_set: HashSet<u16> = self.glyph_ids.into_iter().collect();
        glyph_set.extend(glyph_ids);
        
        // Ensure .notdef is first
        let mut new_ids = vec![0];
        for id in glyph_set {
            if id != 0 {
                new_ids.push(id);
            }
        }
        
        self.glyph_ids = new_ids;
        self
    }
    
    /// Add glyphs for text
    pub fn with_text(mut self, text: &str) -> Result<Self, SubsetError> {
        // This is a simplified implementation
        // In reality, this would need to map characters to glyphs using cmap
        // For now, just return self unchanged
        // TODO: Implement proper character to glyph mapping
        Ok(self)
    }
    
    /// Set Identity-H encoding
    pub fn identity_h(mut self) -> Self {
        self.encoding = Some(FontEncoding::Identity { vertical: false });
        self
    }
    
    /// Set Identity-V encoding
    pub fn identity_v(mut self) -> Self {
        self.encoding = Some(FontEncoding::Identity { vertical: true });
        self
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
    
    /// Mark as symbolic font
    pub fn symbolic(mut self) -> Self {
        self.is_symbolic = true;
        self
    }
    
    /// Build the subset
    pub fn build(self) -> Result<PdfSubsetResult, SubsetError> {
        // Default to Identity-H if no encoding specified
        let encoding = self.encoding.unwrap_or(FontEncoding::Identity { vertical: false });
        
        let mut context = PdfFontContext {
            encoding,
            max_cid: self.max_cid,
            preserve_identity: self.preserve_identity,
            is_symbolic: self.is_symbolic,
        };
        
        // Apply max_cid if specified
        if let Some(max_cid) = self.max_cid {
            context = context.with_max_cid(max_cid);
        }
        
        subset_and_map_for_pdf(self.provider, &self.glyph_ids, context)
    }
}

/// Convenience function to create a PDF subset builder
pub fn subset_for_pdf<T: FontTableProvider>(provider: &T) -> PdfSubsetBuilder<'_, T> {
    PdfSubsetBuilder::new(provider)
}