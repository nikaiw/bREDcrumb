use super::cave::CaveFinder;
use super::{PatchError, PatchResult, PatchStrategy};
use crate::storage::BinaryFormat;
use goblin::mach::MachO;

pub fn patch_macho(
    data: &[u8],
    string: &str,
    strategy: PatchStrategy,
) -> Result<(Vec<u8>, PatchResult), PatchError> {
    let macho = MachO::parse(data, 0)?;
    let string_bytes = string.as_bytes();
    let needed_size = string_bytes.len() + 1;

    match strategy {
        PatchStrategy::Cave => patch_macho_cave(data, &macho, string_bytes, needed_size),
        PatchStrategy::Section => patch_macho_section(data, &macho, string_bytes),
        PatchStrategy::Extend => patch_macho_extend(data, &macho, string_bytes),
        PatchStrategy::Overlay => unreachable!("Overlay is handled in mod.rs"),
    }
}

fn patch_macho_cave(
    data: &[u8],
    macho: &MachO,
    string_bytes: &[u8],
    needed_size: usize,
) -> Result<(Vec<u8>, PatchResult), PatchError> {
    // Look for caves in __DATA or __TEXT segments
    let mut best_cave: Option<super::cave::CodeCave> = None;
    let mut best_section_name = String::new();

    for segment in &macho.segments {
        let seg_name = segment.name().unwrap_or("");
        if seg_name != "__DATA" && seg_name != "__TEXT" && seg_name != "__LINKEDIT" {
            continue;
        }

        if let Ok(sections) = segment.sections() {
            for section in sections {
                let section_start = section.0.offset as usize;
                let section_size = section.0.size as usize;

                if section_start == 0 || section_size == 0 {
                    continue;
                }

                let section_end = section_start + section_size;
                if section_end > data.len() {
                    continue;
                }

                let caves =
                    CaveFinder::find_caves_in_range(data, section_start, section_end, needed_size);
                if let Some(cave) = caves.into_iter().next() {
                    if best_cave.is_none() || cave.size > best_cave.as_ref().unwrap().size {
                        best_cave = Some(cave);
                        best_section_name = section.0.name().unwrap_or("unknown").to_string();
                    }
                }
            }
        }
    }

    // If no cave in segments, search entire file
    if best_cave.is_none() {
        best_cave = CaveFinder::find_best_cave(data, needed_size);
    }

    match best_cave {
        Some(cave) => {
            let mut patched = data.to_vec();
            patched[cave.file_offset..cave.file_offset + string_bytes.len()]
                .copy_from_slice(string_bytes);
            patched[cave.file_offset + string_bytes.len()] = 0;

            let va = calculate_va_from_offset(macho, cave.file_offset);

            Ok((
                patched,
                PatchResult {
                    format: if macho.is_64 {
                        BinaryFormat::MachO64
                    } else {
                        BinaryFormat::MachO32
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

fn patch_macho_section(
    data: &[u8],
    macho: &MachO,
    string_bytes: &[u8],
) -> Result<(Vec<u8>, PatchResult), PatchError> {
    // For Mach-O, adding sections is complex. We'll extend __LINKEDIT instead.
    let mut patched = data.to_vec();

    // Find __LINKEDIT segment
    let linkedit = macho
        .segments
        .iter()
        .find(|s| s.name().unwrap_or("") == "__LINKEDIT")
        .ok_or(PatchError::PatchFailed(
            "No __LINKEDIT segment found".to_string(),
        ))?;

    let write_offset = (linkedit.fileoff + linkedit.filesize) as usize;

    // Extend file
    let aligned_size = align_up(string_bytes.len() + 1, 16);
    patched.resize(write_offset + aligned_size, 0);

    // Write string
    patched[write_offset..write_offset + string_bytes.len()].copy_from_slice(string_bytes);

    // Note: A full implementation would update the segment's filesize
    // This simplified version just appends data

    Ok((
        patched,
        PatchResult {
            format: if macho.is_64 {
                BinaryFormat::MachO64
            } else {
                BinaryFormat::MachO32
            },
            strategy_used: "section (__LINKEDIT extension)".to_string(),
            virtual_address: None,
            file_offset: Some(write_offset as u64),
        },
    ))
}

fn patch_macho_extend(
    data: &[u8],
    macho: &MachO,
    string_bytes: &[u8],
) -> Result<(Vec<u8>, PatchResult), PatchError> {
    // Extend __DATA segment's last section
    let mut patched = data.to_vec();

    // Find __DATA segment
    let data_segment = macho
        .segments
        .iter()
        .find(|s| s.name().unwrap_or("") == "__DATA")
        .or_else(|| macho.segments.iter().last())
        .ok_or(PatchError::PatchFailed("No segment found".to_string()))?;

    let seg_name = data_segment.name().unwrap_or("unknown");
    let segment_end = (data_segment.fileoff + data_segment.filesize) as usize;
    let write_offset = segment_end;

    // Extend file
    patched.resize(write_offset + string_bytes.len() + 1, 0);

    // Write string
    patched[write_offset..write_offset + string_bytes.len()].copy_from_slice(string_bytes);

    let va = data_segment.vmaddr + data_segment.vmsize;

    Ok((
        patched,
        PatchResult {
            format: if macho.is_64 {
                BinaryFormat::MachO64
            } else {
                BinaryFormat::MachO32
            },
            strategy_used: format!("extend ({})", seg_name),
            virtual_address: Some(va),
            file_offset: Some(write_offset as u64),
        },
    ))
}

fn calculate_va_from_offset(macho: &MachO, file_offset: usize) -> Option<u64> {
    for segment in &macho.segments {
        let seg_start = segment.fileoff as usize;
        let seg_end = seg_start + segment.filesize as usize;

        if file_offset >= seg_start && file_offset < seg_end {
            let offset_in_segment = file_offset - seg_start;
            return Some(segment.vmaddr + offset_in_segment as u64);
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
