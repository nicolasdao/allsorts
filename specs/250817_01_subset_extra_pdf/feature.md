# Technical Requirement: PDF-Specific Font Subsetting Features for Allsorts

## Executive Summary

This document specifies additional PDF-specific features for Allsorts that would significantly simplify font subsetting in PDF workflows. These features build upon the core `subset_and_map` function to provide comprehensive support for PDF font structures, reducing implementation complexity and potential bugs in PDF processing applications.

## Problem Context

### PDF Font Subsetting Challenges

Even with the glyph ID mapping from `subset_and_map`, PDF processors must still:

1. **Update Binary Font Data**: Composite glyphs contain hardcoded GID references
2. **Rebuild CIDToGIDMap**: Complex binary structure requiring careful byte ordering
3. **Validate Coverage**: Ensure all CIDs map to valid GIDs
4. **Handle Edge Cases**: Empty glyphs, missing mappings, circular dependencies

### Current Pain Points

```rust
// Even with mapping, we need complex code like:
let (font_data, mapping) = subset_and_map(&provider, &glyph_ids)?;

// Problem 1: Manually patch binary composite references
for composite_glyph in find_composites(&font_data) {
    for component_ref in composite_glyph.components {
        // Complex binary patching with endianness concerns
        patch_gid_reference(&mut font_data, component_ref, &mapping)?;
    }
}

// Problem 2: Rebuild CIDToGIDMap from scratch
let mut new_cid_map = Vec::new();
for cid in 0..=max_cid {
    let old_gid = old_cid_map[cid];
    let new_gid = mapping.get(&old_gid).unwrap_or(0);  // Risk!
    new_cid_map.write_u16_be(new_gid);
}

// Problem 3: No validation that this is correct!
```

## Feature 1: Composite Glyph Reference Updater

### Purpose

Automatically update glyph ID references within composite glyphs in the font binary data, eliminating the need for manual binary patching.

### Function Signature

```rust
/// Update all composite glyph component references in a font
///
/// This function modifies the font data in-place, updating all component
/// glyph ID references in composite glyphs according to the provided mapping.
///
/// # Arguments
/// * `font_data` - Mutable reference to the font file bytes
/// * `mapping` - Glyph ID remapping (old GID -> new GID)
///
/// # Returns
/// * `Ok(UpdateStats)` - Statistics about what was updated
/// * `Err(SubsetError)` - If the font data is invalid or update fails
///
/// # Safety
/// This function validates all changes to ensure font integrity is maintained.
///
/// # Example
/// ```rust
/// let (mut font_data, mapping) = subset_and_map(&provider, &glyph_ids)?;
/// let stats = update_composite_references(&mut font_data, &mapping)?;
/// println!("Updated {} composite glyphs with {} component references", 
///          stats.composites_updated, stats.references_updated);
/// ```
pub fn update_composite_references(
    font_data: &mut [u8],
    mapping: &HashMap<u16, u16>,
) -> Result<UpdateStats, SubsetError>

#[derive(Debug, Clone)]
pub struct UpdateStats {
    /// Number of composite glyphs that were updated
    pub composites_updated: usize,
    /// Total number of component references updated
    pub references_updated: usize,
    /// References that couldn't be mapped (defaulted to 0)
    pub unmapped_references: Vec<u16>,
    /// Whether any CFF subroutines were updated
    pub cff_subroutines_updated: bool,
}
```

### Detailed Behavior

#### TrueType/OpenType Fonts

1. **Parse glyf table location**
2. **For each glyph in the font:**
   ```rust
   if is_composite_glyph(glyph) {
       for component in glyph.components {
           let old_gid = read_u16_be(&font_data[component.offset]);
           if let Some(&new_gid) = mapping.get(&old_gid) {
               write_u16_be(&mut font_data[component.offset], new_gid);
               stats.references_updated += 1;
           } else {
               stats.unmapped_references.push(old_gid);
               write_u16_be(&mut font_data[component.offset], 0);  // .notdef
           }
       }
       stats.composites_updated += 1;
   }
   ```

#### CFF Fonts

1. **Parse CFF CharStrings**
2. **Update subroutine references if they reference glyphs**
3. **Handle CFF2 variation data**

### Error Handling

```rust
pub enum UpdateError {
    /// Font format not recognized
    UnknownFormat,
    /// glyf table not found (for TrueType)
    MissingGlyfTable,
    /// CFF table not found (for CFF fonts)
    MissingCFFTable,
    /// Invalid composite glyph structure
    InvalidComposite(u16),
    /// Component reference out of bounds
    InvalidComponentRef { glyph: u16, component: u16 },
}
```

## Feature 2: Enhanced Subset Result Structure

### Purpose

Provide comprehensive information about the subsetting operation beyond just the mapping, enabling better decision-making and debugging.

### Data Structure

```rust
/// Comprehensive result of font subsetting operation
#[derive(Debug, Clone)]
pub struct SubsetResult {
    /// The subset font data
    pub data: Vec<u8>,
    
    /// Mapping from old GID to new GID
    pub glyph_mapping: HashMap<u16, u16>,
    
    /// Inverse mapping for reverse lookups
    pub reverse_mapping: HashMap<u16, u16>,
    
    /// Glyphs that were added as dependencies (not explicitly requested)
    pub added_glyphs: Vec<u16>,
    
    /// Glyphs that were requested but not found
    pub missing_glyphs: Vec<u16>,
    
    /// Original font metadata
    pub original_info: FontInfo,
    
    /// Subset font metadata
    pub subset_info: FontInfo,
    
    /// Statistics about the subsetting
    pub stats: SubsetStats,
}

#[derive(Debug, Clone)]
pub struct FontInfo {
    /// Number of glyphs in the font
    pub glyph_count: u16,
    /// Maximum glyph ID
    pub max_gid: u16,
    /// Font format (TrueType, CFF, CFF2)
    pub format: FontFormat,
    /// Whether font contains composite glyphs
    pub has_composites: bool,
    /// Size in bytes
    pub size: usize,
}

#[derive(Debug, Clone)]
pub struct SubsetStats {
    /// Size reduction achieved
    pub size_reduction_bytes: i64,
    /// Size reduction percentage
    pub size_reduction_percent: f32,
    /// Number of composite glyphs included
    pub composite_glyphs: usize,
    /// Number of simple glyphs included
    pub simple_glyphs: usize,
    /// Tables that were removed
    pub removed_tables: Vec<String>,
}

/// Extended subset function returning comprehensive result
pub fn subset_detailed(
    provider: &impl FontTableProvider,
    glyph_ids: &[u16],
) -> Result<SubsetResult, SubsetError>
```

### Usage Example

```rust
let result = subset_detailed(&provider, &glyph_ids)?;

// Check if any dependencies were added
if !result.added_glyphs.is_empty() {
    println!("Added {} dependency glyphs", result.added_glyphs.len());
}

// Use reverse mapping for lookups
let old_gid = result.reverse_mapping.get(&new_gid).unwrap();

// Check subsetting efficiency
println!("Reduced font size by {:.1}%", result.stats.size_reduction_percent);
```

## Feature 3: PDF-Specific Subsetting Helper

### Purpose

Provide a high-level function specifically designed for PDF workflows that handles CIDToGIDMap generation and validation.

### Function Signature

```rust
/// Subset a font for PDF usage with automatic CIDToGIDMap handling
///
/// This function understands PDF font structures and generates all
/// necessary mappings and data for PDF font embedding.
///
/// # Arguments
/// * `provider` - Font table provider
/// * `glyph_ids` - Glyphs to include
/// * `pdf_context` - PDF-specific context information
///
/// # Returns
/// Complete PDF-ready font subsetting result
pub fn subset_for_pdf(
    provider: &impl FontTableProvider,
    glyph_ids: &[u16],
    pdf_context: &PdfFontContext,
) -> Result<PdfSubsetResult, SubsetError>

#[derive(Debug, Clone)]
pub struct PdfFontContext {
    /// Existing CIDToGIDMap if available (None for Identity)
    pub cid_to_gid_map: Option<Vec<u16>>,
    /// Maximum CID used in the document
    pub max_cid: u16,
    /// Whether this is a CID font
    pub is_cid_font: bool,
    /// Font's writing mode (horizontal/vertical)
    pub writing_mode: WritingMode,
}

#[derive(Debug, Clone)]
pub struct PdfSubsetResult {
    /// The subset font data with updated references
    pub font_data: Vec<u8>,
    
    /// Glyph ID mapping
    pub glyph_mapping: HashMap<u16, u16>,
    
    /// New CIDToGIDMap ready for PDF embedding (big-endian)
    pub cid_to_gid_map: Vec<u8>,
    
    /// Validation results
    pub validation: ValidationResult,
    
    /// Warnings about potential issues
    pub warnings: Vec<PdfWarning>,
}

#[derive(Debug, Clone)]
pub struct ValidationResult {
    /// All CIDs map to valid GIDs
    pub all_cids_mapped: bool,
    /// List of CIDs that map to GID 0 (missing glyph)
    pub missing_glyph_cids: Vec<u16>,
    /// CIDs that map to unmapped GIDs
    pub unmapped_cids: Vec<u16>,
}

#[derive(Debug, Clone)]
pub enum PdfWarning {
    /// CID maps to GID 0 (will render as missing)
    MissingGlyph { cid: u16 },
    /// GID not found in mapping
    UnmappedGlyph { gid: u16 },
    /// Composite glyph reference couldn't be resolved
    BrokenComposite { glyph: u16, component: u16 },
    /// CIDToGIDMap larger than necessary
    OversizedCidMap { actual: u16, needed: u16 },
}
```

### Detailed Behavior

1. **Subset the font with mapping**
   ```rust
   let (font_data, mapping) = subset_and_map(provider, glyph_ids)?;
   ```

2. **Update composite references automatically**
   ```rust
   update_composite_references(&mut font_data, &mapping)?;
   ```

3. **Generate new CIDToGIDMap**
   ```rust
   let mut cid_map = Vec::with_capacity((max_cid + 1) * 2);
   for cid in 0..=max_cid {
       let old_gid = context.cid_to_gid_map
           .map(|m| m[cid as usize])
           .unwrap_or(cid);  // Identity if no map
       
       let new_gid = mapping.get(&old_gid).copied().unwrap_or(0);
       
       // Write big-endian for PDF
       cid_map.extend_from_slice(&new_gid.to_be_bytes());
       
       // Track issues
       if new_gid == 0 && old_gid != 0 {
           validation.missing_glyph_cids.push(cid);
       }
   }
   ```

4. **Validate and warn**
   ```rust
   if !validation.missing_glyph_cids.is_empty() {
       warnings.push(PdfWarning::MissingGlyph { 
           cid: validation.missing_glyph_cids[0] 
       });
   }
   ```

### Usage Example

```rust
// In PDF processor
let context = PdfFontContext {
    cid_to_gid_map: Some(existing_map),
    max_cid: 1000,
    is_cid_font: true,
    writing_mode: WritingMode::Horizontal,
};

let result = subset_for_pdf(&provider, &used_glyphs, &context)?;

// Check for issues
for warning in &result.warnings {
    match warning {
        PdfWarning::MissingGlyph { cid } => {
            eprintln!("Warning: CID {} will render as missing glyph", cid);
        }
        _ => {}
    }
}

// Use the ready-to-embed data
embed_font_in_pdf(result.font_data);
embed_cid_map_in_pdf(result.cid_to_gid_map);
```

## Feature 4: Validation and Debugging Utilities

### Purpose

Provide utilities to validate subsetting results and debug issues.

### Function Signatures

```rust
/// Validate that a glyph mapping covers all required glyphs
pub fn validate_mapping_coverage(
    mapping: &HashMap<u16, u16>,
    required_gids: &[u16],
    cid_to_gid: Option<&[u16]>,
) -> Result<ValidationReport, ValidationError>

#[derive(Debug)]
pub struct ValidationReport {
    /// Glyphs that should be mapped but aren't
    pub unmapped_glyphs: Vec<u16>,
    /// CIDs that would map to missing glyphs
    pub affected_cids: Vec<u16>,
    /// Suggested fixes
    pub suggestions: Vec<String>,
    /// Overall validation status
    pub is_valid: bool,
}

/// Debug utility to dump human-readable mapping information
pub fn debug_mapping(
    mapping: &HashMap<u16, u16>,
    font_name: &str,
) -> String {
    // Returns formatted string like:
    // Font: Arial
    // Mapping (5 glyphs):
    //   GID 0 -> 0 (.notdef)
    //   GID 19 -> 1 (char 'A')
    //   GID 21 -> 2 (char 'C')
    //   ...
}

/// Detect potential issues with a subset font
pub fn diagnose_subset_issues(
    original_font: &[u8],
    subset_font: &[u8],
    mapping: &HashMap<u16, u16>,
) -> DiagnosticReport

#[derive(Debug)]
pub struct DiagnosticReport {
    /// Composite glyphs with potentially broken references
    pub broken_composites: Vec<u16>,
    /// Glyphs that reference missing components
    pub missing_components: Vec<(u16, u16)>,
    /// Encoding issues detected
    pub encoding_issues: Vec<String>,
    /// Recommended actions
    pub recommendations: Vec<String>,
}
```

## Feature 5: Builder Pattern for Complex Subsetting

### Purpose

Provide a fluent API for complex subsetting scenarios with multiple options.

### API Design

```rust
/// Builder for advanced font subsetting operations
pub struct SubsetBuilder<'a> {
    provider: &'a dyn FontTableProvider,
    glyph_ids: Vec<u16>,
    options: SubsetOptions,
}

impl<'a> SubsetBuilder<'a> {
    /// Create new builder
    pub fn new(provider: &'a dyn FontTableProvider) -> Self
    
    /// Add glyphs to subset
    pub fn with_glyphs(mut self, glyph_ids: &[u16]) -> Self
    
    /// Add glyphs by unicode characters
    pub fn with_characters(mut self, chars: &str) -> Self
    
    /// Configure for PDF usage
    pub fn for_pdf(mut self, max_cid: u16) -> Self
    
    /// Set CIDToGIDMap for updates
    pub fn with_cid_map(mut self, map: &[u16]) -> Self
    
    /// Automatically fix composite references
    pub fn fix_composites(mut self, enabled: bool) -> Self
    
    /// Set validation level
    pub fn validation_level(mut self, level: ValidationLevel) -> Self
    
    /// Execute subsetting
    pub fn build(self) -> Result<SubsetResult, SubsetError>
}

#[derive(Debug, Clone)]
pub enum ValidationLevel {
    /// No validation (fastest)
    None,
    /// Basic validation (check critical glyphs)
    Basic,
    /// Standard validation (recommended)
    Standard,
    /// Strict validation (comprehensive but slower)
    Strict,
}
```

### Usage Example

```rust
let result = SubsetBuilder::new(&provider)
    .with_glyphs(&used_glyphs)
    .for_pdf(max_cid)
    .with_cid_map(&existing_map)
    .fix_composites(true)
    .validation_level(ValidationLevel::Standard)
    .build()?;

// All composite references fixed, CIDToGIDMap generated, validated
```

## Implementation Priority

### Phase 1: Core Features (Highest Priority)
1. `update_composite_references` - Critical for correctness
2. `SubsetResult` structure - Provides necessary metadata

### Phase 2: PDF-Specific Features (High Priority)
3. `subset_for_pdf` - Dramatically simplifies PDF workflows
4. Validation utilities - Helps catch issues early

### Phase 3: Convenience Features (Nice to Have)
5. Builder pattern - Better API ergonomics
6. Debug utilities - Helpful for development

## Testing Requirements

### Unit Tests

```rust
#[test]
fn test_composite_reference_update() {
    // Test that all composite references are updated correctly
}

#[test]
fn test_cid_to_gid_generation() {
    // Test CIDToGIDMap generation with various mappings
}

#[test]
fn test_validation_catches_missing_glyphs() {
    // Ensure validation detects unmapped glyphs
}
```

### Integration Tests

```rust
#[test]
fn test_pdf_workflow_end_to_end() {
    // Complete PDF subsetting workflow
    let context = PdfFontContext { /* ... */ };
    let result = subset_for_pdf(&provider, &glyphs, &context)?;
    
    // Verify PDF renders correctly with result
    assert!(result.validation.all_cids_mapped);
    assert!(result.warnings.is_empty());
}
```

### Benchmark Tests

```rust
#[bench]
fn bench_composite_update_performance() {
    // Ensure reference updating is fast
}

#[bench]
fn bench_pdf_subsetting_vs_manual() {
    // Compare automated vs manual workflow
}
```

## Success Criteria

1. **Correctness**: All composite references updated accurately
2. **Completeness**: CIDToGIDMap covers all required CIDs
3. **Performance**: < 10% overhead vs manual implementation
4. **Usability**: Reduces PDF subsetting code by > 70%
5. **Reliability**: Catches common errors before they cause rendering issues

## Migration Path

### Before (Complex Manual Process)

```rust
// 100+ lines of error-prone manual code
let (font_data, mapping) = subset_and_map(&provider, &glyphs)?;
// ... manually patch composites ...
// ... manually build CIDToGIDMap ...
// ... hope nothing broke ...
```

### After (Simple Automated Process)

```rust
// 5 lines of validated, correct code
let result = subset_for_pdf(&provider, &glyphs, &pdf_context)?;
if !result.warnings.is_empty() {
    log::warn!("Font subsetting warnings: {:?}", result.warnings);
}
embed_font(result.font_data, result.cid_to_gid_map);
```

## Conclusion

These PDF-specific features would transform Allsorts from a general-purpose font subsetting library into a comprehensive solution for PDF font processing. By handling the complex details of composite glyph references, CIDToGIDMap generation, and validation, these features would eliminate entire categories of bugs and reduce implementation complexity by an order of magnitude.

The phased implementation approach allows for incremental value delivery, with the most critical features (composite reference updating) implemented first, followed by progressively more convenient features. This ensures that even a partial implementation provides significant value to PDF processing applications.