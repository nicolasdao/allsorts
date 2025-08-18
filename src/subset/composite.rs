use std::collections::HashMap;
use crate::binary::read::ReadScope;
use crate::error::ParseError;
use crate::tables::OpenTypeFont;
use crate::tables::loca::LocaTable;
use crate::tables::{HeadTable, MaxpTable};
use crate::tag;

/// Statistics about composite reference updates
#[derive(Debug, Clone, Default, PartialEq)]
pub struct UpdateStats {
    /// Number of composite glyphs that were updated
    pub composites_updated: usize,
    /// Total number of component references that were updated
    pub references_updated: usize,
    /// List of glyph IDs that were referenced but not in the mapping
    pub unmapped_references: Vec<u16>,
    /// Whether CFF subroutines were updated (for CFF fonts)
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
    font_data: &mut [u8],
    mapping: &HashMap<u16, u16>,
) -> Result<UpdateStats, ParseError> {
    // Implementation added after tests were written
    // Parse the font to get table information and collect offsets
    let (glyf_offset, loca_offsets) = {
        let scope = ReadScope::new(&*font_data);
        let sfnt = scope.read::<OpenTypeFont<'_>>()?;
        
        // Check font type and get offset table
        let offset_table = match &sfnt.data {
            crate::tables::OpenTypeData::Single(offset_table) => offset_table,
            crate::tables::OpenTypeData::Collection(_) => {
                // For TTC, would need to handle multiple fonts
                return Err(ParseError::MissingValue);
            }
        };
        
        // Get table offsets from font directory
        let glyf_record = offset_table.table_records.iter()
            .find(|record| record.table_tag == tag::GLYF);
        let loca_record = offset_table.table_records.iter()
            .find(|record| record.table_tag == tag::LOCA);
        let head_record = offset_table.table_records.iter()
            .find(|record| record.table_tag == tag::HEAD);
        
        if let (Some(glyf_record), Some(loca_record), Some(head_record)) = 
               (glyf_record, loca_record, head_record) {
            // Parse head table to get index format
            let head_data = sfnt.scope.offset_length(head_record.offset as usize, head_record.length as usize)?;
            let head = head_data.read::<HeadTable>()?;
            
            // Parse maxp table to get number of glyphs
            let maxp_record = offset_table.table_records.iter()
                .find(|record| record.table_tag == tag::MAXP)
                .ok_or(ParseError::MissingValue)?;
            let maxp_data = sfnt.scope.offset_length(maxp_record.offset as usize, maxp_record.length as usize)?;
            let maxp = maxp_data.read::<MaxpTable>()?;
            
            // Parse loca table to get glyph offsets
            let loca_data = sfnt.scope.offset_length(loca_record.offset as usize, loca_record.length as usize)?;
            let loca = loca_data.read_dep::<LocaTable<'_>>((maxp.num_glyphs, head.index_to_loc_format))?;
            
            // Collect offsets to avoid borrowing issues
            let offsets: Vec<u32> = loca.offsets.iter().collect();
            
            (glyf_record.offset as usize, offsets)
        } else if offset_table.table_records.iter().any(|r| r.table_tag == tag::CFF) {
            // CFF font - return stats indicating CFF handling
            let mut stats = UpdateStats::default();
            stats.cff_subroutines_updated = false; // Not implemented yet
            return Ok(stats);
        } else if offset_table.table_records.iter().any(|r| r.table_tag == tag::CFF2) {
            // CFF2 font - return stats indicating CFF2 handling  
            let mut stats = UpdateStats::default();
            stats.cff_subroutines_updated = false; // Not implemented yet
            return Ok(stats);
        } else {
            return Err(ParseError::MissingValue);
        }
    };
    
    // Now we can mutate font_data since scope is dropped
    update_ttf_composites(font_data, glyf_offset, &loca_offsets, mapping)
}

fn update_ttf_composites(
    font_data: &mut [u8],
    glyf_offset: usize,
    loca_offsets: &[u32],
    mapping: &HashMap<u16, u16>,
) -> Result<UpdateStats, ParseError> {
    let mut stats = UpdateStats::default();
    
    // Return early if no mapping provided
    if mapping.is_empty() {
        return Ok(stats);
    }
    
    // Iterate through all glyphs using loca offsets
    for glyph_id in 0..loca_offsets.len().saturating_sub(1) {
        let start = loca_offsets[glyph_id] as usize;
        let end = loca_offsets[glyph_id + 1] as usize;
        
        if end > start {  // Non-empty glyph
            let glyph_offset = glyf_offset + start;
            
            // Safety check for bounds
            if glyph_offset + 2 > font_data.len() {
                continue;
            }
            
            // Check if composite (numberOfContours < 0)
            let num_contours = i16::from_be_bytes([
                font_data[glyph_offset], 
                font_data[glyph_offset + 1]
            ]);
            
            if num_contours < 0 {
                // Parse and update composite components
                update_composite_glyph(
                    &mut font_data[glyph_offset..glyph_offset + (end - start)],
                    mapping,
                    &mut stats
                )?;
            }
        }
    }
    
    Ok(stats)
}

fn update_composite_glyph(
    glyph_data: &mut [u8],
    mapping: &HashMap<u16, u16>,
    stats: &mut UpdateStats,
) -> Result<(), ParseError> {
    // Skip bounding box (10 bytes: numberOfContours + 4 x i16)
    let mut offset = 10;
    
    // Safety check
    if glyph_data.len() < offset + 4 {
        return Ok(());
    }
    
    loop {
        // Check bounds
        if offset + 4 > glyph_data.len() {
            break;
        }
        
        // Read flags (2 bytes)
        let flags = u16::from_be_bytes([glyph_data[offset], glyph_data[offset + 1]]);
        offset += 2;
        
        // Read and update glyph index (2 bytes)
        let old_gid = u16::from_be_bytes([glyph_data[offset], glyph_data[offset + 1]]);
        let new_gid = mapping.get(&old_gid).copied().unwrap_or_else(|| {
            if !stats.unmapped_references.contains(&old_gid) {
                stats.unmapped_references.push(old_gid);
            }
            0  // Default to .notdef
        });
        
        // Write updated glyph index
        glyph_data[offset..offset + 2].copy_from_slice(&new_gid.to_be_bytes());
        stats.references_updated += 1;
        offset += 2;
        
        // Skip arguments and transformation data based on flags
        offset += calculate_component_size(flags);
        
        // Check if we've gone beyond bounds
        if offset > glyph_data.len() {
            break;
        }
        
        // Check for more components (MORE_COMPONENTS flag is bit 5)
        if flags & 0x0020 == 0 {
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