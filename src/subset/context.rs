//! Font encoding context for subsetting operations

/// Language variants for CJK fonts
#[derive(Debug, Clone, PartialEq)]
pub enum CJKLanguage {
    /// Chinese font encoding
    Chinese(ChineseVariant),
    /// Japanese font encoding
    Japanese(JapaneseVariant),
    /// Korean font encoding
    Korean(KoreanVariant),
}

/// Chinese font variants
#[derive(Debug, Clone, PartialEq)]
pub enum ChineseVariant {
    /// Simplified Chinese (GB/GBK encodings)
    Simplified,
    /// Traditional Chinese (CNS/Big5 encodings)
    Traditional,
    /// Hong Kong Chinese (HKSCS encodings)
    HongKong,
}

/// Japanese font variants
#[derive(Debug, Clone, PartialEq)]
pub enum JapaneseVariant {
    /// JIS-based encodings
    JIS,
    /// Shift-JIS based encodings (RKSJ)
    ShiftJIS,
    /// Unicode-based Japanese encodings
    Unicode,
}

/// Korean font variants
#[derive(Debug, Clone, PartialEq)]
pub enum KoreanVariant {
    /// KSC5601 based encodings
    KSC,
    /// Unified Hangul Code encodings
    UHC,
    /// Unicode-based Korean encodings
    Unicode,
}

/// Font encoding types for CID fonts
#[derive(Debug, Clone, PartialEq)]
pub enum FontEncoding {
    /// Identity mapping where CID equals GID
    /// Used by most modern PDFs
    Identity { 
        /// True for vertical writing (Identity-V), false for horizontal (Identity-H)
        vertical: bool 
    },
    
    /// CJK predefined encodings
    CJK {
        /// The CJK language variant
        language: CJKLanguage,
        /// Original encoding name from PDF
        encoding_name: String,
        /// True for vertical writing
        vertical: bool,
        /// If true, requires external CMap data
        requires_cmap_data: bool,
    },
    
    /// Adobe standard collections
    AdobeCollection {
        /// Registry name (typically "Adobe")
        registry: String,
        /// Ordering (e.g., "GB1", "CNS1", "Japan1", "Korea1")
        ordering: String,
        /// Supplement version number
        supplement: u16,
    },
    
    /// Custom encoding
    Custom(String),
}

impl FontEncoding {
    /// Parse encoding name from PDF font dictionary
    pub fn from_pdf_name(name: &str) -> Option<Self> {
        // Phase 1 encodings
        match name {
            "Identity-H" => return Some(FontEncoding::Identity { vertical: false }),
            "Identity-V" => return Some(FontEncoding::Identity { vertical: true }),
            _ => {}
        }
        
        // Phase 3: CJK encodings
        let vertical = name.ends_with("-V");
        
        // Chinese encodings
        // Common patterns: GB-EUC-H, GB-EUC-V, GBK-EUC-H, GBK-EUC-V, UniGB-UTF16-H, etc.
        if name.starts_with("GB-EUC-") && (name.ends_with("-H") || name.ends_with("-V")) {
            return Some(FontEncoding::CJK {
                language: CJKLanguage::Chinese(ChineseVariant::Simplified),
                encoding_name: name.to_string(),
                vertical,
                requires_cmap_data: true,
            });
        }
        if name.starts_with("GBK-EUC-") && (name.ends_with("-H") || name.ends_with("-V")) {
            return Some(FontEncoding::CJK {
                language: CJKLanguage::Chinese(ChineseVariant::Simplified),
                encoding_name: name.to_string(),
                vertical,
                requires_cmap_data: true,
            });
        }
        if name.starts_with("UniGB-") {
            return Some(FontEncoding::CJK {
                language: CJKLanguage::Chinese(ChineseVariant::Simplified),
                encoding_name: name.to_string(),
                vertical,
                requires_cmap_data: false,
            });
        }
        
        // Traditional Chinese encodings
        if name.starts_with("CNS-EUC-") && (name.ends_with("-H") || name.ends_with("-V")) {
            return Some(FontEncoding::CJK {
                language: CJKLanguage::Chinese(ChineseVariant::Traditional),
                encoding_name: name.to_string(),
                vertical,
                requires_cmap_data: true,
            });
        }
        if (name == "B5pc-H" || name == "B5pc-V" || 
            name.starts_with("ETen-B5-") || 
            name.starts_with("HKscs-B5-")) {
            let variant = if name.starts_with("HK") {
                ChineseVariant::HongKong
            } else {
                ChineseVariant::Traditional
            };
            return Some(FontEncoding::CJK {
                language: CJKLanguage::Chinese(variant),
                encoding_name: name.to_string(),
                vertical,
                requires_cmap_data: true,
            });
        }
        if name.starts_with("UniCNS-") {
            return Some(FontEncoding::CJK {
                language: CJKLanguage::Chinese(ChineseVariant::Traditional),
                encoding_name: name.to_string(),
                vertical,
                requires_cmap_data: false,
            });
        }
        
        // Japanese encodings
        if (name.starts_with("90ms-RKSJ-") || name.starts_with("83pv-RKSJ-")) && 
           (name.ends_with("-H") || name.ends_with("-V")) {
            return Some(FontEncoding::CJK {
                language: CJKLanguage::Japanese(JapaneseVariant::ShiftJIS),
                encoding_name: name.to_string(),
                vertical,
                requires_cmap_data: true,
            });
        }
        if name.starts_with("UniJIS-") {
            return Some(FontEncoding::CJK {
                language: CJKLanguage::Japanese(JapaneseVariant::Unicode),
                encoding_name: name.to_string(),
                vertical,
                requires_cmap_data: false,
            });
        }
        if name == "H" || name == "V" || name == "EUC-H" || name == "EUC-V" {
            return Some(FontEncoding::CJK {
                language: CJKLanguage::Japanese(JapaneseVariant::JIS),
                encoding_name: name.to_string(),
                vertical: name == "V" || name == "EUC-V",
                requires_cmap_data: true,
            });
        }
        
        // Korean encodings
        if name.starts_with("KSCms-UHC-") && (name.ends_with("-H") || name.ends_with("-V")) {
            return Some(FontEncoding::CJK {
                language: CJKLanguage::Korean(KoreanVariant::UHC),
                encoding_name: name.to_string(),
                vertical,
                requires_cmap_data: true,
            });
        }
        if name.starts_with("KSC-EUC-") && (name.ends_with("-H") || name.ends_with("-V")) {
            return Some(FontEncoding::CJK {
                language: CJKLanguage::Korean(KoreanVariant::KSC),
                encoding_name: name.to_string(),
                vertical,
                requires_cmap_data: true,
            });
        }
        if name.starts_with("UniKS-") {
            return Some(FontEncoding::CJK {
                language: CJKLanguage::Korean(KoreanVariant::Unicode),
                encoding_name: name.to_string(),
                vertical,
                requires_cmap_data: false,
            });
        }
        
        // Adobe collections
        if name.starts_with("Adobe-") {
            return Self::parse_adobe_collection(name);
        }
        
        None
    }
    
    /// Parse Adobe collection names
    fn parse_adobe_collection(name: &str) -> Option<FontEncoding> {
        // Format: Adobe-Registry-Supplement
        let parts: Vec<&str> = name.split('-').collect();
        if parts.len() == 3 && parts[0] == "Adobe" {
            let registry = "Adobe".to_string();
            let ordering = parts[1].to_string();
            let supplement = parts[2].parse::<u16>().ok()?;
            
            return Some(FontEncoding::AdobeCollection {
                registry,
                ordering,
                supplement,
            });
        }
        None
    }
    
    /// Get the Unicode base for CJK encodings
    pub fn get_unicode_base(&self) -> Option<u32> {
        match self {
            FontEncoding::CJK { language, .. } => {
                match language {
                    CJKLanguage::Chinese(_) => Some(0x4E00),  // CJK Unified start
                    CJKLanguage::Japanese(_) => Some(0x3040),  // Hiragana start
                    CJKLanguage::Korean(_) => Some(0xAC00),   // Hangul start
                }
            }
            _ => None,
        }
    }
    
    /// Check if this encoding requires CID font treatment
    pub fn requires_cid(&self) -> bool {
        match self {
            FontEncoding::Identity { .. } => true,
            FontEncoding::CJK { .. } => true,
            FontEncoding::AdobeCollection { .. } => true,
            FontEncoding::Custom(_) => false,
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