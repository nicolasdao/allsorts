# Allsorts CID Font Detection Issue - Complete Report

## Executive Summary
The `subset_and_map` API in allsorts (tag 0.15.2) is incorrectly returning `SubsetResult::Simple` for CID fonts instead of `SubsetResult::Cid`. This prevents proper CIDToGIDMap generation, causing character rendering issues in PDFs where characters display as '?' or boxes.

## Files Provided
1. **This report**: `specs/250820_02_fix_bug/execute.md`
2. **Failing unit test**: @specs/250820_02_fix_bug/test_cid_detection_standalone.rs 
3. **Actual CID font binary**: @specs/250820_02_fix_bug/calibri_cid_font.bin (34KB)
4. **Test PDF with CID fonts**: `specs/250820_02_fix_bug/page8.pdf` (198KB)

## The Issue

### Current Behavior (INCORRECT)
```rust
let result = subset_and_map(&provider, &glyph_ids, &SubsetProfile::Pdf, CmapTarget::Unicode);
// Returns: SubsetResult::Simple { font_data, glyph_mapping }
// Missing: CIDToGIDMap
```

### Expected Behavior (CORRECT)
```rust
let result = subset_and_map(&provider, &glyph_ids, &SubsetProfile::Pdf, CmapTarget::Unicode);
// Should return: SubsetResult::Cid { font_data, glyph_mapping, cid_to_gid_map }
// Includes: CIDToGIDMap for proper glyph rendering
```

## Test Results

Running the provided test produces:
```
Testing with font data: 34240 bytes
Glyph IDs to subset: [0, 3, 43, 45, 46, 54, 69, 71, 80, 91, 93, 100, 102, 104, 106, 107, 143, 159, 178]
ERROR: CID font returned SubsetResult::Simple!
  Font data size: 18816 bytes
  Glyph mapping entries: 19
  This will cause rendering issues - missing CIDToGIDMap!

thread 'test_cid_font_returns_cid_result' panicked at:
CID font with Identity-H encoding must return SubsetResult::Cid, not SubsetResult::Simple!
```

## How to Reproduce

1. **Setup**:
```toml
# Cargo.toml
[dependencies]
allsorts = { git = "https://github.com/nicolasdao/allsorts.git", tag = "0.15.2" }
```

2. **Run the test**:
```bash
# Copy test file to tests/ directory
cargo test test_cid_font_returns_cid_result -- --nocapture
```

3. **Observe**: Test fails because API returns `SubsetResult::Simple` instead of `SubsetResult::Cid`

## Font Characteristics

The test font (@specs/250820_02_fix_bug/calibri_cid_font.bin) has these properties:
- **Format**: TrueType (has glyf table)
- **Usage in PDF**: CIDFontType2 with Identity-H encoding
- **Total glyphs**: ~225 glyphs
- **Sparse GIDs**: Uses GID 143 for bullet (•), GID 178 for other special chars
- **CID mapping**: Requires CID to GID mapping for correct rendering

## Impact

Without the CIDToGIDMap:
- ✅ Text extraction works (ToUnicode preserved)
- ❌ Visual rendering broken (characters show as '?' or boxes)
- ❌ 99.9% of glyphs map to GID 0
- ❌ PDF viewers cannot render the text correctly

## Detection Heuristics

The API should detect CID fonts when:
1. **Sparse GIDs**: Max GID (178) > actual glyph count (225) is normal, but using GID 143 for a font with fewer glyphs indicates CID usage
2. **Identity mapping expected**: Font expects CID == GID mapping
3. **PDF context**: Font is used as Type0 with Identity-H encoding

## Suggested Solutions

### Option 1: Auto-detection
```rust
fn is_cid_font(glyph_ids: &[u16], glyph_count: usize) -> bool {
    let max_gid = *glyph_ids.iter().max().unwrap_or(&0);
    // If requesting GIDs beyond normal range, likely CID
    max_gid > glyph_count || 
    glyph_ids.iter().any(|&gid| gid > 255 && glyph_count < 256)
}
```

### Option 2: Explicit parameter
```rust
pub fn subset_and_map_with_hint(
    provider: &impl FontTableProvider,
    glyph_ids: &[u16],
    profile: &SubsetProfile,
    cmap_target: CmapTarget,
    is_cid: bool, // New parameter
) -> Result<SubsetResult>
```

### Option 3: Check font tables
Look for specific table combinations that indicate CID usage in PDFs.

## Questions for Maintainers

1. Is the CID vs Simple detection based on font table analysis?
2. Should we pass a hint about the font's usage context (e.g., PDF Type0)?
3. Is there an existing way to force CID treatment that we're missing?
4. Would a PR with auto-detection logic be welcome?

## Test Data

All necessary files are provided:
- @specs/250820_02_fix_bug/calibri_cid_font.bin: The actual 34KB CID font extracted from a production PDF
- @specs/250820_02_fix_bug/test_cid_detection_standalone.rs: Complete failing test case
- The font works correctly when identity preservation is used (workaround)

## Next Steps

We need the API to:
1. Detect this font as a CID font
2. Generate the CIDToGIDMap during subsetting
3. Return `SubsetResult::Cid` with the map

This is blocking proper font subsetting in our PDF compression library, forcing us to use identity preservation which results in 35% larger files.

Please let me know if you need any additional information or test cases. I'm happy to help test fixes or provide more examples of CID fonts that exhibit this issue.