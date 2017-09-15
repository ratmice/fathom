//! endian: be

Offset32 = u32;

/// https://www.microsoft.com/typography/otspec/cmap.htm
CMap = {
    /// Table version number (0).
    version: u16,
    /// Number of encoding tables that follow.
    numTables: u16,
    encodingRecords: [EncodingRecord; numTables],
};

EncodingRecord = {
    /// Platform ID.
    platformID: u16,
    /// Platform-specific encoding ID.
    encodingID: u16,
    /// Byte offset from beginning of table to the subtable for this encoding.
    offset: Offset32,
};

CMapSubtable =
    | Format0
    // TODO: Format2
    // TODO: Format4
    | Format6
    | Format8
    // TODO: Format10
    | Format12
    | Format13
    | Format14
    ;

/// Format 0: Byte encoding table
Format0 = {
    /// Format number is set to 0.
    format: u16 = 0,
    /// This is the length in bytes of the subtable.
    length: u16,
    /// Please see “Note on the language field in 'cmap' subtables“ in this document.
    language: u16,
    /// An array that maps character codes to glyph index values.
    glyphIdArray: [u8; 256],
};

// TODO: Format2

// TODO: Format4

/// Format 6: Trimmed table mapping
Format6 = {
    /// Format number is set to 6.
    format: u16 = 6,
    /// This is the length in bytes of the subtable.
    length: u16,
    /// Please see “Note on the language field in 'cmap' subtables“ in this
    /// document.
    language: u16,
    /// First character code of subrange.
    first_code: u16,
    /// Number of character codes in subrange.
    entry_count: u16,
    /// Array of glyph index values for character codes in the range.
    glyph_id_array: [u16; entry_count],
};

/// Format 8: mixed 16-bit and 32-bit coverage
Format8 = = {
    /// Subtable format; set to 8.
    format: u16 = 8,
    /// Reserved; set to 0
    reserved: u16,
    /// Byte length of this subtable (including the header)
    length: u32,
    /// Please see “Note on the language field in 'cmap' subtables“ in this
    /// document.
    language: u32,
    /// Tightly packed array of bits (8K bytes total) indicating whether the
    /// particular 16-bit (index) value is the start of a 32-bit character code
    is32: [u8; 8192],
    /// Number of groupings which follow
    num_groups: u32,
    /// Array of SequentialMapGroup records.
    groups: [Format8SequentialMapGroup; num_groups],
};

Format8SequentialMapGroup = = {
    /// First character code in this group; note that if this group is for one
    /// or more 16-bit character codes (which is determined from the is32
    /// array), this 32-bit value will have the high 16-bits set to zero
    start_char_code: u32,
    /// Last character code in this group; same condition as listed above for
    /// the `start_char_code`
    end_char_code: u32,
    /// Glyph index corresponding to the starting character code
    start_glyph_id: u32,
};

// TODO: Format10

// Format 12: Segmented coverage
Format12 = {
    /// Subtable format; set to 12.
    format: u16 = 12,
    /// Reserved; set to 0
    reserved: u16,
    /// Byte length of this subtable (including the header)
    length: u32,
    /// Please see “Note on the language field in 'cmap' subtables“ in this
    /// document.
    language: u32,
    /// Number of groupings which follow
    num_groups: u32,
    /// Array of SequentialMapGroup records.
    groups: [Format12SequentialMapGroup; num_groups],
};

Format12SequentialMapGroup = {
    /// First character code in this group
    start_char_code: u32,
    /// Last character code in this group
    end_char_code: u32,
    /// Glyph index corresponding to the starting character code
    start_glyph_id: u32,
};

// Format 13: Many-to-one range mappings
Format13 = {
    /// Subtable format; set to 13.
    format: u16 = 13,
    /// Reserved; set to 0
    reserved: u16,
    /// Byte length of this subtable (including the header)
    length: u32,
    /// Please see “Note on the language field in 'cmap' subtables“ in this
    /// document.
    language: u32,
    /// Number of groupings which follow
    num_groups: u32,
    /// Array of ConstantMapGroup records.
    groups: [ConstantMapGroup; num_groups],
};

ConstantMapGroup = {
    /// First character code in this group
    start_char_code: u32,
    /// Last character code in this group
    end_char_code: u32,
    /// Glyph index to be used for all the characters in the group's range.
    start_glyph_id: u32,
};

/// Format 14: Unicode Variation Sequences
Format14 = {
    /// Subtable format. Set to 14.
    format: u16,
    /// Byte length of this subtable (including this header)
    length: u32,
    /// Number of variation Selector Records
    num_var_selector_records: u32,
    /// Array of VariationSelector records.
    var_selector: [VariationSelector; num_var_selector_records],
};

VariationSelector = {
    /// Variation selector
    var_selector: u24,
    /// Offset from the start of the format 14 subtable to Default UVS Table. May be 0.
    default_uvs_offset: Offset32,
    /// Offset from the start of the format 14 subtable to Non-Default UVS Table. May be 0.
    non_default_uvs_offset: Offset32,
};

/// Default UVS table
DefaultUVS = {
    /// Number of Unicode character ranges.
    num_unicode_value_ranges: u32,
    /// Array of UnicodeRange records.
    ranges: [UnicodeRange; num_unicode_value_ranges],
};

UnicodeRange = {
    /// First value in this range
    start_unicode_value: u24,
    /// Number of additional values in this range
    additional_count: u8,
};

/// NonDefaultUVS Table
NonDefaultUVS = {
    /// Number of UVS Mappings that follow
    num_uvs_mappings: u32,
    /// Array of UVSMapping records.
    uvs_mappings: [UVSMapping; num_uvs_mappings]
};

UVSMapping = {
    /// Base Unicode value of the UVS
    unicode_value: u24,
    /// Glyph ID of the UVS
    glyph_id: u16,
};