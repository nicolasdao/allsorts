# CID Font Subsetting API Enhancement for Allsorts

## Executive Summary

The current `subset_and_map` API in allsorts does not properly handle CID (Character Identifier) fonts used in PDFs, resulting in broken glyph rendering when fonts are aggressively subsetted. This document proposes an enhancement to fix this critical issue, enabling proper CID font subsetting that can reduce PDF file sizes by up to 35% while maintaining correct character rendering.

## Problem Statement

### Current Behavior

When subsetting CID fonts (Type 0 fonts with CIDFontType0/CIDFontType2 descendants), the `subset_and_map` API returns a mapping of `old_gid -> new_gid` for only the glyphs that were included in the subset. However, CID fonts in PDFs require a complete CIDToGIDMap that maps EVERY possible CID (Character ID) to its corresponding GID (Glyph ID), not just the ones we're subsetting.

### The Three-Level Mapping Chain

CID fonts use a three-level mapping:
```
Character Code -> CID -> GID -> Glyph Data
```

1. **Character Code -> CID**: Handled by the Encoding CMap (e.g., Identity-H)
2. **CID -> GID**: Handled by the CIDToGIDMap
3. **GID -> Glyph Data**: Handled by the font file (glyf/CFF tables)

### The Problem in Practice

When we subset a CID font from 256 glyphs to 25 glyphs:
- **Current API returns**: Mapping for 25 glyphs (old_gid -> new_gid)
- **What we need**: CIDToGIDMap with entries for ALL 65536 possible CIDs

Without the complete mapping, unmapped CIDs default to GID 0 (.notdef), causing characters to render as "?" or boxes.

### Real-World Impact

Testing with actual PDF files shows:
- **With identity preservation** (including all glyphs): 100KB file size, correct rendering ✅
- **With aggressive subsetting** (only used glyphs): 65KB file size, broken rendering ❌
- **Potential savings lost**: 35KB (35% additional compression)

## Proposed Solution

### Option 1: Enhanced `subset_and_map` API (Recommended)

Enhance the existing API to detect and handle CID fonts specially:

```rust
pub fn subset_and_map(
    provider: &impl FontTableProvider,
    glyph_ids: &[u16],
    profile: &SubsetProfile,
    cmap_target: CmapTarget,
) -> Result<SubsetResult, SubsetError> {
    // Returns enhanced result structure
}

pub enum SubsetResult {
    /// Standard TrueType/OpenType font
    Simple {
        font_data: Vec<u8>,
        glyph_mapping: HashMap<u16, u16>,
    },
    /// CID font requiring special handling
    Cid {
        font_data: Vec<u8>,
        glyph_mapping: HashMap<u16, u16>,
        cid_to_gid_map: Vec<u8>, // Complete CIDToGIDMap for PDF embedding
    },
}
```

### Option 2: New Dedicated API

Add a specialized API for CID font subsetting:

```rust
pub fn subset_cid_font(
    provider: &impl FontTableProvider,
    used_cids: &[u16],              // CIDs actually used in the document
    original_cid_to_gid: Option<&[u16]>, // Original CIDToGIDMap (None = Identity)
    max_cid: u16,                    // Maximum CID that needs to be mapped
    profile: &SubsetProfile,
    cmap_target: CmapTarget,
) -> Result<CidSubsetResult, SubsetError> {
    // Implementation
}

pub struct CidSubsetResult {
    pub font_data: Vec<u8>,
    pub cid_to_gid_map: Vec<u8>,    // Ready-to-embed CIDToGIDMap
    pub glyph_mapping: HashMap<u16, u16>, // For reference
}
```

## Test Cases

### Test 1: Basic CID Font Subsetting

```rust
#[test]
fn test_cid_font_subsetting_basic() {
    // Setup: Load a CID font (e.g., Calibri from a PDF)
    let font_data = include_bytes!("../fixtures/calibri_cid.otf");
    let provider = create_font_provider(font_data);
    
    // We want to keep only these glyphs (from actual usage):
    // GID 0 (.notdef), GID 45 (dash), GID 46 (period), GID 143 (bullet)
    let glyph_ids = vec![0, 45, 46, 143];
    
    // The original font uses Identity CIDToGIDMap
    // So CID 45 -> GID 45, CID 46 -> GID 46, CID 143 -> GID 143
    
    // Perform subsetting
    let result = subset_and_map(
        &provider,
        &glyph_ids,
        &SubsetProfile::Pdf,
        CmapTarget::Unicode,
    ).unwrap();
    
    match result {
        SubsetResult::Cid { font_data, glyph_mapping, cid_to_gid_map } => {
            // Verify the subset font has 4 glyphs
            let subset_provider = create_font_provider(&font_data);
            let maxp = read_maxp_table(&subset_provider);
            assert_eq!(maxp.num_glyphs, 4);
            
            // Verify glyph mapping (old -> new)
            assert_eq!(glyph_mapping[&0], 0);   // .notdef stays at 0
            assert_eq!(glyph_mapping[&45], 1);  // dash becomes GID 1
            assert_eq!(glyph_mapping[&46], 2);  // period becomes GID 2
            assert_eq!(glyph_mapping[&143], 3); // bullet becomes GID 3
            
            // CRITICAL: Verify CIDToGIDMap has correct entries
            // The map should be at least 144*2 bytes (for CIDs 0-143)
            assert!(cid_to_gid_map.len() >= 288);
            
            // Check specific CID mappings in the generated map
            let cid_45_gid = u16::from_be_bytes([cid_to_gid_map[90], cid_to_gid_map[91]]);
            let cid_46_gid = u16::from_be_bytes([cid_to_gid_map[92], cid_to_gid_map[93]]);
            let cid_143_gid = u16::from_be_bytes([cid_to_gid_map[286], cid_to_gid_map[287]]);
            
            assert_eq!(cid_45_gid, 1);  // CID 45 maps to new GID 1
            assert_eq!(cid_46_gid, 2);  // CID 46 maps to new GID 2
            assert_eq!(cid_143_gid, 3); // CID 143 maps to new GID 3
            
            // Unmapped CIDs should map to 0 (.notdef)
            let cid_100_gid = u16::from_be_bytes([cid_to_gid_map[200], cid_to_gid_map[201]]);
            assert_eq!(cid_100_gid, 0); // Unmapped CID -> .notdef
        }
        _ => panic!("Expected CID font result"),
    }
}
```

### Test 2: CID Font with Existing CIDToGIDMap

```rust
#[test]
fn test_cid_font_with_existing_map() {
    // Setup: Font with non-identity CIDToGIDMap
    let font_data = include_bytes!("../fixtures/times_cid_mapped.otf");
    let provider = create_font_provider(font_data);
    
    // Original mapping (simplified example):
    // CID 65 ('A') -> GID 100
    // CID 66 ('B') -> GID 101  
    // CID 67 ('C') -> GID 102
    let original_cid_to_gid = vec![
        // ... entries for CIDs 0-64 ...
        0, 100,  // CID 65 -> GID 100
        0, 101,  // CID 66 -> GID 101
        0, 102,  // CID 67 -> GID 102
        // ... more entries ...
    ];
    
    // We want to keep GIDs: 0, 100, 102 (keeping 'A' and 'C')
    let glyph_ids = vec![0, 100, 102];
    
    // Perform subsetting with original map context
    let result = subset_cid_font(
        &provider,
        &[65, 67],  // Used CIDs
        Some(&original_cid_to_gid),
        255,        // max_cid
        &SubsetProfile::Pdf,
        CmapTarget::Unicode,
    ).unwrap();
    
    // Verify the new CIDToGIDMap
    let map = result.cid_to_gid_map;
    
    // CID 65 should map to new GID for old GID 100
    let cid_65_gid = u16::from_be_bytes([map[130], map[131]]);
    assert_eq!(cid_65_gid, 1); // Old GID 100 -> new GID 1
    
    // CID 66 should map to 0 (not included)
    let cid_66_gid = u16::from_be_bytes([map[132], map[133]]);
    assert_eq!(cid_66_gid, 0); // Not included -> .notdef
    
    // CID 67 should map to new GID for old GID 102
    let cid_67_gid = u16::from_be_bytes([map[134], map[135]]);
    assert_eq!(cid_67_gid, 2); // Old GID 102 -> new GID 2
}
```

### Test 3: Edge Case - High CID Values

```rust
#[test]
fn test_cid_font_high_cid_values() {
    // Real-world case: Fonts often have special characters at high CIDs
    let font_data = include_bytes!("../fixtures/arial_cid.otf");
    let provider = create_font_provider(font_data);
    
    // Include glyphs with high CIDs (common for Asian fonts)
    let glyph_ids = vec![0, 143, 8212, 8226]; // .notdef, bullet, em-dash, bullet
    
    let result = subset_and_map(
        &provider,
        &glyph_ids,
        &SubsetProfile::Pdf,
        CmapTarget::Unicode,
    ).unwrap();
    
    match result {
        SubsetResult::Cid { cid_to_gid_map, .. } => {
            // Map must be large enough for highest CID
            assert!(cid_to_gid_map.len() >= 8227 * 2);
            
            // Verify high CID mappings work
            let cid_8226_offset = 8226 * 2;
            let cid_8226_gid = u16::from_be_bytes([
                cid_to_gid_map[cid_8226_offset],
                cid_to_gid_map[cid_8226_offset + 1],
            ]);
            assert_ne!(cid_8226_gid, 0); // Should map to valid GID
        }
        _ => panic!("Expected CID font result"),
    }
}
```

### Test 4: Regression Test - PDF Glyph Zero Detector

```rust
#[test]
fn test_no_glyph_zero_issues() {
    // This test ensures our fix prevents the "glyph zero" problem
    // where characters render as '?' in PDF viewers
    
    let font_data = include_bytes!("../fixtures/calibri_from_pdf.otf");
    let provider = create_font_provider(font_data);
    
    // Subset with typical usage from a real PDF
    let used_glyphs = vec![
        0, 32, 33, 45, 46, 65, 66, 67, 68, 69, 70, 71, 72, 73, 74,
        97, 98, 99, 100, 101, 102, 103, 104, 105, 143, // bullet
    ];
    
    let result = subset_and_map(
        &provider,
        &used_glyphs,
        &SubsetProfile::Pdf,
        CmapTarget::Unicode,
    ).unwrap();
    
    match result {
        SubsetResult::Cid { cid_to_gid_map, .. } => {
            // Simulate what PDF readers do: look up common CIDs
            let test_cids = vec![32, 65, 97, 143]; // space, A, a, bullet
            
            for cid in test_cids {
                let offset = cid * 2;
                let gid = u16::from_be_bytes([
                    cid_to_gid_map[offset],
                    cid_to_gid_map[offset + 1],
                ]);
                
                // No CID should map to 0 if it was in our used set
                if used_glyphs.contains(&cid) {
                    assert_ne!(gid, 0, "CID {} incorrectly maps to GID 0", cid);
                }
            }
        }
        _ => panic!("Expected CID font result"),
    }
}
```

## Implementation Guidance

### 1. Detecting CID Fonts

A font is a CID font if:
- It's embedded in a PDF as part of a Type 0 font structure
- The font tables indicate CFF with CID structure
- The font is TrueType but used with a CIDToGIDMap in PDF

```rust
fn is_cid_font(provider: &impl FontTableProvider) -> bool {
    // Check for CFF table with CID structure
    if let Ok(Some(cff_data)) = provider.table_data(tag::CFF) {
        // Parse CFF header and check for CID
        if let Ok(cff) = parse_cff(cff_data) {
            return cff.is_cid_keyed();
        }
    }
    
    // For TrueType fonts, we need external context (from PDF structure)
    // This might need to be passed as a parameter
    false
}
```

### 2. Building the CIDToGIDMap

```rust
fn build_cid_to_gid_map(
    original_map: Option<&[u16]>,
    glyph_remapping: &HashMap<u16, u16>,
    max_cid: u16,
) -> Vec<u8> {
    let map_size = (max_cid as usize + 1) * 2;
    let mut map = vec![0u8; map_size];
    
    // If we have an original map, update it with new GIDs
    if let Some(orig) = original_map {
        for cid in 0..=max_cid {
            let cid_offset = cid as usize * 2;
            if cid_offset + 1 < orig.len() {
                // Get original GID for this CID
                let old_gid = u16::from_be_bytes([orig[cid_offset], orig[cid_offset + 1]]);
                
                // Map to new GID if it was included
                let new_gid = glyph_remapping.get(&old_gid).copied().unwrap_or(0);
                
                // Write to map (big-endian)
                map[cid_offset] = (new_gid >> 8) as u8;
                map[cid_offset + 1] = (new_gid & 0xFF) as u8;
            }
        }
    } else {
        // Identity mapping: CID == original GID
        for cid in 0..=max_cid {
            let new_gid = glyph_remapping.get(&cid).copied().unwrap_or(0);
            let cid_offset = cid as usize * 2;
            map[cid_offset] = (new_gid >> 8) as u8;
            map[cid_offset + 1] = (new_gid & 0xFF) as u8;
        }
    }
    
    map
}
```

### 3. Integration Points

The fix should integrate with existing code paths:

```rust
// In subset_with_mapping function
pub fn subset_with_mapping(
    provider: &impl FontTableProvider,
    glyph_ids: &[u16],
    profile: &SubsetProfile,
    cmap_target: CmapTarget,
) -> Result<SubsetResult, SubsetError> {
    // ... existing validation ...
    
    // Detect if this is a CID font
    let is_cid = detect_cid_font(provider);
    
    if is_cid {
        // Special handling for CID fonts
        let (font_data, mapping) = perform_subsetting(provider, glyph_ids, profile)?;
        
        // Determine max CID (from context or analysis)
        let max_cid = determine_max_cid(provider, glyph_ids);
        
        // Build complete CIDToGIDMap
        let cid_to_gid_map = build_cid_to_gid_map(None, &mapping, max_cid);
        
        Ok(SubsetResult::Cid {
            font_data,
            glyph_mapping: mapping,
            cid_to_gid_map,
        })
    } else {
        // Existing behavior for regular fonts
        let (font_data, mapping) = perform_subsetting(provider, glyph_ids, profile)?;
        
        Ok(SubsetResult::Simple {
            font_data,
            glyph_mapping: mapping,
        })
    }
}
```

## Expected Outcomes

### Correctness
- All subsetted CID fonts render correctly in PDF viewers
- No "?" or box characters for included glyphs
- Pass the "PDF Glyph Zero Detector" test

### Performance
- 35% additional file size reduction for PDFs with CID fonts
- Example: 100KB → 65KB for typical business PDFs
- No performance regression in subsetting speed

### Compatibility
- Backward compatible with existing API users
- Existing non-CID font subsetting unchanged
- Clear migration path for users

## Success Criteria

1. **All tests pass**: The provided test cases should pass
2. **Real-world validation**: Test with actual PDF files:
   - Calibri, Arial, Times New Roman CID fonts
   - Asian fonts with high CID values
   - Fonts with custom CIDToGIDMaps
3. **No rendering issues**: PDFs render correctly in:
   - Adobe Acrobat
   - Chrome/Firefox PDF viewers
   - macOS Preview
   - pdftotext extracts correct text

## References

- [PDF Reference 1.7 - Section 5.6: CID-Keyed Fonts](https://www.adobe.com/devnet/pdf/pdf_reference.html)
- [OpenType Specification - CFF Table](https://docs.microsoft.com/en-us/typography/opentype/spec/cff)
- [CID Font Technology Overview](https://adobe-type-tools.github.io/font-tech-notes/pdfs/5092.CIDFontTechnology.pdf)

## Appendix: Current Bug Demonstration

```bash
# Current behavior with aggressive subsetting
$ ./pdf_compress_cli input.pdf output.pdf --aggressive
$ ./pdf_glyph_zero_detector output.pdf

❌ RESULT: Glyph mapping issues CONFIRMED!
Missing glyphs: 65498/65536 (99.9%)

# With proposed fix
$ ./pdf_compress_cli input.pdf output.pdf --aggressive
$ ./pdf_glyph_zero_detector output.pdf

✅ RESULT: No missing glyphs detected!
File size: 65KB (vs 100KB with identity preservation)
```

## Contact

For questions or clarifications about this specification:
- GitHub Issue: [Link to issue in allsorts fork]
- Related PR in pdf-compress: [Link to PR showing the problem]
- Test files available at: [Link to test PDF files]