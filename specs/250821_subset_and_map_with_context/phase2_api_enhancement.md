# Phase 2: API Enhancement & Convenience Wrappers

## Executive Summary

Build on Phase 1's foundation by adding convenience APIs, better error handling, and improved ergonomics. This phase makes the context-aware API easier to use while maintaining backward compatibility.

## 1. What We're Building

### 1.1 New Components

```rust
// Dedicated PDF context structure
pub struct PdfFontContext {
    pub encoding: FontEncoding,
    pub max_cid: Option<u16>,        // Optional max CID override
    pub preserve_identity: bool,      // Force CID preservation
    pub is_symbolic: bool,            // Hint for symbolic fonts
}

// Convenience wrapper for PDF use cases
pub fn subset_and_map_for_pdf(
    provider: &impl FontTableProvider,
    glyph_ids: &[u16],
    pdf_context: PdfFontContext,
) -> Result<PdfSubsetResult, SubsetError>

// Enhanced result type for PDF
pub struct PdfSubsetResult {
    pub font_data: Vec<u8>,
    pub glyph_mapping: HashMap<u16, u16>,
    pub cid_to_gid_map: Vec<u8>,
    pub font_type: PdfFontType,
    pub encoding_used: FontEncoding,
    pub statistics: SubsetStatistics,
}

// Enhanced error types
pub enum SubsetError {
    // ... existing variants ...
    UnsupportedEncoding(String),
    InvalidContext(String),
    CidGenerationFailed(String),
}
```

### 1.2 Key Features

1. **Simplified PDF API**: Single function for PDF use cases
2. **Better Defaults**: Smart defaults for common scenarios
3. **Rich Results**: Include metadata and statistics
4. **Improved Errors**: Actionable error messages
5. **Builder Pattern**: For complex configurations

## 2. Why This Phase

### 2.1 Developer Experience
- **Reduces Boilerplate**: One function call instead of multiple steps
- **Self-Documenting**: Clear parameter names and types
- **Fewer Mistakes**: Validation and smart defaults
- **Better Debugging**: Rich error messages and statistics

### 2.2 Maintainability
- **Separation of Concerns**: PDF-specific logic isolated
- **Extensibility**: Easy to add new PDF features
- **Testing**: Cleaner test structure

## 3. Technical Implementation

### 3.1 File Structure

```
src/subset/
├── mod.rs                    (existing)
├── context.rs                (from Phase 1, enhanced)
├── cid_map.rs               (from Phase 1)
├── pdf.rs                   (new - PDF-specific API)
├── builder.rs               (new - builder pattern)
└── error.rs                 (enhanced error types)
```

### 3.2 Implementation Details

#### 3.2.1 PDF Context Structure (`src/subset/pdf.rs`)

```rust
use crate::subset::context::{FontContext, FontEncoding};
use crate::subset::{SubsetProfile, CmapTarget, SubsetResult};
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
#[derive(Debug, Clone)]
pub struct PdfFontContext {
    /// The encoding used in the PDF
    pub encoding: FontEncoding,
    
    /// Optional maximum CID value
    /// If None, will be calculated from glyph_ids
    pub max_cid: Option<u16>,
    
    /// Force preservation of original glyph IDs
    /// Useful for compatibility but results in larger files
    pub preserve_identity: bool,
    
    /// Indicates if this is a symbolic font
    /// Helps with detection heuristics
    pub is_symbolic: bool,
}

impl PdfFontContext {
    /// Create context for Identity-H encoding (most common)
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
    
    /// Create context from PDF font dictionary values
    pub fn from_pdf_dict(encoding_name: &str, flags: u32) -> Result<Self, SubsetError> {
        let encoding = FontEncoding::from_pdf_name(encoding_name)
            .ok_or_else(|| SubsetError::UnsupportedEncoding(encoding_name.to_string()))?;
        
        Ok(PdfFontContext {
            encoding,
            max_cid: None,
            preserve_identity: false,
            is_symbolic: flags & 0x04 != 0,  // Check symbolic flag
        })
    }
    
    /// Set maximum CID value
    pub fn with_max_cid(mut self, max_cid: u16) -> Self {
        self.max_cid = Some(max_cid);
        self
    }
    
    /// Enable identity preservation
    pub fn preserve_identity(mut self) -> Self {
        self.preserve_identity = true;
        self
    }
}

/// Subsetting statistics
#[derive(Debug, Clone)]
pub struct SubsetStatistics {
    pub original_glyph_count: usize,
    pub subset_glyph_count: usize,
    pub original_size_estimate: usize,
    pub subset_size: usize,
    pub cid_map_size: usize,
    pub reduction_percentage: f32,
}

/// Result of PDF-specific subsetting
#[derive(Debug)]
pub struct PdfSubsetResult {
    /// The subsetted font data
    pub font_data: Vec<u8>,
    
    /// Mapping from old to new glyph IDs
    pub glyph_mapping: HashMap<u16, u16>,
    
    /// CIDToGIDMap for PDF embedding
    pub cid_to_gid_map: Vec<u8>,
    
    /// Type of font for PDF
    pub font_type: PdfFontType,
    
    /// Encoding that was used
    pub encoding_used: FontEncoding,
    
    /// Statistics about the subsetting
    pub statistics: SubsetStatistics,
}

impl PdfSubsetResult {
    /// Get the size reduction achieved
    pub fn size_reduction(&self) -> f32 {
        self.statistics.reduction_percentage
    }
    
    /// Check if this is a CID font
    pub fn is_cid_font(&self) -> bool {
        matches!(self.font_type, PdfFontType::CidType0 | PdfFontType::CidType2)
    }
    
    /// Get the CIDToGIDMap if this is a CID font
    pub fn cid_to_gid_map(&self) -> Option<&[u8]> {
        if self.is_cid_font() {
            Some(&self.cid_to_gid_map)
        } else {
            None
        }
    }
}
```

#### 3.2.2 Main PDF API Function (`src/subset/pdf.rs` continued)

```rust
/// Subset a font for PDF embedding with full context support
///
/// This is the recommended API for PDF use cases. It handles:
/// - Automatic CID detection
/// - Correct CIDToGIDMap generation
/// - Statistics and metadata
/// - Validation and error handling
///
/// # Example
/// ```rust
/// let context = PdfFontContext::identity_h();
/// let result = subset_and_map_for_pdf(&provider, &glyph_ids, context)?;
/// 
/// println!("Reduced size by {:.1}%", result.size_reduction());
/// if let Some(cid_map) = result.cid_to_gid_map() {
///     pdf_font.embed_cid_to_gid_map(cid_map);
/// }
/// ```
pub fn subset_and_map_for_pdf(
    provider: &impl FontTableProvider,
    glyph_ids: &[u16],
    pdf_context: PdfFontContext,
) -> Result<PdfSubsetResult, SubsetError> {
    // Validate inputs
    if glyph_ids.is_empty() || glyph_ids[0] != 0 {
        return Err(SubsetError::InvalidContext(
            "Glyph IDs must start with 0 (.notdef)".to_string()
        ));
    }
    
    // Determine font type from structure
    let font_type = detect_pdf_font_type(provider)?;
    
    // Create appropriate context
    let context = FontContext::PdfType0 {
        encoding: pdf_context.encoding.clone(),
    };
    
    // Perform subsetting with context
    let subset_result = subset_and_map_with_context(
        provider,
        glyph_ids,
        &SubsetProfile::Pdf,
        CmapTarget::Unicode,
        context,
    )?;
    
    // Extract results based on type
    let (font_data, glyph_mapping, cid_to_gid_map) = match subset_result {
        SubsetResult::Cid { font_data, glyph_mapping, cid_to_gid_map } => {
            (font_data, glyph_mapping, cid_to_gid_map)
        }
        SubsetResult::Simple { font_data, glyph_mapping } => {
            // Simple font - no CID map needed
            (font_data, glyph_mapping, Vec::new())
        }
    };
    
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

/// Detect PDF font type from structure
fn detect_pdf_font_type(provider: &impl FontTableProvider) -> Result<PdfFontType, SubsetError> {
    use crate::tag;
    
    if provider.has_table(tag::CFF) || provider.has_table(tag::CFF2) {
        // CFF-based font
        if is_cid_keyed_cff(provider)? {
            Ok(PdfFontType::CidType0)
        } else {
            Ok(PdfFontType::Simple)
        }
    } else {
        // TrueType-based font
        // Check if it should be treated as CID
        if should_be_cid_type2(provider)? {
            Ok(PdfFontType::CidType2)
        } else {
            Ok(PdfFontType::Simple)
        }
    }
}

fn calculate_statistics(
    provider: &impl FontTableProvider,
    subset_data: &[u8],
    requested_glyphs: usize,
    glyph_mapping: &HashMap<u16, u16>,
    cid_map: &[u8],
) -> Result<SubsetStatistics, SubsetError> {
    // Estimate original size (simplified)
    let original_size_estimate = estimate_font_size(provider)?;
    let subset_size = subset_data.len();
    let cid_map_size = cid_map.len();
    
    let reduction = if original_size_estimate > 0 {
        let total_new_size = subset_size + cid_map_size;
        let reduction = original_size_estimate.saturating_sub(total_new_size);
        (reduction as f32 / original_size_estimate as f32) * 100.0
    } else {
        0.0
    };
    
    Ok(SubsetStatistics {
        original_glyph_count: get_glyph_count(provider)? as usize,
        subset_glyph_count: glyph_mapping.len(),
        original_size_estimate,
        subset_size,
        cid_map_size,
        reduction_percentage: reduction,
    })
}
```

#### 3.2.3 Builder Pattern API (`src/subset/builder.rs`)

```rust
use crate::subset::pdf::{PdfFontContext, PdfSubsetResult};
use crate::subset::context::FontEncoding;

/// Builder for PDF font subsetting with fluent API
pub struct PdfSubsetBuilder<'a> {
    provider: &'a dyn FontTableProvider,
    glyph_ids: Vec<u16>,
    encoding: Option<FontEncoding>,
    max_cid: Option<u16>,
    preserve_identity: bool,
    is_symbolic: bool,
}

impl<'a> PdfSubsetBuilder<'a> {
    /// Create a new builder
    pub fn new(provider: &'a dyn FontTableProvider) -> Self {
        PdfSubsetBuilder {
            provider,
            glyph_ids: vec![0],  // Start with .notdef
            encoding: None,
            max_cid: None,
            preserve_identity: false,
            is_symbolic: false,
        }
    }
    
    /// Add glyph IDs to subset
    pub fn with_glyphs(mut self, glyph_ids: &[u16]) -> Self {
        for &gid in glyph_ids {
            if gid != 0 && !self.glyph_ids.contains(&gid) {
                self.glyph_ids.push(gid);
            }
        }
        self
    }
    
    /// Add glyphs from Unicode text
    pub fn with_text(mut self, text: &str) -> Result<Self, SubsetError> {
        // Map characters to glyphs
        let mut font = Font::new(Box::new(self.provider.clone()))?;
        for ch in text.chars() {
            let (gid, _) = font.lookup_glyph_index(ch, MatchingPresentation::NotRequired, None);
            if gid != 0 && !self.glyph_ids.contains(&gid) {
                self.glyph_ids.push(gid);
            }
        }
        Ok(self)
    }
    
    /// Set encoding
    pub fn with_encoding(mut self, encoding: FontEncoding) -> Self {
        self.encoding = Some(encoding);
        self
    }
    
    /// Use Identity-H encoding (common case)
    pub fn identity_h(self) -> Self {
        self.with_encoding(FontEncoding::Identity { vertical: false })
    }
    
    /// Use Identity-V encoding
    pub fn identity_v(self) -> Self {
        self.with_encoding(FontEncoding::Identity { vertical: true })
    }
    
    /// Set maximum CID
    pub fn with_max_cid(mut self, max_cid: u16) -> Self {
        self.max_cid = Some(max_cid);
        self
    }
    
    /// Enable identity preservation
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
        
        let context = PdfFontContext {
            encoding,
            max_cid: self.max_cid,
            preserve_identity: self.preserve_identity,
            is_symbolic: self.is_symbolic,
        };
        
        subset_and_map_for_pdf(self.provider, &self.glyph_ids, context)
    }
}

/// Convenience function to start building
pub fn subset_for_pdf(provider: &dyn FontTableProvider) -> PdfSubsetBuilder {
    PdfSubsetBuilder::new(provider)
}
```

#### 3.2.4 Enhanced Error Types (`src/subset/error.rs`)

```rust
use std::fmt;

/// Enhanced subset error with better context
#[derive(Debug)]
pub enum SubsetError {
    // Existing variants
    NotDef,
    Parse(ParseError),
    Write(WriteError),
    TooManyGlyphs,
    CFF(CFFError),
    InvalidFontCount,
    
    // New variants for Phase 2
    /// Encoding is not supported
    UnsupportedEncoding(String),
    
    /// Invalid context provided
    InvalidContext(String),
    
    /// CID map generation failed
    CidGenerationFailed(String),
    
    /// Font type detection failed
    FontTypeDetection(String),
}

impl fmt::Display for SubsetError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            SubsetError::UnsupportedEncoding(enc) => {
                write!(f, "Encoding '{}' is not supported. Supported encodings: Identity-H, Identity-V", enc)
            }
            SubsetError::InvalidContext(msg) => {
                write!(f, "Invalid subsetting context: {}", msg)
            }
            SubsetError::CidGenerationFailed(msg) => {
                write!(f, "Failed to generate CIDToGIDMap: {}", msg)
            }
            SubsetError::FontTypeDetection(msg) => {
                write!(f, "Could not determine font type: {}", msg)
            }
            // ... existing variants
            _ => write!(f, "{:?}", self),
        }
    }
}

impl std::error::Error for SubsetError {}
```

### 3.3 Testing Strategy

#### 3.3.1 API Usage Tests

```rust
#[test]
fn test_pdf_context_convenience() {
    // Test convenience constructors
    let ctx = PdfFontContext::identity_h();
    assert_eq!(ctx.encoding, FontEncoding::Identity { vertical: false });
    assert!(!ctx.preserve_identity);
    
    let ctx = PdfFontContext::identity_v();
    assert_eq!(ctx.encoding, FontEncoding::Identity { vertical: true });
}

#[test]
fn test_pdf_api_simple_case() {
    let provider = create_test_provider();
    let context = PdfFontContext::identity_h();
    
    let result = subset_and_map_for_pdf(
        &provider,
        &[0, 1, 2, 3],
        context,
    ).unwrap();
    
    assert!(result.is_cid_font());
    assert!(result.statistics.reduction_percentage > 0.0);
}

#[test]
fn test_builder_pattern() {
    let provider = create_test_provider();
    
    let result = subset_for_pdf(&provider)
        .with_glyphs(&[42, 43, 44])
        .identity_h()
        .with_max_cid(1000)
        .build()
        .unwrap();
    
    assert_eq!(result.encoding_used, FontEncoding::Identity { vertical: false });
    assert!(!result.cid_to_gid_map.is_empty());
}

#[test]
fn test_builder_with_text() {
    let provider = create_test_provider();
    
    let result = subset_for_pdf(&provider)
        .with_text("Hello, World!")
        .unwrap()
        .identity_h()
        .build()
        .unwrap();
    
    // Should have glyphs for the text
    assert!(result.glyph_mapping.len() > 1);
}
```

#### 3.3.2 Error Handling Tests

```rust
#[test]
fn test_missing_notdef_error() {
    let provider = create_test_provider();
    let context = PdfFontContext::identity_h();
    
    let result = subset_and_map_for_pdf(
        &provider,
        &[1, 2, 3],  // Missing 0
        context,
    );
    
    assert!(matches!(result, Err(SubsetError::InvalidContext(_))));
}

#[test]
fn test_unsupported_encoding_error() {
    let result = PdfFontContext::from_pdf_dict("Unknown-Encoding", 0);
    assert!(matches!(result, Err(SubsetError::UnsupportedEncoding(_))));
    
    if let Err(e) = result {
        let msg = e.to_string();
        assert!(msg.contains("not supported"));
        assert!(msg.contains("Identity-H"));
    }
}
```

### 3.4 Migration Examples

```rust
// Example 1: Simple Identity-H case
let result = subset_and_map_for_pdf(
    &provider,
    &glyph_ids,
    PdfFontContext::identity_h(),
)?;

// Example 2: With custom settings
let context = PdfFontContext::identity_h()
    .with_max_cid(65535)
    .preserve_identity();
    
let result = subset_and_map_for_pdf(&provider, &glyph_ids, context)?;

// Example 3: Using builder for text
let result = subset_for_pdf(&provider)
    .with_text("Sample text for subsetting")?
    .identity_h()
    .build()?;

// Example 4: From PDF dictionary
let context = PdfFontContext::from_pdf_dict(&pdf_font.encoding, pdf_font.flags)?;
let result = subset_and_map_for_pdf(&provider, &used_glyphs, context)?;
```

## 4. Success Criteria

### 4.1 Functional Requirements
- ✅ `subset_and_map_for_pdf` works for all Identity encodings
- ✅ Builder pattern supports common workflows
- ✅ Statistics provide useful metrics
- ✅ Error messages are actionable

### 4.2 Usability Requirements
- API requires < 5 lines for common cases
- Documentation includes working examples
- Migration from Phase 1 is straightforward
- No breaking changes to Phase 1 API

### 4.3 Quality Metrics
- Test coverage > 90% for new code
- All examples in documentation compile
- Performance overhead < 5% vs Phase 1

## 5. Dependencies on Phase 1

This phase requires Phase 1 to be complete:
- `FontEncoding` enum
- `FontContext` enum
- `subset_and_map_with_context` function
- Identity encoding support

## 6. Estimated Timeline

| Task | Duration | Notes |
|------|----------|-------|
| PDF context structure | 2 hours | Build on Phase 1 types |
| Main PDF API function | 3 hours | Integration logic |
| Builder pattern | 3 hours | Fluent API design |
| Enhanced errors | 2 hours | Better messages |
| Statistics calculation | 2 hours | Size metrics |
| Unit tests | 3 hours | API coverage |
| Documentation | 2 hours | Examples and guide |
| **Total** | **17 hours** | ~2.5 days |

## 7. Future Enhancements

Phase 2 sets up for:
- Phase 3: CJK encoding support (new FontEncoding variants)
- Phase 4: Auto-detection (enhance builder)
- Phase 5: Advanced patterns (statistics help identify)

---

*Document Version: 1.0*  
*Created: 2024-08-21*  
*Phase: 2 of 5*  
*Priority: HIGH - Improves developer experience*