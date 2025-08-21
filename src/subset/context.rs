//! Font encoding context for subsetting operations

/// Font encoding types for CID fonts
#[derive(Debug, Clone, PartialEq)]
pub enum FontEncoding {
    /// Identity mapping where CID equals GID
    /// Used by most modern PDFs
    Identity { 
        /// True for vertical writing (Identity-V), false for horizontal (Identity-H)
        vertical: bool 
    },
}

impl FontEncoding {
    /// Parse encoding name from PDF font dictionary
    pub fn from_pdf_name(name: &str) -> Option<Self> {
        match name {
            "Identity-H" => Some(FontEncoding::Identity { vertical: false }),
            "Identity-V" => Some(FontEncoding::Identity { vertical: true }),
            _ => None,  // Phase 1: Return None for unsupported encodings
        }
    }
    
    /// Check if this encoding requires CID font treatment
    pub fn requires_cid(&self) -> bool {
        match self {
            FontEncoding::Identity { .. } => true,
        }
    }
}

/// Context for font subsetting operations
#[derive(Debug, Clone, PartialEq)]
pub enum FontContext {
    /// No context provided - use heuristics
    Unknown,
    
    /// PDF Type0 (CID) font with encoding
    PdfType0 { 
        /// The encoding used for this PDF font
        encoding: FontEncoding,
    },
}

impl Default for FontContext {
    fn default() -> Self {
        FontContext::Unknown
    }
}