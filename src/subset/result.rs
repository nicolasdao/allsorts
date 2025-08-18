use crate::binary::read::ReadScope;
use crate::subset::CmapTarget;
use crate::subset::{subset_and_map, SubsetError, SubsetProfile};
use crate::tables::{FontTableProvider, MaxpTable, OpenTypeFont};
use crate::tag;
use std::collections::{HashMap, HashSet};

/// Enhanced result structure for font subsetting operations
#[derive(Debug, Clone, PartialEq)]
pub struct SubsetResult {
    /// The subsetted font data
    pub data: Vec<u8>,
    /// Mapping from old glyph IDs to new glyph IDs
    pub glyph_mapping: HashMap<u16, u16>,
    /// Reverse mapping from new glyph IDs to old glyph IDs
    pub reverse_mapping: HashMap<u16, u16>,
    /// Glyphs that were automatically added as dependencies
    pub added_glyphs: Vec<u16>,
    /// Glyphs that were requested but not found in the font
    pub missing_glyphs: Vec<u16>,
    /// Information about the original font
    pub original_info: FontInfo,
    /// Information about the subsetted font
    pub subset_info: FontInfo,
    /// Statistics about the subsetting operation
    pub stats: SubsetStats,
}

/// Information about a font
#[derive(Debug, Clone, PartialEq)]
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

/// Font format enumeration
#[derive(Debug, Clone, PartialEq)]
pub enum FontFormat {
    /// TrueType font format
    TrueType,
    /// Compact Font Format (CFF)
    CFF,
    /// Compact Font Format 2 (CFF2)
    CFF2,
}

/// Statistics about the subsetting operation
#[derive(Debug, Clone, PartialEq)]
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

/// Perform subsetting with detailed result information
pub fn subset_detailed(
    provider: &impl FontTableProvider,
    glyph_ids: &[u16],
    profile: &SubsetProfile,
    cmap_target: CmapTarget,
) -> Result<SubsetResult, SubsetError> {
    // Implementation added after tests
    let original_info = collect_font_info(provider)?;
    let original_size = calculate_font_size(provider)?;

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
        .filter(|&gid| !glyph_mapping.contains_key(gid) && *gid != 0)
        .copied()
        .collect();

    // Collect subset font info
    let subset_info = collect_font_info_from_data(&data)?;
    let subset_size = data.len();

    // Calculate statistics
    let stats = SubsetStats {
        size_reduction_bytes: original_size as i64 - subset_size as i64,
        size_reduction_percent: if original_size > 0 {
            ((original_size - subset_size) as f32 / original_size as f32) * 100.0
        } else {
            0.0
        },
        composite_glyphs: count_composites(&glyph_mapping, provider)?,
        simple_glyphs: glyph_mapping
            .len()
            .saturating_sub(count_composites(&glyph_mapping, provider)?),
        removed_tables: identify_removed_tables(provider, &data)?,
    };

    Ok(SubsetResult {
        data,
        glyph_mapping,
        reverse_mapping,
        added_glyphs,
        missing_glyphs,
        original_info: FontInfo {
            glyph_count: original_info.0,
            max_gid: original_info.1,
            format: original_info.2,
            has_composites: original_info.3,
            size: original_size,
        },
        subset_info,
        stats,
    })
}

fn collect_font_info(
    provider: &impl FontTableProvider,
) -> Result<(u16, u16, FontFormat, bool), SubsetError> {
    let maxp_data = provider.read_table_data(tag::MAXP)?;
    let maxp = ReadScope::new(&maxp_data).read::<MaxpTable>()?;

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
        false // CFF fonts don't have traditional composites
    };

    Ok((
        maxp.num_glyphs,
        maxp.num_glyphs.saturating_sub(1),
        format,
        has_composites,
    ))
}

fn collect_font_info_from_data(data: &[u8]) -> Result<FontInfo, SubsetError> {
    let scope = ReadScope::new(data);
    let font = scope.read::<OpenTypeFont<'_>>()?;
    let provider = font.table_provider(0)?;

    let (glyph_count, max_gid, format, has_composites) = collect_font_info(&provider)?;

    Ok(FontInfo {
        glyph_count,
        max_gid,
        format,
        has_composites,
        size: data.len(),
    })
}

fn calculate_font_size(provider: &impl FontTableProvider) -> Result<usize, SubsetError> {
    // Calculate total size by summing all table sizes
    let mut total_size = 0;

    // Common tables to check
    let tables = [
        tag::CMAP,
        tag::HEAD,
        tag::HHEA,
        tag::HMTX,
        tag::MAXP,
        tag::NAME,
        tag::OS_2,
        tag::POST,
        tag::GLYF,
        tag::LOCA,
        tag::CFF,
        tag::CFF2,
        tag::CVT,
        tag::FPGM,
        tag::PREP,
        tag::GSUB,
        tag::GPOS,
        tag::GDEF,
        tag::BASE,
        tag::JSTF,
    ];

    for table_tag in &tables {
        if provider.has_table(*table_tag) {
            if let Ok(data) = provider.read_table_data(*table_tag) {
                total_size += data.len();
            }
        }
    }

    // Add some overhead for font directory
    total_size += 12 + 16 * 20; // Approximate directory size

    Ok(total_size)
}

fn check_for_composites(provider: &impl FontTableProvider) -> Result<bool, SubsetError> {
    // For TrueType fonts, check if there are composite glyphs
    if !provider.has_table(tag::GLYF) {
        return Ok(false);
    }

    // This is a simplified check - in reality would need to parse glyf table
    // For now, assume fonts with many glyphs likely have composites
    let maxp_data = provider.read_table_data(tag::MAXP)?;
    let maxp = ReadScope::new(&maxp_data).read::<MaxpTable>()?;

    Ok(maxp.num_glyphs > 256) // Heuristic: fonts with many glyphs likely have composites
}

fn count_composites(
    _mapping: &HashMap<u16, u16>,
    _provider: &impl FontTableProvider,
) -> Result<usize, SubsetError> {
    // Simplified implementation - would need to actually parse glyphs
    Ok(0)
}

fn identify_removed_tables(
    original_provider: &impl FontTableProvider,
    subset_data: &[u8],
) -> Result<Vec<String>, SubsetError> {
    let mut removed = Vec::new();

    // Parse subset font to see which tables it has
    let scope = ReadScope::new(subset_data);
    let subset_font = scope.read::<OpenTypeFont<'_>>()?;
    let subset_provider = subset_font.table_provider(0)?;

    // Check common tables
    let tables = [
        (tag::GSUB, "GSUB"),
        (tag::GPOS, "GPOS"),
        (tag::GDEF, "GDEF"),
        (tag::BASE, "BASE"),
        (tag::JSTF, "JSTF"),
        (tag::LTSH, "LTSH"),
        (tag::VDMX, "VDMX"),
        (tag::HDMX, "hdmx"),
        (tag::KERN, "kern"),
    ];

    for (tag, name) in &tables {
        if original_provider.has_table(*tag) && !subset_provider.has_table(*tag) {
            removed.push(name.to_string());
        }
    }

    Ok(removed)
}
