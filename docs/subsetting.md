# Font Subsetting Guide

## Table of Contents

1. [Overview](#overview)
2. [Quick Start](#quick-start)
3. [Core APIs](#core-apis)
   - [subset](#subset---basic-subsetting)
   - [subset_and_map](#subset_and_map---subsetting-with-glyph-mapping)
   - [subset_and_map_with_hint](#subset_and_map_with_hint---explicit-cid-control)
   - [subset_and_map_with_context](#subset_and_map_with_context---context-aware-cid-support-v0160)
4. [Advanced APIs](#advanced-apis)
   - [SubsetBuilder](#subsetbuilder---fluent-api)
   - [subset_detailed](#subset_detailed---comprehensive-results)
   - [subset_for_pdf](#subset_for_pdf---pdf-optimization)
5. [CID Font Support](#cid-font-support)
6. [Composite Glyph Handling](#composite-glyph-handling)
7. [Validation & Debugging](#validation--debugging)
8. [Common Scenarios](#common-scenarios)
9. [Performance Guide](#performance-guide)
10. [Troubleshooting](#troubleshooting)
11. [API Reference](#api-reference)

## Overview

Font subsetting is the process of extracting a subset of glyphs from a font file to create a smaller, optimized font containing only the characters you need. This is essential for:

- **Web fonts**: Reduce download size for faster page loads
- **PDF embedding**: Minimize document size while preserving text
- **Application bundling**: Include only required characters
- **Font optimization**: Remove unused glyphs and tables

Allsorts provides comprehensive subsetting capabilities with special support for:
- Glyph ID remapping and tracking
- CID-keyed fonts (common in PDFs)
- **CJK font encodings** (Chinese, Japanese, Korean) - [See CJK Support Guide](cjk_support.md)
- Composite glyph dependency resolution
- PDF-specific optimizations
- Validation and debugging tools

## Quick Start

### Simple Text Subsetting

```rust
use allsorts::subset::{subset, SubsetProfile, CmapTarget};
use allsorts::binary::read::ReadScope;
use allsorts::tables::OpenTypeFont;

// Load font
let font_data = std::fs::read("font.ttf")?;
let scope = ReadScope::new(&font_data);
let font_file = scope.read::<OpenTypeFont>()?;
let provider = font_file.table_provider(0)?;

// Subset for specific glyphs
let glyph_ids = vec![0, 42, 43, 100]; // Always start with 0 (.notdef)
let subset_data = subset(
    &provider,
    &glyph_ids,
    &SubsetProfile::Pdf,
    CmapTarget::Unicode,
)?;

// Save subset font
std::fs::write("subset.ttf", subset_data)?;
```

### Subsetting with Tracking

```rust
use allsorts::subset::{subset_and_map, SubsetResult};

// Subset and track how glyphs are remapped
let result = subset_and_map(
    &provider,
    &glyph_ids,
    &SubsetProfile::Pdf,
    CmapTarget::Unicode,
)?;

match result {
    SubsetResult::Simple { font_data, glyph_mapping } => {
        println!("Created {} byte subset", font_data.len());
        for (old_id, new_id) in &glyph_mapping {
            println!("Glyph {} -> {}", old_id, new_id);
        }
    }
    SubsetResult::Cid { font_data, glyph_mapping, cid_to_gid_map } => {
        println!("Created CID font with CIDToGIDMap");
    }
}
```

## Core APIs

### `subset` - Basic Subsetting

The simplest subsetting function that creates a subset font without tracking glyph remapping.

**Signature:**
```rust
pub fn subset(
    provider: &impl FontTableProvider,
    glyph_ids: &[u16],
    profile: &SubsetProfile,
    cmap_target: CmapTarget,
) -> Result<Vec<u8>, SubsetError>
```

**Parameters:**
- `provider`: Font table provider from parsed font
- `glyph_ids`: Array of glyph IDs to include (must start with 0)
- `profile`: Controls which tables to include
  - `SubsetProfile::Pdf`: Minimal tables for PDF
  - `SubsetProfile::Minimal`: Required tables only
  - `SubsetProfile::Custom(Vec<u32>)`: Custom table selection
- `cmap_target`: Character mapping format
  - `CmapTarget::Unrestricted`: Keep all mappings
  - `CmapTarget::Unicode`: Unicode only
  - `CmapTarget::MacRoman`: Mac Roman encoding

**Example:**
```rust
// Subset for PDF embedding
let subset_data = subset(
    &provider,
    &[0, 1, 2, 3], // Glyphs to include
    &SubsetProfile::Pdf,
    CmapTarget::Unicode,
)?;
```

### `subset_and_map` - Subsetting with Glyph Mapping

Enhanced subsetting that tracks how glyph IDs are remapped and handles CID fonts.

**Signature:**
```rust
pub fn subset_and_map(
    provider: &impl FontTableProvider,
    glyph_ids: &[u16],
    profile: &SubsetProfile,
    cmap_target: CmapTarget,
) -> Result<SubsetResult, SubsetError>
```

**Return Value:**
```rust
pub enum SubsetResult {
    /// Standard TrueType/OpenType font
    Simple {
        font_data: Vec<u8>,
        glyph_mapping: HashMap<u16, u16>, // old -> new
    },
    /// CID-keyed font (for PDFs)
    Cid {
        font_data: Vec<u8>,
        glyph_mapping: HashMap<u16, u16>,
        cid_to_gid_map: Vec<u8>, // Binary CIDToGIDMap
    },
}
```

**Example with Glyph Tracking:**
```rust
// Track character to glyph mapping
let mut char_to_old_gid = HashMap::new();
char_to_old_gid.insert('A', 42);
char_to_old_gid.insert('B', 43);

let result = subset_and_map(
    &provider,
    &[0, 42, 43],
    &SubsetProfile::Pdf,
    CmapTarget::Unicode,
)?;

// Update mappings with new IDs
match result {
    SubsetResult::Simple { glyph_mapping, .. } => {
        for (ch, old_gid) in &char_to_old_gid {
            let new_gid = glyph_mapping[old_gid];
            println!("{} was glyph {}, now {}", ch, old_gid, new_gid);
        }
    }
    _ => {}
}
```

### `subset_and_map_with_hint` - Explicit CID Control

Allows explicit specification of whether a font should be treated as CID.

**Signature:**
```rust
pub fn subset_and_map_with_hint(
    provider: &impl FontTableProvider,
    glyph_ids: &[u16],
    profile: &SubsetProfile,
    cmap_target: CmapTarget,
    force_cid: bool,
) -> Result<SubsetResult, SubsetError>
```

**When to Use:**
- You know from PDF structure that a font uses Identity-H encoding
- Auto-detection isn't triggering for your specific font
- You need explicit control over CID treatment

**Example:**
```rust
// Force CID treatment for a TrueType font used in PDF
let result = subset_and_map_with_hint(
    &provider,
    &glyph_ids,
    &SubsetProfile::Pdf,
    CmapTarget::Unicode,
    true, // Force CID treatment
)?;

if let SubsetResult::Cid { cid_to_gid_map, .. } = result {
    // Embed CIDToGIDMap in PDF
    pdf_font.set_cid_to_gid_map(cid_to_gid_map);
}
```

### `subset_and_map_with_context` - Context-Aware CID Support (v0.16.0+)

**New in v0.16.0:** Provides context-aware subsetting with correct CIDToGIDMap generation for Identity encodings, fixing the issue where characters render as '?' in PDFs.

**Signature:**
```rust
pub fn subset_and_map_with_context(
    provider: &impl FontTableProvider,
    glyph_ids: &[u16],
    profile: &SubsetProfile,
    cmap_target: CmapTarget,
    context: FontContext,
) -> Result<SubsetResult, SubsetError>
```

**Context Types:**
```rust
pub enum FontEncoding {
    /// Identity mapping where CID equals GID (Identity-H/V)
    Identity { vertical: bool },
}

pub enum FontContext {
    /// No context provided - use heuristics
    Unknown,
    /// PDF Type0 (CID) font with encoding
    PdfType0 { encoding: FontEncoding },
}
```

**When to Use:**
- You know the PDF encoding (e.g., Identity-H, Identity-V)
- You need correct CIDToGIDMap generation for Identity encodings
- You want to ensure proper character rendering in PDFs

**Example - Identity-H Encoding:**
```rust
use allsorts::subset::{subset_and_map_with_context, FontContext, FontEncoding};

// Specify Identity-H encoding for horizontal text
let context = FontContext::PdfType0 {
    encoding: FontEncoding::Identity { vertical: false },
};

let result = subset_and_map_with_context(
    &provider,
    &[0, 42, 43],
    &SubsetProfile::Pdf,
    CmapTarget::Unicode,
    context,
)?;

if let SubsetResult::Cid { cid_to_gid_map, glyph_mapping, .. } = result {
    // CIDToGIDMap is now correctly generated for Identity-H
    // For Identity encoding: CID == original GID
    // So CID 42 maps to glyph_mapping[42]
    pdf_font.set_cid_to_gid_map(cid_to_gid_map);
}
```

**Example - Identity-V Encoding:**
```rust
// Specify Identity-V encoding for vertical text
let context = FontContext::PdfType0 {
    encoding: FontEncoding::Identity { vertical: true },
};

let result = subset_and_map_with_context(
    &provider,
    &glyph_ids,
    &SubsetProfile::Pdf,
    CmapTarget::Unicode,
    context,
)?;
```

**Key Benefits:**
- **Fixes rendering issues**: Correctly generates CIDToGIDMap for Identity encodings
- **90% coverage**: Identity-H/V encodings are used in most modern PDFs
- **Backward compatible**: Unknown context falls back to existing behavior
- **Future-proof**: Extensible for additional encodings in future phases

### Phase 2: PDF Convenience APIs (v0.16.0+)

**New in v0.16.0:** Phase 2 adds convenience wrappers and enhanced APIs specifically designed for PDF font subsetting, building on top of the Phase 1 context-aware subsetting.

#### PDF-Specific Convenience Function

```rust
use allsorts::subset::pdf::{subset_and_map_for_pdf, PdfFontContext};

// Simple Identity-H subsetting
let context = PdfFontContext::identity_h();
let result = subset_and_map_for_pdf(&provider, &glyph_ids, context)?;

// Access enhanced results
println!("Font type: {:?}", result.font_type);
println!("Size reduction: {:.1}%", result.statistics.reduction_percentage);
println!("Subset glyphs: {}", result.statistics.subset_glyph_count);
```

#### PdfFontContext Constructors

```rust
// Quick constructors for common encodings
let context_h = PdfFontContext::identity_h();  // Identity-H (horizontal)
let context_v = PdfFontContext::identity_v();  // Identity-V (vertical)

// Create from PDF font dictionary
let context = PdfFontContext::from_pdf_dict("Identity-H", pdf_flags)?;

// Advanced configuration
let context = PdfFontContext::identity_h()
    .with_max_cid(255)
    .preserve_identity();
```

#### Builder Pattern API

```rust
use allsorts::subset::phase2::builder::subset_for_pdf;

// Fluent builder interface
let result = subset_for_pdf(&provider)
    .with_glyphs(&[0, 42, 43])
    .identity_h()
    .with_max_cid(255)
    .build()?;

// Chain multiple operations
let result = subset_for_pdf(&provider)
    .with_glyphs(&[0, 100, 200])
    .with_glyphs(&[300, 400])  // Adds more glyphs
    .identity_v()               // Vertical text
    .symbolic()                 // Mark as symbolic font
    .build()?;
```

#### Enhanced Statistics

```rust
// Get detailed subsetting statistics
let result = subset_and_map_for_pdf(&provider, &glyph_ids, context)?;

let stats = &result.statistics;
println!("Subsetting Statistics:");
println!("  Original glyphs: {}", stats.original_glyph_count);
println!("  Subset glyphs: {}", stats.subset_glyph_count);
println!("  Original size: {} bytes", stats.original_size_estimate);
println!("  Subset size: {} bytes", stats.subset_size);
println!("  CID map size: {} bytes", stats.cid_map_size);
println!("  Size reduction: {:.1}%", stats.reduction_percentage);
```

#### Font Type Detection

```rust
// Automatic font type detection
let result = subset_and_map_for_pdf(&provider, &glyph_ids, context)?;

match result.font_type {
    PdfFontType::Simple => println!("Simple font (Type 1/TrueType)"),
    PdfFontType::CidType0 => println!("CID Type 0 (CFF)"),
    PdfFontType::CidType2 => println!("CID Type 2 (TrueType)"),
}

// Check if CID font
if result.is_cid_font() {
    let cid_map = result.cid_to_gid_map().unwrap();
    pdf_font.set_cid_to_gid_map(cid_map);
}
```

#### Complete Phase 2 Example

```rust
use allsorts::subset::phase2::{
    pdf::{subset_and_map_for_pdf, PdfFontContext},
    builder::subset_for_pdf,
};

// Method 1: Direct API with context
fn subset_with_context(provider: &impl FontTableProvider) -> Result<(), SubsetError> {
    let context = PdfFontContext::identity_h()
        .with_max_cid(255);
    
    let result = subset_and_map_for_pdf(
        provider,
        &[0, 42, 43],
        context,
    )?;
    
    println!("Created {} byte subset", result.font_data.len());
    println!("Reduced size by {:.1}%", result.size_reduction());
    
    Ok(())
}

// Method 2: Builder pattern
fn subset_with_builder(provider: &impl FontTableProvider) -> Result<(), SubsetError> {
    let result = subset_for_pdf(provider)
        .with_glyphs(&[0, 42, 43])
        .identity_h()
        .preserve_identity()
        .build()?;
    
    // Access all the same information
    println!("Font type: {:?}", result.font_type);
    println!("Encoding: {:?}", result.encoding_used);
    
    Ok(())
}
```

**Phase 2 Key Features:**
- **Convenience wrappers**: Simpler API for common PDF use cases
- **Enhanced statistics**: Detailed information about subsetting operations
- **Font type detection**: Automatic identification of font types
- **Builder pattern**: Fluent interface for complex configurations
- **Better error messages**: Enhanced error variants for debugging
- **Phase 1 integration**: Internally uses the proven context-aware subsetting

### Phase 3: CJK Encoding Support (v0.16.1+)

**New in v0.16.1:** Phase 3 adds comprehensive support for Chinese, Japanese, and Korean (CJK) font encodings, enabling correct CIDToGIDMap generation for fonts with predefined CMap encodings beyond Identity-H/V. The CJK support is fully integrated into the main subsetting pipeline.

#### Supported CJK Encodings

```rust
use allsorts::subset::context::{FontEncoding, CJKLanguage, ChineseVariant};
use allsorts::subset::cjk::BuiltinCMapProvider;
use allsorts::subset::pdf::PdfFontContext;

// Chinese encoding example
let encoding = FontEncoding::from_pdf_name("GB-EUC-H").unwrap();
let cmap_provider = Box::new(BuiltinCMapProvider::new());

let context = PdfFontContext::from_pdf_dict("GB-EUC-H", 0)?
    .with_max_cid(8000)
    .with_cmap_provider(cmap_provider);

let result = subset_and_map_for_pdf(&provider, &glyph_ids, context)?;
```

**Supported Encodings:**
- **Chinese**: GB-EUC-H/V, GBK-EUC-H/V, CNS-EUC-H/V, B5pc-H/V, ETen-B5, HKscs-B5, UniGB/UniCNS
- **Japanese**: 90ms-RKSJ-H/V, 83pv-RKSJ-H/V, H, V, EUC-H/V, UniJIS-UTF16-H/V
- **Korean**: KSCms-UHC-H/V, KSC-EUC-H/V, UniKS-UTF16-H/V
- **Adobe Collections**: Adobe-GB1-*, Adobe-CNS1-*, Adobe-Japan1-*, Adobe-Korea1-*

See the [CJK Support Guide](cjk_support.md) for detailed documentation and examples.

### Phase 4: Detection & Auto-Configuration (v0.16.2+)

**New in v0.16.2:** Phase 4 adds intelligent encoding detection and auto-configuration capabilities, reducing the manual configuration burden for PDF font subsetting.

#### Automatic Encoding Detection

```rust
use allsorts::subset::auto::auto_subset_for_pdf;
use allsorts::subset::detection::{DetectionConfidence, PdfFontInfo};

// Automatic detection with minimal configuration
let result = auto_subset_for_pdf(&provider)
    .with_glyphs(&[0, 143, 159, 178])  // Sparse CID glyphs
    .build()?;

// Access detection information
println!("Detected encoding: {:?}", result.detection.encoding);
println!("Confidence: {:?}", result.detection.confidence);
println!("Reasoning: {:?}", result.detection.reasoning);
```

#### Detection with PDF Metadata

```rust
// Provide PDF font information for better detection
let pdf_info = PdfFontInfo {
    encoding_name: Some("Identity-H".to_string()),
    font_name: Some("ArialMT".to_string()),
    flags: 0x04,  // Symbolic font flag
    registry: Some("Adobe".to_string()),
    ordering: Some("Identity".to_string()),
    supplement: Some(0),
    to_unicode: None,
};

let result = auto_subset_for_pdf(&provider)
    .with_glyphs(&glyph_ids)
    .with_pdf_info(pdf_info)
    .build()?;

// Certain confidence when explicit encoding is provided
assert_eq!(result.confidence(), &DetectionConfidence::Certain);
```

#### Pattern-Based Detection

The system automatically detects encoding patterns from glyph IDs:

```rust
// CJK dense pattern (sequential high IDs)
let cjk_glyphs = vec![0, 8000, 8001, 8002, 8003];

// ASCII/Latin pattern
let ascii_glyphs = vec![0, 65, 66, 67, 68];  // A, B, C, D

// Identity-H pattern (sparse CIDs)
let identity_glyphs = vec![0, 143, 159, 178];

// Auto-detection will identify the correct pattern
let result = auto_subset_for_pdf(&provider)
    .with_glyphs(&cjk_glyphs)
    .build()?;

// Detected as CJK encoding with high confidence
```

#### Confidence Levels

```rust
use allsorts::subset::detection::DetectionConfidence;

// Set minimum confidence threshold
let result = auto_subset_for_pdf(&provider)
    .with_glyphs(&ambiguous_glyphs)
    .min_confidence(DetectionConfidence::High)
    .build();

// Will fail if detection confidence is below High
if let Err(e) = result {
    println!("Detection confidence too low: {}", e);
}
```

**Confidence Levels:**
- **Certain**: Explicit encoding specified in PDF metadata
- **High**: Strong pattern match (90%+ confidence)
- **Medium**: Some patterns match (70-90% confidence)
- **Low**: Guessing based on heuristics (<70% confidence)

#### Manual Override

```rust
use allsorts::subset::context::{FontEncoding, CJKLanguage, JapaneseVariant};

// Override automatic detection with specific encoding
let encoding = FontEncoding::CJK {
    language: CJKLanguage::Japanese(JapaneseVariant::Unicode),
    encoding_name: "UniJIS-UTF16-H".to_string(),
    vertical: false,
    requires_cmap_data: false,
};

let result = auto_subset_for_pdf(&provider)
    .with_glyphs(&glyph_ids)
    .override_encoding(encoding)
    .build()?;

// Always returns Certain confidence for manual override
```

#### Statistical Analysis

The detection system analyzes glyph statistics:

```rust
use allsorts::subset::detection::GlyphStatistics;

// Analyze glyph distribution
let stats = GlyphStatistics::from_glyph_ids(&glyph_ids);

println!("Glyph Statistics:");
println!("  Min GID: {}", stats.min_gid);
println!("  Max GID: {}", stats.max_gid);
println!("  Density: {:.2}", stats.density);
println!("  Has CJK: {}", stats.has_cjk_range);
println!("  Has Kana: {}", stats.has_kana_range);
println!("  Has ASCII: {}", stats.has_ascii_range);
```

#### Complete Auto-Configuration Example

```rust
use allsorts::subset::auto::auto_subset_for_pdf;
use allsorts::subset::detection::{EncodingDetector, PdfFontInfo};

fn subset_pdf_font_auto(
    provider: &dyn FontTableProvider,
    glyph_ids: &[u16],
    pdf_metadata: Option<PdfFontInfo>,
) -> Result<Vec<u8>, SubsetError> {
    // Build with auto-detection
    let mut builder = auto_subset_for_pdf(provider)
        .with_glyphs(glyph_ids);
    
    // Add PDF metadata if available
    if let Some(info) = pdf_metadata {
        builder = builder.with_pdf_info(info);
    }
    
    // Build and get results
    let result = builder
        .min_confidence(DetectionConfidence::Medium)
        .build()?;
    
    // Report detection results
    println!("Auto-detection Results:");
    println!("  Encoding: {:?}", result.detection.encoding);
    println!("  Confidence: {:?}", result.detection.confidence);
    
    for reason in result.reasoning() {
        println!("  - {}", reason);
    }
    
    if !result.alternatives().is_empty() {
        println!("  Alternative encodings:");
        for alt in result.alternatives() {
            println!("    - {:?}", alt);
        }
    }
    
    Ok(result.subset_result.font_data)
}
```

**Key Features:**
- **Zero-configuration API**: Works automatically for most cases
- **Pattern matching**: Identifies Identity-H, CJK, ASCII, and Symbol patterns
- **Statistical analysis**: Analyzes glyph distributions for better detection
- **Caching**: Avoids recomputation for identical inputs
- **Confidence levels**: Provides transparency about detection certainty
- **Manual override**: Allows explicit encoding specification when needed

## Advanced APIs

### SubsetBuilder - Fluent API

A builder pattern API that provides an intuitive interface for complex subsetting operations.

**Creating and Using the Builder:**

```rust
use allsorts::subset::builder::{SubsetBuilder, ValidationLevel};

let result = SubsetBuilder::new(&provider)
    .with_characters("Hello, World!")?  // Add glyphs from text
    .with_glyphs(&[100, 101, 102])      // Add specific glyph IDs
    .for_pdf(255)                        // Configure for PDF with max CID
    .validation_level(ValidationLevel::Strict)
    .build()?;

println!("Created {} byte subset", result.data.len());
println!("Size reduction: {:.1}%", result.stats.size_reduction_percent);
```

**Builder Methods:**

| Method | Purpose | Example |
|--------|---------|---------|
| `with_characters(text)` | Add glyphs from Unicode text | `.with_characters("Hello 世界")` |
| `with_glyphs(ids)` | Add specific glyph IDs | `.with_glyphs(&[0, 42, 100])` |
| `for_pdf(max_cid)` | Configure for PDF embedding | `.for_pdf(255)` |
| `with_cid_map(map)` | Set custom CID mapping | `.with_cid_map(&[0, 42, 43])` |
| `fix_composites(bool)` | Fix composite references | `.fix_composites(true)` |
| `validation_level(level)` | Set validation strictness | `.validation_level(ValidationLevel::Strict)` |
| `with_profile(profile)` | Select subset profile | `.with_profile(SubsetProfile::Web)` |
| `with_cmap_target(target)` | Set character map format | `.with_cmap_target(CmapTarget::Unicode)` |

**Validation Levels:**

```rust
pub enum ValidationLevel {
    None,     // No validation (fastest)
    Basic,    // Check .notdef is present
    Standard, // Verify glyph IDs exist (default)
    Strict,   // Check duplicates and dependencies
}
```

**Complete Example:**

```rust
// Multi-language PDF subset with validation
fn create_pdf_subset(
    provider: &dyn FontTableProvider,
    languages: &[(&str, &str)], // (language_code, text)
) -> Result<Vec<u8>, SubsetError> {
    let mut builder = SubsetBuilder::new(provider);
    
    // Add text from each language
    for (lang, text) in languages {
        println!("Adding {} text: {}", lang, text);
        builder = builder.with_characters(text)?;
    }
    
    // Configure for PDF and build
    let result = builder
        .for_pdf(65535)  // Support full Unicode range
        .validation_level(ValidationLevel::Strict)
        .build()?;
    
    // Report statistics
    println!("Subset statistics:");
    println!("  Original: {} bytes", result.original_info.size);
    println!("  Subset: {} bytes", result.subset_info.size);
    println!("  Reduction: {:.1}%", result.stats.size_reduction_percent);
    println!("  Glyphs: {}", result.glyph_mapping.len());
    
    Ok(result.data)
}

// Usage
let languages = vec![
    ("en", "Hello, World!"),
    ("zh", "你好，世界！"),
    ("ja", "こんにちは、世界！"),
    ("ar", "مرحبا بالعالم!"),
];

let subset = create_pdf_subset(&provider, &languages)?;
```

### `subset_detailed` - Comprehensive Results

Provides detailed information about the subsetting operation including statistics and metadata.

**Signature:**
```rust
pub fn subset_detailed(
    provider: &impl FontTableProvider,
    glyphs: &[u16],
    profile: &SubsetProfile,
    cmap_target: CmapTarget,
) -> Result<SubsetResult, SubsetError>
```

**Return Structure:**
```rust
pub struct SubsetResult {
    pub data: Vec<u8>,                        // Subset font data
    pub glyph_mapping: HashMap<u16, u16>,     // Old -> new mapping
    pub reverse_mapping: HashMap<u16, u16>,   // New -> old mapping
    pub added_glyphs: Vec<u16>,               // Dependencies added
    pub missing_glyphs: Vec<u16>,             // Requested but not found
    pub original_info: FontInfo,              // Original font metadata
    pub subset_info: FontInfo,                // Subset font metadata
    pub stats: SubsetStats,                   // Statistics
}

pub struct SubsetStats {
    pub size_reduction_bytes: i64,
    pub size_reduction_percent: f32,
    pub composite_glyphs: usize,
    pub simple_glyphs: usize,
    pub removed_tables: Vec<String>,
}
```

**Example:**
```rust
let result = subset_detailed(
    &provider,
    &[0, 42, 43, 100],
    &SubsetProfile::Pdf,
    CmapTarget::Unicode,
)?;

// Analyze subsetting impact
println!("Subsetting Analysis:");
println!("  Original: {} glyphs, {} bytes", 
         result.original_info.glyph_count,
         result.original_info.size);
println!("  Subset: {} glyphs, {} bytes",
         result.subset_info.glyph_count,
         result.subset_info.size);
println!("  Size saved: {} bytes ({:.1}%)",
         result.stats.size_reduction_bytes,
         result.stats.size_reduction_percent);

// Check for issues
if !result.missing_glyphs.is_empty() {
    eprintln!("Warning: {} glyphs not found", 
              result.missing_glyphs.len());
}

if !result.added_glyphs.is_empty() {
    println!("Added {} dependency glyphs", 
             result.added_glyphs.len());
}
```

### `subset_for_pdf` - PDF Optimization

Specialized subsetting for PDF embedding with CID font support and validation.

**Signature:**
```rust
pub fn subset_for_pdf(
    provider: &impl FontTableProvider,
    glyph_ids: &[u16],
    pdf_context: &PdfFontContext,
) -> Result<PdfSubsetResult, SubsetError>
```

**Data Structures:**
```rust
pub struct PdfFontContext {
    pub cid_to_gid_map: Option<Vec<u16>>,
    pub max_cid: u16,
    pub is_cid_font: bool,
    pub writing_mode: WritingMode,
}

pub struct PdfSubsetResult {
    pub font_data: Vec<u8>,
    pub glyph_mapping: HashMap<u16, u16>,
    pub cid_to_gid_map: Vec<u8>,
    pub validation: ValidationResult,
    pub warnings: Vec<PdfWarning>,
}
```

**Example:**
```rust
use allsorts::subset::pdf::{subset_for_pdf, PdfFontContext, WritingMode};

// Configure for CID font with horizontal text
let pdf_context = PdfFontContext {
    cid_to_gid_map: None,
    max_cid: 255,
    is_cid_font: true,
    writing_mode: WritingMode::Horizontal,
};

let result = subset_for_pdf(
    &provider,
    &[0, 42, 43, 100],
    &pdf_context,
)?;

// Handle warnings
for warning in &result.warnings {
    match warning {
        PdfWarning::MissingGlyph { cid } => {
            eprintln!("CID {} references missing glyph", cid);
        }
        PdfWarning::BrokenComposite { glyph, component } => {
            eprintln!("Glyph {} has broken reference to {}", 
                     glyph, component);
        }
        _ => {}
    }
}

// Embed in PDF
pdf_font_descriptor.embed_font(result.font_data);
pdf_font_descriptor.set_cid_to_gid_map(result.cid_to_gid_map);
```

## CID Font Support

### Understanding CID Fonts

CID (Character Identifier) fonts are essential for PDF documents, especially those containing Asian text. They use a two-level mapping:

1. **Character Code → CID**: Handled by Encoding (e.g., Identity-H)
2. **CID → GID**: Handled by CIDToGIDMap
3. **GID → Glyph**: Handled by font tables

### Automatic CID Detection

Allsorts automatically detects CID fonts based on:

1. **CFF Structure**: PostScript fonts with CID tables
2. **Sparse Glyph IDs**: Glyphs like 143, 159, 178 for special characters
3. **Dense Sequential**: Large ranges of sequential glyphs (CJK fonts)
4. **PDF Context**: When using PDF subset profile

```rust
// Automatic detection example
let glyph_ids = vec![0, 3, 143, 178]; // Sparse IDs trigger CID detection

let result = subset_and_map(
    &provider,
    &glyph_ids,
    &SubsetProfile::Pdf,
    CmapTarget::Unicode,
)?;

if let SubsetResult::Cid { cid_to_gid_map, .. } = result {
    println!("CID font detected!");
    println!("CIDToGIDMap size: {} bytes", cid_to_gid_map.len());
}
```

### CIDToGIDMap Format

The CIDToGIDMap is a binary array mapping CIDs to GIDs:

```rust
// Reading CIDToGIDMap
fn get_gid_for_cid(cid: u16, cid_to_gid_map: &[u8]) -> u16 {
    let offset = (cid as usize) * 2;
    if offset + 1 < cid_to_gid_map.len() {
        u16::from_be_bytes([
            cid_to_gid_map[offset],
            cid_to_gid_map[offset + 1],
        ])
    } else {
        0 // .notdef
    }
}

// Example: CID 45 maps to GID 3
let gid = get_gid_for_cid(45, &cid_to_gid_map);
```

### Working with Different CID Types

```rust
// Japanese font with JIS encoding
let result = subset_and_map_with_hint(
    &provider,
    &jis_glyph_ids,
    &SubsetProfile::Pdf,
    CmapTarget::Unicode,
    true, // Force CID for JIS fonts
)?;

// Chinese font with GB encoding
let result = subset_and_map_with_hint(
    &provider,
    &gb_glyph_ids,
    &SubsetProfile::Pdf,
    CmapTarget::Unicode,
    true, // Force CID for GB fonts
)?;
```

## Composite Glyph Handling

### Understanding Composite Glyphs

Composite glyphs are made of references to other glyphs. Common examples:
- Accented characters (é = e + ´)
- Complex ligatures
- CJK ideographs with components

### Automatic Dependency Resolution

Allsorts automatically includes component glyphs:

```rust
// Request only 'é' (glyph 245)
let glyph_ids = vec![0, 245];

let result = subset_and_map(&provider, &glyph_ids, 
                           &SubsetProfile::Pdf, 
                           CmapTarget::Unicode)?;

// Mapping includes dependencies
// {
//   0: 0,    // .notdef
//   245: 1,  // é (requested)
//   101: 2,  // e (component)
//   150: 3   // ´ (component)
// }
```

### Fixing Composite References

After subsetting, composite glyph references must be updated:

```rust
use allsorts::subset::composite::update_composite_references;

// Fix references after subsetting
let stats = update_composite_references(&mut font_data, &glyph_mapping)?;

println!("Fixed {} composite glyphs", stats.composites_updated);
println!("Updated {} references", stats.references_updated);

if !stats.unmapped_references.is_empty() {
    eprintln!("Warning: {} references couldn't be mapped",
              stats.unmapped_references.len());
}
```

### Builder API Auto-Fix

The builder API can automatically fix composites:

```rust
let result = SubsetBuilder::new(&provider)
    .with_glyphs(&[0, 245])
    .fix_composites(true)  // Enable auto-fix
    .build()?;

// Or use PDF mode which enables it automatically
let result = SubsetBuilder::new(&provider)
    .with_glyphs(&[0, 245])
    .for_pdf(255)  // Auto-enables composite fixing
    .build()?;
```

## Validation & Debugging

### Validate Glyph Mapping

```rust
use allsorts::subset::validation::validate_mapping_coverage;

let report = validate_mapping_coverage(
    &glyph_mapping,
    &required_glyphs,
    Some(&cid_to_gid_map),
)?;

if !report.is_valid {
    eprintln!("Validation failed!");
    eprintln!("Unmapped glyphs: {:?}", report.unmapped_glyphs);
    
    for suggestion in report.suggestions {
        eprintln!("Suggestion: {}", suggestion);
    }
}
```

### Debug Mapping Output

```rust
use allsorts::subset::validation::debug_mapping;

let debug_output = debug_mapping(&glyph_mapping, "MyFont");
println!("{}", debug_output);

// Output:
// Font: MyFont
// Mapping (4 glyphs):
//   GID 0 -> 0 (.notdef)
//   GID 42 -> 1
//   GID 43 -> 2
//   GID 100 -> 3
```

### Diagnose Subset Issues

```rust
use allsorts::subset::validation::diagnose_subset_issues;

let report = diagnose_subset_issues(
    &original_font_data,
    &subset_font_data,
    &glyph_mapping,
);

if !report.broken_composites.is_empty() {
    println!("Found {} broken composites", 
             report.broken_composites.len());
    
    for (glyph, component) in &report.missing_components {
        println!("  Glyph {} missing component {}", 
                 glyph, component);
    }
}

for recommendation in report.recommendations {
    println!("Recommendation: {}", recommendation);
}
```

## Common Scenarios

### Web Font Optimization

```rust
fn optimize_web_font(
    font_path: &str,
    text: &str,
) -> Result<Vec<u8>, Box<dyn Error>> {
    // Load font
    let font_data = std::fs::read(font_path)?;
    let scope = ReadScope::new(&font_data);
    let font_file = scope.read::<OpenTypeFont>()?;
    let provider = font_file.table_provider(0)?;
    
    // Use builder for easy text subsetting
    let result = SubsetBuilder::new(&provider)
        .with_characters(text)?
        .with_profile(SubsetProfile::Web)
        .with_cmap_target(CmapTarget::Unicode)
        .build()?;
    
    println!("Web font optimization:");
    println!("  Original: {} KB", result.original_info.size / 1024);
    println!("  Optimized: {} KB", result.data.len() / 1024);
    println!("  Saved: {:.1}%", result.stats.size_reduction_percent);
    
    Ok(result.data)
}
```

### PDF Font Embedding

```rust
fn embed_font_in_pdf(
    provider: &dyn FontTableProvider,
    used_characters: &str,
) -> Result<PdfEmbeddedFont, Box<dyn Error>> {
    // Build subset for PDF
    let result = SubsetBuilder::new(provider)
        .with_characters(used_characters)?
        .for_pdf(255)  // Support basic CIDs
        .validation_level(ValidationLevel::Strict)
        .build()?;
    
    // Check if CID font
    let (is_cid, cid_map) = match result {
        SubsetResult::Cid { cid_to_gid_map, .. } => {
            (true, Some(cid_to_gid_map))
        }
        _ => (false, None)
    };
    
    Ok(PdfEmbeddedFont {
        data: result.data,
        is_cid_font: is_cid,
        cid_to_gid_map: cid_map,
        glyph_mapping: result.glyph_mapping,
    })
}
```

### Multi-Language Document

```rust
fn create_multilingual_subset(
    provider: &dyn FontTableProvider,
    documents: Vec<(String, String)>, // (language, text)
) -> Result<Vec<u8>, Box<dyn Error>> {
    let mut builder = SubsetBuilder::new(provider);
    
    // Collect all unique characters
    let mut all_text = String::new();
    for (lang, text) in &documents {
        println!("Processing {} text: {} chars", 
                 lang, text.chars().count());
        all_text.push_str(text);
    }
    
    // Remove duplicates by converting to set and back
    let unique_chars: HashSet<char> = all_text.chars().collect();
    let unique_text: String = unique_chars.into_iter().collect();
    
    // Build subset
    let result = builder
        .with_characters(&unique_text)?
        .for_pdf(65535)  // Full Unicode range
        .build()?;
    
    println!("Created multilingual subset:");
    println!("  Languages: {}", documents.len());
    println!("  Unique characters: {}", unique_text.chars().count());
    println!("  Glyphs in subset: {}", result.glyph_mapping.len());
    println!("  Size: {} KB", result.data.len() / 1024);
    
    Ok(result.data)
}
```

### Font Analysis Before Subsetting

```rust
fn analyze_font_for_subsetting(
    font_path: &str,
    sample_text: &str,
) -> Result<(), Box<dyn Error>> {
    let font_data = std::fs::read(font_path)?;
    let scope = ReadScope::new(&font_data);
    let font_file = scope.read::<OpenTypeFont>()?;
    let provider = font_file.table_provider(0)?;
    
    // Get font metadata
    let mut font = Font::new(Box::new(provider.clone()))?;
    
    // Map characters to glyphs
    let mut glyph_ids = vec![0];
    let mut missing_chars = Vec::new();
    
    for ch in sample_text.chars() {
        let (gid, _) = font.lookup_glyph_index(
            ch,
            MatchingPresentation::NotRequired,
            None,
        );
        
        if gid != 0 {
            if !glyph_ids.contains(&gid) {
                glyph_ids.push(gid);
            }
        } else {
            missing_chars.push(ch);
        }
    }
    
    println!("Font Analysis:");
    println!("  Total glyphs in font: {}", font.num_glyphs());
    println!("  Characters requested: {}", sample_text.chars().count());
    println!("  Glyphs needed: {}", glyph_ids.len());
    println!("  Missing characters: {:?}", missing_chars);
    
    // Estimate subset size
    let subset_ratio = glyph_ids.len() as f32 / font.num_glyphs() as f32;
    let estimated_size = (font_data.len() as f32 * subset_ratio) as usize;
    
    println!("  Estimated subset size: {} KB ({:.1}% of original)",
             estimated_size / 1024,
             subset_ratio * 100.0);
    
    Ok(())
}
```

## Performance Guide

### Memory Optimization

```rust
// Reuse providers for multiple operations
let provider = font_file.table_provider(0)?;

// Process multiple subsets
for text in texts {
    let result = SubsetBuilder::new(&provider)  // Reuse provider
        .with_characters(text)?
        .build()?;
    // Process result...
}
```

### Batch Processing

```rust
fn batch_subset_fonts(
    font_paths: Vec<&str>,
    text: &str,
) -> Result<Vec<Vec<u8>>, Box<dyn Error>> {
    let mut results = Vec::new();
    
    for path in font_paths {
        let font_data = std::fs::read(path)?;
        let scope = ReadScope::new(&font_data);
        let font_file = scope.read::<OpenTypeFont>()?;
        let provider = font_file.table_provider(0)?;
        
        let result = SubsetBuilder::new(&provider)
            .with_characters(text)?
            .validation_level(ValidationLevel::Basic) // Faster
            .build()?;
        
        results.push(result.data);
    }
    
    Ok(results)
}
```

### Performance Benchmarks

| Operation | 10 glyphs | 100 glyphs | 1000 glyphs |
|-----------|-----------|------------|-------------|
| Basic subset | ~1ms | ~5ms | ~40ms |
| With mapping | ~1ms | ~5ms | ~41ms |
| CID detection | +0.1ms | +0.2ms | +0.5ms |
| Composite fix | +0.5ms | +2ms | +10ms |
| Validation | +0.2ms | +1ms | +5ms |

### Optimization Tips

1. **Use appropriate validation level**
   ```rust
   // Fast, minimal validation
   .validation_level(ValidationLevel::Basic)
   
   // Thorough validation (slower)
   .validation_level(ValidationLevel::Strict)
   ```

2. **Pre-calculate glyph IDs**
   ```rust
   // Cache character to glyph mappings
   let mut char_to_glyph = HashMap::new();
   // ... populate cache once ...
   
   // Reuse for multiple subsets
   let glyph_ids: Vec<u16> = text.chars()
       .filter_map(|ch| char_to_glyph.get(&ch))
       .copied()
       .collect();
   ```

3. **Choose minimal profiles**
   ```rust
   // Only include necessary tables
   SubsetProfile::Minimal  // Fastest
   SubsetProfile::Pdf      // PDF-optimized
   SubsetProfile::Web      // Includes layout tables
   ```

## Troubleshooting

### Common Issues and Solutions

#### "NotDef" Error
**Problem:** `SubsetError::NotDef`
**Solution:** Always include glyph 0 as the first element:
```rust
let mut glyph_ids = vec![0];  // Start with .notdef
glyph_ids.extend(your_glyphs);
```

#### Characters Rendering as Boxes
**Problem:** Characters show as '?' or □ in PDF
**Solution:** Ensure CID fonts are properly detected:
```rust
// Use explicit hint if auto-detection fails
let result = subset_and_map_with_hint(
    &provider, &glyph_ids,
    &SubsetProfile::Pdf, CmapTarget::Unicode,
    true,  // Force CID treatment
)?;
```

#### Missing Composite Components
**Problem:** Accented characters broken
**Solution:** Enable composite fixing:
```rust
SubsetBuilder::new(&provider)
    .fix_composites(true)
    // or
    .for_pdf(255)  // Auto-enables fixing
```

#### Parse(BadIndex) with CFF Fonts
**Problem:** Error with glyph IDs >= 225
**Solution:** Update to version 0.16.1 or later (bug fixed)

#### Non-Sequential New IDs
**Problem:** Mapping has gaps in new IDs
**Solution:** Remove duplicates from input:
```rust
let glyph_ids: Vec<u16> = requested_glyphs
    .into_iter()
    .collect::<HashSet<_>>()  // Remove duplicates
    .into_iter()
    .collect();
```

### Debugging Techniques

1. **Enable verbose output**
   ```rust
   let debug_info = debug_mapping(&mapping, "Problem Font");
   eprintln!("{}", debug_info);
   ```

2. **Validate before and after**
   ```rust
   // Validate input
   let report = validate_mapping_coverage(
       &glyph_mapping,
       &original_glyphs,
       None,
   )?;
   
   // Diagnose issues
   let diagnosis = diagnose_subset_issues(
       &original_font,
       &subset_font,
       &glyph_mapping,
   );
   ```

3. **Compare with reference implementation**
   ```rust
   // Test with known-good font
   let test_font = include_bytes!("test-font.ttf");
   // Compare results...
   ```

## API Reference

### Error Types

```rust
pub enum SubsetError {
    /// Missing .notdef glyph at position 0
    NotDef,
    /// Font parsing error
    Parse(ParseError),
    /// Font writing error  
    Write(WriteError),
    /// Too many glyphs for format
    TooManyGlyphs,
    /// CFF font processing error
    CFF(CFFError),
    /// Invalid font count in CFF
    InvalidFontCount,
}
```

### Subset Profiles

```rust
pub enum SubsetProfile {
    /// Minimal tables for PDF embedding
    Pdf,
    /// Only required OpenType tables
    Minimal,
    /// Custom table selection
    Custom(Vec<u32>),
}
```

### Character Map Targets

```rust
pub enum CmapTarget {
    /// Keep all character mappings
    Unrestricted,
    /// Only Unicode mappings
    Unicode,
    /// Mac Roman encoding only
    MacRoman,
}
```

### Complete Function List

| Function | Module | Purpose |
|----------|--------|---------|
| `subset` | `subset` | Basic subsetting without mapping |
| `subset_and_map` | `subset` | Subsetting with glyph ID tracking |
| `subset_and_map_with_hint` | `subset` | Subsetting with explicit CID control |
| `subset_and_map_with_context` | `subset` | Context-aware subsetting with correct Identity encoding support |
| `subset_detailed` | `subset::result` | Detailed subsetting with statistics |
| `subset_for_pdf` | `subset::pdf` | PDF-optimized subsetting |
| `SubsetBuilder::new` | `subset::builder` | Fluent builder API |
| `update_composite_references` | `subset::composite` | Fix composite glyph references |
| `validate_mapping_coverage` | `subset::validation` | Validate glyph mappings |
| `diagnose_subset_issues` | `subset::validation` | Diagnose subset problems |
| `debug_mapping` | `subset::validation` | Debug output for mappings |

---

*For more examples and advanced usage, see the [test suite](../tests/) and [API documentation](https://docs.rs/allsorts/).*