# Phase 1: Core Identity Encoding Support

## Executive Summary

Implement the minimal viable `subset_and_map_with_context` API that correctly handles Identity-H/V encodings, which covers 90% of modern PDF use cases and immediately fixes the client's CIDToGIDMap issue.

## 1. What We're Building

### 1.1 Core Components

```rust
// New types for encoding context
pub enum FontEncoding {
    Identity { vertical: bool },  // Phase 1: Only this variant
    // Future phases will add more variants
}

pub enum FontContext {
    Unknown,
    PdfType0 { encoding: FontEncoding },
    // Future phases will add more variants
}

// Main API function
pub fn subset_and_map_with_context(
    provider: &impl FontTableProvider,
    glyph_ids: &[u16],
    profile: &SubsetProfile,
    cmap_target: CmapTarget,
    context: FontContext,
) -> Result<SubsetResult, SubsetError>
```

### 1.2 Key Behavior

For Identity encodings (CID == original GID), the function will:
1. Perform standard subsetting
2. Generate correct CIDToGIDMap where `map[cid] = glyph_mapping[cid]`
3. Return `SubsetResult::Cid` with the correct map

## 2. Why This Phase First

### 2.1 Immediate Value
- **Fixes Production Issue**: Solves the client's rendering problem where characters show as '?'
- **90% Coverage**: Identity-H/V encodings are used in most modern PDFs
- **Minimal Complexity**: Simple mapping logic (CID == GID)
- **Low Risk**: Doesn't affect existing APIs

### 2.2 Business Impact
- Enables 40% file size reduction while maintaining correct rendering
- Unblocks PDF optimization for the majority of use cases
- Provides foundation for future encoding support

## 3. Technical Implementation

### 3.1 File Structure

```
src/subset/
├── mod.rs                    (existing, needs updates)
├── context.rs                (new - context types)
└── cid_map.rs               (new - CIDToGIDMap generation)
```

### 3.2 Implementation Details

#### 3.2.1 Context Types (`src/subset/context.rs`)

```rust
use std::fmt;

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
        encoding: FontEncoding,
    },
}

impl Default for FontContext {
    fn default() -> Self {
        FontContext::Unknown
    }
}
```

#### 3.2.2 CIDToGIDMap Generation (`src/subset/cid_map.rs`)

```rust
use std::collections::HashMap;

/// Build a CIDToGIDMap for Identity encodings
/// 
/// For Identity-H/V encodings, CID values equal the original GID values.
/// This function creates a map from CIDs to the new (remapped) GID values.
///
/// # Arguments
/// * `glyph_mapping` - HashMap of original GID to new GID
/// * `max_cid` - Maximum CID value to include in the map
///
/// # Returns
/// Binary CIDToGIDMap in big-endian format (2 bytes per entry)
pub fn build_identity_cid_map(
    glyph_mapping: &HashMap<u16, u16>,
    max_cid: u16,
) -> Vec<u8> {
    let map_size = ((max_cid as usize) + 1) * 2;
    let mut cid_to_gid_map = vec![0u8; map_size];
    
    // For Identity encoding: CID == original GID
    // So we map: CID -> glyph_mapping[CID]
    for cid in 0..=max_cid {
        let new_gid = glyph_mapping.get(&cid).copied().unwrap_or(0);
        let offset = (cid as usize) * 2;
        cid_to_gid_map[offset..offset + 2].copy_from_slice(&new_gid.to_be_bytes());
    }
    
    cid_to_gid_map
}

/// Build CIDToGIDMap based on encoding type
pub fn build_cid_to_gid_map_for_encoding(
    encoding: &FontEncoding,
    glyph_mapping: &HashMap<u16, u16>,
    max_cid: u16,
) -> Result<Vec<u8>, SubsetError> {
    match encoding {
        FontEncoding::Identity { .. } => {
            Ok(build_identity_cid_map(glyph_mapping, max_cid))
        }
    }
}

/// Determine the maximum CID value based on context
pub fn determine_max_cid(
    context: &FontContext,
    glyph_ids: &[u16],
) -> u16 {
    match context {
        FontContext::PdfType0 { encoding } => {
            match encoding {
                FontEncoding::Identity { .. } => {
                    // For Identity, max CID equals max original GID
                    glyph_ids.iter().copied().max().unwrap_or(0)
                }
            }
        }
        FontContext::Unknown => {
            // Conservative: use the highest glyph ID
            glyph_ids.iter().copied().max().unwrap_or(0)
        }
    }
}
```

#### 3.2.3 Main API Function (`src/subset/mod.rs` - additions)

```rust
use crate::subset::context::{FontContext, FontEncoding};
use crate::subset::cid_map::{build_cid_to_gid_map_for_encoding, determine_max_cid};

/// Create a subset font with glyph ID mapping and context-aware CID support
///
/// This function extends `subset_and_map` by accepting a context parameter that
/// provides information about how the font will be used, enabling correct
/// CIDToGIDMap generation for PDF embedding.
///
/// # Arguments
/// * `provider` - Font table provider
/// * `glyph_ids` - List of glyph IDs to include (must start with 0/.notdef)
/// * `profile` - Subset profile determining which tables to include
/// * `cmap_target` - Target character mapping format
/// * `context` - Usage context for the font (e.g., PDF encoding information)
///
/// # Returns
/// * `SubsetResult::Simple` for standard fonts
/// * `SubsetResult::Cid` for CID fonts with correct CIDToGIDMap
///
/// # Example
/// ```rust
/// use allsorts::subset::{subset_and_map_with_context, FontContext, FontEncoding};
/// 
/// let context = FontContext::PdfType0 {
///     encoding: FontEncoding::Identity { vertical: false },
/// };
/// 
/// let result = subset_and_map_with_context(
///     &provider,
///     &[0, 42, 43],
///     &SubsetProfile::Pdf,
///     CmapTarget::Unicode,
///     context,
/// )?;
/// 
/// if let SubsetResult::Cid { cid_to_gid_map, .. } = result {
///     // Use the correctly generated CIDToGIDMap for PDF embedding
///     pdf_font.set_cid_to_gid_map(cid_to_gid_map);
/// }
/// ```
pub fn subset_and_map_with_context(
    provider: &impl FontTableProvider,
    glyph_ids: &[u16],
    profile: &SubsetProfile,
    cmap_target: CmapTarget,
    context: FontContext,
) -> Result<SubsetResult, SubsetError> {
    // Step 1: Perform standard subsetting with mapping
    let (font_data, glyph_mapping) = subset_with_mapping(
        provider,
        glyph_ids,
        profile,
        cmap_target,
    )?;
    
    // Step 2: Determine if CID treatment is needed
    let needs_cid = match &context {
        FontContext::PdfType0 { encoding } => encoding.requires_cid(),
        FontContext::Unknown => {
            // Fall back to existing detection logic
            detect_cid_font(provider, glyph_ids)
        }
    };
    
    // Step 3: Generate appropriate result
    if needs_cid {
        let max_cid = determine_max_cid(&context, glyph_ids);
        
        // Generate CIDToGIDMap based on encoding
        let cid_to_gid_map = match &context {
            FontContext::PdfType0 { encoding } => {
                build_cid_to_gid_map_for_encoding(encoding, &glyph_mapping, max_cid)?
            }
            FontContext::Unknown => {
                // Fall back to existing (potentially incorrect) generation
                // This maintains backward compatibility
                build_cid_to_gid_map(None, &glyph_mapping, max_cid)
            }
        };
        
        Ok(SubsetResult::Cid {
            font_data,
            glyph_mapping,
            cid_to_gid_map,
        })
    } else {
        Ok(SubsetResult::Simple {
            font_data,
            glyph_mapping,
        })
    }
}
```

### 3.3 Testing Strategy

#### 3.3.1 Unit Tests

```rust
// tests/subset_context_tests.rs

#[test]
fn test_identity_h_encoding_detection() {
    let encoding = FontEncoding::from_pdf_name("Identity-H");
    assert_eq!(encoding, Some(FontEncoding::Identity { vertical: false }));
    assert!(encoding.unwrap().requires_cid());
}

#[test]
fn test_identity_v_encoding_detection() {
    let encoding = FontEncoding::from_pdf_name("Identity-V");
    assert_eq!(encoding, Some(FontEncoding::Identity { vertical: true }));
}

#[test]
fn test_unsupported_encoding_returns_none() {
    // Phase 1: Other encodings not yet supported
    assert_eq!(FontEncoding::from_pdf_name("GB-EUC-H"), None);
    assert_eq!(FontEncoding::from_pdf_name("90ms-RKSJ-H"), None);
}

#[test]
fn test_identity_cid_map_generation() {
    let mut mapping = HashMap::new();
    mapping.insert(0, 0);    // .notdef stays at 0
    mapping.insert(42, 1);   // GID 42 -> new GID 1
    mapping.insert(100, 2);  // GID 100 -> new GID 2
    
    let cid_map = build_identity_cid_map(&mapping, 100);
    
    // Verify CID 42 maps to new GID 1
    assert_eq!(u16::from_be_bytes([cid_map[84], cid_map[85]]), 1);
    
    // Verify CID 100 maps to new GID 2  
    assert_eq!(u16::from_be_bytes([cid_map[200], cid_map[201]]), 2);
    
    // Verify unmapped CID maps to 0
    assert_eq!(u16::from_be_bytes([cid_map[50], cid_map[51]]), 0);
}
```

#### 3.3.2 Integration Tests

```rust
// tests/identity_encoding_integration.rs

#[test]
fn test_subset_with_identity_h_context() {
    let font_data = include_bytes!("../tests/fonts/Calibri-CID.ttf");
    let provider = create_provider(font_data);
    
    let context = FontContext::PdfType0 {
        encoding: FontEncoding::Identity { vertical: false },
    };
    
    let result = subset_and_map_with_context(
        &provider,
        &[0, 143, 159, 178],
        &SubsetProfile::Pdf,
        CmapTarget::Unicode,
        context,
    ).unwrap();
    
    // Should return CID variant with correct map
    match result {
        SubsetResult::Cid { cid_to_gid_map, glyph_mapping, .. } => {
            // Verify CID 143 maps to its new GID
            let new_gid_143 = glyph_mapping[&143];
            let offset = 143 * 2;
            let mapped_gid = u16::from_be_bytes([
                cid_to_gid_map[offset],
                cid_to_gid_map[offset + 1],
            ]);
            assert_eq!(mapped_gid, new_gid_143);
        }
        _ => panic!("Expected CID result for Identity-H encoding"),
    }
}

#[test]
fn test_backwards_compatibility_unknown_context() {
    // Verify existing behavior still works
    let result = subset_and_map_with_context(
        &provider,
        &glyph_ids,
        &SubsetProfile::Pdf,
        CmapTarget::Unicode,
        FontContext::Unknown,
    ).unwrap();
    
    // Should still work, using heuristic detection
    assert!(matches!(result, SubsetResult::Simple { .. }) || 
            matches!(result, SubsetResult::Cid { .. }));
}
```

### 3.4 Migration Guide for Client

```rust
// BEFORE - Incorrect CIDToGIDMap
let result = subset_and_map(
    &provider,
    &glyph_ids,
    &SubsetProfile::Pdf,
    CmapTarget::Unicode,
)?;

// AFTER - Correct CIDToGIDMap for Identity-H
let context = FontContext::PdfType0 {
    encoding: FontEncoding::Identity { vertical: false },
};

let result = subset_and_map_with_context(
    &provider,
    &glyph_ids,
    &SubsetProfile::Pdf,
    CmapTarget::Unicode,
    context,
)?;

// The CIDToGIDMap will now be correct for Identity-H encoding
```

## 4. Success Criteria

### 4.1 Functional Requirements
- ✅ Identity-H encoding produces correct CIDToGIDMap
- ✅ Identity-V encoding produces correct CIDToGIDMap
- ✅ Unknown context falls back to existing behavior
- ✅ All existing tests continue to pass

### 4.2 Performance Requirements
- API overhead < 1ms for typical fonts
- CIDToGIDMap generation < 10ms for 10,000 CIDs
- Memory usage proportional to max CID value

### 4.3 Quality Metrics
- Test coverage > 95% for new code
- Zero breaking changes to existing API
- Clear documentation with examples

## 5. Risks & Mitigations

| Risk | Impact | Mitigation |
|------|--------|------------|
| Breaking existing API | High | Keep `subset_and_map` unchanged, add new function |
| Incorrect CID mapping | High | Comprehensive test suite with real fonts |
| Performance regression | Medium | Benchmark before/after, optimize hot paths |
| Client migration issues | Low | Provide clear migration guide and examples |

## 6. Dependencies

### 6.1 Internal Dependencies
- Existing `subset_with_mapping` function
- Existing `detect_cid_font` function
- HashMap from std::collections

### 6.2 External Dependencies
- None for Phase 1

## 7. Estimated Timeline

| Task | Duration | Notes |
|------|----------|-------|
| Implement context types | 2 hours | Simple enums and traits |
| Implement CID map generation | 3 hours | Core logic for Identity encoding |
| Integrate with subset_and_map | 2 hours | Hook up the pieces |
| Write unit tests | 3 hours | Comprehensive coverage |
| Write integration tests | 2 hours | Test with real fonts |
| Documentation | 2 hours | API docs and examples |
| **Total** | **14 hours** | ~2 days of work |

## 8. Next Steps

After Phase 1 is complete and validated:
1. Release as version 0.16.0 or 0.15.4
2. Client can immediately use for Identity-H/V fonts
3. Proceed to Phase 2 for API convenience improvements
4. Gather feedback on API ergonomics

## Appendix A: Example Usage

```rust
use allsorts::subset::{
    subset_and_map_with_context, 
    FontContext, 
    FontEncoding,
    SubsetProfile,
    CmapTarget,
    SubsetResult,
};

fn subset_pdf_font_with_identity_encoding(
    font_bytes: &[u8],
    used_glyphs: &[u16],
    is_vertical: bool,
) -> Result<PdfFontData, Box<dyn Error>> {
    let provider = create_provider(font_bytes)?;
    
    // Specify Identity encoding
    let context = FontContext::PdfType0 {
        encoding: FontEncoding::Identity { vertical: is_vertical },
    };
    
    // Perform subsetting with context
    let result = subset_and_map_with_context(
        &provider,
        used_glyphs,
        &SubsetProfile::Pdf,
        CmapTarget::Unicode,
        context,
    )?;
    
    // Extract CID font data
    match result {
        SubsetResult::Cid { font_data, glyph_mapping, cid_to_gid_map } => {
            println!("Created CID font with {} byte CIDToGIDMap", 
                     cid_to_gid_map.len());
            
            Ok(PdfFontData {
                font_bytes: font_data,
                cid_to_gid_map: Some(cid_to_gid_map),
                glyph_mapping,
            })
        }
        SubsetResult::Simple { .. } => {
            Err("Expected CID font for Identity encoding".into())
        }
    }
}
```

## Appendix B: Test Font Requirements

For Phase 1 testing, we need:

1. **Calibri-CID.ttf** - The client's problematic font
2. **NotoSans-Regular.ttf** - Standard TrueType font
3. **NotoSansCJK-Regular.otf** - CFF-based CID font
4. **Arial.ttf** - Simple font for regression testing

---

*Document Version: 1.0*  
*Created: 2024-08-21*  
*Phase: 1 of 5*  
*Priority: CRITICAL - Fixes production issue*