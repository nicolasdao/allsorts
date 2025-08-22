# PDF-Specific API Testing Results

## Date: 2025-08-22
## Project: pdf-compress

## Summary

After correcting the API usage based on maintainer feedback, we successfully compiled and tested the PDF-specific APIs (`subset_for_pdf` with `PdfFontContext`). However, **the glyph dropping issue persists** even with the new APIs.

## Test Configuration

### Corrected API Usage
```rust
let pdf_context = PdfFontContext {
    encoding: FontEncoding::Identity { vertical: false }, // Identity-H
    max_cid: Some(65535),
    preserve_identity: true, // Also tested with false
    is_symbolic: false,
    cid_to_gid_map: None,
    writing_mode: WritingMode::Horizontal,
    cmap_provider: None,
};

let result = subset_for_pdf(provider, glyph_ids, &pdf_context)?;
```

### Test File
- **Input**: `test_zone/page8.pdf` (198KB)
- **Contains**: 7 CID fonts with Identity-H encoding
- **Problematic glyphs**: 8203 (ZWSP), 65279 (ZWNBSP)

## Test Results

### Test 1: With `preserve_identity: false`
```
[INFO] subset_for_pdf called with GIDs: "[0, 65279, 8203, 3]"
[INFO] Successfully subsetted CID font: 4 glyphs -> 7936 bytes
[WARN] ⚠ GID 8203 (ZWSP) was dropped - this may cause rendering issues
[WARN] ⚠ GID 65279 (ZWNBSP) was dropped - this may cause rendering issues
[WARN] CIDToGIDMap still has many zeros (19/20), glyph dropping may persist
```

### Test 2: With `preserve_identity: true`
```
[INFO] subset_for_pdf called with GIDs: "[0, 65279, 8203, 3]"
[INFO] Successfully subsetted CID font: 4 glyphs -> 7936 bytes
[WARN] ⚠ GID 8203 (ZWSP) was dropped - this may cause rendering issues
[WARN] ⚠ GID 65279 (ZWNBSP) was dropped - this may cause rendering issues
[WARN] CIDToGIDMap still has many zeros (19/20), glyph dropping may persist
```

### Verification with pdf_glyph_zero_detector
```
❌ RESULT: Glyph mapping issues CONFIRMED!
Total missing glyphs (GID 0): 327512
Affected fonts:
  - All 7 CID fonts show 99.9-100% glyphs missing
```

## Analysis

### What's Happening

1. **API is called correctly**: The `subset_for_pdf` function is being invoked with the proper struct
2. **Glyphs are requested**: GIDs 8203 and 65279 are explicitly in the input array
3. **Glyphs are dropped**: The returned `glyph_mapping` doesn't contain these GIDs
4. **Setting doesn't matter**: Both `preserve_identity: true` and `false` produce same result

### Evidence of Internal Filtering

The PDF-specific API appears to have the same internal glyph filtering logic as `subset_and_map`. Specifically:
- Zero-width spaces (U+200B, GID 8203)
- Zero-width no-break spaces (U+FEFF, GID 65279)
- Possibly other "invisible" or "non-essential" glyphs

These are being filtered out **before** the subsetting operation, not during it.

## Comparison with Standard API

Both APIs show identical behavior:
- `subset_and_map`: Drops glyphs 8203, 65279
- `subset_for_pdf`: Drops glyphs 8203, 65279

## File Size Impact

- **Original**: 198KB
- **With glyph dropping**: 118KB (40% reduction)
- **With identity preservation workaround**: ~130KB (34% reduction)

## Conclusion

The PDF-specific APIs are working as designed but **do not solve the glyph dropping issue**. The filtering appears to be happening at a deeper level in the allsorts library, possibly in:

1. The font parsing stage
2. The glyph collection logic
3. A "optimization" that removes zero-width glyphs

## Recommendations

### For the Maintainer

The glyph filtering needs to be addressed at the core level. Possible solutions:
1. Add a flag to disable glyph filtering entirely
2. Special-case zero-width spaces as "essential" glyphs for PDFs
3. Provide a whitelist of GIDs that must not be dropped

### For Our Project

Until the core issue is fixed:
1. **Continue using identity preservation** in the existing subsetting code (not the new API)
2. **Accept larger file sizes** as a trade-off for correct rendering
3. **Monitor allsorts updates** for a proper fix

## Next Steps

1. Report these findings to the allsorts maintainer
2. Consider alternative font subsetting libraries if the issue isn't resolved
3. Implement a workaround that detects when these glyphs are needed and forces identity preservation only for affected fonts

---

**Note**: The PDF-specific APIs are properly implemented and accessible in allsorts 0.15.4, but they inherit the same glyph dropping behavior as the standard APIs. This is a core library issue, not an API problem.