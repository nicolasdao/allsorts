# Allsorts Issue: Glyph Dropping in CID Font Subsetting with Identity-H Encoding

## Issue Summary

When subsetting TrueType CID fonts with Identity-H encoding using `subset_for_pdf` API, certain glyphs (particularly common punctuation like hyphen GID 45, period GID 46, and bullet GID 143) are being dropped from the subset even when explicitly requested. This results in PDFs where 68-93% of characters render as '?' despite text extraction working correctly.

## Environment

- **Allsorts version**: 0.16.2 (git = "https://github.com/nicolasdao/allsorts.git", tag = "0.16.2")
- **Font types affected**: TrueType CID fonts with Identity-H encoding
- **Test fonts**: Calibri, Arial, Times New Roman from Windows
- **PDF context**: Subsetting fonts from existing PDFs with Identity CIDToGIDMap

## The Problem

### What's Happening
1. We request specific GIDs to be subset, including GIDs like 45 (hyphen), 46 (period), 143 (bullet)
2. The `subset_for_pdf` API returns successfully but the returned `glyph_mapping` is missing these GIDs
3. The resulting CIDToGIDMap has these CIDs mapping to 0 (notdef), causing '?' to appear in PDF viewers

### Example
```rust
// Request includes GID 45 and 46
let requested_gids = vec![0, 49, 69, 81, 3, 91, 45, 155, 13, 79, 17, 106, 54, 68, 102, 25, 80, 160, 46, 165...];
info!("Requesting {} glyphs including GID 45 and 46", requested_gids.len());

let result = subset_for_pdf(provider, &requested_gids, &pdf_context)?;

// But the returned mapping doesn't include them!
assert!(result.glyph_mapping.contains_key(&45)); // FAILS
assert!(result.glyph_mapping.contains_key(&46)); // FAILS
```

## How to Reproduce

### Step 1: Create Test Case
```rust
use allsorts::subset::pdf::{subset_for_pdf, PdfFontContext, WritingMode};
use allsorts::subset::FontEncoding;
use allsorts::font_data::FontData;
use allsorts::binary::read::ReadScope;
use std::fs;

fn test_cid_glyph_dropping() {
    // Load a TrueType font (e.g., Calibri)
    let font_data = fs::read("calibri.ttf").unwrap();
    let scope = ReadScope::new(&font_data);
    let font_file = scope.read::<FontData>().unwrap();
    let provider = font_file.table_provider(0).unwrap();
    
    // Request common glyphs including punctuation
    let requested_gids = vec![
        0,   // .notdef
        3,   // space
        45,  // hyphen-minus (CRITICAL: gets dropped)
        46,  // period (CRITICAL: gets dropped)
        65,  // A
        97,  // a
        143, // bullet (CRITICAL: gets dropped)
    ];
    
    // Configure for Identity-H CID font
    let pdf_context = PdfFontContext {
        encoding: FontEncoding::Identity { vertical: false },
        max_cid: Some(255),
        preserve_identity: false,  // Using optimal subsetting
        is_symbolic: false,
        cid_to_gid_map: None,
        writing_mode: WritingMode::Horizontal,
        cmap_provider: None,
    };
    
    // Perform subsetting
    let result = subset_for_pdf(&provider, &requested_gids, &pdf_context).unwrap();
    
    // Check which glyphs were actually included
    println!("Requested {} glyphs", requested_gids.len());
    println!("Got {} in mapping", result.glyph_mapping.len());
    
    for gid in &requested_gids {
        if !result.glyph_mapping.contains_key(gid) {
            println!("WARNING: GID {} was requested but not in result mapping!", gid);
        }
    }
    
    // Check CIDToGIDMap for zeros
    let mut zero_count = 0;
    for i in (0..result.cid_to_gid_map.len()).step_by(2) {
        let gid = u16::from_be_bytes([
            result.cid_to_gid_map[i],
            result.cid_to_gid_map[i + 1]
        ]);
        if gid == 0 && i > 0 {  // Skip CID 0 which should map to GID 0
            zero_count += 1;
        }
    }
    println!("CIDToGIDMap has {} non-zero entries out of {}", 
             result.cid_to_gid_map.len() / 2 - zero_count,
             result.cid_to_gid_map.len() / 2);
}
```

### Step 2: Expected vs Actual Results

**Expected:**
- All requested GIDs should be in the returned `glyph_mapping`
- CIDToGIDMap should have proper mappings for all requested glyphs

**Actual:**
- GIDs 45, 46, 143 (and others) are missing from `glyph_mapping`
- CIDToGIDMap has these CIDs mapping to 0 (notdef)
- PDF viewers show '?' for these characters

## Analysis

### Identity-H Context
For TrueType fonts with Identity-H encoding in PDFs:
- The PDF uses "Identity" CIDToGIDMap, meaning CID values ARE the GID values
- Character codes map to CIDs through the encoding (Identity-H means char code = CID)
- The subsetting should preserve these specific GID mappings

### Suspected Issue
It appears the `subset_for_pdf` API might be:
1. Validating GIDs against some internal criteria and dropping "invalid" ones
2. Or having issues with certain glyph types (punctuation, symbols)
3. Or incorrectly handling the Identity mapping case

## Workaround

Setting `preserve_identity: true` in `PdfFontContext` works but results in much larger files as it includes all glyphs up to the maximum GID.

## Request for Fix

Could you please investigate why `subset_for_pdf` is dropping these specific glyphs even when they're explicitly requested? The glyphs exist in the font (verified by checking the font's glyph count), they're being requested in the input vector, but they're not appearing in the output mapping.

This is critical for PDF compression tools as these missing glyphs are common punctuation marks that make compressed PDFs unreadable.

## Additional Debug Info

When running with logging, we see:
```
Font Calibri-Bold has 203 total glyphs (valid GID range: 0-202)
Requesting GIDs: [0, 49, 69, 81, 3, 91, 45, 155, 13, 79, 17, 106, 54, 68, 102, 25, 80, 160, 46, 165...]
subset_for_pdf returned 15 mapped glyphs (but we requested 44!)
WARNING: GID 45 was dropped
WARNING: GID 46 was dropped  
WARNING: GID 143 was dropped
```

The issue affects multiple fonts (Calibri, Arial, Times New Roman) so it's not font-specific.

Thank you for looking into this!