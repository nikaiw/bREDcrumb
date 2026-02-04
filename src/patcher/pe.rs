use super::cave::CaveFinder;
use super::{PatchError, PatchResult, PatchStrategy};
use crate::storage::BinaryFormat;
use goblin::pe::PE;

pub fn patch_pe(
    data: &[u8],
    string: &str,
    strategy: PatchStrategy,
) -> Result<(Vec<u8>, PatchResult), PatchError> {
    let pe = PE::parse(data)?;
    let string_bytes = string.as_bytes();
    let needed_size = string_bytes.len() + 1; // +1 for null terminator

    match strategy {
        PatchStrategy::Cave => patch_pe_cave(data, &pe, string_bytes, needed_size),
        PatchStrategy::Section => patch_pe_section(data, &pe, string_bytes),
        PatchStrategy::Extend => patch_pe_extend(data, &pe, string_bytes),
        PatchStrategy::Overlay => unreachable!("Overlay is handled in mod.rs"),
    }
}

fn patch_pe_cave(
    data: &[u8],
    pe: &PE,
    string_bytes: &[u8],
    needed_size: usize,
) -> Result<(Vec<u8>, PatchResult), PatchError> {
    // Look for caves in data sections (.rdata, .data, .rsrc)
    let data_sections: Vec<_> = pe
        .sections
        .iter()
        .filter(|s| {
            let name = String::from_utf8_lossy(&s.name);
            name.starts_with(".rdata")
                || name.starts_with(".data")
                || name.starts_with(".rsrc")
                || name.starts_with(".text")
        })
        .collect();

    let mut best_cave: Option<super::cave::CodeCave> = None;
    let mut best_section_name = String::new();

    for section in &data_sections {
        let section_start = section.pointer_to_raw_data as usize;
        let section_end = section_start + section.size_of_raw_data as usize;

        if section_end > data.len() {
            continue;
        }

        let caves = CaveFinder::find_caves_in_range(data, section_start, section_end, needed_size);
        if let Some(cave) = caves.into_iter().next() {
            if best_cave.is_none() || cave.size > best_cave.as_ref().unwrap().size {
                best_cave = Some(cave);
                best_section_name = String::from_utf8_lossy(&section.name)
                    .trim_end_matches('\0')
                    .to_string();
            }
        }
    }

    // If no cave in data sections, search entire file
    if best_cave.is_none() {
        best_cave = CaveFinder::find_best_cave(data, needed_size);
    }

    match best_cave {
        Some(cave) => {
            let mut patched = data.to_vec();
            patched[cave.file_offset..cave.file_offset + string_bytes.len()]
                .copy_from_slice(string_bytes);
            // Null terminator
            patched[cave.file_offset + string_bytes.len()] = 0;

            // Calculate virtual address if possible
            let va = calculate_va_from_offset(pe, cave.file_offset);

            Ok((
                patched,
                PatchResult {
                    format: if pe.is_64 {
                        BinaryFormat::PE64
                    } else {
                        BinaryFormat::PE32
                    },
                    strategy_used: format!("cave ({})", best_section_name),
                    virtual_address: va,
                    file_offset: Some(cave.file_offset as u64),
                },
            ))
        }
        None => {
            let largest = CaveFinder::largest_cave_size(data);
            Err(PatchError::NoCaveFound {
                needed: needed_size,
                found: largest,
            })
        }
    }
}

fn patch_pe_section(
    data: &[u8],
    pe: &PE,
    string_bytes: &[u8],
) -> Result<(Vec<u8>, PatchResult), PatchError> {
    // Add a new .rtstr section
    let mut patched = data.to_vec();

    // Calculate alignment
    let file_alignment = pe
        .header
        .optional_header
        .map(|oh| oh.windows_fields.file_alignment)
        .unwrap_or(0x200) as usize;
    let section_alignment = pe
        .header
        .optional_header
        .map(|oh| oh.windows_fields.section_alignment)
        .unwrap_or(0x1000) as usize;

    // Find where sections end
    let last_section = pe.sections.last().ok_or(PatchError::PatchFailed(
        "No sections found in PE".to_string(),
    ))?;

    let raw_data_end =
        last_section.pointer_to_raw_data as usize + last_section.size_of_raw_data as usize;
    let virtual_end = last_section.virtual_address as usize + last_section.virtual_size as usize;

    // Align to file alignment
    let new_section_offset = align_up(raw_data_end, file_alignment);
    let new_section_va = align_up(virtual_end, section_alignment) as u32;
    let new_section_size = align_up(string_bytes.len() + 1, file_alignment);

    // Extend file to accommodate new section data
    patched.resize(new_section_offset + new_section_size, 0);

    // Write string data
    patched[new_section_offset..new_section_offset + string_bytes.len()]
        .copy_from_slice(string_bytes);

    // Update PE headers
    // Find section table offset
    let section_table_offset = pe.header.dos_header.pe_pointer as usize
        + 4  // PE signature
        + 20 // COFF header
        + pe.header.coff_header.size_of_optional_header as usize;

    let num_sections = pe.header.coff_header.number_of_sections as usize;
    let new_section_entry_offset = section_table_offset + num_sections * 40;

    // Check if there's space for new section header
    let headers_size = pe
        .header
        .optional_header
        .map(|oh| oh.windows_fields.size_of_headers)
        .unwrap_or(0x400) as usize;

    if new_section_entry_offset + 40 > headers_size {
        return Err(PatchError::PatchFailed(
            "No space for new section header".to_string(),
        ));
    }

    // Write new section header
    let section_header = create_section_header(
        b".rtstr\0\0",
        new_section_va,
        string_bytes.len() as u32 + 1,
        new_section_offset as u32,
        new_section_size as u32,
    );
    patched[new_section_entry_offset..new_section_entry_offset + 40]
        .copy_from_slice(&section_header);

    // Update number of sections
    let num_sections_offset = pe.header.dos_header.pe_pointer as usize + 4 + 2;
    let new_num_sections = (num_sections + 1) as u16;
    patched[num_sections_offset..num_sections_offset + 2]
        .copy_from_slice(&new_num_sections.to_le_bytes());

    // Update SizeOfImage
    if pe.header.optional_header.is_some() {
        let size_of_image_offset = pe.header.dos_header.pe_pointer as usize + 4 + 20 + 56;

        let new_size_of_image = align_up(
            new_section_va as usize + new_section_size,
            section_alignment,
        ) as u32;
        patched[size_of_image_offset..size_of_image_offset + 4]
            .copy_from_slice(&new_size_of_image.to_le_bytes());
    }

    let image_base = pe
        .header
        .optional_header
        .map(|oh| oh.windows_fields.image_base)
        .unwrap_or(0);

    Ok((
        patched,
        PatchResult {
            format: if pe.is_64 {
                BinaryFormat::PE64
            } else {
                BinaryFormat::PE32
            },
            strategy_used: "section (.rtstr)".to_string(),
            virtual_address: Some(image_base + new_section_va as u64),
            file_offset: Some(new_section_offset as u64),
        },
    ))
}

fn patch_pe_extend(
    data: &[u8],
    pe: &PE,
    string_bytes: &[u8],
) -> Result<(Vec<u8>, PatchResult), PatchError> {
    // Extend the last section
    let mut patched = data.to_vec();

    let file_alignment = pe
        .header
        .optional_header
        .map(|oh| oh.windows_fields.file_alignment)
        .unwrap_or(0x200) as usize;

    let last_section_idx = pe.sections.len() - 1;
    let last_section = &pe.sections[last_section_idx];

    let section_raw_end =
        last_section.pointer_to_raw_data as usize + last_section.size_of_raw_data as usize;

    // Write at the end of the section's raw data
    let write_offset = section_raw_end;
    let new_raw_size =
        last_section.size_of_raw_data as usize + align_up(string_bytes.len() + 1, file_alignment);

    // Extend file
    patched.resize(last_section.pointer_to_raw_data as usize + new_raw_size, 0);

    // Write string
    patched[write_offset..write_offset + string_bytes.len()].copy_from_slice(string_bytes);

    // Update section header - size of raw data
    let section_table_offset = pe.header.dos_header.pe_pointer as usize
        + 4
        + 20
        + pe.header.coff_header.size_of_optional_header as usize;
    let section_entry_offset = section_table_offset + last_section_idx * 40;

    // Update SizeOfRawData (offset 16 in section header)
    patched[section_entry_offset + 16..section_entry_offset + 20]
        .copy_from_slice(&(new_raw_size as u32).to_le_bytes());

    // Update VirtualSize if needed (offset 8 in section header)
    let new_virtual_size = last_section.virtual_size.max(new_raw_size as u32);
    patched[section_entry_offset + 8..section_entry_offset + 12]
        .copy_from_slice(&new_virtual_size.to_le_bytes());

    let image_base = pe
        .header
        .optional_header
        .map(|oh| oh.windows_fields.image_base)
        .unwrap_or(0);

    let va_offset = last_section.virtual_address as u64
        + (write_offset - last_section.pointer_to_raw_data as usize) as u64;

    Ok((
        patched,
        PatchResult {
            format: if pe.is_64 {
                BinaryFormat::PE64
            } else {
                BinaryFormat::PE32
            },
            strategy_used: format!(
                "extend ({})",
                String::from_utf8_lossy(&last_section.name).trim_end_matches('\0')
            ),
            virtual_address: Some(image_base + va_offset),
            file_offset: Some(write_offset as u64),
        },
    ))
}

fn calculate_va_from_offset(pe: &PE, file_offset: usize) -> Option<u64> {
    let image_base = pe
        .header
        .optional_header
        .map(|oh| oh.windows_fields.image_base)?;

    for section in &pe.sections {
        let section_start = section.pointer_to_raw_data as usize;
        let section_end = section_start + section.size_of_raw_data as usize;

        if file_offset >= section_start && file_offset < section_end {
            let offset_in_section = file_offset - section_start;
            return Some(image_base + section.virtual_address as u64 + offset_in_section as u64);
        }
    }

    None
}

fn align_up(value: usize, alignment: usize) -> usize {
    if alignment == 0 {
        return value;
    }
    (value + alignment - 1) & !(alignment - 1)
}

fn create_section_header(
    name: &[u8; 8],
    virtual_address: u32,
    virtual_size: u32,
    raw_data_ptr: u32,
    raw_data_size: u32,
) -> [u8; 40] {
    let mut header = [0u8; 40];

    // Name (8 bytes)
    header[0..8].copy_from_slice(name);

    // VirtualSize (4 bytes)
    header[8..12].copy_from_slice(&virtual_size.to_le_bytes());

    // VirtualAddress (4 bytes)
    header[12..16].copy_from_slice(&virtual_address.to_le_bytes());

    // SizeOfRawData (4 bytes)
    header[16..20].copy_from_slice(&raw_data_size.to_le_bytes());

    // PointerToRawData (4 bytes)
    header[20..24].copy_from_slice(&raw_data_ptr.to_le_bytes());

    // PointerToRelocations (4 bytes) - 0
    // PointerToLineNumbers (4 bytes) - 0
    // NumberOfRelocations (2 bytes) - 0
    // NumberOfLineNumbers (2 bytes) - 0

    // Characteristics (4 bytes) - IMAGE_SCN_CNT_INITIALIZED_DATA | IMAGE_SCN_MEM_READ
    let characteristics: u32 = 0x40000040;
    header[36..40].copy_from_slice(&characteristics.to_le_bytes());

    header
}
