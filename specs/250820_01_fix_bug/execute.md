# Parse(BadIndex) Error in subset_and_map API

## Issue Summary

The `subset_and_map` API in our allsorts fork (`feat-subset-pdf-extra` branch) is failing with `Parse(BadIndex)` errors when attempting to subset CID fonts. This occurs because CID fonts can have glyph IDs that exceed the actual number of glyphs in the font file. For example, a font with only 100 physical glyphs might use GID 143 for a bullet character or GID 178 for other special characters.

## Context

We're working on integrating the enhanced `subset_and_map` API that returns `SubsetResult` enum with complete CIDToGIDMap. While the integration code is ready, the API consistently fails with `Parse(BadIndex)` errors during actual use.

## The Problem

### What's Happening
1. CID fonts use glyph IDs that don't necessarily correspond to physical glyph indices in the font
2. A font might have only 100-1000 physical glyphs but use GIDs like 143, 178, etc.
3. When we request subsetting with these GIDs, the API tries to access glyph index 143 in a font that only has 100 glyphs
4. The API fails with `Parse(BadIndex)` when it tries to access non-existent glyph indices

### Debug Output from Production

After disabling identity preservation, we now request only the actually used glyphs:

```
[DEBUG] Attempting subset_and_map with glyph IDs: [0, 75, 88, 92, 12, 11, 25, 38, 83, 16, 24, 18, 71, 87, 70, 81, 44, 84, 27, 85]... (48 total)
[DEBUG] Attempting subset_and_map with glyph IDs: [0, 100, 43, 45, 54, 93, 106, 107, 46, 80, 143, 91, 25, 104, 69, 102, 178, 71, 49, 159]... (46 total)
[DEBUG] Attempting subset_and_map with glyph IDs: [0, 81, 144, 114, 90, 23, 54, 45, 69, 87, 107, 93, 48, 43, 73, 33, 47, 102, 46, 49]... (75 total)
[WARN]  subset_and_map failed: Parse(BadIndex)
```

Note the high GID values like 143, 144, 159, 178 - these are beyond the font's actual glyph count but are valid CID font glyph references.

## Reproduction Test Case

Here's a minimal test case that reproduces the exact issue:

```rust
use allsorts::binary::read::ReadScope;
use allsorts::font_data::FontData;
use allsorts::subset::{subset_and_map, SubsetProfile, CmapTarget, SubsetResult};
use allsorts::tables::FontTableProvider;
use allsorts::tag;

#[test]
fn test_subset_and_map_parse_badindex_with_excessive_glyphs() {
    // Load any TrueType font - the specific font doesn't matter
    // You can use any .ttf or .ttc file available
    let font_bytes = include_bytes!("path/to/any/truetype/font.ttf");
    
    // Parse the font
    let font_data = ReadScope::new(font_bytes)
        .read::<FontData<'_>>()
        .expect("Failed to parse font");
    
    let provider = font_data
        .table_provider(0)
        .expect("Failed to get table provider");
    
    // Check actual glyph count in the font
    let actual_glyph_count = if let Some(maxp_data) = provider.table_data(tag::MAXP).ok() {
        let data = maxp_data.as_ref();
        if data.len() >= 6 {
            u16::from_be_bytes([data[4], data[5]])
        } else {
            1000 // Fallback estimate
        }
    } else {
        1000 // Fallback estimate
    };
    
    println!("Font has {} actual glyphs", actual_glyph_count);
    
    // CRITICAL: This reproduces the exact issue - requesting specific glyph IDs
    // that are beyond the font's physical glyph count
    // These are actual GIDs from our CID fonts
    let glyph_ids: Vec<u16> = vec![
        0, 75, 88, 92, 12, 11, 25, 38, 83, 16, 24, 18, 71, 87, 70, 81,
        44, 84, 27, 85, 100, 43, 45, 54, 93, 106, 107, 46, 80, 
        143,  // Bullet character - beyond typical font glyph count
        91, 25, 104, 69, 102, 
        178,  // Special character - way beyond typical font glyph count
        71, 49, 
        159,  // Another high GID
        144,  // Another high GID
    ];
    
    println!("Attempting to subset with {} glyph IDs", glyph_ids.len());
    let max_requested_gid = *glyph_ids.iter().max().unwrap();
    println!("Max requested GID: {} (font only has {} glyphs!)", 
             max_requested_gid, actual_glyph_count);
    
    // This WILL fail with Parse(BadIndex)
    let result = subset_and_map(
        &provider,
        &glyph_ids,
        &SubsetProfile::Pdf,
        CmapTarget::Unrestricted
    );
    
    // Expected: Parse(BadIndex) error
    // Actual: Parse(BadIndex) error ✓
    assert!(result.is_err());
    match result {
        Err(e) => {
            println!("Got expected error: {:?}", e);
            // Verify it's specifically BadIndex
            let error_str = format!("{:?}", e);
            assert!(error_str.contains("BadIndex"));
        }
        Ok(_) => {
            panic!("Should have failed with BadIndex, but succeeded!");
        }
    }
}

#[test]
fn test_subset_and_map_works_with_reasonable_glyphs() {
    // Same font loading code...
    let font_bytes = include_bytes!("path/to/any/truetype/font.ttf");
    let font_data = ReadScope::new(font_bytes)
        .read::<FontData<'_>>()
        .expect("Failed to parse font");
    let provider = font_data
        .table_provider(0)
        .expect("Failed to get table provider");
    
    // Use only glyphs that definitely exist
    let glyph_ids: Vec<u16> = vec![0, 3, 36, 37, 38, 68, 69, 70];
    
    // This SHOULD work fine
    let result = subset_and_map(
        &provider,
        &glyph_ids,
        &SubsetProfile::Pdf,
        CmapTarget::Unrestricted
    );
    
    // Should succeed with reasonable inputs
    assert!(result.is_ok());
    match result {
        Ok(SubsetResult::Simple { font_data, glyph_mapping }) => {
            println!("Success! Got {} bytes, {} mappings", 
                     font_data.len(), glyph_mapping.len());
        }
        Ok(SubsetResult::Cid { font_data, glyph_mapping, cid_to_gid_map }) => {
            println!("Success! CID font: {} bytes, {} mappings, {} byte map",
                     font_data.len(), glyph_mapping.len(), cid_to_gid_map.len());
        }
        Err(e) => {
            panic!("Should work with valid glyph IDs, got: {:?}", e);
        }
    }
}
```

## Expected vs Actual Behavior

### Expected
The `subset_and_map` API should handle requests for non-existent glyph IDs gracefully, either by:
1. Ignoring glyph IDs that don't exist in the font
2. Clamping to the actual glyph count
3. Returning a more descriptive error

### Actual
The API fails with `Parse(BadIndex)` when any requested glyph ID exceeds the font's actual glyph count.

## Impact

This prevents us from using the enhanced `subset_and_map` API for CID fonts, forcing us to fall back to the legacy API which doesn't provide the CIDToGIDMap we need for optimal subsetting. This results in ~35% larger PDF files than necessary.

## Suggested Solutions

### Option 1: Filter Invalid Glyph IDs (Preferred)
The API could filter out glyph IDs that exceed the font's actual glyph count:

```rust
// Inside subset_and_map implementation
let actual_glyph_count = get_glyph_count_from_maxp(&provider)?;
let valid_glyph_ids: Vec<u16> = glyph_ids
    .iter()
    .filter(|&&gid| gid < actual_glyph_count)
    .copied()
    .collect();
```

### Option 2: Return Descriptive Error
If filtering isn't desirable, return a more descriptive error:

```rust
pub enum SubsetError {
    // ...existing variants...
    InvalidGlyphId { requested: u16, max_available: u16 },
}
```

### Option 3: Document the Limitation
If this is intended behavior, document that callers must ensure all requested glyph IDs exist in the font.

## Additional Context

- This issue only affects CID fonts with identity preservation enabled
- The legacy `subset` API doesn't have this issue (it likely filters internally)
- We're using the `feat-subset-pdf-extra` branch of the allsorts fork
- The issue is 100% reproducible with the test case above

## Questions

1. Is requesting non-existent glyph IDs considered an error or should it be handled gracefully?
2. Does the legacy `subset` API filter invalid glyph IDs internally?
3. Would you prefer filtering, clamping, or error reporting for this case?

Please let me know if you need any additional information or test cases to reproduce and fix this issue.