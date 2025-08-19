# Technical Implementation Plan: PDF-Specific Font Subsetting Features for Allsorts (TDD Approach)

## Executive Summary

This plan details the implementation of PDF-specific font subsetting features for the Allsorts library, building upon the existing `subset_and_map` functionality. The implementation will be delivered in three phases using Test-Driven Development (TDD), prioritizing the most critical features first to provide immediate value while maintaining backward compatibility.

## TDD Methodology

This implementation follows strict TDD practices:
1. **Write tests first**: All tests are written before implementation
2. **Red-Green-Refactor**: Tests must fail initially, then pass with minimal code
3. **Incremental development**: Each feature is built test-by-test
4. **Continuous validation**: Run `cargo test` after each change

## Architecture Overview

### Current State
- **Core subsetting**: `subset.rs` contains the main subsetting logic
- **Glyph mapping**: `subset_and_map` returns old-to-new GID mappings
- **Font formats**: Supports TTF (glyf), CFF, and CFF2
- **Composite handling**: `glyf/subset.rs` recursively includes composite dependencies

### Proposed Architecture
```
src/
├── subset.rs                  # Existing core subsetting
├── subset/
│   ├── mod.rs                # Module exports
│   ├── pdf.rs                # PDF-specific features (NEW)
│   ├── composite.rs          # Composite reference updater (NEW)
│   ├── result.rs             # Enhanced result structures (NEW)
│   ├── builder.rs            # Builder pattern API (NEW)
│   └── validation.rs         # Validation utilities (NEW)
```

## Pre-Implementation Phase: Test Infrastructure Setup

### Step 1: Verify Test Health
```bash
# Run existing tests to ensure clean baseline
cargo test
# All tests must pass before proceeding
```

### Step 2: Create Test Module Structure
```bash
# Create test files for new features
touch tests/pdf_subset.rs
touch tests/composite_update.rs
touch tests/subset_result.rs
```

## Phase 1: Core Features (Week 1)

### Feature 1.1: Composite Glyph Reference Updater

#### Step 1: Define API Signatures (No Implementation)

File: `src/subset/composite.rs`

```rust
// Only signatures and documentation - no implementation yet!
use std::collections::HashMap;
use crate::error::SubsetError;

/// Statistics about composite reference updates
#[derive(Debug, Clone, Default, PartialEq)]
pub struct UpdateStats {
    pub composites_updated: usize,
    pub references_updated: usize,
    pub unmapped_references: Vec<u16>,
    pub cff_subroutines_updated: bool,
}

/// Updates composite glyph references in font data based on glyph ID mapping
/// 
/// # Arguments
/// * `font_data` - Mutable font data to update in-place
/// * `mapping` - Old to new glyph ID mapping
/// 
/// # Returns
/// Statistics about the update operation
pub fn update_composite_references(
    _font_data: &mut [u8],
    _mapping: &HashMap<u16, u16>,
) -> Result<UpdateStats, SubsetError> {
    todo!("Implementation will come after tests")
}
```

#### Step 2: Write Unit Tests (TDD - Red Phase)

File: `tests/composite_update.rs`

```rust
mod common;

use std::collections::HashMap;
use allsorts::subset::composite::{update_composite_references, UpdateStats};
use allsorts::error::SubsetError;

#[test]
fn test_update_stats_default() {
    // Test that UpdateStats has sensible defaults
    let stats = UpdateStats::default();
    assert_eq!(stats.composites_updated, 0);
    assert_eq!(stats.references_updated, 0);
    assert!(stats.unmapped_references.is_empty());
    assert!(!stats.cff_subroutines_updated);
}

#[test]
fn test_composite_reference_update_empty_mapping() {
    // Test with empty mapping
    let mut font_data = common::read_fixture_font("opentype/Klei.otf");
    let mapping = HashMap::new();
    
    let result = update_composite_references(&mut font_data, &mapping);
    assert!(result.is_ok());
    
    let stats = result.unwrap();
    assert_eq!(stats.composites_updated, 0);
}

#[test]
fn test_composite_reference_update_ttf() {
    // Test updating composite references in TrueType font
    let mut font_data = common::read_fixture_font("opentype/composite_glyphs.ttf");
    let mapping = HashMap::from([
        (0, 0),    // .notdef
        (100, 1),  // component glyph
        (200, 2),  // composite using 100
    ]);
    
    let result = update_composite_references(&mut font_data, &mapping);
    assert!(result.is_ok());
    
    let stats = result.unwrap();
    assert!(stats.composites_updated > 0);
    assert!(stats.references_updated > 0);
    assert!(stats.unmapped_references.is_empty());
}

#[test]
fn test_composite_reference_unmapped() {
    // Test handling of unmapped references
    let mut font_data = common::read_fixture_font("opentype/composite_glyphs.ttf");
    let mapping = HashMap::from([
        (0, 0),    // .notdef only
    ]);
    
    let result = update_composite_references(&mut font_data, &mapping);
    assert!(result.is_ok());
    
    let stats = result.unwrap();
    assert!(!stats.unmapped_references.is_empty());
}

#[test]
fn test_composite_reference_cff() {
    // Test CFF font handling
    let mut font_data = common::read_fixture_font("opentype/Klei.otf");
    let mapping = HashMap::from([
        (0, 0),
        (1, 1),
        (2, 2),
    ]);
    
    let result = update_composite_references(&mut font_data, &mapping);
    assert!(result.is_ok());
    
    let stats = result.unwrap();
    // CFF fonts have subroutines instead of composites
    assert!(stats.cff_subroutines_updated || stats.composites_updated == 0);
}

#[test]
#[should_panic(expected = "UnknownFormat")]
fn test_composite_reference_invalid_format() {
    // Test with invalid font data
    let mut font_data = vec![0; 100];
    let mapping = HashMap::new();
    
    update_composite_references(&mut font_data, &mapping).unwrap();
}
```

#### Step 3: Run Tests to Verify Failure (TDD - Red Phase)

```bash
# Run the new tests - they should fail since implementation is not done
cargo test composite_update
# Expected: All tests fail with "not yet implemented" panic
```

#### Step 4: Implement Minimal Code (TDD - Green Phase)

File: `src/subset/composite.rs`

```rust
use std::collections::HashMap;
use crate::binary::read::ReadScope;
use crate::binary::write::{WriteBinary, WriteBuffer};
use crate::error::{ParseError, WriteError};
use crate::tables::glyf::{GlyfTable, CompositeGlyph, GlyfRecord};
use crate::tag;

#[derive(Debug, Clone, Default, PartialEq)]
pub struct UpdateStats {
    pub composites_updated: usize,
    pub references_updated: usize,
    pub unmapped_references: Vec<u16>,
    pub cff_subroutines_updated: bool,
}

pub fn update_composite_references(
    font_data: &mut [u8],
    mapping: &HashMap<u16, u16>,
) -> Result<UpdateStats, SubsetError> {
    // Implementation added after tests were written
    let scope = ReadScope::new(font_data);
    let sfnt = scope.read::<SfntHeader>()?;
    
    if let Some(glyf_record) = sfnt.find_table(tag::GLYF) {
        update_ttf_composites(font_data, glyf_record, mapping)
    } else if let Some(cff_record) = sfnt.find_table(tag::CFF) {
        update_cff_composites(font_data, cff_record, mapping)
    } else if let Some(cff2_record) = sfnt.find_table(tag::CFF2) {
        update_cff2_composites(font_data, cff2_record, mapping)
    } else {
        Err(SubsetError::UnknownFormat)
    }
}

fn update_ttf_composites(
    font_data: &mut [u8],
    glyf_record: TableRecord,
    mapping: &HashMap<u16, u16>,
) -> Result<UpdateStats, SubsetError> {
    let mut stats = UpdateStats::default();
    
    // Parse loca to find glyph boundaries
    let loca = parse_loca_table(font_data)?;
    let glyf_offset = glyf_record.offset as usize;
    
    // Iterate through all glyphs
    for (glyph_id, (start, end)) in loca.offsets.windows(2).enumerate() {
        if end > start {  // Non-empty glyph
            let glyph_offset = glyf_offset + start;
            let glyph_data = &mut font_data[glyph_offset..glyph_offset + (end - start)];
            
            // Check if composite (numberOfContours < 0)
            let num_contours = i16::from_be_bytes([glyph_data[0], glyph_data[1]]);
            if num_contours < 0 {
                // Parse and update composite components
                update_composite_glyph(glyph_data, mapping, &mut stats)?;
            }
        }
    }
    
    Ok(stats)
}

fn update_composite_glyph(
    glyph_data: &mut [u8],
    mapping: &HashMap<u16, u16>,
    stats: &mut UpdateStats,
) -> Result<(), SubsetError> {
    // Skip bounding box (10 bytes: numberOfContours + 4 x i16)
    let mut offset = 10;
    
    loop {
        // Read flags (2 bytes)
        let flags = u16::from_be_bytes([glyph_data[offset], glyph_data[offset + 1]]);
        offset += 2;
        
        // Read and update glyph index (2 bytes)
        let old_gid = u16::from_be_bytes([glyph_data[offset], glyph_data[offset + 1]]);
        let new_gid = mapping.get(&old_gid).copied().unwrap_or_else(|| {
            stats.unmapped_references.push(old_gid);
            0  // Default to .notdef
        });
        
        // Write updated glyph index
        glyph_data[offset..offset + 2].copy_from_slice(&new_gid.to_be_bytes());
        stats.references_updated += 1;
        offset += 2;
        
        // Skip arguments and transformation data based on flags
        offset += calculate_component_size(flags);
        
        // Check for more components
        if flags & 0x0020 == 0 {  // MORE_COMPONENTS flag
            break;
        }
    }
    
    stats.composites_updated += 1;
    Ok(())
}

fn calculate_component_size(flags: u16) -> usize {
    let mut size = 0;
    
    // Arguments
    if flags & 0x0001 != 0 {  // ARG_1_AND_2_ARE_WORDS
        size += 4;  // 2 x i16
    } else {
        size += 2;  // 2 x i8
    }
    
    // Transformation matrix
    if flags & 0x0008 != 0 {  // WE_HAVE_A_SCALE
        size += 2;  // F2Dot14
    } else if flags & 0x0040 != 0 {  // WE_HAVE_AN_X_AND_Y_SCALE
        size += 4;  // 2 x F2Dot14
    } else if flags & 0x0080 != 0 {  // WE_HAVE_A_TWO_BY_TWO
        size += 8;  // 4 x F2Dot14
    }
    
    size
}
```

#### Implementation Notes:
1. **Direct binary patching**: Updates font bytes in-place for efficiency
2. **Flag parsing**: Correctly handles composite glyph component flags
3. **Error recovery**: Uses .notdef (0) for unmapped references
4. **Statistics tracking**: Returns detailed update information

#### Step 5: Run Tests to Verify Success (TDD - Green Phase)

```bash
# Run tests again - they should now pass
cargo test composite_update
# Expected: All tests pass
```

#### Step 6: Refactor if Needed (TDD - Refactor Phase)

- Review code for clarity and efficiency
- Ensure no duplicate code
- Run tests again to ensure refactoring didn't break anything

### Feature 1.2: Enhanced Subset Result Structure

#### Step 1: Define API Signatures (No Implementation)

File: `src/subset/result.rs`

```rust
// Only signatures and types - no implementation yet!
use std::collections::{HashMap, HashSet};
use crate::tables::FontTableProvider;
use crate::error::SubsetError;

#[derive(Debug, Clone, PartialEq)]
pub struct SubsetResult {
    pub data: Vec<u8>,
    pub glyph_mapping: HashMap<u16, u16>,
    pub reverse_mapping: HashMap<u16, u16>,
    pub added_glyphs: Vec<u16>,
    pub missing_glyphs: Vec<u16>,
    pub original_info: FontInfo,
    pub subset_info: FontInfo,
    pub stats: SubsetStats,
}

#[derive(Debug, Clone)]
pub struct FontInfo {
    pub glyph_count: u16,
    pub max_gid: u16,
    pub format: FontFormat,
    pub has_composites: bool,
    pub size: usize,
}

#[derive(Debug, Clone, PartialEq)]
pub enum FontFormat {
    TrueType,
    CFF,
    CFF2,
}

#[derive(Debug, Clone)]
pub struct SubsetStats {
    pub size_reduction_bytes: i64,
    pub size_reduction_percent: f32,
    pub composite_glyphs: usize,
    pub simple_glyphs: usize,
    pub removed_tables: Vec<String>,
}

pub fn subset_detailed(
    _provider: &impl FontTableProvider,
    _glyph_ids: &[u16],
    _profile: &SubsetProfile,
    _cmap_target: CmapTarget,
) -> Result<SubsetResult, SubsetError> {
    todo!("Implementation will come after tests")
}
```

#### Step 2: Write Unit Tests (TDD - Red Phase)

File: `tests/subset_result.rs`

```rust
mod common;

use std::collections::HashMap;
use allsorts::subset::result::{
    subset_detailed, SubsetResult, FontInfo, FontFormat, SubsetStats
};
use allsorts::subset::{SubsetProfile, CmapTarget};

#[test]
fn test_subset_result_structure() {
    // Test that SubsetResult contains expected fields
    let result = SubsetResult {
        data: vec![1, 2, 3],
        glyph_mapping: HashMap::from([(0, 0), (1, 1)]),
        reverse_mapping: HashMap::from([(0, 0), (1, 1)]),
        added_glyphs: vec![],
        missing_glyphs: vec![],
        original_info: FontInfo {
            glyph_count: 100,
            max_gid: 99,
            format: FontFormat::TrueType,
            has_composites: false,
            size: 1000,
        },
        subset_info: FontInfo {
            glyph_count: 2,
            max_gid: 1,
            format: FontFormat::TrueType,
            has_composites: false,
            size: 500,
        },
        stats: SubsetStats {
            size_reduction_bytes: 500,
            size_reduction_percent: 50.0,
            composite_glyphs: 0,
            simple_glyphs: 2,
            removed_tables: vec![],
        },
    };
    
    assert_eq!(result.data.len(), 3);
    assert_eq!(result.glyph_mapping.len(), 2);
    assert_eq!(result.stats.size_reduction_percent, 50.0);
}

#[test]
fn test_subset_detailed_basic() {
    let font_buffer = common::read_fixture_font("opentype/Klei.otf");
    let provider = create_provider(&font_buffer);
    
    let result = subset_detailed(
        &provider,
        &[0, 1, 2],
        &SubsetProfile::Pdf,
        CmapTarget::Unrestricted,
    );
    
    assert!(result.is_ok());
    let subset = result.unwrap();
    
    // Verify structure
    assert!(!subset.data.is_empty());
    assert!(subset.glyph_mapping.contains_key(&0));
    assert_eq!(subset.glyph_mapping[&0], 0); // .notdef always maps to 0
    
    // Verify statistics
    assert!(subset.stats.size_reduction_bytes > 0);
    assert!(subset.stats.size_reduction_percent > 0.0);
}

#[test]
fn test_subset_detailed_missing_glyphs() {
    let font_buffer = common::read_fixture_font("opentype/Klei.otf");
    let provider = create_provider(&font_buffer);
    
    // Request non-existent glyph IDs
    let result = subset_detailed(
        &provider,
        &[0, 9999, 10000],
        &SubsetProfile::Minimal,
        CmapTarget::Unrestricted,
    );
    
    assert!(result.is_ok());
    let subset = result.unwrap();
    
    // Should identify missing glyphs
    assert!(!subset.missing_glyphs.is_empty());
    assert!(subset.missing_glyphs.contains(&9999));
    assert!(subset.missing_glyphs.contains(&10000));
}

#[test]
fn test_subset_detailed_added_glyphs() {
    let font_buffer = common::read_fixture_font("opentype/composite_glyphs.ttf");
    let provider = create_provider(&font_buffer);
    
    // Request composite glyph - should add dependencies
    let result = subset_detailed(
        &provider,
        &[0, 200], // 200 is composite that uses other glyphs
        &SubsetProfile::Default,
        CmapTarget::Unrestricted,
    );
    
    assert!(result.is_ok());
    let subset = result.unwrap();
    
    // Should have added dependency glyphs
    assert!(!subset.added_glyphs.is_empty());
}
```

#### Step 3: Run Tests to Verify Failure

```bash
cargo test subset_result
# Expected: Tests fail with "not yet implemented"
```

#### Step 4: Implement Minimal Code (TDD - Green Phase)

File: `src/subset/result.rs`

```rust
// Now add the actual implementation
pub fn subset_detailed(
    provider: &impl FontTableProvider,
    glyph_ids: &[u16],
    profile: &SubsetProfile,
    cmap_target: CmapTarget,
) -> Result<SubsetResult, SubsetError> {
    // Implementation added after tests
    let original_info = collect_font_info(provider)?;
    
    // Track requested glyphs
    let requested_glyphs: HashSet<u16> = glyph_ids.iter().copied().collect();
    
    // Perform subsetting with mapping
    let (data, glyph_mapping) = subset_and_map(provider, glyph_ids, profile, cmap_target)?;
    
    // Build reverse mapping
    let reverse_mapping: HashMap<u16, u16> = glyph_mapping
        .iter()
        .map(|(&old, &new)| (new, old))
        .collect();
    
    // Identify added glyphs (dependencies)
    let added_glyphs: Vec<u16> = glyph_mapping
        .keys()
        .filter(|&gid| !requested_glyphs.contains(gid))
        .copied()
        .collect();
    
    // Identify missing glyphs (requested but not found)
    let missing_glyphs: Vec<u16> = requested_glyphs
        .iter()
        .filter(|&gid| !glyph_mapping.contains_key(gid))
        .copied()
        .collect();
    
    // Collect subset font info
    let subset_info = collect_font_info_from_data(&data)?;
    
    // Calculate statistics
    let stats = SubsetStats {
        size_reduction_bytes: original_info.size as i64 - subset_info.size as i64,
        size_reduction_percent: ((original_info.size - subset_info.size) as f32 
                                / original_info.size as f32) * 100.0,
        composite_glyphs: count_composites(&glyph_mapping, provider)?,
        simple_glyphs: glyph_mapping.len() - count_composites(&glyph_mapping, provider)?,
        removed_tables: identify_removed_tables(provider, &data)?,
    };
    
    Ok(SubsetResult {
        data,
        glyph_mapping,
        reverse_mapping,
        added_glyphs,
        missing_glyphs,
        original_info,
        subset_info,
        stats,
    })
}

fn collect_font_info(provider: &impl FontTableProvider) -> Result<FontInfo, SubsetError> {
    let maxp = read_maxp_table(provider)?;
    let format = if provider.has_table(tag::GLYF) {
        FontFormat::TrueType
    } else if provider.has_table(tag::CFF2) {
        FontFormat::CFF2
    } else {
        FontFormat::CFF
    };
    
    let has_composites = if format == FontFormat::TrueType {
        check_for_composites(provider)?
    } else {
        false  // CFF fonts don't have traditional composites
    };
    
    Ok(FontInfo {
        glyph_count: maxp.num_glyphs,
        max_gid: maxp.num_glyphs - 1,
        format,
        has_composites,
        size: calculate_font_size(provider)?,
    })
}
```

#### Step 5: Run Tests to Verify Success

```bash
cargo test subset_result
# Expected: All tests pass
```

## Phase 2: PDF-Specific Features (Week 2)

### Feature 2.1: PDF-Specific Subsetting Helper

#### Step 1: Define API Signatures (No Implementation)

File: `src/subset/pdf.rs`

```rust
use std::collections::HashMap;
use crate::subset::{SubsetProfile, CmapTarget, subset_and_map};
use crate::subset::composite::update_composite_references;

#[derive(Debug, Clone)]
pub struct PdfFontContext {
    pub cid_to_gid_map: Option<Vec<u16>>,
    pub max_cid: u16,
    pub is_cid_font: bool,
    pub writing_mode: WritingMode,
}

#[derive(Debug, Clone, PartialEq)]
pub enum WritingMode {
    Horizontal,
    Vertical,
}

#[derive(Debug, Clone)]
pub struct PdfSubsetResult {
    pub font_data: Vec<u8>,
    pub glyph_mapping: HashMap<u16, u16>,
    pub cid_to_gid_map: Vec<u8>,
    pub validation: ValidationResult,
    pub warnings: Vec<PdfWarning>,
}

#[derive(Debug, Clone)]
pub struct ValidationResult {
    pub all_cids_mapped: bool,
    pub missing_glyph_cids: Vec<u16>,
    pub unmapped_cids: Vec<u16>,
}

#[derive(Debug, Clone)]
pub enum PdfWarning {
    MissingGlyph { cid: u16 },
    UnmappedGlyph { gid: u16 },
    BrokenComposite { glyph: u16, component: u16 },
    OversizedCidMap { actual: u16, needed: u16 },
}

pub fn subset_for_pdf(
    _provider: &impl FontTableProvider,
    _glyph_ids: &[u16],
    _pdf_context: &PdfFontContext,
) -> Result<PdfSubsetResult, SubsetError> {
    todo!("Implementation will come after tests")
}
```

#### Step 2: Write Unit Tests (TDD - Red Phase)

File: `tests/pdf_subset.rs`

```rust
mod common;

use std::collections::HashMap;
use allsorts::subset::pdf::{
    subset_for_pdf, PdfFontContext, PdfSubsetResult, WritingMode, PdfWarning
};

#[test]
fn test_pdf_context_defaults() {
    let context = PdfFontContext {
        cid_to_gid_map: None,
        max_cid: 255,
        is_cid_font: false,
        writing_mode: WritingMode::Horizontal,
    };
    
    assert_eq!(context.max_cid, 255);
    assert_eq!(context.writing_mode, WritingMode::Horizontal);
}

#[test]
fn test_subset_for_pdf_basic() {
    let font_buffer = common::read_fixture_font("opentype/Klei.otf");
    let provider = create_provider(&font_buffer);
    
    let context = PdfFontContext {
        cid_to_gid_map: None,
        max_cid: 100,
        is_cid_font: false,
        writing_mode: WritingMode::Horizontal,
    };
    
    let result = subset_for_pdf(&provider, &[0, 1, 2], &context);
    assert!(result.is_ok());
    
    let pdf_result = result.unwrap();
    assert!(!pdf_result.font_data.is_empty());
    assert_eq!(pdf_result.cid_to_gid_map.len(), 202); // (100+1) * 2 bytes
    assert!(pdf_result.validation.all_cids_mapped);
    assert!(pdf_result.warnings.is_empty());
}

#[test]
fn test_subset_for_pdf_with_cid_map() {
    let font_buffer = common::read_fixture_font("opentype/Klei.otf");
    let provider = create_provider(&font_buffer);
    
    let context = PdfFontContext {
        cid_to_gid_map: Some(vec![0, 19, 21, 110]),
        max_cid: 3,
        is_cid_font: true,
        writing_mode: WritingMode::Horizontal,
    };
    
    let result = subset_for_pdf(&provider, &[0, 19, 21, 110], &context);
    assert!(result.is_ok());
    
    let pdf_result = result.unwrap();
    assert_eq!(pdf_result.cid_to_gid_map.len(), 8); // 4 CIDs * 2 bytes
}

#[test]
fn test_subset_for_pdf_missing_glyphs() {
    let font_buffer = common::read_fixture_font("opentype/Klei.otf");
    let provider = create_provider(&font_buffer);
    
    let context = PdfFontContext {
        cid_to_gid_map: Some(vec![0, 9999]), // Non-existent glyph
        max_cid: 1,
        is_cid_font: true,
        writing_mode: WritingMode::Horizontal,
    };
    
    let result = subset_for_pdf(&provider, &[0], &context);
    assert!(result.is_ok());
    
    let pdf_result = result.unwrap();
    assert!(!pdf_result.warnings.is_empty());
    
    // Should have warning about missing glyph
    let has_missing_warning = pdf_result.warnings.iter().any(|w| {
        matches!(w, PdfWarning::MissingGlyph { .. })
    });
    assert!(has_missing_warning);
}
```

#### Step 3: Run Tests to Verify Failure

```bash
cargo test pdf_subset
# Expected: Tests fail
```

#### Step 4: Implement Minimal Code (TDD - Green Phase)

File: `src/subset/pdf.rs`

```rust
// Add implementation after tests
pub fn subset_for_pdf(
    provider: &impl FontTableProvider,
    glyph_ids: &[u16],
    pdf_context: &PdfFontContext,
) -> Result<PdfSubsetResult, SubsetError> {
    // Implementation added after tests were written
    let (mut font_data, glyph_mapping) = subset_and_map(
        provider,
        glyph_ids,
        &SubsetProfile::Pdf,
        CmapTarget::Unrestricted,
    )?;
    
    // Step 2: Update composite references
    let update_stats = update_composite_references(&mut font_data, &glyph_mapping)?;
    
    // Step 3: Generate CIDToGIDMap
    let (cid_to_gid_map, validation) = generate_cid_to_gid_map(
        &pdf_context,
        &glyph_mapping,
    )?;
    
    // Step 4: Collect warnings
    let mut warnings = Vec::new();
    
    // Check for missing glyphs
    for &cid in &validation.missing_glyph_cids {
        warnings.push(PdfWarning::MissingGlyph { cid });
    }
    
    // Check for unmapped references
    for &gid in &update_stats.unmapped_references {
        warnings.push(PdfWarning::UnmappedGlyph { gid });
    }
    
    // Check for oversized CID map
    let max_used_cid = find_max_used_cid(&pdf_context.cid_to_gid_map, &glyph_mapping);
    if max_used_cid < pdf_context.max_cid {
        warnings.push(PdfWarning::OversizedCidMap {
            actual: pdf_context.max_cid,
            needed: max_used_cid,
        });
    }
    
    Ok(PdfSubsetResult {
        font_data,
        glyph_mapping,
        cid_to_gid_map,
        validation,
        warnings,
    })
}

fn generate_cid_to_gid_map(
    context: &PdfFontContext,
    mapping: &HashMap<u16, u16>,
) -> Result<(Vec<u8>, ValidationResult), SubsetError> {
    let mut cid_map = Vec::with_capacity((context.max_cid as usize + 1) * 2);
    let mut missing_glyph_cids = Vec::new();
    let mut unmapped_cids = Vec::new();
    
    for cid in 0..=context.max_cid {
        // Get original GID for this CID
        let old_gid = if let Some(ref existing_map) = context.cid_to_gid_map {
            existing_map.get(cid as usize).copied().unwrap_or(cid)
        } else {
            // Identity mapping if no existing map
            cid
        };
        
        // Map to new GID
        let new_gid = mapping.get(&old_gid).copied().unwrap_or_else(|| {
            if old_gid != 0 {  // Don't warn about .notdef
                if context.cid_to_gid_map.is_some() {
                    unmapped_cids.push(cid);
                } else {
                    missing_glyph_cids.push(cid);
                }
            }
            0  // Default to .notdef
        });
        
        // Write as big-endian for PDF
        cid_map.extend_from_slice(&new_gid.to_be_bytes());
    }
    
    let validation = ValidationResult {
        all_cids_mapped: missing_glyph_cids.is_empty() && unmapped_cids.is_empty(),
        missing_glyph_cids,
        unmapped_cids,
    };
    
    Ok((cid_map, validation))
}

fn find_max_used_cid(
    cid_to_gid_map: &Option<Vec<u16>>,
    glyph_mapping: &HashMap<u16, u16>,
) -> u16 {
    if let Some(map) = cid_to_gid_map {
        // Find highest CID that maps to a used glyph
        for (cid, &gid) in map.iter().enumerate().rev() {
            if glyph_mapping.contains_key(&gid) {
                return cid as u16;
            }
        }
    }
    0
}
```

#### Step 5: Run Tests to Verify Success

```bash
cargo test pdf_subset
# Expected: All tests pass
```

### Feature 2.2: Validation and Debugging Utilities

#### Step 1: Define API Signatures (No Implementation)

File: `src/subset/validation.rs`

```rust
// Only signatures - no implementation yet!
use std::collections::HashMap;
use std::fmt::Write;

#[derive(Debug)]
pub struct ValidationReport {
    pub unmapped_glyphs: Vec<u16>,
    pub affected_cids: Vec<u16>,
    pub suggestions: Vec<String>,
    pub is_valid: bool,
}

pub fn validate_mapping_coverage(
    _mapping: &HashMap<u16, u16>,
    _required_gids: &[u16],
    _cid_to_gid: Option<&[u16]>,
) -> Result<ValidationReport, ValidationError> {
    todo!("Implementation will come after tests")
}

pub fn debug_mapping(
    _mapping: &HashMap<u16, u16>,
    _font_name: &str,
) -> String {
    todo!("Implementation will come after tests")
}

pub fn diagnose_subset_issues(
    _original_font: &[u8],
    _subset_font: &[u8],
    _mapping: &HashMap<u16, u16>,
) -> DiagnosticReport {
    todo!("Implementation will come after tests")
}
```

#### Step 2: Write Unit Tests (TDD - Red Phase)

File: `tests/validation.rs`

```rust
mod common;

use std::collections::HashMap;
use allsorts::subset::validation::{
    validate_mapping_coverage, debug_mapping, diagnose_subset_issues,
    ValidationReport, DiagnosticReport
};

#[test]
fn test_validate_mapping_coverage_complete() {
    let mapping = HashMap::from([
        (0, 0),
        (1, 1),
        (2, 2),
    ]);
    
    let result = validate_mapping_coverage(
        &mapping,
        &[0, 1, 2],
        None,
    );
    
    assert!(result.is_ok());
    let report = result.unwrap();
    assert!(report.is_valid);
    assert!(report.unmapped_glyphs.is_empty());
}

#[test]
fn test_validate_mapping_coverage_missing() {
    let mapping = HashMap::from([
        (0, 0),
        (1, 1),
    ]);
    
    let result = validate_mapping_coverage(
        &mapping,
        &[0, 1, 2, 3], // 2 and 3 are not mapped
        None,
    );
    
    assert!(result.is_ok());
    let report = result.unwrap();
    assert!(!report.is_valid);
    assert_eq!(report.unmapped_glyphs.len(), 2);
    assert!(report.unmapped_glyphs.contains(&2));
    assert!(report.unmapped_glyphs.contains(&3));
}

#[test]
fn test_debug_mapping_output() {
    let mapping = HashMap::from([
        (0, 0),
        (100, 1),
        (200, 2),
    ]);
    
    let output = debug_mapping(&mapping, "TestFont");
    
    assert!(output.contains("TestFont"));
    assert!(output.contains("3 glyphs"));
    assert!(output.contains(".notdef"));
    assert!(output.contains("GID 0 -> 0"));
}

#[test]
fn test_diagnose_subset_issues() {
    let original = common::read_fixture_font("opentype/composite_glyphs.ttf");
    let subset = common::read_fixture_font("opentype/Klei.otf");
    let mapping = HashMap::from([(0, 0), (1, 1)]);
    
    let report = diagnose_subset_issues(&original, &subset, &mapping);
    
    // Should produce a diagnostic report
    assert!(report.broken_composites.is_empty() || !report.broken_composites.is_empty());
    if !report.broken_composites.is_empty() {
        assert!(!report.recommendations.is_empty());
    }
}
```

#### Step 3: Run Tests to Verify Failure

```bash
cargo test validation
# Expected: Tests fail
```

#### Step 4: Implement Code (TDD - Green Phase)

File: `src/subset/validation.rs`

```rust
// Add implementation after tests
pub fn validate_mapping_coverage(
    mapping: &HashMap<u16, u16>,
    required_gids: &[u16],
    cid_to_gid: Option<&[u16]>,
) -> Result<ValidationReport, ValidationError> {
    // Implementation added after tests were written
    let mut unmapped_glyphs = Vec::new();
    let mut affected_cids = Vec::new();
    let mut suggestions = Vec::new();
    
    // Check all required glyphs are mapped
    for &gid in required_gids {
        if !mapping.contains_key(&gid) {
            unmapped_glyphs.push(gid);
            
            // Find affected CIDs
            if let Some(cid_map) = cid_to_gid {
                for (cid, &mapped_gid) in cid_map.iter().enumerate() {
                    if mapped_gid == gid {
                        affected_cids.push(cid as u16);
                    }
                }
            }
        }
    }
    
    // Generate suggestions
    if !unmapped_glyphs.is_empty() {
        suggestions.push(format!(
            "Add {} unmapped glyphs to the subset: {:?}",
            unmapped_glyphs.len(),
            &unmapped_glyphs[..unmapped_glyphs.len().min(5)]
        ));
    }
    
    if !affected_cids.is_empty() {
        suggestions.push(format!(
            "{} CIDs will render as missing glyphs",
            affected_cids.len()
        ));
    }
    
    Ok(ValidationReport {
        unmapped_glyphs,
        affected_cids,
        suggestions,
        is_valid: unmapped_glyphs.is_empty(),
    })
}

pub fn debug_mapping(
    mapping: &HashMap<u16, u16>,
    font_name: &str,
) -> String {
    let mut output = String::new();
    
    writeln!(&mut output, "Font: {}", font_name).unwrap();
    writeln!(&mut output, "Mapping ({} glyphs):", mapping.len()).unwrap();
    
    // Sort by new ID for readability
    let mut sorted: Vec<_> = mapping.iter().collect();
    sorted.sort_by_key(|(_, &new_id)| new_id);
    
    for (&old_id, &new_id) in sorted.iter().take(20) {
        let glyph_name = if old_id == 0 {
            ".notdef".to_string()
        } else {
            format!("glyph_{}", old_id)
        };
        writeln!(&mut output, "  GID {} -> {} ({})", old_id, new_id, glyph_name).unwrap();
    }
    
    if mapping.len() > 20 {
        writeln!(&mut output, "  ... and {} more", mapping.len() - 20).unwrap();
    }
    
    output
}

#[derive(Debug)]
pub struct DiagnosticReport {
    pub broken_composites: Vec<u16>,
    pub missing_components: Vec<(u16, u16)>,
    pub encoding_issues: Vec<String>,
    pub recommendations: Vec<String>,
}

pub fn diagnose_subset_issues(
    original_font: &[u8],
    subset_font: &[u8],
    mapping: &HashMap<u16, u16>,
) -> DiagnosticReport {
    let mut report = DiagnosticReport::default();
    
    // Check composite glyphs
    if let Ok(composites) = find_composite_glyphs(subset_font) {
        for (glyph_id, components) in composites {
            for component_gid in components {
                if !mapping.values().any(|&v| v == component_gid) {
                    report.broken_composites.push(glyph_id);
                    report.missing_components.push((glyph_id, component_gid));
                }
            }
        }
    }
    
    // Check encoding
    if let Ok(cmap) = parse_cmap_table(subset_font) {
        if cmap.is_empty() {
            report.encoding_issues.push("No character mappings found".to_string());
        }
    }
    
    // Generate recommendations
    if !report.broken_composites.is_empty() {
        report.recommendations.push(
            "Run update_composite_references to fix broken composite glyphs".to_string()
        );
    }
    
    if !report.encoding_issues.is_empty() {
        report.recommendations.push(
            "Consider using CmapTarget::Unicode for web compatibility".to_string()
        );
    }
    
    report
}
```

#### Step 5: Run Tests to Verify Success

```bash
cargo test validation
# Expected: All tests pass
```

## Phase 3: Convenience Features (Week 3)

### Feature 3.1: Builder Pattern API

#### Step 1: Define API Signatures (No Implementation)

File: `src/subset/builder.rs`

```rust
use std::collections::HashSet;
use crate::tables::FontTableProvider;
use crate::subset::{SubsetProfile, CmapTarget};

pub struct SubsetBuilder<'a> {
    provider: &'a dyn FontTableProvider,
    glyph_ids: Vec<u16>,
    cmap_target: CmapTarget,
    profile: SubsetProfile,
    fix_composites: bool,
    validation_level: ValidationLevel,
    pdf_context: Option<PdfFontContext>,
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum ValidationLevel {
    None,
    Basic,
    Standard,
    Strict,
}

impl<'a> SubsetBuilder<'a> {
    pub fn new(provider: &'a dyn FontTableProvider) -> Self {
        todo!("Implementation will come after tests")
    }
    
    pub fn with_glyphs(self, _glyph_ids: &[u16]) -> Self {
        todo!("Implementation will come after tests")
    }
    
    pub fn with_characters(self, _chars: &str) -> Result<Self, SubsetError> {
        todo!("Implementation will come after tests")
    }
    
    pub fn for_pdf(self, _max_cid: u16) -> Self {
        todo!("Implementation will come after tests")
    }
    
    pub fn with_cid_map(self, _map: &[u16]) -> Self {
        todo!("Implementation will come after tests")
    }
    
    pub fn fix_composites(self, _enabled: bool) -> Self {
        todo!("Implementation will come after tests")
    }
    
    pub fn validation_level(self, _level: ValidationLevel) -> Self {
        todo!("Implementation will come after tests")
    }
    
    pub fn with_profile(self, _profile: SubsetProfile) -> Self {
        todo!("Implementation will come after tests")
    }
    
    pub fn with_cmap_target(self, _target: CmapTarget) -> Self {
        todo!("Implementation will come after tests")
    }
    
    pub fn build(self) -> Result<SubsetResult, SubsetError> {
        todo!("Implementation will come after tests")
    }
}
```

#### Step 2: Write Unit Tests (TDD - Red Phase)

File: `tests/builder.rs`

```rust
mod common;

use allsorts::subset::builder::{SubsetBuilder, ValidationLevel};
use allsorts::subset::{SubsetProfile, CmapTarget};

#[test]
fn test_builder_basic() {
    let font_buffer = common::read_fixture_font("opentype/Klei.otf");
    let provider = create_provider(&font_buffer);
    
    let result = SubsetBuilder::new(&provider)
        .with_glyphs(&[0, 1, 2])
        .build();
    
    assert!(result.is_ok());
    let subset = result.unwrap();
    assert_eq!(subset.glyph_mapping.len(), 3);
}

#[test]
fn test_builder_with_characters() {
    let font_buffer = common::read_fixture_font("opentype/Klei.otf");
    let provider = create_provider(&font_buffer);
    
    let result = SubsetBuilder::new(&provider)
        .with_characters("Hello")
        .unwrap()
        .build();
    
    assert!(result.is_ok());
    let subset = result.unwrap();
    assert!(subset.glyph_mapping.len() >= 5); // At least H, e, l, o + .notdef
}

#[test]
fn test_builder_for_pdf() {
    let font_buffer = common::read_fixture_font("opentype/Klei.otf");
    let provider = create_provider(&font_buffer);
    
    let result = SubsetBuilder::new(&provider)
        .with_glyphs(&[0, 1, 2])
        .for_pdf(255)
        .build();
    
    assert!(result.is_ok());
    // PDF mode should auto-enable composite fixing
}

#[test]
fn test_builder_validation_levels() {
    let font_buffer = common::read_fixture_font("opentype/Klei.otf");
    let provider = create_provider(&font_buffer);
    
    // Test None validation - should accept anything
    let result = SubsetBuilder::new(&provider)
        .with_glyphs(&[9999]) // Invalid glyph
        .validation_level(ValidationLevel::None)
        .build();
    assert!(result.is_ok());
    
    // Test Strict validation - should reject invalid
    let result = SubsetBuilder::new(&provider)
        .with_glyphs(&[9999]) // Invalid glyph
        .validation_level(ValidationLevel::Strict)
        .build();
    assert!(result.is_err());
}

#[test]
fn test_builder_chaining() {
    let font_buffer = common::read_fixture_font("opentype/Klei.otf");
    let provider = create_provider(&font_buffer);
    
    let result = SubsetBuilder::new(&provider)
        .with_glyphs(&[0, 1, 2])
        .with_profile(SubsetProfile::Pdf)
        .with_cmap_target(CmapTarget::Unicode)
        .fix_composites(true)
        .validation_level(ValidationLevel::Standard)
        .build();
    
    assert!(result.is_ok());
}
```

#### Step 3: Run Tests to Verify Failure

```bash
cargo test builder
# Expected: Tests fail
```

#### Step 4: Implement Code (TDD - Green Phase)

File: `src/subset/builder.rs`

```rust
// Add implementation after tests
impl<'a> SubsetBuilder<'a> {
    pub fn new(provider: &'a dyn FontTableProvider) -> Self {
        // Implementation added after tests were written
        Self {
            provider,
            glyph_ids: vec![0],  // Always include .notdef
            cmap_target: CmapTarget::Unrestricted,
            profile: SubsetProfile::Minimal,
            fix_composites: false,
            validation_level: ValidationLevel::Standard,
            pdf_context: None,
        }
    }
    
    pub fn with_glyphs(mut self, glyph_ids: &[u16]) -> Self {
        // Ensure .notdef is first
        if !glyph_ids.is_empty() && glyph_ids[0] != 0 {
            self.glyph_ids = vec![0];
            self.glyph_ids.extend_from_slice(glyph_ids);
        } else {
            self.glyph_ids = glyph_ids.to_vec();
        }
        
        // Remove duplicates while preserving order
        let mut seen = HashSet::new();
        self.glyph_ids.retain(|&gid| seen.insert(gid));
        
        self
    }
    
    pub fn with_characters(mut self, chars: &str) -> Result<Self, SubsetError> {
        // Map characters to glyph IDs
        let mut font = Font::new(Box::new(self.provider.clone()))?;
        
        for ch in chars.chars() {
            let (glyph_id, _) = font.lookup_glyph_index(
                ch,
                MatchingPresentation::NotRequired,
                None,
            );
            if glyph_id != 0 && !self.glyph_ids.contains(&glyph_id) {
                self.glyph_ids.push(glyph_id);
            }
        }
        
        Ok(self)
    }
    
    pub fn for_pdf(mut self, max_cid: u16) -> Self {
        self.profile = SubsetProfile::Pdf;
        self.pdf_context = Some(PdfFontContext {
            cid_to_gid_map: None,
            max_cid,
            is_cid_font: true,
            writing_mode: WritingMode::Horizontal,
        });
        self.fix_composites = true;  // Auto-enable for PDF
        self
    }
    
    pub fn with_cid_map(mut self, map: &[u16]) -> Self {
        if let Some(ref mut ctx) = self.pdf_context {
            ctx.cid_to_gid_map = Some(map.to_vec());
        }
        self
    }
    
    pub fn fix_composites(mut self, enabled: bool) -> Self {
        self.fix_composites = enabled;
        self
    }
    
    pub fn validation_level(mut self, level: ValidationLevel) -> Self {
        self.validation_level = level;
        self
    }
    
    pub fn with_profile(mut self, profile: SubsetProfile) -> Self {
        self.profile = profile;
        self
    }
    
    pub fn with_cmap_target(mut self, target: CmapTarget) -> Self {
        self.cmap_target = target;
        self
    }
    
    pub fn build(self) -> Result<SubsetResult, SubsetError> {
        // Validate input based on level
        match self.validation_level {
            ValidationLevel::None => {},
            ValidationLevel::Basic => {
                if self.glyph_ids.is_empty() || self.glyph_ids[0] != 0 {
                    return Err(SubsetError::NotDef);
                }
            },
            ValidationLevel::Standard => {
                validate_glyph_ids(&self.glyph_ids, self.provider)?;
            },
            ValidationLevel::Strict => {
                strict_validate_glyph_ids(&self.glyph_ids, self.provider)?;
            },
        }
        
        // Perform subsetting
        let result = if let Some(pdf_ctx) = self.pdf_context {
            // PDF-specific subsetting
            let pdf_result = subset_for_pdf(
                self.provider,
                &self.glyph_ids,
                &pdf_ctx,
            )?;
            
            // Convert to SubsetResult
            SubsetResult {
                data: pdf_result.font_data,
                glyph_mapping: pdf_result.glyph_mapping,
                // ... other fields
            }
        } else {
            // Standard subsetting
            subset_detailed(
                self.provider,
                &self.glyph_ids,
                &self.profile,
                self.cmap_target,
            )?
        };
        
        // Fix composites if requested
        if self.fix_composites {
            let mut data = result.data;
            update_composite_references(&mut data, &result.glyph_mapping)?;
            return Ok(SubsetResult { data, ..result });
        }
        
        Ok(result)
    }
}

fn validate_glyph_ids(
    glyph_ids: &[u16],
    provider: &dyn FontTableProvider,
) -> Result<(), SubsetError> {
    // Check .notdef is first
    if glyph_ids.is_empty() || glyph_ids[0] != 0 {
        return Err(SubsetError::NotDef);
    }
    
    // Check all IDs are valid
    let maxp = read_maxp_table(provider)?;
    for &gid in glyph_ids {
        if gid >= maxp.num_glyphs {
            return Err(SubsetError::InvalidGlyphId(gid));
        }
    }
    
    Ok(())
}
```

## Integration with Existing Code

### Modifications to `src/subset.rs`

```rust
// Add at the top
pub mod composite;
pub mod pdf;
pub mod result;
pub mod builder;
pub mod validation;

// Re-export main APIs
pub use composite::{update_composite_references, UpdateStats};
pub use pdf::{subset_for_pdf, PdfFontContext, PdfSubsetResult, WritingMode};
pub use result::{subset_detailed, SubsetResult, FontInfo, SubsetStats};
pub use builder::{SubsetBuilder, ValidationLevel};
pub use validation::{validate_mapping_coverage, debug_mapping, diagnose_subset_issues};
```

### New Error Variants in `src/error.rs`

```rust
#[derive(Debug)]
pub enum SubsetError {
    // Existing variants...
    
    // New variants
    UnknownFormat,
    InvalidGlyphId(u16),
    MissingGlyfTable,
    MissingCFFTable,
    InvalidComposite(u16),
    InvalidComponentRef { glyph: u16, component: u16 },
    ValidationError(String),
}
```

#### Step 5: Run Tests to Verify Success

```bash
cargo test builder
# Expected: All tests pass
```

#### Step 6: Run Full Test Suite

```bash
# Run all tests to ensure no regressions
cargo test
# Expected: All tests pass
```

## Testing Strategy (TDD-Driven)

### Test Execution Order

1. **Pre-implementation Tests**: Write all tests before implementation
2. **Red Phase**: Verify tests fail with `todo!()` implementations
3. **Green Phase**: Add minimal code to make tests pass
4. **Refactor Phase**: Improve code while keeping tests green
5. **Integration**: Run full suite after each feature

### Test Organization Following Project Guidelines

```bash
# Test structure
tests/
├── common.rs           # Shared utilities (from existing)
├── pdf_subset.rs       # PDF-specific tests
├── composite_update.rs # Composite reference tests
├── subset_result.rs    # Result structure tests
├── validation.rs       # Validation tests
├── builder.rs          # Builder API tests
└── fonts/              # Test fixtures (existing)
```

### Unit Tests

#### Test file: `tests/pdf_subset.rs`

```rust
#[test]
fn test_composite_reference_update() {
    let font_data = read_test_font("composite.ttf");
    let glyph_ids = vec![0, 100, 200];  // 200 is composite using 100
    
    let (mut subset_data, mapping) = subset_and_map(&provider, &glyph_ids)?;
    let stats = update_composite_references(&mut subset_data, &mapping)?;
    
    assert_eq!(stats.composites_updated, 1);
    assert_eq!(stats.references_updated, 1);
    assert!(stats.unmapped_references.is_empty());
    
    // Verify composite now references new ID
    let composite = parse_composite_glyph(&subset_data, mapping[&200])?;
    assert_eq!(composite.components[0].glyph_index, mapping[&100]);
}

#[test]
fn test_cid_to_gid_generation() {
    let context = PdfFontContext {
        cid_to_gid_map: Some(vec![0, 19, 21, 110]),
        max_cid: 3,
        is_cid_font: true,
        writing_mode: WritingMode::Horizontal,
    };
    
    let mapping = HashMap::from([
        (0, 0),
        (19, 1),
        (21, 2),
        (110, 3),
    ]);
    
    let (cid_map, validation) = generate_cid_to_gid_map(&context, &mapping)?;
    
    assert_eq!(cid_map.len(), 8);  // 4 CIDs * 2 bytes
    assert_eq!(cid_map[0..2], [0, 0]);  // CID 0 -> GID 0
    assert_eq!(cid_map[2..4], [0, 1]);  // CID 1 -> GID 1 (was 19)
    assert!(validation.all_cids_mapped);
}

#[test]
fn test_pdf_workflow() {
    let provider = get_test_provider();
    let context = PdfFontContext {
        cid_to_gid_map: None,  // Identity mapping
        max_cid: 255,
        is_cid_font: false,
        writing_mode: WritingMode::Horizontal,
    };
    
    let result = subset_for_pdf(&provider, &[0, 1, 2, 3], &context)?;
    
    assert!(result.warnings.is_empty());
    assert!(result.validation.all_cids_mapped);
    assert_eq!(result.cid_to_gid_map.len(), 512);  // 256 * 2
}

#[test]
fn test_builder_pattern() {
    let result = SubsetBuilder::new(&provider)
        .with_glyphs(&[0, 10, 20, 30])
        .for_pdf(100)
        .fix_composites(true)
        .validation_level(ValidationLevel::Strict)
        .build()?;
    
    assert_eq!(result.glyph_mapping.len(), 4);
    assert!(result.stats.size_reduction_percent > 0.0);
}
```

### Continuous Testing During Development

```bash
# After each implementation step:
cargo test --lib           # Run unit tests
cargo test --test pdf_subset  # Run specific integration test
cargo test                 # Run full suite before commit

# For debugging:
cargo test -- --nocapture  # Show println! output
cargo test test_name       # Run specific test
```

### Integration Tests

```rust
#[test]
fn test_real_pdf_font_subsetting() {
    // Test with actual PDF font requirements
    let font = load_font("NotoSans-Regular.ttf");
    let text = "Hello, World!";
    
    let result = SubsetBuilder::new(&font)
        .with_characters(text)
        .for_pdf(255)
        .build()?;
    
    // Verify font works in PDF
    let pdf_doc = create_test_pdf();
    pdf_doc.embed_font(result.font_data, result.cid_to_gid_map);
    pdf_doc.draw_text(text);
    
    assert!(pdf_doc.is_valid());
}
```

### Test Coverage Requirements

```bash
# Install tarpaulin if not already installed
cargo install cargo-tarpaulin

# Check test coverage
cargo tarpaulin --out Html
# Target: >80% coverage for new code
```

### Benchmark Tests

```rust
#[bench]
fn bench_composite_update(b: &mut Bencher) {
    let font_data = load_complex_font();
    let mapping = create_large_mapping();
    
    b.iter(|| {
        let mut data = font_data.clone();
        update_composite_references(&mut data, &mapping)
    });
}

#[bench]
fn bench_pdf_subsetting(b: &mut Bencher) {
    let provider = get_provider();
    let context = create_pdf_context();
    
    b.iter(|| {
        subset_for_pdf(&provider, &test_glyphs(), &context)
    });
}
```

## Migration Guide

### Before (Manual Process)
```rust
// 100+ lines of error-prone code
let (font_data, mapping) = subset_and_map(&provider, &glyphs)?;

// Manually update composites
for glyph in find_composites(&font_data) {
    // Complex binary manipulation
}

// Manually build CIDToGIDMap
let mut cid_map = Vec::new();
for cid in 0..=max_cid {
    // Complex mapping logic
}
```

### After (Automated Process)
```rust
// 5 lines with validation
let result = subset_for_pdf(&provider, &glyphs, &pdf_context)?;
if !result.warnings.is_empty() {
    log::warn!("Subsetting warnings: {:?}", result.warnings);
}
embed_font(result.font_data, result.cid_to_gid_map);
```

## Performance Considerations

### Expected Performance
- **Composite reference update**: < 5ms for typical fonts (1000 glyphs)
- **CIDToGIDMap generation**: < 1ms for 256 CIDs
- **Overall overhead**: < 10% vs manual implementation
- **Memory usage**: One additional HashMap + temporary buffers

### Optimization Opportunities
1. **Lazy evaluation**: Only update composites if requested
2. **Parallel processing**: Update composites in parallel for large fonts
3. **Caching**: Cache parsed table offsets
4. **Streaming**: Support streaming for very large fonts

## Risk Mitigation

### Technical Risks
1. **Binary format changes**: Extensive testing with various font formats
2. **Endianness issues**: Explicit big-endian handling throughout
3. **Composite complexity**: Handle all flag combinations correctly

### Mitigation Strategies
1. **Comprehensive testing**: Unit, integration, and fuzz testing
2. **Gradual rollout**: Phase implementation over 3 weeks
3. **Backward compatibility**: All existing APIs remain unchanged
4. **Documentation**: Extensive docs and examples

## Success Metrics

### Quantitative Metrics
- ✅ All tests passing (100% coverage for new code)
- ✅ < 10% performance overhead
- ✅ > 70% code reduction for PDF workflows
- ✅ Zero breaking changes

### Qualitative Metrics
- ✅ Simplified API that's intuitive to use
- ✅ Comprehensive error messages and warnings
- ✅ Clear documentation with examples
- ✅ Positive user feedback

## Timeline (TDD-Adjusted)

### Week 1: Core Features
- Day 1: Write all tests for composite reference updater
- Day 2: Implement composite reference updater (make tests pass)
- Day 3: Write all tests for enhanced result structure
- Day 4: Implement enhanced result structure (make tests pass)
- Day 5: Refactor, documentation, and full test suite validation

### Week 2: PDF Features
- Day 1: Write all tests for PDF subsetting helper
- Day 2: Implement PDF subsetting helper (make tests pass)
- Day 3: Write all tests for validation utilities
- Day 4: Implement validation utilities (make tests pass)
- Day 5: Integration tests and cross-feature validation

### Week 3: Polish and Release
- Day 1: Write all tests for builder pattern API
- Day 2: Implement builder pattern API (make tests pass)
- Day 3: Performance optimization with benchmark validation
- Day 4: Documentation, examples, and test coverage analysis
- Day 5: Final testing, lint checks, and release preparation

### Daily TDD Workflow

```bash
# Start of day
cargo test  # Ensure clean baseline

# For each feature:
# 1. Write test
vim tests/feature_test.rs

# 2. Run test (should fail)
cargo test feature_test

# 3. Write implementation
vim src/subset/feature.rs

# 4. Run test (should pass)
cargo test feature_test

# 5. Run full suite
cargo test

# End of day
cargo test          # Full suite
cargo clippy        # Lint check
cargo fmt --check   # Format check
```

## TDD Success Criteria

### Test-Driven Metrics
- ✅ All tests written before implementation
- ✅ 100% test coverage for new code
- ✅ All tests pass in CI/CD pipeline
- ✅ No test flakiness or intermittent failures
- ✅ Tests serve as living documentation

### Quality Gates

```bash
# Before merging any PR:
cargo test                    # All tests pass
cargo clippy -- -D warnings   # No clippy warnings
cargo fmt --check             # Code is formatted
cargo doc --no-deps          # Documentation builds
cargo tarpaulin --min 80     # Coverage >= 80%
```

## Conclusion

This implementation plan provides a clear TDD-driven roadmap for adding PDF-specific font subsetting features to Allsorts. By writing tests first, we ensure that:

1. **Requirements are clear**: Tests define the expected behavior
2. **Code is testable**: TDD forces good design decisions
3. **Regressions are prevented**: Comprehensive test suite catches issues early
4. **Documentation is built-in**: Tests serve as usage examples

The phased approach ensures that critical features are delivered first while maintaining code quality and backward compatibility. The test-first methodology guarantees that every feature is properly validated before implementation.

The implementation will dramatically simplify PDF font processing workflows, reducing complexity by over 70% while improving reliability through automated validation and error handling. With comprehensive test coverage from the start, this positions Allsorts as the premier choice for PDF font subsetting in the Rust ecosystem.