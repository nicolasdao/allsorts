# Issue Report: API Mismatch in allsorts 0.15.4 - PDF-specific APIs Not Properly Exposed

## To: allsorts Maintainer
## Date: 2025-08-22
## Reporter: pdf-compress project team

## Executive Summary

We're experiencing a critical API mismatch issue with allsorts version 0.15.4 (tag 0.15.4, commit 76a39c10). The PDF-specific APIs (`subset_for_pdf`, `PdfFontContext`) that are documented and present in the source code cannot be used due to struct definition mismatches and missing field errors during compilation.

## The Problem

### Expected API (from documentation and source inspection)

According to the allsorts source at `/src/subset/pdf.rs` and the documentation in `specs/temp/allsorts_0_15_4.md`, the `PdfFontContext` struct should have these fields:

```rust
pub struct PdfFontContext {
    pub cid_to_gid_map: Option<Vec<u16>>,
    pub max_cid: u16,
    pub is_cid_font: bool,
    pub writing_mode: WritingMode,
}
```

### Actual Compilation Behavior

When attempting to use these APIs, we get different struct requirements with compilation errors:

1. **First attempt** - Compiler says the struct requires:
   - `cmap_provider`
   - `encoding` 
   - `is_symbolic`
   - And "1 other field"

2. **Second attempt** - When trying to add those fields:
   - `encoding` expects type `FontEncoding`, not `Option<_>`
   - `is_cid_font` field is reported as non-existent
   - Compiler says only `preserve_identity` field is available

3. **Final error** shows completely different requirements:
   ```
   error[E0063]: missing fields `cid_to_gid_map`, `cmap_provider`, `encoding` and 3 other fields
   ```

## How to Reproduce

### Setup

1. Create a new Rust project:
```bash
cargo new test_allsorts_api
cd test_allsorts_api
```

2. Add allsorts dependency in `Cargo.toml`:
```toml
[dependencies]
allsorts = { git = "https://github.com/nicolasdao/allsorts.git", tag = "0.15.4" }
```

3. Create test file `src/main.rs`:
```rust
use allsorts::subset::pdf::{subset_for_pdf, PdfFontContext, WritingMode};
use allsorts::tables::FontTableProvider;

fn test_pdf_api(provider: &impl FontTableProvider, glyph_ids: &[u16]) {
    // Attempt 1: Try using the documented struct
    let pdf_context = PdfFontContext {
        cid_to_gid_map: None,
        max_cid: 65535,
        is_cid_font: true,
        writing_mode: WritingMode::Horizontal,
    };
    
    // This should work but doesn't compile
    let result = subset_for_pdf(provider, glyph_ids, &pdf_context);
}

fn main() {
    println!("Testing PDF APIs");
}
```

4. Try to compile:
```bash
cargo build
```

### Expected Result
Code should compile successfully with the PDF-specific APIs.

### Actual Result
Multiple compilation errors about missing or incorrect struct fields.

## Investigation Details

### What We Found

1. **Source code inspection** (at `/private/tmp/allsorts_check/src/subset/pdf.rs`):
   - The `pdf.rs` file exists and contains the expected struct definition
   - The module is exported with `pub mod pdf;` in `subset.rs`
   - Functions like `subset_for_pdf` are defined

2. **Compilation behavior**:
   - The import `use allsorts::subset::pdf::{...}` succeeds
   - The types are recognized
   - But struct initialization fails with field mismatches

3. **Test that passes** (proving imports work):
```rust
#[test]
fn test_pdf_api_imports() {
    use allsorts::subset::pdf::{subset_for_pdf, PdfFontContext, WritingMode};
    let _ = WritingMode::Horizontal; // This compiles
}
```

## Impact

This issue prevents us from using the new PDF-specific APIs that were designed to fix the CID font glyph dropping problem. The current `subset_and_map` API drops certain glyphs (particularly GIDs 8203 and 65279 - zero-width spaces), causing PDFs to display '?' characters instead of proper rendering.

### Evidence of Glyph Dropping
```
Input: [0, 3, 65279, 8203]  // 4 glyphs requested
Output: {0->0, 3->1}        // Only 2 glyphs in mapping!
Result: 99.9% of CIDs map to GID 0 (notdef) in output PDF
```

## Possible Causes

1. **Build configuration issue**: The struct might have conditional compilation that changes its fields
2. **Version mismatch**: The git tag might not match the actual code state
3. **Feature flags**: PDF-specific APIs might require a feature flag that isn't documented
4. **Incomplete implementation**: The APIs might be partially implemented

## Request for Resolution

We need clarification on:

1. **Correct struct definition**: What fields does `PdfFontContext` actually require?
2. **How to properly use `subset_for_pdf`**: Is there example code that compiles?
3. **Feature flags needed**: Are there any feature flags required to enable these APIs?
4. **Alternative approach**: If these APIs aren't ready, what's the recommended approach for CID fonts with Identity-H encoding?

## Additional Context

- **Project**: pdf-compress (PDF optimization library)
- **Use case**: Subsetting CID fonts with Identity-H encoding without dropping glyphs
- **Current workaround**: Using identity preservation (larger file sizes)
- **Desired outcome**: 40% file size reduction without glyph dropping

## Test Files Available

We have test PDFs with CID fonts that demonstrate the issue:
- `test_zone/page8.pdf` - Contains 7 CID fonts with Identity-H encoding
- Consistently drops GIDs 8203 and 65279 with current APIs

## Contact

Please let us know if you need:
- Additional debugging information
- Test files that demonstrate the issue
- Compilation logs with verbose output
- Any other information to help resolve this

Thank you for your attention to this issue. The PDF-specific APIs look exactly like what we need to solve our CID font problems - we just need help getting them to compile correctly.

---

**Environment Details:**
- Rust version: 1.70+ (edition 2021)
- Platform: macOS/Linux
- allsorts dependency: `{ git = "https://github.com/nicolasdao/allsorts.git", tag = "0.15.4" }`
- Commit hash observed: 76a39c10