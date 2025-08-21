# CID Font API Integration - Final Status Report

## Date: 2025-08-20

## Summary
The allsorts API has been updated (tag 0.15.3) to correctly detect CID fonts and return `SubsetResult::Cid`. However, the CIDToGIDMap provided by the API is mostly zeros, which doesn't correctly map CIDs to the new glyph IDs after subsetting.

## Current Status

### ✅ Successes
1. **API Detection Fixed**: The API now correctly returns `SubsetResult::Cid` for CID fonts
2. **No Parse(BadIndex) Errors**: The sparse glyph ID issue has been resolved
3. **File Size Reduction**: Achieving 40% reduction (198KB → 118KB)
4. **Text Extraction Works**: PDFs remain searchable and text can be extracted

### ❌ Remaining Issues
1. **Incorrect CIDToGIDMap**: The API provides a CIDToGIDMap that's mostly zeros
   - Size: 130,560 bytes (should be 131,072 for full range)
   - Content: Mostly zeros, causing glyphs to map to GID 0
   - Result: Characters display as '?' in PDF viewers

## Technical Analysis

### The API Returns
```rust
SubsetResult::Cid {
    font_data: Vec<u8>,          // ✅ Correct subsetted font
    glyph_mapping: HashMap<u16, u16>, // ✅ Correct old->new GID mapping
    cid_to_gid_map: Vec<u8>,     // ❌ Mostly zeros, incorrect
}
```

### What We Need
For Identity-H encoded CID fonts in PDFs:
- CID == original GID (by definition of Identity-H)
- We need: CID → original GID → new GID
- The API's CIDToGIDMap should encode this transformation

### The Problem
The API doesn't understand the PDF context:
1. It doesn't know these fonts use Identity-H encoding
2. It doesn't know that CID == original GID
3. It provides a generic CIDToGIDMap that doesn't match our needs

## Current Workaround

### Option 1: Identity Preservation (Currently Working)
```rust
// Preserve original glyph IDs
preserve_cid_identity: true
```
- ✅ Correct rendering
- ❌ Larger file sizes (no optimal subsetting)

### Option 2: Generate CIDToGIDMap from Glyph Mapping
```rust
// Use glyph_mapping to build CIDToGIDMap
// Assumes CID == original GID (Identity-H)
generate_cid_to_gid_map_from_identity(&glyph_remapping, max_cid)
```
- ✅ Should work in theory
- ❌ Currently not working due to mapping issues

## Code Locations

### Key Files
1. **src/font_subsetting/subsetting.rs**
   - Lines 980-1160: API integration and CIDToGIDMap handling
   - Line 1132: Decision point for using API map vs generating

2. **src/font_subsetting/allsorts_integration.rs**
   - Lines 40-75: Handling SubsetResult enum
   - Correctly extracts CID variant

3. **src/cmap.rs**
   - Lines 314-360: CIDToGIDMap generation functions

## Recommendations

### Short Term (For Production)
Use identity preservation for CID fonts:
- Ensures correct rendering
- Acceptable file sizes
- No risk of character issues

### Medium Term
The allsorts API needs enhancement:
1. Accept encoding hint (Identity-H)
2. Generate correct CIDToGIDMap for PDF context
3. Or provide raw mapping data for us to generate the map

### Long Term
Consider alternative approaches:
1. Direct CFF/TrueType table manipulation
2. Custom CID font subsetting implementation
3. Integration with other font libraries

## Test Results

### File Sizes
- Original: 198KB
- With API (broken rendering): 118KB (40% reduction)
- With identity preservation: ~130KB (34% reduction)

### Rendering Tests
```bash
# With API CIDToGIDMap
./pdf_glyph_zero_detector output.pdf
# Result: 99.9% glyphs missing (map to GID 0)

# With identity preservation
./pdf_glyph_zero_detector output_identity.pdf
# Result: All glyphs render correctly
```

## Conclusion

While the allsorts API now correctly identifies CID fonts (fixing the Parse(BadIndex) issue), it doesn't provide a usable CIDToGIDMap for PDF contexts. The identity preservation workaround remains the most reliable solution for production use.

The core issue is a mismatch between what the font subsetting library provides (generic font subsetting) and what PDF needs (context-aware CID mapping). This requires either:
1. Enhancements to the allsorts API
2. Custom CIDToGIDMap generation
3. Continued use of identity preservation

## Files Modified
- Cargo.toml: Updated to allsorts tag 0.15.3
- src/font_subsetting/subsetting.rs: Added logic to handle API CIDToGIDMap

## Dependencies
- allsorts = { git = "https://github.com/nicolasdao/allsorts.git", tag = "0.15.3" }