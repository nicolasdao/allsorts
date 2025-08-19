# PDF-Specific Font Subsetting API Documentation

## Table of Contents
1. [Overview](#overview)
2. [Phase 1: Core Features](#phase-1-core-features)
   - [Composite Glyph Reference Updater](#composite-glyph-reference-updater)
   - [Enhanced Subset Result Structure](#enhanced-subset-result-structure)
3. [Phase 2: PDF-Specific Features](#phase-2-pdf-specific-features)
   - [PDF-Specific Subsetting Helper](#pdf-specific-subsetting-helper)
   - [Validation and Debugging Utilities](#validation-and-debugging-utilities)
4. [Phase 3: Builder Pattern API](#phase-3-builder-pattern-api)
   - [SubsetBuilder Overview](#subsetbuilder-overview)
   - [Validation Levels](#validation-levels)
   - [Builder Methods](#builder-methods)
   - [Complete Builder Examples](#complete-builder-examples)
5. [Integration Guide](#integration-guide)
6. [Complete Examples](#complete-examples)
7. [Performance Considerations](#performance-considerations)
8. [Error Handling](#error-handling)

## Overview

The PDF-specific font subsetting features extend Allsorts' capabilities with specialized functionality for embedding fonts in PDF documents. These features were developed to address common challenges in PDF font subsetting:

- **Composite glyph integrity**: Ensuring composite glyphs maintain correct references after subsetting
- **CID font support**: Proper handling of CID-keyed fonts commonly used in PDFs
- **Validation and diagnostics**: Tools to detect and fix subsetting issues
- **Detailed result tracking**: Comprehensive information about subsetting operations

### Architecture

The implementation is organized into five main modules:

```
src/subset/
├── composite.rs    # Composite glyph reference updating
├── result.rs       # Enhanced result structures
├── pdf.rs          # PDF-specific subsetting
├── validation.rs   # Validation and debugging utilities
└── builder.rs      # Builder Pattern API for unified interface
```

## Phase 1: Core Features

### Composite Glyph Reference Updater

**Module**: `src/subset/composite.rs`

#### Purpose

Composite glyphs in TrueType fonts contain references to other glyphs (components). When subsetting, these component references must be updated to reflect new glyph IDs in the subset font. Without this update, composite glyphs would reference non-existent glyphs, causing rendering issues.

#### API Reference

```rust
pub fn update_composite_references(
    font_data: &mut [u8],
    mapping: &HashMap<u16, u16>
) -> Result<UpdateStats, SubsetError>
```

**Parameters:**
- `font_data`: Mutable reference to the font file bytes
- `mapping`: HashMap mapping old glyph IDs to new glyph IDs

**Returns:**
- `UpdateStats`: Statistics about the update operation
- `SubsetError`: Error if update fails

```rust
pub struct UpdateStats {
    /// Number of composite glyphs that were updated
    pub composites_updated: usize,
    /// Total number of component references updated
    pub references_updated: usize,
    /// List of component references that couldn't be mapped
    pub unmapped_references: Vec<u16>,
    /// Whether CFF/CFF2 subroutines were updated
    pub cff_subroutines_updated: bool,
}
```

#### How It Works

1. **Parses font tables**: Reads the font's glyf, loca, head, and maxp tables
2. **Identifies composites**: Scans through glyphs to find those with the composite flag
3. **Updates references**: For each composite glyph:
   - Extracts component glyph IDs
   - Maps them using the provided mapping
   - Writes updated IDs back to the font data
4. **Handles multiple formats**: Supports both short (16-bit) and long (32-bit) loca formats

#### Usage Example

```rust
use allsorts::subset::composite::update_composite_references;
use std::collections::HashMap;

// After subsetting, you have a mapping of old to new glyph IDs
let mapping = HashMap::from([
    (0, 0),   // .notdef stays at 0
    (42, 1),  // Glyph 42 becomes 1
    (43, 2),  // Glyph 43 becomes 2
    (100, 3), // Composite glyph 100 becomes 3
]);

// Update the font data in-place
let stats = update_composite_references(&mut font_data, &mapping)?;

println!("Updated {} composite glyphs", stats.composites_updated);
println!("Fixed {} component references", stats.references_updated);

if !stats.unmapped_references.is_empty() {
    eprintln!("Warning: {} references couldn't be mapped", 
              stats.unmapped_references.len());
}
```

### Enhanced Subset Result Structure

**Module**: `src/subset/result.rs`

#### Purpose

Provides detailed information about subsetting operations, including bidirectional glyph mappings, statistics, and metadata. This is essential for:
- Debugging subsetting issues
- Tracking which glyphs were included/excluded
- Understanding the impact of subsetting on file size
- Maintaining glyph relationships for advanced typography

#### API Reference

```rust
pub fn subset_detailed(
    provider: &impl FontTableProvider,
    glyphs: &[u16],
    profile: &SubsetProfile,
    cmap_target: CmapTarget,
) -> Result<SubsetResult, SubsetError>
```

**Parameters:**
- `provider`: Font table provider for reading font data
- `glyphs`: Array of glyph IDs to include in subset
- `profile`: Subsetting profile (e.g., Minimal, Web, Pdf)
- `cmap_target`: Target character map format

**Returns:**
- `SubsetResult`: Comprehensive subsetting results

```rust
pub struct SubsetResult {
    /// The subsetted font data
    pub data: Vec<u8>,
    /// Mapping from old to new glyph IDs
    pub glyph_mapping: HashMap<u16, u16>,
    /// Reverse mapping from new to old glyph IDs
    pub reverse_mapping: HashMap<u16, u16>,
    /// Glyphs added beyond those requested (e.g., composites)
    pub added_glyphs: Vec<u16>,
    /// Requested glyphs that couldn't be found
    pub missing_glyphs: Vec<u16>,
    /// Information about the original font
    pub original_info: FontInfo,
    /// Information about the subset font
    pub subset_info: FontInfo,
    /// Subsetting statistics
    pub stats: SubsetStats,
}

pub struct FontInfo {
    /// Number of glyphs in the font
    pub glyph_count: u16,
    /// Maximum glyph ID in the font
    pub max_gid: u16,
    /// Font format (TrueType, CFF, or CFF2)
    pub format: FontFormat,
    /// Whether the font contains composite glyphs
    pub has_composites: bool,
    /// Size of the font data in bytes
    pub size: usize,
}

pub struct SubsetStats {
    /// Size reduction in bytes
    pub size_reduction_bytes: i64,
    /// Size reduction as a percentage
    pub size_reduction_percent: f32,
    /// Number of composite glyphs in the subset
    pub composite_glyphs: usize,
    /// Number of simple glyphs in the subset
    pub simple_glyphs: usize,
    /// Tables that were removed during subsetting
    pub removed_tables: Vec<String>,
}
```

#### Usage Example

```rust
use allsorts::subset::result::subset_detailed;
use allsorts::subset::{SubsetProfile, CmapTarget};

// Subset with detailed tracking
let result = subset_detailed(
    &provider,
    &[0, 42, 43, 100], // Glyph IDs to include
    &SubsetProfile::Pdf,
    CmapTarget::Unrestricted,
)?;

// Access comprehensive information
println!("Original font: {} glyphs, {} bytes", 
         result.original_info.glyph_count,
         result.original_info.size);

println!("Subset font: {} glyphs, {} bytes",
         result.subset_info.glyph_count,
         result.subset_info.size);

println!("Size reduction: {:.1}%", result.stats.size_reduction_percent);

// Check if specific glyphs were included
if !result.missing_glyphs.is_empty() {
    eprintln!("Warning: Couldn't find glyphs: {:?}", result.missing_glyphs);
}

// Track additional glyphs that were included
if !result.added_glyphs.is_empty() {
    println!("Added {} dependency glyphs", result.added_glyphs.len());
}

// Use mappings for further processing
for (old_id, new_id) in &result.glyph_mapping {
    println!("Glyph {} -> {}", old_id, new_id);
}
```

## Phase 2: PDF-Specific Features

### PDF-Specific Subsetting Helper

**Module**: `src/subset/pdf.rs`

#### Purpose

Optimizes font subsetting specifically for PDF embedding, handling:
- CID-keyed fonts (common in PDFs for Asian text)
- CIDToGIDMap generation for PDF renderers
- PDF-specific validation requirements
- Composite glyph fixing integrated into the workflow

#### API Reference

```rust
pub fn subset_for_pdf(
    provider: &impl FontTableProvider,
    glyph_ids: &[u16],
    pdf_context: &PdfFontContext,
) -> Result<PdfSubsetResult, SubsetError>
```

**Parameters:**
- `provider`: Font table provider
- `glyph_ids`: Glyphs to include in subset
- `pdf_context`: PDF-specific configuration

**Data Structures:**

```rust
pub struct PdfFontContext {
    /// Optional existing CID to GID mapping
    pub cid_to_gid_map: Option<Vec<u16>>,
    /// Maximum CID value in the font
    pub max_cid: u16,
    /// Whether this is a CID font
    pub is_cid_font: bool,
    /// Writing mode for the font
    pub writing_mode: WritingMode,
}

pub enum WritingMode {
    Horizontal,
    Vertical,
}

pub struct PdfSubsetResult {
    /// The subsetted font data with fixed composites
    pub font_data: Vec<u8>,
    /// Mapping from old to new glyph IDs
    pub glyph_mapping: HashMap<u16, u16>,
    /// CIDToGIDMap for PDF embedding (big-endian)
    pub cid_to_gid_map: Vec<u8>,
    /// Validation results
    pub validation: ValidationResult,
    /// Warnings generated during subsetting
    pub warnings: Vec<PdfWarning>,
}

pub struct ValidationResult {
    /// Whether all CIDs are properly mapped
    pub all_cids_mapped: bool,
    /// CIDs that reference missing glyphs
    pub missing_glyph_cids: Vec<u16>,
    /// CIDs that couldn't be mapped
    pub unmapped_cids: Vec<u16>,
}

pub enum PdfWarning {
    MissingGlyph { cid: u16 },
    UnmappedGlyph { gid: u16 },
    BrokenComposite { glyph: u16, component: u16 },
    OversizedCidMap { actual: u16, needed: u16 },
}
```

#### How CIDToGIDMap Works

The CIDToGIDMap is a critical structure for PDF fonts:

1. **Purpose**: Maps Character IDs (CIDs) to Glyph IDs (GIDs) in the subset font
2. **Format**: Array of 16-bit values in big-endian byte order
3. **Size**: (max_cid + 1) * 2 bytes
4. **Usage**: PDF renderers use this to find the correct glyph for each character

#### Usage Example

```rust
use allsorts::subset::pdf::{subset_for_pdf, PdfFontContext, WritingMode};

// Configure for CID font with identity mapping
let pdf_context = PdfFontContext {
    cid_to_gid_map: None, // Use identity mapping
    max_cid: 255,         // Support CIDs 0-255
    is_cid_font: true,
    writing_mode: WritingMode::Horizontal,
};

// Subset for PDF embedding
let result = subset_for_pdf(
    &provider,
    &[0, 42, 43, 100],
    &pdf_context,
)?;

// The font data is ready for PDF embedding
write_to_pdf(result.font_data);

// Embed the CIDToGIDMap in the PDF
pdf_font_descriptor.set_cid_to_gid_map(result.cid_to_gid_map);

// Check for warnings
for warning in &result.warnings {
    match warning {
        PdfWarning::MissingGlyph { cid } => {
            eprintln!("CID {} references a missing glyph", cid);
        },
        PdfWarning::BrokenComposite { glyph, component } => {
            eprintln!("Glyph {} has broken reference to {}", glyph, component);
        },
        _ => {}
    }
}

// Validate the mapping
if !result.validation.all_cids_mapped {
    eprintln!("Some CIDs couldn't be mapped: {:?}", 
              result.validation.unmapped_cids);
}
```

### Validation and Debugging Utilities

**Module**: `src/subset/validation.rs`

#### Purpose

Provides tools to validate glyph mappings and diagnose subsetting issues. Essential for:
- Ensuring subset integrity before PDF embedding
- Debugging rendering issues
- Providing actionable feedback to developers

#### API Reference

##### Mapping Coverage Validation

```rust
pub fn validate_mapping_coverage(
    mapping: &HashMap<u16, u16>,
    required_gids: &[u16],
    cid_to_gid: Option<&[u16]>,
) -> Result<ValidationReport, ValidationError>
```

**Purpose**: Validates that all required glyphs are present in the mapping.

**Parameters:**
- `mapping`: The glyph ID mapping to validate
- `required_gids`: Glyphs that must be present
- `cid_to_gid`: Optional CID-to-GID mapping to check

**Returns:**
```rust
pub struct ValidationReport {
    /// Glyphs that are unmapped
    pub unmapped_glyphs: Vec<u16>,
    /// CIDs affected by unmapped glyphs
    pub affected_cids: Vec<u16>,
    /// Suggestions for fixing issues
    pub suggestions: Vec<String>,
    /// Whether the mapping is valid
    pub is_valid: bool,
}
```

##### Debug Mapping Output

```rust
pub fn debug_mapping(
    mapping: &HashMap<u16, u16>,
    font_name: &str,
) -> String
```

**Purpose**: Generates human-readable representation of a glyph mapping.

##### Subset Issue Diagnosis

```rust
pub fn diagnose_subset_issues(
    original_font: &[u8],
    subset_font: &[u8],
    mapping: &HashMap<u16, u16>,
) -> DiagnosticReport
```

**Purpose**: Analyzes a subset font for potential issues.

**Returns:**
```rust
pub struct DiagnosticReport {
    /// Composite glyphs with broken references
    pub broken_composites: Vec<u16>,
    /// Missing component glyphs (parent, component)
    pub missing_components: Vec<(u16, u16)>,
    /// Character encoding issues
    pub encoding_issues: Vec<String>,
    /// Recommendations for fixing issues
    pub recommendations: Vec<String>,
}
```

#### Usage Examples

##### Example 1: Validate Before PDF Embedding

```rust
use allsorts::subset::validation::validate_mapping_coverage;

// Validate that all required glyphs are mapped
let required_glyphs = vec![0, 42, 43, 100, 101];
let report = validate_mapping_coverage(
    &glyph_mapping,
    &required_glyphs,
    Some(&cid_to_gid_map),
)?;

if !report.is_valid {
    eprintln!("Validation failed!");
    eprintln!("Unmapped glyphs: {:?}", report.unmapped_glyphs);
    eprintln!("Affected CIDs: {:?}", report.affected_cids);
    
    // Show suggestions
    for suggestion in report.suggestions {
        eprintln!("Suggestion: {}", suggestion);
    }
    
    return Err("Cannot proceed with invalid mapping");
}
```

##### Example 2: Debug Mapping for Troubleshooting

```rust
use allsorts::subset::validation::debug_mapping;

// Generate debug output
let debug_output = debug_mapping(&glyph_mapping, "Arial Unicode MS");
println!("{}", debug_output);

// Output:
// Font: Arial Unicode MS
// Mapping (156 glyphs):
//   GID 0 -> 0 (.notdef)
//   GID 42 -> 1 (glyph_42)
//   GID 43 -> 2 (glyph_43)
//   ... and 153 more
```

##### Example 3: Diagnose Subset Issues

```rust
use allsorts::subset::validation::diagnose_subset_issues;

// Diagnose potential issues
let report = diagnose_subset_issues(
    &original_font_data,
    &subset_font_data,
    &glyph_mapping,
);

if !report.broken_composites.is_empty() {
    eprintln!("Found {} broken composite glyphs", 
              report.broken_composites.len());
    
    for (glyph, component) in &report.missing_components {
        eprintln!("  Glyph {} missing component {}", glyph, component);
    }
}

// Apply recommendations
for recommendation in report.recommendations {
    println!("Recommended action: {}", recommendation);
}
```

## Phase 3: Builder Pattern API

### SubsetBuilder Overview

**Module**: `src/subset/builder.rs`

#### Purpose

The Builder Pattern API provides a fluent, intuitive interface for font subsetting that unifies all features from Phases 1 and 2. It was created to:

- **Simplify the API**: Replace complex function calls with chainable methods
- **Improve discoverability**: All options are available as methods on a single builder
- **Provide sensible defaults**: Works out-of-the-box with minimal configuration
- **Enable progressive enhancement**: Start simple, add complexity as needed
- **Reduce errors**: Type-safe validation levels and automatic consistency checks

#### Core Structure

```rust
pub struct SubsetBuilder<'a> {
    provider: &'a dyn FontTableProvider,
    glyph_ids: Vec<u16>,
    cmap_target: CmapTarget,
    profile: SubsetProfile,
    fix_composites: bool,
    validation_level: ValidationLevel,
    pdf_context: Option<PdfFontContext>,
}
```

### Validation Levels

The builder supports four validation levels to balance safety and performance:

```rust
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum ValidationLevel {
    /// No validation - accepts any input (fastest, least safe)
    None,
    /// Basic validation - ensures .notdef is present
    Basic,
    /// Standard validation - checks glyph IDs exist (default)
    Standard,
    /// Strict validation - comprehensive checks including duplicates
    Strict,
}
```

**When to use each level:**
- `None`: When you trust the input completely and need maximum performance
- `Basic`: For simple fonts where you only care about .notdef
- `Standard`: Default choice for most applications
- `Strict`: When debugging or working with untrusted fonts

### Builder Methods

#### Creating a Builder

```rust
pub fn new(provider: &'a dyn FontTableProvider) -> Self
```

**Purpose**: Creates a new builder with default settings.

**Default Configuration**:
- Includes glyph 0 (.notdef)
- Uses `SubsetProfile::Minimal`
- Sets `CmapTarget::Unrestricted`
- Enables `ValidationLevel::Standard`
- Composite fixing disabled

**Example**:
```rust
let builder = SubsetBuilder::new(&provider);
```

#### Adding Glyphs by ID

```rust
pub fn with_glyphs(self, glyph_ids: &[u16]) -> Self
```

**Purpose**: Specifies which glyphs to include by their IDs.

**Input**: Array of glyph IDs (e.g., `[0, 42, 100]`)

**Behavior**:
- Automatically ensures .notdef (glyph 0) is first
- Removes duplicate IDs while preserving order
- Replaces any previously set glyphs

**Example**:
```rust
let builder = SubsetBuilder::new(&provider)
    .with_glyphs(&[0, 42, 43, 100]);
```

#### Adding Glyphs by Character

```rust
pub fn with_characters(self, chars: &str) -> Result<Self, SubsetError>
```

**Purpose**: Maps Unicode characters to glyphs using the font's cmap table.

**Input**: String of characters to include (e.g., "Hello, World!")

**Output**: `Result` containing the builder or an error if character mapping fails

**Behavior**:
- Uses the font's character map to find glyph IDs
- Automatically includes .notdef
- Skips characters that don't exist in the font
- Preserves existing glyphs (additive)

**Example**:
```rust
let builder = SubsetBuilder::new(&provider)
    .with_characters("Hello, 世界!")?
    .with_glyphs(&[100, 101]); // Can combine with explicit IDs
```

#### PDF Configuration

```rust
pub fn for_pdf(self, max_cid: u16) -> Self
```

**Purpose**: Optimizes subsetting for PDF embedding.

**Input**: Maximum CID value to support (typically 255 or 65535)

**Automatic Configuration**:
- Sets profile to `SubsetProfile::Pdf`
- Enables composite glyph fixing
- Creates PDF context for CID font handling
- Sets horizontal writing mode by default

**Example**:
```rust
let builder = SubsetBuilder::new(&provider)
    .with_characters("Sample text")?  
    .for_pdf(255); // Support CIDs 0-255
```

#### CID Mapping

```rust
pub fn with_cid_map(self, map: &[u16]) -> Self
```

**Purpose**: Sets a custom CID-to-GID mapping for PDF fonts.

**Input**: Array where index is CID and value is GID

**Use Case**: When you have an existing CID mapping from a PDF

**Example**:
```rust
let cid_map = vec![0, 42, 43, 100]; // CID 1->GID 42, CID 2->GID 43, etc.
let builder = SubsetBuilder::new(&provider)
    .for_pdf(255)
    .with_cid_map(&cid_map);
```

#### Composite Glyph Handling

```rust
pub fn fix_composites(self, enabled: bool) -> Self
```

**Purpose**: Controls whether to update composite glyph references.

**Input**: Boolean flag (true to enable, false to disable)

**When to Enable**:
- Always for PDF embedding (auto-enabled by `for_pdf()`)
- When subsetting TrueType fonts with composite glyphs
- When you see rendering issues with composite characters

**Example**:
```rust
let builder = SubsetBuilder::new(&provider)
    .with_glyphs(&[0, 100, 101])
    .fix_composites(true);
```

#### Validation Configuration

```rust
pub fn validation_level(self, level: ValidationLevel) -> Self
```

**Purpose**: Sets how strictly to validate input glyphs.

**Input**: `ValidationLevel` enum value

**Validation Checks by Level**:
- `None`: No checks
- `Basic`: Ensures .notdef is first
- `Standard`: Checks all glyph IDs are valid
- `Strict`: Also checks for duplicates and dependencies

**Example**:
```rust
let builder = SubsetBuilder::new(&provider)
    .with_glyphs(&[0, 9999]) // Invalid glyph
    .validation_level(ValidationLevel::None); // Skip validation
```

#### Profile Selection

```rust
pub fn with_profile(self, profile: SubsetProfile) -> Self
```

**Purpose**: Selects which tables to include in the subset.

**Available Profiles**:
- `SubsetProfile::Minimal`: Smallest possible font
- `SubsetProfile::Web`: Optimized for web use
- `SubsetProfile::Pdf`: Optimized for PDF embedding
- `SubsetProfile::Print`: Includes all printing-related tables

**Example**:
```rust
let builder = SubsetBuilder::new(&provider)
    .with_characters("Hello")?  
    .with_profile(SubsetProfile::Web);
```

#### Character Map Target

```rust
pub fn with_cmap_target(self, target: CmapTarget) -> Self
```

**Purpose**: Specifies the character mapping format for the subset.

**Options**:
- `CmapTarget::Unrestricted`: Keep all cmap subtables
- `CmapTarget::Unicode`: Only Unicode mappings
- `CmapTarget::WindowsUnicode`: Windows-specific Unicode

**Example**:
```rust
let builder = SubsetBuilder::new(&provider)
    .with_characters("Hello")?  
    .with_cmap_target(CmapTarget::Unicode);
```

#### Building the Subset

```rust
pub fn build(self) -> Result<SubsetResult, SubsetError>
```

**Purpose**: Executes the subsetting operation with all configured options.

**Output**: `SubsetResult` containing:
- `data: Vec<u8>` - The subsetted font bytes
- `glyph_mapping: HashMap<u16, u16>` - Old to new glyph ID mapping
- `reverse_mapping: HashMap<u16, u16>` - New to old glyph ID mapping
- `stats: SubsetStats` - Statistics about the operation
- Additional metadata about the subset

**Process**:
1. Validates input based on validation level
2. Performs subsetting (PDF-specific or standard)
3. Fixes composite references if enabled
4. Collects statistics and metadata
5. Returns comprehensive result

**Example**:
```rust
let result = SubsetBuilder::new(&provider)
    .with_characters("Hello, World!")?  
    .for_pdf(255)
    .validation_level(ValidationLevel::Strict)
    .build()?;

println!("Subset size: {} bytes", result.data.len());
println!("Included {} glyphs", result.glyph_mapping.len());
```

### Complete Builder Examples

#### Example 1: Simple Text Subsetting

```rust
use allsorts::subset::builder::SubsetBuilder;

// Subset a font for specific text
fn subset_for_text(provider: &dyn FontTableProvider, text: &str) -> Result<Vec<u8>, SubsetError> {
    SubsetBuilder::new(provider)
        .with_characters(text)?
        .build()
        .map(|result| result.data)
}

// Usage
let subset_data = subset_for_text(&provider, "Hello, World!")?;
println!("Created subset with {} bytes", subset_data.len());
```

#### Example 2: PDF Embedding with Validation

```rust
use allsorts::subset::builder::{SubsetBuilder, ValidationLevel};

fn create_pdf_font(
    provider: &dyn FontTableProvider,
    text: &str,
) -> Result<(Vec<u8>, HashMap<u16, u16>), Box<dyn Error>> {
    let result = SubsetBuilder::new(provider)
        .with_characters(text)?
        .for_pdf(255)  // Auto-enables composite fixing
        .validation_level(ValidationLevel::Strict)
        .build()?;
    
    // Check for issues
    if !result.missing_glyphs.is_empty() {
        eprintln!("Warning: {} glyphs not found", result.missing_glyphs.len());
    }
    
    println!("Size reduction: {:.1}%", result.stats.size_reduction_percent);
    
    Ok((result.data, result.glyph_mapping))
}
```

#### Example 3: Advanced Configuration with Multiple Sources

```rust
use allsorts::subset::builder::{SubsetBuilder, ValidationLevel};
use allsorts::subset::{SubsetProfile, CmapTarget};

fn advanced_subset(
    provider: &dyn FontTableProvider,
    text: &str,
    extra_glyphs: &[u16],
) -> Result<SubsetResult, SubsetError> {
    SubsetBuilder::new(provider)
        // Add glyphs from text
        .with_characters(text)?
        // Add specific glyph IDs
        .with_glyphs(extra_glyphs)
        // Configure for PDF
        .for_pdf(65535)
        // Use specific profile
        .with_profile(SubsetProfile::Pdf)
        // Target Unicode cmap
        .with_cmap_target(CmapTarget::Unicode)
        // Enable composite fixing
        .fix_composites(true)
        // Use strict validation
        .validation_level(ValidationLevel::Strict)
        // Build the subset
        .build()
}

// Usage
let result = advanced_subset(
    &provider,
    "Hello, 世界!",
    &[100, 101, 102], // Additional glyphs
)?;

println!("Subset statistics:");
println!("  Original size: {} bytes", result.original_info.size);
println!("  Subset size: {} bytes", result.subset_info.size);
println!("  Size reduction: {:.1}%", result.stats.size_reduction_percent);
println!("  Glyphs included: {}", result.glyph_mapping.len());
```

#### Example 4: Multi-language PDF with Error Handling

```rust
use allsorts::subset::builder::{SubsetBuilder, ValidationLevel};

fn subset_for_languages(
    provider: &dyn FontTableProvider,
    languages: &[(&str, &str)], // (language_code, text)
) -> Result<Vec<SubsetResult>, Box<dyn Error>> {
    let mut results = Vec::new();
    
    for (lang, text) in languages {
        println!("Processing {} text...", lang);
        
        let result = SubsetBuilder::new(provider)
            .with_characters(text)
            .map_err(|e| format!("Failed to map characters for {}: {}", lang, e))?
            .for_pdf(65535)
            .validation_level(ValidationLevel::Standard)
            .build()
            .map_err(|e| format!("Failed to subset for {}: {}", lang, e))?;
        
        // Report statistics
        println!("  {} glyphs for {}", result.glyph_mapping.len(), lang);
        println!("  Size: {} bytes", result.data.len());
        
        results.push(result);
    }
    
    Ok(results)
}

// Usage
let languages = vec![
    ("en", "Hello, World!"),
    ("zh", "你好，世界！"),
    ("ja", "こんにちは、世界！"),
    ("ar", "مرحبا بالعالم!"),
];

let subsets = subset_for_languages(&provider, &languages)?;
println!("Created {} language-specific subsets", subsets.len());
```

#### Example 5: Diagnostic Workflow

```rust
use allsorts::subset::builder::{SubsetBuilder, ValidationLevel};
use allsorts::subset::validation::diagnose_subset_issues;

fn subset_with_diagnostics(
    provider: &dyn FontTableProvider,
    original_font: &[u8],
    text: &str,
) -> Result<SubsetResult, Box<dyn Error>> {
    // Try strict validation first
    let result = SubsetBuilder::new(provider)
        .with_characters(text)?
        .for_pdf(255)
        .validation_level(ValidationLevel::Strict)
        .build();
    
    match result {
        Ok(subset) => {
            // Diagnose any potential issues
            let report = diagnose_subset_issues(
                original_font,
                &subset.data,
                &subset.glyph_mapping,
            );
            
            if !report.broken_composites.is_empty() {
                println!("Note: {} composite glyphs may need attention", 
                        report.broken_composites.len());
            }
            
            Ok(subset)
        },
        Err(e) => {
            // Fallback to less strict validation
            println!("Strict validation failed: {}, trying standard...", e);
            
            SubsetBuilder::new(provider)
                .with_characters(text)?
                .for_pdf(255)
                .validation_level(ValidationLevel::Standard)
                .build()
        }
    }
}
```

## Integration Guide

### Complete PDF Subsetting Workflow

Here's how to integrate all features for a complete PDF font subsetting workflow:

```rust
use allsorts::subset::{
    composite::update_composite_references,
    result::subset_detailed,
    pdf::{subset_for_pdf, PdfFontContext, WritingMode},
    validation::{validate_mapping_coverage, diagnose_subset_issues},
    SubsetProfile, CmapTarget,
};
use std::collections::HashMap;

pub fn subset_font_for_pdf(
    font_data: &[u8],
    text: &str,
    max_cid: u16,
) -> Result<PdfEmbeddableFont, Box<dyn Error>> {
    // Step 1: Create font provider
    let scope = ReadScope::new(font_data);
    let font_file = scope.read::<OpenTypeFont>()?;
    let provider = font_file.table_provider(0)?;
    
    // Step 2: Map characters to glyphs
    let mut font = Font::new(provider.box_clone())?;
    let mut glyph_ids = vec![0]; // Always include .notdef
    
    for ch in text.chars() {
        let (gid, _) = font.lookup_glyph_index(
            ch,
            MatchingPresentation::NotRequired,
            None,
        );
        if gid != 0 && !glyph_ids.contains(&gid) {
            glyph_ids.push(gid);
        }
    }
    
    // Step 3: Configure PDF context
    let pdf_context = PdfFontContext {
        cid_to_gid_map: None,
        max_cid,
        is_cid_font: true,
        writing_mode: WritingMode::Horizontal,
    };
    
    // Step 4: Perform PDF-optimized subsetting
    let subset_result = subset_for_pdf(
        &provider,
        &glyph_ids,
        &pdf_context,
    )?;
    
    // Step 5: Validate the result
    let validation_report = validate_mapping_coverage(
        &subset_result.glyph_mapping,
        &glyph_ids,
        None,
    )?;
    
    if !validation_report.is_valid {
        // Handle validation failure
        for suggestion in validation_report.suggestions {
            eprintln!("Validation issue: {}", suggestion);
        }
    }
    
    // Step 6: Check for warnings
    for warning in &subset_result.warnings {
        match warning {
            PdfWarning::BrokenComposite { glyph, component } => {
                eprintln!("Warning: Glyph {} has broken reference to {}", 
                         glyph, component);
            },
            _ => {}
        }
    }
    
    // Step 7: Return embeddable font
    Ok(PdfEmbeddableFont {
        font_data: subset_result.font_data,
        cid_to_gid_map: subset_result.cid_to_gid_map,
        glyph_count: subset_result.glyph_mapping.len() as u16,
    })
}
```

### Using with Existing Allsorts APIs

The new features integrate seamlessly with existing Allsorts functionality:

```rust
use allsorts::{Font, font::MatchingPresentation};
use allsorts::subset::{subset_and_map, SubsetResult};
use allsorts::subset::composite::update_composite_references;

// Use standard Allsorts for initial subsetting
let result = subset_and_map(
    &provider,
    &glyph_ids,
    &SubsetProfile::Pdf,
    CmapTarget::Unrestricted,
)?;

// Extract font data and mapping from SubsetResult
let (mut font_data, mapping) = match result {
    SubsetResult::Simple { font_data, glyph_mapping } => (font_data, glyph_mapping),
    SubsetResult::Cid { font_data, glyph_mapping, .. } => (font_data, glyph_mapping),
};

// Apply composite fixing from new features
let stats = update_composite_references(&mut font_data, &mapping)?;

// Continue with standard Allsorts operations
let subset_font = Font::new(provider_from_data(&font_data))?;
```

## Complete Examples

### Example 1: Subset Font for Multi-language PDF

```rust
use allsorts::subset::pdf::{subset_for_pdf, PdfFontContext, WritingMode};

fn subset_for_multilingual_pdf(
    font_data: &[u8],
    texts: &[(&str, WritingMode)],
) -> Result<Vec<PdfSubsetResult>, Box<dyn Error>> {
    let scope = ReadScope::new(font_data);
    let font_file = scope.read::<OpenTypeFont>()?;
    let provider = font_file.table_provider(0)?;
    
    let mut results = Vec::new();
    
    for (text, writing_mode) in texts {
        // Collect glyphs for this text
        let mut font = Font::new(provider.box_clone())?;
        let mut glyph_ids = vec![0];
        
        for ch in text.chars() {
            let (gid, _) = font.lookup_glyph_index(
                ch,
                MatchingPresentation::NotRequired,
                None,
            );
            if gid != 0 && !glyph_ids.contains(&gid) {
                glyph_ids.push(gid);
            }
        }
        
        // Create PDF context for this text's writing mode
        let pdf_context = PdfFontContext {
            cid_to_gid_map: None,
            max_cid: 0xFFFF,
            is_cid_font: true,
            writing_mode: writing_mode.clone(),
        };
        
        // Subset for this text
        let result = subset_for_pdf(&provider, &glyph_ids, &pdf_context)?;
        results.push(result);
    }
    
    Ok(results)
}

// Usage
let texts = vec![
    ("Hello World", WritingMode::Horizontal),
    ("こんにちは", WritingMode::Horizontal),
    ("中文竖排", WritingMode::Vertical),
];

let subsets = subset_for_multilingual_pdf(&font_data, &texts)?;
```

### Example 2: Debug and Fix Broken Subset

```rust
use allsorts::subset::{
    composite::update_composite_references,
    validation::{diagnose_subset_issues, debug_mapping},
};

fn fix_broken_subset(
    original_font: &[u8],
    broken_subset: &mut Vec<u8>,
    mapping: &HashMap<u16, u16>,
) -> Result<(), Box<dyn Error>> {
    // Diagnose issues
    let report = diagnose_subset_issues(
        original_font,
        broken_subset,
        mapping,
    );
    
    println!("Diagnostic Report:");
    println!("  Broken composites: {}", report.broken_composites.len());
    println!("  Missing components: {}", report.missing_components.len());
    println!("  Encoding issues: {}", report.encoding_issues.len());
    
    // Print debug mapping
    let debug_output = debug_mapping(mapping, "Problem Font");
    println!("\nGlyph Mapping:\n{}", debug_output);
    
    // Apply fixes
    if !report.broken_composites.is_empty() {
        println!("Fixing composite references...");
        let stats = update_composite_references(broken_subset, mapping)?;
        println!("  Fixed {} composites", stats.composites_updated);
        println!("  Updated {} references", stats.references_updated);
        
        if !stats.unmapped_references.is_empty() {
            println!("  Warning: {} references remain unmapped", 
                    stats.unmapped_references.len());
        }
    }
    
    // Apply other recommendations
    for recommendation in report.recommendations {
        println!("Recommendation: {}", recommendation);
    }
    
    Ok(())
}
```

## Performance Considerations

### Memory Usage

- **In-place updates**: `update_composite_references` modifies font data in-place, avoiding additional allocations
- **Lazy evaluation**: Validation utilities only parse what's necessary
- **Efficient data structures**: HashMap for O(1) glyph ID lookups

### Speed Optimizations

- **Binary patching**: Direct byte manipulation for composite updates
- **Parallel processing**: Multiple validation checks can run concurrently
- **Early termination**: Validation stops at first critical error

### Best Practices

1. **Reuse providers**: Create FontTableProvider once and reuse
2. **Batch operations**: Process multiple glyphs together
3. **Cache mappings**: Store and reuse glyph mappings when possible
4. **Validate early**: Check inputs before expensive operations

## Error Handling

### Error Types

All APIs use `SubsetError` for consistency:

```rust
pub enum SubsetError {
    /// Missing .notdef glyph
    NotDef,
    /// Invalid glyph ID
    InvalidGlyphId(u16),
    /// Parse error
    ParseError(ParseError),
    /// Write error
    WriteError(WriteError),
    /// Custom error
    Custom(String),
}
```

### Error Recovery Strategies

```rust
// Strategy 1: Fallback to standard subsetting
let result = subset_for_pdf(&provider, &glyphs, &context)
    .or_else(|_| {
        // Fallback to standard subsetting without PDF optimizations
        subset_detailed(&provider, &glyphs, &SubsetProfile::Minimal, 
                       CmapTarget::Unrestricted)
    })?;

// Strategy 2: Collect partial results
let mut successful_glyphs = vec![];
let mut failed_glyphs = vec![];

for glyph in requested_glyphs {
    match validate_single_glyph(glyph) {
        Ok(_) => successful_glyphs.push(glyph),
        Err(_) => failed_glyphs.push(glyph),
    }
}

// Strategy 3: Auto-fix known issues
match update_composite_references(&mut font_data, &mapping) {
    Ok(stats) => println!("Fixed {} composites", stats.composites_updated),
    Err(SubsetError::ParseError(_)) => {
        println!("Font might not have composite glyphs, skipping fix");
    },
    Err(e) => return Err(e),
}
```

## Testing

All features have comprehensive test coverage:

- **Composite updates**: 6 tests in `tests/composite_update.rs`
- **Result structures**: 4 tests in `tests/subset_result.rs`  
- **PDF subsetting**: 5 tests in `tests/pdf_subset.rs`
- **Validation**: 5 tests in `tests/validation.rs`
- **Builder API**: 6 tests in `tests/builder.rs`

Run tests with:
```bash
cargo test --test composite_update
cargo test --test subset_result
cargo test --test pdf_subset
cargo test --test validation
cargo test --test builder

# Or run all feature tests at once
cargo test --tests
```

## Version History

- **Phase 1** (Completed): Core features for composite handling and detailed results
- **Phase 2** (Completed): PDF-specific optimizations and validation tools
- **Phase 3** (Completed): Builder Pattern API for unified interface

## License

These features are part of the Allsorts library and follow the same Apache 2.0 license.