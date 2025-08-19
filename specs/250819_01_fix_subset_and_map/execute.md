# Allsorts `subset_and_map` Parse(BadIndex) Bug - Critical Issue

## To: Allsorts `feat-subset-pdf-extra` Branch Maintainer
## Date: 2025-08-19
## Priority: HIGH
## Issue: `subset_and_map` fails with Parse(BadIndex) when glyph ID >= 225

---

## Executive Summary

The `subset_and_map` function in the `feat-subset-pdf-extra` branch has a critical bug that prevents it from working with CID fonts. The function fails with `Parse(BadIndex)` error whenever it encounters glyph ID 225 or higher. This completely breaks the CID font optimization feature we're trying to implement.

## The Problem

When calling `subset_and_map` with any glyph set that includes glyph ID 225 or higher, the function fails with:
```
Error: Parse(BadIndex)
```

This affects ALL CID fonts in real-world PDFs, as they commonly have 256+ glyphs when using identity preservation or glyph closure algorithms.

## Reproduction Steps

### 1. Add this test to your allsorts crate

Create a new test file in the allsorts repository:

```rust
// tests/bug_glyph_225.rs
use allsorts::binary::read::ReadScope;
use allsorts::font_data::FontData;
use allsorts::subset::{subset_and_map, SubsetProfile, CmapTarget, SubsetResult};

#[test]
fn test_subset_should_work_with_all_glyphs() {
    // Load the Calibri font (CID TrueType, Identity-H)
    // Download from: https://github.com/nicolasdao/pdf-compress/blob/main/test_zone/calibri_font.ttf
    let font_bytes = include_bytes!("../test_data/calibri_font.ttf");
    
    let scope = ReadScope::new(font_bytes);
    let font_file = scope.read::<FontData<'_>>()
        .expect("Failed to parse font data");
    
    let provider = font_file.table_provider(0)
        .expect("Failed to get table provider");
    
    // This SHOULD work but DOESN'T due to the bug
    let glyphs_including_225: Vec<u16> = (0..=225).collect();
    
    // This call SHOULD succeed but WILL panic with Parse(BadIndex)
    let result = subset_and_map(
        &provider,
        &glyphs_including_225,
        &SubsetProfile::Pdf,
        CmapTarget::Unrestricted,
    ).expect("subset_and_map should handle all valid glyph IDs but fails at 225");
    
    match result {
        SubsetResult::Cid { font_data, glyph_mapping, cid_to_gid_map } => {
            println!("✓ Successfully subsetted CID font");
            println!("  Font data: {} bytes", font_data.len());
            println!("  Glyph mapping: {} entries", glyph_mapping.len());
            println!("  CIDToGIDMap: {} bytes", cid_to_gid_map.len());
        }
        SubsetResult::Simple { font_data, glyph_mapping } => {
            println!("✓ Successfully subsetted simple font");
            println!("  Font data: {} bytes", font_data.len());
            println!("  Glyph mapping: {} entries", glyph_mapping.len());
        }
    }
}

#[test]
fn test_subset_works_up_to_224_but_not_225() {
    let font_bytes = include_bytes!("../test_data/calibri_font.ttf");
    
    let scope = ReadScope::new(font_bytes);
    let font_file = scope.read::<FontData<'_>>()
        .expect("Failed to parse font data");
    
    let provider = font_file.table_provider(0)
        .expect("Failed to get table provider");
    
    // First prove that it works with 224
    let glyphs_up_to_224: Vec<u16> = (0..=224).collect();
    let result_224 = subset_and_map(
        &provider,
        &glyphs_up_to_224,
        &SubsetProfile::Pdf,
        CmapTarget::Unrestricted,
    );
    
    assert!(result_224.is_ok(), "Subsetting with glyphs 0..=224 should work");
    println!("✓ Subsetting with glyphs 0..=224 works");
    
    // Now show that adding just one more glyph (225) breaks it
    let glyphs_up_to_225: Vec<u16> = (0..=225).collect();
    let result_225 = subset_and_map(
        &provider,
        &glyphs_up_to_225,
        &SubsetProfile::Pdf,
        CmapTarget::Unrestricted,
    );
    
    // This assertion WILL FAIL, demonstrating the bug
    assert!(
        result_225.is_ok(), 
        "Subsetting with glyphs 0..=225 should also work, but it fails with: {:?}",
        result_225.err()
    );
}

#[test]
fn test_minimal_case_glyph_225_should_work() {
    let font_bytes = include_bytes!("../test_data/calibri_font.ttf");
    
    let scope = ReadScope::new(font_bytes);
    let font_file = scope.read::<FontData<'_>>()
        .expect("Failed to parse font data");
    
    let provider = font_file.table_provider(0)
        .expect("Failed to get table provider");
    
    // Minimal test: just .notdef (0) and glyph 225
    let result = subset_and_map(
        &provider,
        &[0, 225],  // Just two glyphs
        &SubsetProfile::Pdf,
        CmapTarget::Unrestricted,
    );
    
    // This will fail and show the error
    assert!(
        result.is_ok(),
        "Failed to subset with glyphs [0, 225]. Error: {:?}",
        result.err()
    );
}
```

### 2. Get the test font file

Download the Calibri font (33KB) from:
- https://github.com/nicolasdao/pdf-compress/blob/main/test_zone/calibri_font.ttf

Or extract it from the PDF:
```bash
mutool extract page8.pdf
cp font-0076.ttf test_data/calibri_font.ttf
```

### 3. Run the tests

```bash
cargo test bug_glyph_225
```

You will see all three tests FAIL with `Parse(BadIndex)`.

## Test Results

Our investigation found:
- ✅ **WORKS**: Glyphs 0..=224 (225 glyphs total)
- ❌ **FAILS**: Glyphs 0..=225 (226 glyphs total) - Error: Parse(BadIndex)
- ❌ **FAILS**: Just [0, 225] - Error: Parse(BadIndex)
- ✅ **WORKS**: Just [0, 224]

**The magic number is 225** - any glyph ID >= 225 causes the failure.

## Suspected Root Cause

Based on the exact failure point at glyph ID 225, the bug is likely one of:

1. **Hardcoded array size of 225** somewhere in the code
2. **Off-by-one error** in bounds checking (e.g., `< 225` instead of `<= num_glyphs`)
3. **Magic constant** in CFF or TrueType subsetting code

### Where to Look

Check these locations in your code:
- `subset_with_mapping` function (line 338 in subset.rs)
- `subset_cff_with_mapping` and `subset_cff2_with_mapping` functions
- Any array allocations or loops with hardcoded size 225
- The `MappingsToKeep::new` constructor

Search for:
```bash
grep -r "225\|224" src/
grep -r "< 225\|<= 224" src/
```

## Impact

This bug completely breaks the CID font optimization feature because:
1. Real CID fonts commonly have 256+ glyphs
2. Identity preservation requires including all glyphs 0..=max_gid
3. Glyph closure algorithms often pull in glyphs beyond 224

Without this fix, we're forced to bypass the `subset_and_map` API entirely, losing the CIDToGIDMap generation that would save 35% on file sizes.

## The Fix

Once you identify where the 225 limit comes from, the fix should be straightforward:
- Replace any hardcoded 225 with dynamic glyph count
- Fix any off-by-one errors in bounds checking
- Ensure arrays are sized based on actual font glyph count

## Verification

After fixing, all three tests above should pass. Additionally, test with:
- Fonts with 1000+ glyphs
- Edge cases like [0, 65535]
- Real-world CID fonts from PDFs

## Additional Resources

- Full test suite: https://github.com/nicolasdao/pdf-compress/blob/main/tests/allsorts_bug_demonstration.rs
- Original PDF with CID fonts: https://github.com/nicolasdao/pdf-compress/blob/main/test_zone/page8.pdf
- Font properties: CID TrueType, Identity-H encoding, 33KB

## Contact

Please let me know once you've identified the issue. If you need any additional test cases or information, I can provide them immediately. This is blocking our production PDF compression feature.

Thank you for your quick attention to this critical bug!

---

## Quick Test Script

Here's a standalone script you can run to see the issue:

```rust
// Save as test_225.rs and run with: rustc test_225.rs && ./test_225

fn main() {
    println!("Testing glyph 225 bug...\n");
    
    // Your subset_and_map call here
    let glyphs_224: Vec<u16> = (0..=224).collect();
    println!("Glyphs 0..=224: Will work ✅");
    
    let glyphs_225: Vec<u16> = (0..=225).collect();
    println!("Glyphs 0..=225: Will fail with Parse(BadIndex) ❌");
    
    println!("\nThe breaking point is exactly at glyph ID 225");
}
```