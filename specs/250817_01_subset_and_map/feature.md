# Technical Requirement: `subset_and_map` Function for Allsorts

## Executive Summary

This document specifies the requirements for adding a `subset_and_map` function to the Allsorts font subsetting library. This function addresses a critical limitation where users need to know how glyph IDs are remapped during subsetting to update external references (particularly in PDF documents).

## Problem Statement

### Current Limitation

The existing `subset` function in Allsorts performs glyph subsetting with automatic renumbering for space efficiency:

```rust
// Current behavior
let subset_data = subset(&provider, &[0, 19, 21, 110, 143])?;
// Result: Glyphs are renumbered to [0, 1, 2, 3, 4]
// Problem: We don't know which original GID maps to which new GID!
```

### Impact on PDF Processing

PDF documents use several structures that reference glyph IDs directly:

1. **CIDToGIDMap**: Maps Character IDs to Glyph IDs
2. **Composite Glyphs**: Reference component glyphs by ID
3. **ToUnicode CMaps**: May reference GIDs directly

Without knowing the remapping, these structures break after subsetting, causing:
- Characters rendering as boxes (□) or question marks (?)
- Missing glyphs in PDF viewers
- Broken composite characters (é, ñ, etc.)

### Real-World Example

```
Original PDF:
- CID 143 → GID 143 (bullet character •)
- Font has GIDs [0-199]

After subsetting without mapping info:
- Font now has GIDs [0-4] (compacted)
- CID 143 still points to GID 143 (doesn't exist!)
- Result: Bullet renders as □
```

## Proposed Solution

### New Function Signature

```rust
/// Subset a font and return both the subset data and the glyph ID mapping
///
/// This function performs the same subsetting as `subset()` but additionally
/// returns a mapping from original glyph IDs to their new IDs in the subset font.
///
/// # Arguments
/// * `provider` - Font table provider for the source font
/// * `glyph_ids` - List of glyph IDs to include in the subset
///
/// # Returns
/// * `Vec<u8>` - The subset font data
/// * `HashMap<u16, u16>` - Mapping from old GID to new GID
///
/// # Errors
/// * `SubsetError::MissingNotdef` - If glyph ID 0 is not included
/// * `SubsetError::DuplicateGlyphId` - If duplicate IDs are provided
/// * `SubsetError::SubsetFailed` - If subsetting operation fails
///
/// # Example
/// ```rust
/// let glyph_ids = vec![0, 19, 21, 110, 143];
/// let (subset_data, mapping) = subset_and_map(&provider, &glyph_ids)?;
/// 
/// // Use mapping to update references
/// assert_eq!(mapping.get(&19), Some(&1));   // GID 19 became GID 1
/// assert_eq!(mapping.get(&143), Some(&4));  // GID 143 became GID 4
/// ```
pub fn subset_and_map(
    provider: &impl FontTableProvider,
    glyph_ids: &[u16],
) -> Result<(Vec<u8>, HashMap<u16, u16>), SubsetError>
```

### Detailed Behavior Specification

#### Input Requirements

1. **Font Provider**
   - Must be a valid font (TrueType, OpenType, CFF, or CFF2)
   - Must contain required tables (head, maxp, etc.)

2. **Glyph IDs**
   - Must include GID 0 (.notdef glyph)
   - Must not contain duplicates
   - Must be valid IDs within the font's glyph count
   - Order in the slice determines priority but not final ordering

#### Processing Steps

1. **Closure Computation**
   ```rust
   // Include composite glyph dependencies
   let closed_glyph_set = compute_closure(glyph_ids);
   ```

2. **Glyph Renumbering**
   ```rust
   // Assign new sequential IDs
   let mut new_gid = 0;
   let mut mapping = HashMap::new();
   
   // .notdef always remains 0
   mapping.insert(0, 0);
   new_gid = 1;
   
   // Assign sequential IDs to remaining glyphs
   for old_gid in closed_glyph_set.iter().skip(1) {
       mapping.insert(*old_gid, new_gid);
       new_gid += 1;
   }
   ```

3. **Font Subsetting**
   - Extract only needed glyphs
   - Update all internal tables with new GIDs
   - Maintain font validity

4. **Mapping Validation**
   - All requested glyphs must be in mapping
   - All mapped values must be unique
   - Mapped values must be sequential from 0

#### Output Specification

1. **Subset Font Data (`Vec<u8>`)**
   - Valid OpenType/TrueType font file
   - Contains only requested glyphs (plus dependencies)
   - All internal references updated to new GIDs
   - Tables optimized for size

2. **Glyph Mapping (`HashMap<u16, u16>`)**
   - Key: Original glyph ID
   - Value: New glyph ID in subset font
   - Contains entries for ALL glyphs in the subset (including dependencies)
   - Does NOT contain entries for excluded glyphs

#### Example Mapping

```rust
// Input
let requested_gids = vec![0, 19, 21, 110, 143];

// If GID 21 is composite referencing GID 65
let (data, mapping) = subset_and_map(&provider, &requested_gids)?;

// Expected mapping
assert_eq!(mapping, HashMap::from([
    (0, 0),    // .notdef preserved
    (19, 1),   // Simple glyph
    (21, 2),   // Composite glyph
    (65, 3),   // Added dependency of GID 21
    (110, 4),  // Simple glyph
    (143, 5),  // Simple glyph
]));
```

### Edge Cases and Error Handling

#### Missing .notdef
```rust
let result = subset_and_map(&provider, &[19, 21]);  // Missing GID 0
assert!(matches!(result, Err(SubsetError::MissingNotdef)));
```

#### Invalid Glyph IDs
```rust
let result = subset_and_map(&provider, &[0, 99999]);  // Invalid GID
assert!(matches!(result, Err(SubsetError::InvalidGlyphId(99999))));
```

#### Duplicate Glyph IDs
```rust
let result = subset_and_map(&provider, &[0, 19, 19]);  // Duplicate
assert!(matches!(result, Err(SubsetError::DuplicateGlyphId(19))));
```

#### Circular Dependencies
```rust
// If GID 10 references GID 11, and GID 11 references GID 10
let result = subset_and_map(&provider, &[0, 10]);
// Should handle gracefully, including both in output
```

### Performance Requirements

1. **Time Complexity**: O(n log n) where n is the number of glyphs
2. **Memory Usage**: Should not exceed 2x the original font size
3. **Mapping Construction**: Should add < 5% overhead vs regular `subset()`

### Testing Requirements

#### Unit Tests

```rust
#[test]
fn test_subset_and_map_simple_glyphs() {
    let (data, mapping) = subset_and_map(&provider, &[0, 10, 20, 30])?;
    assert_eq!(mapping.len(), 4);
    assert_eq!(mapping[&0], 0);
    assert_eq!(mapping[&10], 1);
    assert_eq!(mapping[&20], 2);
    assert_eq!(mapping[&30], 3);
}

#[test]
fn test_subset_and_map_with_composites() {
    // Test that composite dependencies are included and mapped
}

#[test]
fn test_subset_and_map_cff_font() {
    // Test with CFF fonts (different internal structure)
}
```

#### Integration Tests

1. **PDF Workflow Test**: Subset font, update CIDToGIDMap using mapping, verify rendering
2. **Composite Glyph Test**: Verify composite references remain valid
3. **Large Font Test**: Test with fonts containing >10,000 glyphs

### Backward Compatibility

The existing `subset` function must remain unchanged:

```rust
/// Existing function - preserved for backward compatibility
pub fn subset(
    provider: &impl FontTableProvider,
    glyph_ids: &[u16],
) -> Result<Vec<u8>, SubsetError> {
    let (data, _mapping) = subset_and_map(provider, glyph_ids)?;
    Ok(data)
}
```

### Implementation Notes

1. **Reuse Existing Logic**: The internal subsetting logic should be shared between `subset` and `subset_and_map`
2. **Lazy Mapping**: Only construct the mapping if requested (for performance)
3. **Font Type Agnostic**: Must work with TrueType, CFF, and CFF2 fonts

### Documentation Requirements

1. **API Documentation**: Full rustdoc with examples
2. **Migration Guide**: How to update from `subset` to `subset_and_map`
3. **PDF Use Case**: Specific example showing CIDToGIDMap update

### Success Criteria

1. **Functionality**: Returns correct mapping for all font types
2. **Performance**: < 5% overhead compared to `subset()`
3. **Compatibility**: Existing `subset()` users unaffected
4. **Testing**: > 95% code coverage with edge cases
5. **Documentation**: Clear examples for PDF workflows

## Alternative Considerations

### Why Not Return Mapping Optionally?

```rust
// Considered but rejected:
pub fn subset(provider: &impl FontTableProvider, glyph_ids: &[u16], return_mapping: bool) 
    -> Result<(Vec<u8>, Option<HashMap<u16, u16>>), SubsetError>
```

**Rejected because:**
- Changes existing API (breaks compatibility)
- Conditional return types are not idiomatic Rust
- Separate functions are clearer in intent

### Why Not Use a Builder Pattern?

```rust
// Considered:
Subsetter::new()
    .with_mapping()
    .subset(&provider, &glyph_ids)
```

**Not chosen for initial implementation because:**
- More complex API change
- Can be added later if needed
- Simple function is sufficient for current need

## Conclusion

The `subset_and_map` function fills a critical gap in Allsorts' functionality, enabling proper PDF font subsetting while maintaining backward compatibility. This minimal addition solves the glyph remapping problem without complicating the existing API.