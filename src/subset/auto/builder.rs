use crate::subset::detection::{EncodingDetector, EncodingDetection, DetectionConfidence, PdfFontInfo};
use crate::subset::context::FontEncoding;
use crate::subset::pdf::{PdfSubsetResult, PdfFontContext, subset_and_map_for_pdf, WritingMode};
use crate::subset::SubsetError;
use crate::tables::FontTableProvider;
use std::collections::HashSet;

/// Builder for automatic font subsetting with encoding detection
pub struct AutoSubsetBuilder<'a> {
    provider: &'a dyn FontTableProvider,
    glyph_ids: Vec<u16>,
    pdf_info: Option<PdfFontInfo>,
    detector: Option<EncodingDetector>,
    override_encoding: Option<FontEncoding>,
    min_confidence: DetectionConfidence,
}

impl<'a> AutoSubsetBuilder<'a> {
    /// Create a new auto-subset builder
    pub fn new(provider: &'a dyn FontTableProvider) -> Self {
        Self {
            provider,
            glyph_ids: Vec::new(),
            pdf_info: None,
            detector: None,
            override_encoding: None,
            min_confidence: DetectionConfidence::Low,
        }
    }
    
    /// Add glyphs to subset
    pub fn with_glyphs(mut self, glyph_ids: &[u16]) -> Self {
        // Deduplicate and add glyphs
        let mut existing: HashSet<u16> = self.glyph_ids.iter().cloned().collect();
        for &gid in glyph_ids {
            if !existing.contains(&gid) {
                existing.insert(gid);
                self.glyph_ids.push(gid);
            }
        }
        self
    }
    
    /// Add PDF font information for better detection
    pub fn with_pdf_info(mut self, info: PdfFontInfo) -> Self {
        self.pdf_info = Some(info);
        self
    }
    
    /// Use a custom detector
    pub fn with_detector(mut self, detector: EncodingDetector) -> Self {
        self.detector = Some(detector);
        self
    }
    
    /// Override auto-detection with specific encoding
    pub fn override_encoding(mut self, encoding: FontEncoding) -> Self {
        self.override_encoding = Some(encoding);
        self
    }
    
    /// Set minimum confidence threshold
    pub fn min_confidence(mut self, confidence: DetectionConfidence) -> Self {
        self.min_confidence = confidence;
        self
    }
    
    /// Build and execute the subset with auto-detection
    pub fn build(mut self) -> Result<AutoSubsetResult, SubsetError> {
        // Ensure we have at least .notdef (glyph 0)
        if self.glyph_ids.is_empty() {
            self.glyph_ids.push(0);
        } else if !self.glyph_ids.contains(&0) {
            self.glyph_ids.insert(0, 0);
        }
        
        // Get or create detector
        let mut detector = if let Some(detector) = self.detector.take() {
            detector
        } else {
            // Create a new detector - we need to box the provider for EncodingDetector
            // For now, create a simple detector without using the provider
            // This is a limitation of the current design - in a real implementation
            // we'd need to restructure this to avoid the boxing issue
            struct DummyProvider;
            impl FontTableProvider for DummyProvider {
                fn table_data(&self, _tag: u32) -> Result<Option<std::borrow::Cow<'_, [u8]>>, crate::error::ParseError> {
                    Ok(Some(std::borrow::Cow::Borrowed(&[])))
                }
                
                fn has_table(&self, _tag: u32) -> bool {
                    false
                }
                
                fn table_tags(&self) -> Option<Vec<u32>> {
                    Some(vec![])
                }
            }
            EncodingDetector::new(Box::new(DummyProvider))
        };
        
        // Perform detection or use override
        let detection = if let Some(encoding) = self.override_encoding.take() {
            EncodingDetection {
                encoding,
                confidence: DetectionConfidence::Certain,
                reasoning: vec!["Manual override specified".to_string()],
                alternatives: Vec::new(),
            }
        } else {
            detector.detect(&self.glyph_ids, self.pdf_info.as_ref())
        };
        
        // Check confidence threshold
        if detection.confidence < self.min_confidence {
            return Err(SubsetError::InvalidContext(
                format!(
                    "Detection confidence ({:?}) is below minimum threshold ({:?})", 
                    detection.confidence, self.min_confidence
                )
            ));
        }
        
        // Create PDF context from detected encoding
        let pdf_context = self.create_pdf_context(&detection.encoding);
        
        // Perform the actual subsetting
        // We need to work around the Sized requirement by using a concrete mock type
        // In practice, this would use a different implementation or the existing function 
        // would need to be modified to accept ?Sized trait objects
        struct DummyProviderWrapper<'p> {
            inner: &'p dyn FontTableProvider,
        }
        
        impl<'p> FontTableProvider for DummyProviderWrapper<'p> {
            fn table_data(&self, tag: u32) -> Result<Option<std::borrow::Cow<'_, [u8]>>, crate::error::ParseError> {
                self.inner.table_data(tag)
            }
            
            fn has_table(&self, tag: u32) -> bool {
                self.inner.has_table(tag)
            }
            
            fn table_tags(&self) -> Option<Vec<u32>> {
                self.inner.table_tags()
            }
        }
        
        let wrapper = DummyProviderWrapper { inner: self.provider };
        let subset_result = subset_and_map_for_pdf(
            &wrapper,
            &self.glyph_ids,
            pdf_context
        )?;
        
        Ok(AutoSubsetResult {
            subset_result,
            detection,
        })
    }
    
    /// Create PDF context from detected encoding
    fn create_pdf_context(&self, encoding: &FontEncoding) -> PdfFontContext {
        match encoding {
            FontEncoding::Identity { vertical } => {
                let mut context = PdfFontContext::identity_h();
                if *vertical {
                    context.encoding = FontEncoding::Identity { vertical: true };
                    context.writing_mode = WritingMode::Vertical;
                }
                context
            },
            FontEncoding::CJK { .. } => {
                let mut context = PdfFontContext::identity_h();
                context.encoding = encoding.clone();
                context.is_symbolic = true;
                context
            },
            _ => {
                let mut context = PdfFontContext::identity_h();
                context.encoding = encoding.clone();
                context
            }
        }
    }
}

/// Result of automatic subsetting with detection information
pub struct AutoSubsetResult {
    /// The PDF subset result
    pub subset_result: PdfSubsetResult,
    /// The encoding detection information
    pub detection: EncodingDetection,
}

impl AutoSubsetResult {
    /// Get the detection confidence level
    pub fn confidence(&self) -> &DetectionConfidence {
        &self.detection.confidence
    }
    
    /// Get the reasoning for the detection
    pub fn reasoning(&self) -> &[String] {
        &self.detection.reasoning
    }
    
    /// Get alternative encoding suggestions
    pub fn alternatives(&self) -> &[FontEncoding] {
        &self.detection.alternatives
    }
}

/// Create an auto-subset builder for PDF embedding
pub fn auto_subset_for_pdf(provider: &dyn FontTableProvider) -> AutoSubsetBuilder<'_> {
    AutoSubsetBuilder::new(provider)
}