Read @README.md and the @docs/subsetting.md to understand this project and its current subsetting capabilities and APIs and then read @specs/250821_subset_and_map_with_context/README.md to understand the general context of the current upcoming changes and then implement the new changes below:

# Phase 1: Core Identity Encoding Support

## Executive Summary

Implement the minimal viable `subset_and_map_with_context` API that correctly handles Identity-H/V encodings, which covers 90% of modern PDF use cases and immediately fixes the client's CIDToGIDMap issue.

## 1. What We're Building

### 1.1 Technical Overview

**Goal:** Add support for context-aware subsetting with Identity encoding to fix CIDToGIDMap generation for PDF fonts.

**Core Problem:** Current subsetting generates incorrect CIDToGIDMap for Identity-H/V encodings, causing characters to render as '?' in PDFs.

**Solution:** Create `subset_and_map_with_context` API that understands Identity encoding semantics where CID == original GID.

### 1.2 Key Components

```rust
// New types for encoding context
pub enum FontEncoding {
    Identity { vertical: bool },  // Phase 1: Only this variant
}

pub enum FontContext {
    Unknown,
    PdfType0 { encoding: FontEncoding },
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

### 1.3 Expected Behavior

For Identity encodings (CID == original GID):
1. Perform standard subsetting with glyph mapping
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

## 3. TDD Implementation Steps

### 3.1 Preparation

1. **Read Documentation**
   - Review existing subset API in `src/subset/mod.rs`
   - Understand current CIDToGIDMap generation logic
   - Study the client's PDF rendering issue

2. **Verify Test Health**
   ```bash
   cargo test
   # Ensure all existing tests pass before starting
   ```
   
   **Stop if any tests fail.** Fix existing failures first.

### 3.2 Analysis & Planning

1. **High-Level Design**
   - **Goal:** Add context-aware subsetting with Identity encoding support
   - **Affected modules:** `src/subset/mod.rs`, new `src/subset/context.rs`, new `src/subset/cid_map.rs`
   - **External dependencies:** None for Phase 1

2. **Break Down Into Tasks**
   - Define `FontEncoding` and `FontContext` types
   - Implement Identity CIDToGIDMap generation logic
   - Create `subset_and_map_with_context` function
   - Integrate with existing subsetting logic

3. **Edge-Case Brainstorm**
   - **Valid inputs:** Identity-H/V encodings, typical glyph lists
   - **Boundary conditions:** Empty glyph list, single glyph, maximum CID
   - **Error scenarios:** Missing .notdef (glyph 0), non-Identity encoding in Phase 1
   - **Performance constraints:** CIDToGIDMap generation must be O(max_cid)

### 3.3 Define APIs & Signatures

Draft interfaces without implementation:

| Component / Function | Inputs | Outputs | Error Cases / Behaviors |
|---------------------|--------|---------|------------------------|
| `FontEncoding::from_pdf_name()` | `name: &str` | `Option<FontEncoding>` | Returns None for unsupported encodings |
| `FontEncoding::requires_cid()` | `&self` | `bool` | True for Identity encodings |
| `build_identity_cid_map()` | `glyph_mapping: &HashMap<u16, u16>, max_cid: u16` | `Vec<u8>` | Handles unmapped CIDs (map to 0) |
| `subset_and_map_with_context()` | `provider, glyph_ids, profile, cmap_target, context` | `Result<SubsetResult, SubsetError>` | Validates .notdef present, handles encoding |

```rust
// src/subset/context.rs - Signatures only, no implementation
pub enum FontEncoding {
    Identity { vertical: bool },
}

impl FontEncoding {
    pub fn from_pdf_name(name: &str) -> Option<Self>;
    pub fn requires_cid(&self) -> bool;
}

pub enum FontContext {
    Unknown,
    PdfType0 { encoding: FontEncoding },
}

// src/subset/cid_map.rs - Signatures only, no implementation
pub fn build_identity_cid_map(
    glyph_mapping: &HashMap<u16, u16>,
    max_cid: u16,
) -> Vec<u8>;

pub fn determine_max_cid(
    context: &FontContext,
    glyph_ids: &[u16],
) -> u16;

// src/subset/mod.rs - Signature only, no implementation
pub fn subset_and_map_with_context(
    provider: &impl FontTableProvider,
    glyph_ids: &[u16],
    profile: &SubsetProfile,
    cmap_target: CmapTarget,
    context: FontContext,
) -> Result<SubsetResult, SubsetError>;
```

## 4. Write Unit Tests (TDD)

**Write tests BEFORE any implementation. All tests should initially fail.**

### 4.1 Set Up Test Files

```
tests/
├── subset_context_tests.rs      (new - Phase 1 unit tests)
└── identity_encoding_tests.rs   (new - Integration tests)
```

### 4.2 Specify Test Cases

**Happy Paths:**
- Identity-H encoding detection and parsing
- Identity-V encoding detection and parsing
- Correct CIDToGIDMap generation for Identity encodings
- Context-aware subsetting returns CID result

**Edge Cases:**
- Empty glyph mapping
- Large CID values (up to u16::MAX)
- Single glyph subsetting

**Error Cases:**
- Unsupported encoding names return None
- Missing .notdef glyph in glyph list
- Invalid context configurations

### 4.3 Unit Tests (Write First - These Will Fail)

```rust
// tests/subset_context_tests.rs

#[cfg(test)]
mod phase1_context_tests {
    use allsorts::subset::context::{FontEncoding, FontContext};
    use allsorts::subset::cid_map::build_identity_cid_map;
    use std::collections::HashMap;

    // BASIC ENUM CONSTRUCTION TESTS
    #[test]
    fn test_font_encoding_identity_h() {
        // WILL FAIL: FontEncoding not implemented yet
        let encoding = FontEncoding::Identity { vertical: false };
        match encoding {
            FontEncoding::Identity { vertical } => assert!(!vertical),
        }
    }

    #[test]
    fn test_font_encoding_identity_v() {
        // WILL FAIL: FontEncoding not implemented yet
        let encoding = FontEncoding::Identity { vertical: true };
        match encoding {
            FontEncoding::Identity { vertical } => assert!(vertical),
        }
    }

    // ENCODING PARSING TESTS
    #[test]
    fn test_font_encoding_from_pdf_name_identity_h() {
        // WILL FAIL: from_pdf_name method not implemented yet
        let encoding = FontEncoding::from_pdf_name("Identity-H");
        assert_eq!(encoding, Some(FontEncoding::Identity { vertical: false }));
    }

    #[test]
    fn test_font_encoding_from_pdf_name_identity_v() {
        // WILL FAIL: from_pdf_name method not implemented yet
        let encoding = FontEncoding::from_pdf_name("Identity-V");
        assert_eq!(encoding, Some(FontEncoding::Identity { vertical: true }));
    }

    #[test]
    fn test_font_encoding_from_pdf_name_unsupported() {
        // WILL FAIL: from_pdf_name method not implemented yet
        // Phase 1: Other encodings return None
        let encoding = FontEncoding::from_pdf_name("GB-EUC-H");
        assert_eq!(encoding, None);
    }

    #[test]
    fn test_font_encoding_from_pdf_name_case_sensitive() {
        // WILL FAIL: from_pdf_name method not implemented yet
        let encoding = FontEncoding::from_pdf_name("identity-h");
        assert_eq!(encoding, None); // Should be case-sensitive
    }

    // CID REQUIREMENT TESTS
    #[test]
    fn test_font_encoding_requires_cid() {
        // WILL FAIL: requires_cid method not implemented yet
        let encoding = FontEncoding::Identity { vertical: false };
        assert!(encoding.requires_cid());
    }

    #[test]
    fn test_font_encoding_requires_cid_vertical() {
        // WILL FAIL: requires_cid method not implemented yet
        let encoding = FontEncoding::Identity { vertical: true };
        assert!(encoding.requires_cid());
    }

    // CONTEXT TESTS
    #[test]
    fn test_font_context_default() {
        // WILL FAIL: Default trait not implemented yet
        let context = FontContext::default();
        assert_eq!(context, FontContext::Unknown);
    }

    #[test]
    fn test_font_context_pdf_type0() {
        // WILL FAIL: FontContext enum not implemented yet
        let context = FontContext::PdfType0 {
            encoding: FontEncoding::Identity { vertical: false },
        };
        match context {
            FontContext::PdfType0 { encoding } => {
                assert_eq!(encoding, FontEncoding::Identity { vertical: false });
            }
            _ => panic!("Expected PdfType0 context"),
        }
    }

    // CID MAP GENERATION TESTS
    #[test]
    fn test_build_identity_cid_map_empty() {
        // WILL FAIL: build_identity_cid_map function not implemented yet
        let mapping = HashMap::new();
        let cid_map = build_identity_cid_map(&mapping, 0);
        assert_eq!(cid_map.len(), 2); // One entry (CID 0) = 2 bytes
        // CID 0 should map to GID 0 (default for unmapped)
        assert_eq!(u16::from_be_bytes([cid_map[0], cid_map[1]]), 0);
    }

    #[test]
    fn test_build_identity_cid_map_basic() {
        // WILL FAIL: build_identity_cid_map function not implemented yet
        let mut mapping = HashMap::new();
        mapping.insert(0, 0);  // .notdef
        mapping.insert(42, 1); // GID 42 -> new GID 1
        mapping.insert(100, 2); // GID 100 -> new GID 2
        
        let cid_map = build_identity_cid_map(&mapping, 100);
        
        // Should have 101 entries * 2 bytes = 202 bytes (CID 0-100)
        assert_eq!(cid_map.len(), 202);
        
        // CID 0 should map to new GID 0
        assert_eq!(u16::from_be_bytes([cid_map[0], cid_map[1]]), 0);
        
        // CID 42 should map to new GID 1
        assert_eq!(u16::from_be_bytes([cid_map[84], cid_map[85]]), 1);
        
        // CID 100 should map to new GID 2
        assert_eq!(u16::from_be_bytes([cid_map[200], cid_map[201]]), 2);
        
        // Unmapped CID 50 should map to 0 (default)
        assert_eq!(u16::from_be_bytes([cid_map[100], cid_map[101]]), 0);
    }

    #[test]
    fn test_build_identity_cid_map_large() {
        // WILL FAIL: build_identity_cid_map function not implemented yet
        let mut mapping = HashMap::new();
        for i in 0..1000u16 {
            mapping.insert(i * 2, i); // Even GIDs map to sequential new GIDs
        }
        
        let cid_map = build_identity_cid_map(&mapping, 2000);
        
        // Should have 2001 entries * 2 bytes = 4002 bytes (CID 0-2000)
        assert_eq!(cid_map.len(), 4002);
        
        // Check some mappings (CID == original GID for Identity encoding)
        assert_eq!(u16::from_be_bytes([cid_map[0], cid_map[1]]), 0);   // CID 0 -> new GID 0
        assert_eq!(u16::from_be_bytes([cid_map[200], cid_map[201]]), 100); // CID 100 -> new GID 50
        assert_eq!(u16::from_be_bytes([cid_map[400], cid_map[401]]), 200); // CID 200 -> new GID 100
        
        // Unmapped CID should map to 0
        assert_eq!(u16::from_be_bytes([cid_map[2], cid_map[3]]), 0);  // CID 1 (odd) -> GID 0
    }

    #[test]
    fn test_build_identity_cid_map_max_cid_boundary() {
        // WILL FAIL: build_identity_cid_map function not implemented yet
        let mut mapping = HashMap::new();
        mapping.insert(65535, 1); // Max u16 value
        
        let cid_map = build_identity_cid_map(&mapping, 65535);
        
        // Should handle max CID value
        let offset = 65535 * 2;
        assert_eq!(u16::from_be_bytes([cid_map[offset], cid_map[offset + 1]]), 1);
    }

    // MAX CID DETERMINATION TESTS
    #[test]
    fn test_determine_max_cid() {
        // WILL FAIL: determine_max_cid function not implemented yet
        use allsorts::subset::cid_map::determine_max_cid;
        
        let context = FontContext::PdfType0 {
            encoding: FontEncoding::Identity { vertical: false },
        };
        
        let glyph_ids = vec![0, 42, 100, 200];
        let max_cid = determine_max_cid(&context, &glyph_ids);
        assert_eq!(max_cid, 200);
    }

    #[test]
    fn test_determine_max_cid_empty_list() {
        // WILL FAIL: determine_max_cid function not implemented yet
        use allsorts::subset::cid_map::determine_max_cid;
        
        let context = FontContext::PdfType0 {
            encoding: FontEncoding::Identity { vertical: false },
        };
        
        let glyph_ids = vec![];
        let max_cid = determine_max_cid(&context, &glyph_ids);
        assert_eq!(max_cid, 0);
    }

    #[test]
    fn test_determine_max_cid_unknown_context() {
        // WILL FAIL: determine_max_cid function not implemented yet
        use allsorts::subset::cid_map::determine_max_cid;
        
        let context = FontContext::Unknown;
        let glyph_ids = vec![0, 42, 100];
        let max_cid = determine_max_cid(&context, &glyph_ids);
        assert_eq!(max_cid, 100); // Should use conservative approach
    }
}
```

### 4.4 Integration Tests (Write First - These Will Fail)

```rust
// tests/identity_encoding_tests.rs

#[cfg(test)]
mod phase1_integration_tests {
    use allsorts::subset::{
        subset_and_map_with_context,
        FontContext,
        FontEncoding,
        SubsetProfile,
        CmapTarget,
        SubsetResult,
    };
    use allsorts::FontTableProvider;
    use std::collections::HashMap;
    
    fn create_provider(font_data: &[u8]) -> impl FontTableProvider {
        // Helper function to create font provider
        // WILL FAIL until helper is implemented
        unimplemented!("Helper function not implemented yet")
    }
    
    #[test]
    fn test_subset_with_identity_h_returns_cid() {
        // WILL FAIL: subset_and_map_with_context function not implemented yet
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
        );
        
        assert!(result.is_ok());
        
        match result.unwrap() {
            SubsetResult::Cid { cid_to_gid_map, glyph_mapping, .. } => {
                // Should be CID result for Identity encoding
                assert!(!cid_to_gid_map.is_empty());
                assert!(glyph_mapping.contains_key(&143));
                // Verify it's the correct size for max CID 178
                assert_eq!(cid_to_gid_map.len(), 179 * 2); // 0-178 inclusive
            }
            _ => panic!("Expected CID result for Identity-H encoding"),
        }
    }
    
    #[test]
    fn test_subset_with_unknown_context_uses_heuristics() {
        // WILL FAIL: subset_and_map_with_context function not implemented yet
        let font_data = include_bytes!("../tests/fonts/NotoSans-Regular.ttf");
        let provider = create_provider(font_data);
        
        let result = subset_and_map_with_context(
            &provider,
            &[0, 1, 2, 3],
            &SubsetProfile::Pdf,
            CmapTarget::Unicode,
            FontContext::Unknown,
        );
        
        assert!(result.is_ok());
        // Should still work with Unknown context using existing logic
        // Result type depends on font characteristics
    }
    
    #[test]
    fn test_identity_cid_map_correctness() {
        // WILL FAIL: subset_and_map_with_context function not implemented yet
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
        
        if let SubsetResult::Cid { cid_to_gid_map, glyph_mapping, .. } = result {
            // CRITICAL TEST: Verify Identity encoding semantics
            // For Identity encoding: CID == original GID
            // So CID 143 should map to glyph_mapping[143]
            
            let new_gid_143 = glyph_mapping[&143];
            let offset = 143 * 2;
            let mapped_gid = u16::from_be_bytes([
                cid_to_gid_map[offset],
                cid_to_gid_map[offset + 1],
            ]);
            assert_eq!(mapped_gid, new_gid_143);
            
            // Verify CID 159 maps correctly
            let new_gid_159 = glyph_mapping[&159];
            let offset = 159 * 2;
            let mapped_gid = u16::from_be_bytes([
                cid_to_gid_map[offset],
                cid_to_gid_map[offset + 1],
            ]);
            assert_eq!(mapped_gid, new_gid_159);
            
            // Verify CID 178 maps correctly
            let new_gid_178 = glyph_mapping[&178];
            let offset = 178 * 2;
            let mapped_gid = u16::from_be_bytes([
                cid_to_gid_map[offset],
                cid_to_gid_map[offset + 1],
            ]);
            assert_eq!(mapped_gid, new_gid_178);
        } else {
            panic!("Expected CID result for Identity encoding");
        }
    }
    
    #[test]
    fn test_missing_notdef_error() {
        // WILL FAIL: subset_and_map_with_context function not implemented yet
        let font_data = include_bytes!("../tests/fonts/NotoSans-Regular.ttf");
        let provider = create_provider(font_data);
        
        let context = FontContext::PdfType0 {
            encoding: FontEncoding::Identity { vertical: false },
        };
        
        let result = subset_and_map_with_context(
            &provider,
            &[1, 2, 3], // Missing 0 (.notdef)
            &SubsetProfile::Pdf,
            CmapTarget::Unicode,
            context,
        );
        
        assert!(result.is_err());
        // Should error because .notdef (glyph 0) is required for PDF fonts
    }

    #[test]
    fn test_identity_v_encoding() {
        // WILL FAIL: subset_and_map_with_context function not implemented yet
        let font_data = include_bytes!("../tests/fonts/NotoSansCJK-Regular.otf");
        let provider = create_provider(font_data);
        
        let context = FontContext::PdfType0 {
            encoding: FontEncoding::Identity { vertical: true },
        };
        
        let result = subset_and_map_with_context(
            &provider,
            &[0, 50, 100],
            &SubsetProfile::Pdf,
            CmapTarget::Unicode,
            context,
        );
        
        assert!(result.is_ok());
        // Should work with Identity-V encoding too
        match result.unwrap() {
            SubsetResult::Cid { .. } => {
                // Success - should generate CID result for Identity-V
            }
            _ => panic!("Expected CID result for Identity-V encoding"),
        }
    }

    #[test]
    fn test_single_glyph_subsetting() {
        // WILL FAIL: subset_and_map_with_context function not implemented yet
        let font_data = include_bytes!("../tests/fonts/Calibri-CID.ttf");
        let provider = create_provider(font_data);
        
        let context = FontContext::PdfType0 {
            encoding: FontEncoding::Identity { vertical: false },
        };
        
        let result = subset_and_map_with_context(
            &provider,
            &[0], // Only .notdef
            &SubsetProfile::Pdf,
            CmapTarget::Unicode,
            context,
        );
        
        assert!(result.is_ok());
        match result.unwrap() {
            SubsetResult::Cid { cid_to_gid_map, glyph_mapping, .. } => {
                assert_eq!(cid_to_gid_map.len(), 2); // Only CID 0 -> 2 bytes
                assert_eq!(glyph_mapping.len(), 1); // Only one glyph mapped
            }
            _ => panic!("Expected CID result"),
        }
    }
}
```

## 5. Implement Functions Incrementally

**For each function or module:**

### 5.1 Minimal Implementation

#### Step 1: Run tests to see failures
```bash
cargo test --test subset_context_tests
# All tests should fail with compilation errors - this is expected
```

#### Step 2: Implement minimal FontEncoding enum
```rust
// src/subset/context.rs
#[derive(Debug, Clone, PartialEq)]
pub enum FontEncoding {
    Identity { vertical: bool },
}
```

#### Step 3: Run specific test
```bash
cargo test --test subset_context_tests test_font_encoding_identity_h
# This one test should now pass
```

### 5.2 Iterate Through Each Component

**TDD Cycle for each component:**

1. **Implement `FontEncoding::from_pdf_name`**
   ```bash
   cargo test --test subset_context_tests --filter from_pdf_name
   # Fix failures until tests pass
   ```

2. **Implement `FontEncoding::requires_cid`**
   ```bash
   cargo test --test subset_context_tests --filter requires_cid
   # Fix failures until tests pass
   ```

3. **Implement `FontContext` enum and Default**
   ```bash
   cargo test --test subset_context_tests --filter font_context
   # Fix failures until tests pass
   ```

4. **Implement `build_identity_cid_map`**
   ```bash
   cargo test --test subset_context_tests --filter build_identity_cid_map
   # Fix failures until tests pass
   ```

5. **Implement `determine_max_cid`**
   ```bash
   cargo test --test subset_context_tests --filter determine_max_cid
   # Fix failures until tests pass
   ```

6. **Implement `subset_and_map_with_context`**
   ```bash
   cargo test --test identity_encoding_tests
   # Fix failures until integration tests pass
   ```

### 5.3 Repeat Until All Tests Pass

Continue implementing incrementally until all unit tests pass:

```bash
cargo test --test subset_context_tests
# Should show all tests passing
```

## 6. Full-Suite Integration

### 6.1 Execute Entire Test Suite

```bash
cargo test
# Run ALL tests to ensure no regressions
```

### 6.2 Fix Regressions

- Address any failures in existing tests
- Ensure integration tests pass
- Verify performance hasn't degraded

## 7. Implementation Details

### 7.1 File Structure

```
src/subset/
├── mod.rs                    (existing, needs updates)
├── context.rs                (new - context types)
└── cid_map.rs               (new - CIDToGIDMap generation)
```

### 7.2 Context Types Implementation (`src/subset/context.rs`)

**Implement ONLY after tests are written and failing:**

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

### 7.3 CIDToGIDMap Generation Implementation (`src/subset/cid_map.rs`)

**Implement ONLY after tests are written and failing:**

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

### 7.4 Main API Function Implementation (`src/subset/mod.rs` - additions)

**Implement ONLY after tests are written and failing:**

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

## 8. Documentation & Review

### 8.1 Update Documentation

- Update API documentation for new functions
- Add examples to README if needed
- Update changelog with new features

### 8.2 Ensure Code Comments

- Document all public interfaces
- Explain Identity encoding semantics
- Add examples for complex logic

### 8.3 Peer Review

- Open pull request for review
- Address feedback and suggestions
- Ensure code follows project conventions

### 8.4 Migration Guide for Client

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

## 9. Success Criteria

### 9.1 Functional Requirements
- ✅ Identity-H encoding produces correct CIDToGIDMap
- ✅ Identity-V encoding produces correct CIDToGIDMap
- ✅ Unknown context falls back to existing behavior
- ✅ All existing tests continue to pass

### 9.2 Performance Requirements
- API overhead < 1ms for typical fonts
- CIDToGIDMap generation < 10ms for 10,000 CIDs
- Memory usage proportional to max CID value

### 9.3 Quality Metrics
- Test coverage > 95% for new code
- Zero breaking changes to existing API
- Clear documentation with examples

## 10. Risks & Mitigations

| Risk | Impact | Mitigation |
|------|--------|------------|
| Breaking existing API | High | Keep `subset_and_map` unchanged, add new function |
| Incorrect CID mapping | High | Comprehensive test suite with real fonts |
| Performance regression | Medium | Benchmark before/after, optimize hot paths |
| Client migration issues | Low | Provide clear migration guide and examples |

## 11. Dependencies

### 11.1 Internal Dependencies
- Existing `subset_with_mapping` function
- Existing `detect_cid_font` function
- HashMap from std::collections

### 11.2 External Dependencies
- None for Phase 1

## 12. Estimated Timeline

| Task | Duration | Notes |
|------|----------|-------|
| Implement context types | 2 hours | Simple enums and traits |
| Implement CID map generation | 3 hours | Core logic for Identity encoding |
| Integrate with subset_and_map | 2 hours | Hook up the pieces |
| Write unit tests | 3 hours | Comprehensive coverage |
| Write integration tests | 2 hours | Test with real fonts |
| Documentation | 2 hours | API docs and examples |
| **Total** | **14 hours** | ~2 days of work |

## 13. Next Steps

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

## 14. TDD Phase 1 Checklist

### Preparation
- [ ] Read existing subset API documentation
- [ ] Understand current CIDToGIDMap generation issue
- [ ] Run `cargo test` to verify all existing tests pass
- [ ] Stop and fix any failing tests before proceeding

### Analysis & Planning
- [ ] Define high-level goal: Context-aware subsetting for Identity encodings
- [ ] Break down into sub-tasks: context types, CID map generation, main API
- [ ] Identify edge cases: empty lists, large CIDs, missing .notdef
- [ ] Plan file structure: `context.rs`, `cid_map.rs`, updates to `mod.rs`

### API Definition
- [ ] Draft `FontEncoding` enum with Identity variant
- [ ] Draft `FontContext` enum with PdfType0 variant
- [ ] Define `build_identity_cid_map` function signature
- [ ] Define `subset_and_map_with_context` function signature
- [ ] Create interface table with inputs, outputs, error cases

### Write Unit Tests (TDD)
- [ ] Create `tests/subset_context_tests.rs`
- [ ] Write failing tests for `FontEncoding` construction
- [ ] Write failing tests for `FontEncoding::from_pdf_name`
- [ ] Write failing tests for `FontEncoding::requires_cid`
- [ ] Write failing tests for `FontContext` variants
- [ ] Write failing tests for `build_identity_cid_map` (empty, basic, large, boundary)
- [ ] Write failing tests for `determine_max_cid`
- [ ] Create `tests/identity_encoding_tests.rs`
- [ ] Write failing integration tests for Identity-H encoding
- [ ] Write failing integration tests for Identity-V encoding
- [ ] Write failing tests for edge cases (missing .notdef, single glyph)
- [ ] Write failing tests for backwards compatibility
- [ ] Verify all tests fail initially: `cargo test --test subset_context_tests`

### Implement Functions Incrementally
- [ ] Implement minimal `FontEncoding` enum
- [ ] Run tests, verify basic enum tests pass
- [ ] Implement `FontEncoding::from_pdf_name`
- [ ] Run tests, verify parsing tests pass
- [ ] Implement `FontEncoding::requires_cid`
- [ ] Run tests, verify CID requirement tests pass
- [ ] Implement `FontContext` enum and Default
- [ ] Run tests, verify context tests pass
- [ ] Implement `build_identity_cid_map`
- [ ] Run tests, verify CID map generation tests pass
- [ ] Implement `determine_max_cid`
- [ ] Run tests, verify max CID tests pass
- [ ] Implement `subset_and_map_with_context`
- [ ] Run integration tests, verify they pass
- [ ] All unit tests passing: `cargo test --test subset_context_tests`
- [ ] All integration tests passing: `cargo test --test identity_encoding_tests`

### Full-Suite Integration
- [ ] Run entire test suite: `cargo test`
- [ ] Verify no regressions in existing functionality
- [ ] Fix any failing tests
- [ ] Ensure all tests pass consistently

### Documentation & Review
- [ ] Add comprehensive API documentation
- [ ] Update project documentation with new features
- [ ] Add code comments explaining Identity encoding semantics
- [ ] Create migration guide for clients
- [ ] Open pull request for review
- [ ] Address review feedback
- [ ] Merge when approved

### Verification
- [ ] Test with client's problematic Calibri-CID.ttf font
- [ ] Verify CIDToGIDMap generation fixes rendering issue
- [ ] Measure performance impact (should be minimal)
- [ ] Confirm 40% file size reduction is maintained
- [ ] Validate backwards compatibility with existing usage

---

*Document Version: 2.0 - TDD Edition*  
*Updated: 2024-08-21*  
*Phase: 1 of 5*  
*Priority: CRITICAL - Fixes production issue*  
*Methodology: Test-Driven Development*