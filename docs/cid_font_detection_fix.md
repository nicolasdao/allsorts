# CID Font Detection Fix for TrueType Fonts

## Overview
Fixed a critical bug in `subset_and_map` where TrueType fonts used as CID fonts in PDFs were incorrectly returning `SubsetResult::Simple` instead of `SubsetResult::Cid`, resulting in missing CIDToGIDMap and rendering issues.

## Problem
- TrueType fonts can be used as CIDFontType2 in PDFs with Identity-H encoding
- The original `detect_cid_font` function only checked for CFF/PostScript CID fonts
- This caused TrueType CID fonts to be incorrectly processed without CIDToGIDMap
- Result: Characters rendered as '?' or boxes in PDF viewers

## Solution

### 1. Enhanced CID Detection
Added `detect_cid_from_glyph_pattern` function that detects CID fonts based on:
- Sparse glyph ID patterns (e.g., GID 143 for bullet, GID 178 for special chars)
- Ratio of maximum GID to total glyph count
- Common CID font usage patterns in PDFs

### 2. New API Function
Added `subset_and_map_with_hint` function that allows explicit CID specification:
```rust
pub fn subset_and_map_with_hint(
    provider: &impl FontTableProvider,
    glyph_ids: &[u16],
    profile: &SubsetProfile,
    cmap_target: CmapTarget,
    force_cid: bool,  // New parameter to force CID treatment
) -> Result<SubsetResult, SubsetError>
```

## Impact
- Fixes rendering issues for TrueType fonts used as CID fonts in PDFs
- Enables proper CIDToGIDMap generation for CIDFontType2
- Maintains backward compatibility with existing API
- Provides explicit control when external context indicates CID usage

## Testing
Added comprehensive tests in `tests/cid_detection.rs`:
- `test_truetype_cid_font_detection`: Verifies TrueType CID fonts are correctly detected
- `test_sparse_glyph_ids_indicate_cid_font`: Tests sparse GID pattern detection
- `test_subset_and_map_with_hint`: Validates the new API with force_cid parameter

## Files Modified
- `src/subset.rs`: Enhanced `detect_cid_font`, added `detect_cid_from_glyph_pattern` and `subset_and_map_with_hint`
- `tests/cid_detection.rs`: New integration tests for CID font detection

## Backward Compatibility
- Existing `subset_and_map` API unchanged
- New heuristics are conservative to avoid false positives
- New `subset_and_map_with_hint` API is additive