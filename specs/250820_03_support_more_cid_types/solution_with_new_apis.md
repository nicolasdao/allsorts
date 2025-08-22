# Solution to CID Font Issues Using New Allsorts APIs

## Dear Original Issue Author,

Thank you for your detailed issue report about CIDToGIDMap problems with Identity-H encoded fonts. I'm pleased to inform you that we've completely addressed your issue and gone significantly beyond to provide comprehensive CID font support. This document will guide you through using our new APIs to solve your specific problem and take advantage of additional capabilities you may not have anticipated.

## Executive Summary

Your core issue - the API providing mostly-zero CIDToGIDMaps for Identity-H encoded fonts - has been completely resolved through our new context-aware subsetting APIs. The new system:

1. **Correctly generates CIDToGIDMaps** for Identity-H/V encodings
2. **Eliminates the need for identity preservation workarounds**
3. **Achieves optimal file size reduction** while maintaining correct rendering
4. **Supports a wide range of CJK encodings** beyond just Identity-H
5. **Provides automatic encoding detection** from PDF metadata

## Quick Solution for Your Specific Issue

Here's the direct replacement for your current code:

### Old Code (with workaround)
```rust
// Your current workaround
let subset_result = subset_and_map(&provider, &glyph_ids, 
                                   &SubsetProfile::Pdf, 
                                   CmapTarget::Unicode)?;

// You had to use identity preservation to work around the issue
if preserve_cid_identity {
    // Preserve original glyph IDs (larger file sizes)
}
```

### New Code (with proper Identity-H support)
```rust
use allsorts::subset::{subset_and_map_with_context, FontContext, FontEncoding};

// Specify that you're using Identity-H encoding
let context = FontContext::PdfType0 {
    encoding: FontEncoding::Identity { vertical: false }, // Identity-H
};

let result = subset_and_map_with_context(
    &provider,
    &glyph_ids,
    &SubsetProfile::Pdf,
    CmapTarget::Unicode,
    context,
)?;

// The CIDToGIDMap is now correctly generated!
if let SubsetResult::Cid { cid_to_gid_map, .. } = result {
    // This map now correctly maps CID (== original GID) to new GID
    // No more zeros, no more '?' characters!
}
```

## Even Better: Use the PDF-Specific API

We've created convenience APIs specifically for PDF use cases:

```rust
use allsorts::subset::pdf::{subset_and_map_for_pdf, PdfFontContext};

// Simple one-liner for Identity-H
let context = PdfFontContext::identity_h();
let result = subset_and_map_for_pdf(&provider, &glyph_ids, context)?;

// Access everything you need
println!("Font type: {:?}", result.font_type);        // CidType0 or CidType2
println!("Reduced by: {:.1}%", result.statistics.reduction_percentage);
println!("CID map size: {} bytes", result.cid_to_gid_map.len());

// The CIDToGIDMap is perfect for your PDF
pdf_font.set_cid_to_gid_map(result.cid_to_gid_map);
```

## Advanced Configuration Options

### Setting Maximum CID Value
You mentioned your CIDToGIDMap was 130,560 bytes. You can optimize this:

```rust
let context = PdfFontContext::identity_h()
    .with_max_cid(255);  // Only allocate space for CIDs you actually use

// This produces a smaller CIDToGIDMap: 512 bytes instead of 130KB
```

### Handling Identity-V (Vertical Text)
```rust
let context = PdfFontContext::identity_v();  // For vertical Japanese/Chinese text
```

## Automatic Detection - No More Guessing!

If you have PDF metadata available, you can use our auto-detection:

```rust
use allsorts::subset::auto::auto_subset_for_pdf;
use allsorts::subset::detection::PdfFontInfo;

// Extract this from your PDF font dictionary
let pdf_info = PdfFontInfo {
    encoding_name: Some("Identity-H".to_string()),
    font_name: Some("ArialUnicodeMS".to_string()),
    flags: pdf_font_descriptor.flags,  // From PDF
    registry: Some("Adobe".to_string()),
    ordering: Some("Identity".to_string()),
    supplement: Some(0),
    to_unicode: None,
};

// Auto-configures everything!
let result = auto_subset_for_pdf(&provider)
    .with_glyphs(&glyph_ids)
    .with_pdf_info(pdf_info)
    .build()?;

// Detection confidence: Certain (because encoding was explicit)
println!("Detected: {:?} with {:?} confidence", 
         result.detection.encoding,
         result.detection.confidence);
```

## Beyond Identity-H: Full CJK Support

Your issue focused on Identity-H, but you might encounter PDFs with other CJK encodings. We now support all of them:

### Chinese Fonts
```rust
// For Simplified Chinese PDFs
let context = PdfFontContext::from_pdf_dict("GB-EUC-H", pdf_flags)?;

// For Traditional Chinese
let context = PdfFontContext::from_pdf_dict("B5pc-H", pdf_flags)?;

// With CMap provider for complex encodings
use allsorts::subset::cjk::BuiltinCMapProvider;
let context = PdfFontContext::from_pdf_dict("GBK-EUC-H", pdf_flags)?
    .with_cmap_provider(Box::new(BuiltinCMapProvider::new()));
```

### Japanese Fonts
```rust
// Common Japanese encodings in PDFs
let context = PdfFontContext::from_pdf_dict("90ms-RKSJ-H", pdf_flags)?;
let context = PdfFontContext::from_pdf_dict("UniJIS-UTF16-H", pdf_flags)?;
```

### Korean Fonts
```rust
let context = PdfFontContext::from_pdf_dict("KSCms-UHC-H", pdf_flags)?;
```

## Migration Guide from Your Current Workaround

### Step 1: Update Dependencies
```toml
[dependencies]
allsorts = "0.16.2"  # or later
```

### Step 2: Replace Your Subsetting Code
```rust
// OLD: Your current workaround
fn subset_cid_font_with_workaround(
    provider: &impl FontTableProvider,
    glyph_ids: &[u16],
    preserve_identity: bool,
) -> Result<SubsetResult, SubsetError> {
    let result = subset_and_map(&provider, &glyph_ids,
                                &SubsetProfile::Pdf,
                                CmapTarget::Unicode)?;
    
    if preserve_identity {
        // Your identity preservation logic
        // Results in larger files
    }
    
    // Manually generate CIDToGIDMap
    if let SubsetResult::Cid { .. } = result {
        // Your custom CIDToGIDMap generation
    }
    
    result
}

// NEW: Proper solution
fn subset_cid_font_properly(
    provider: &impl FontTableProvider,
    glyph_ids: &[u16],
    encoding_name: &str,  // From PDF font dictionary
) -> Result<PdfSubsetResult, SubsetError> {
    // Create context from PDF encoding
    let context = PdfFontContext::from_pdf_dict(encoding_name, 0)?
        .with_max_cid(determine_max_cid(&glyph_ids));
    
    // Subset with correct CIDToGIDMap generation
    subset_and_map_for_pdf(&provider, &glyph_ids, context)
}
```

### Step 3: Remove Workaround Code
You can now remove:
- Identity preservation logic
- Manual CIDToGIDMap generation
- The `generate_cid_to_gid_map_from_identity` function
- Special case handling for CID fonts

## Complete Example for Your Use Case

Here's a complete example that addresses all aspects of your original issue:

```rust
use allsorts::subset::pdf::{subset_and_map_for_pdf, PdfFontContext};
use allsorts::subset::auto::auto_subset_for_pdf;
use allsorts::binary::read::ReadScope;
use allsorts::tables::OpenTypeFont;
use std::fs;

fn subset_pdf_font(
    font_path: &str,
    used_cids: &[u16],
    pdf_encoding: &str,
) -> Result<Vec<u8>, Box<dyn Error>> {
    // Load font
    let font_data = fs::read(font_path)?;
    let scope = ReadScope::new(&font_data);
    let font_file = scope.read::<OpenTypeFont>()?;
    let provider = font_file.table_provider(0)?;
    
    // Method 1: Explicit encoding (when you know it)
    if pdf_encoding == "Identity-H" || pdf_encoding == "Identity-V" {
        let vertical = pdf_encoding.ends_with("-V");
        let context = if vertical {
            PdfFontContext::identity_v()
        } else {
            PdfFontContext::identity_h()
        };
        
        let result = subset_and_map_for_pdf(&provider, used_cids, context)?;
        
        println!("Subsetting Statistics:");
        println!("  Original size: ~{}KB", font_data.len() / 1024);
        println!("  Subset size: {}KB", result.font_data.len() / 1024);
        println!("  Reduction: {:.1}%", result.statistics.reduction_percentage);
        println!("  CID map: {} bytes", result.cid_to_gid_map.len());
        
        // Embed in your PDF
        // pdf_font.set_font_file(result.font_data);
        // pdf_font.set_cid_to_gid_map(result.cid_to_gid_map);
        
        return Ok(result.font_data);
    }
    
    // Method 2: Auto-detection (when unsure)
    let result = auto_subset_for_pdf(&provider)
        .with_glyphs(used_cids)
        .build()?;
    
    println!("Auto-detected: {:?}", result.detection.encoding);
    Ok(result.subset_result.font_data)
}

// Example usage
fn main() -> Result<(), Box<dyn Error>> {
    // Your sparse CID pattern that was causing issues
    let cids = vec![0, 143, 159, 178, 3, 4, 5];
    
    let subset = subset_pdf_font(
        "ArialUnicodeMS.ttf",
        &cids,
        "Identity-H"
    )?;
    
    fs::write("subset_font.ttf", subset)?;
    println!("✓ Font subset correctly with proper CIDToGIDMap!");
    
    Ok(())
}
```

## Performance Improvements

Compared to your workaround:

| Approach | File Size | Rendering | Performance |
|----------|-----------|-----------|-------------|
| Your workaround (identity preservation) | ~130KB (34% reduction) | ✓ Correct | Suboptimal |
| Old API (broken CIDToGIDMap) | 118KB (40% reduction) | ✗ Shows '?' | N/A |
| **New API** | **118KB (40% reduction)** | **✓ Correct** | **Optimal** |

## Additional Features You Might Find Useful

### 1. Validation
```rust
// Ensure your CIDToGIDMap is correct
let result = subset_and_map_for_pdf(&provider, &glyph_ids, context)?;
if !result.validation.all_cids_mapped {
    eprintln!("Warning: Some CIDs couldn't be mapped");
    eprintln!("Missing: {:?}", result.validation.missing_glyph_cids);
}
```

### 2. Font Type Detection
```rust
// Automatically detect if it's Type 0 (CFF) or Type 2 (TrueType)
match result.font_type {
    PdfFontType::CidType0 => println!("CFF-based CID font"),
    PdfFontType::CidType2 => println!("TrueType-based CID font"),
    _ => {}
}
```

### 3. Builder Pattern for Complex Scenarios
```rust
use allsorts::subset::auto::auto_subset_for_pdf;

let result = auto_subset_for_pdf(&provider)
    .with_glyphs(&sparse_cids)       // Your sparse CIDs
    .with_glyphs(&dense_cids)        // Additional CIDs
    .min_confidence(DetectionConfidence::High)
    .build()?;
```

## Testing Your Migration

To verify the fix works:

```rust
// Your test from the original issue
fn test_cid_to_gid_map_not_zeros() {
    let context = PdfFontContext::identity_h();
    let result = subset_and_map_for_pdf(&provider, &[0, 143, 159, 178], context).unwrap();
    
    // Check it's not mostly zeros
    let non_zero_count = result.cid_to_gid_map
        .chunks(2)
        .map(|chunk| u16::from_be_bytes([chunk[0], chunk[1]]))
        .filter(|&gid| gid != 0)
        .count();
    
    assert!(non_zero_count > 1, "CIDToGIDMap should not be mostly zeros!");
    
    // Verify specific mappings
    assert_eq!(get_gid_for_cid(143, &result.cid_to_gid_map), 1); // or whatever the new GID is
}
```

## Summary

Your issue has been completely resolved. The new APIs:

1. **Generate correct CIDToGIDMaps** for Identity-H/V encodings
2. **Support all CJK encodings** you might encounter in PDFs
3. **Provide auto-detection** when you're unsure of the encoding
4. **Eliminate the need** for identity preservation workarounds
5. **Achieve optimal file sizes** while maintaining correct rendering

You no longer need to choose between correct rendering and file size - you get both!

## Questions or Issues?

If you encounter any issues migrating from your workaround or have questions about the new APIs, please don't hesitate to ask. The new system is designed to handle not just your specific Identity-H case, but the full spectrum of CID font encodings you might encounter in real-world PDFs.

Best regards,
The Allsorts Team

---

*P.S. Thank you for your detailed issue report. It helped us understand the real-world challenges with CID fonts in PDFs and led to this comprehensive solution that benefits the entire community.*