use super::cave::CaveFinder;
use super::{PatchError, PatchResult, PatchStrategy};
use crate::storage::BinaryFormat;
use goblin::elf::Elf;

pub fn patch_elf(
    data: &[u8],
    string: &str,
    strategy: PatchStrategy,
) -> Result<(Vec<u8>, PatchResult), PatchError> {
    let elf = Elf::parse(data)?;
    let string_bytes = string.as_bytes();
    let needed_size = string_bytes.len() + 1;

    match strategy {
        PatchStrategy::Cave => patch_elf_cave(data, &elf, string_bytes, needed_size),
        PatchStrategy::Section => patch_elf_section(data, &elf, string_bytes),
        PatchStrategy::Extend => patch_elf_extend(data, &elf, string_bytes),
        PatchStrategy::Overlay => unreachable!("Overlay is handled in mod.rs"),
    }
}

fn patch_elf_cave(
    data: &[u8],
    elf: &Elf,
    string_bytes: &[u8],
    needed_size: usize,
) -> Result<(Vec<u8>, PatchResult), PatchError> {
    // Look for caves in data sections (.rodata, .data, .bss)
    let data_sections: Vec<_> = elf
        .section_headers
        .iter()
        .filter(|s| {
            let name = elf.shdr_strtab.get_at(s.sh_name).unwrap_or("");
            name == ".rodata" || name == ".data" || name == ".note.gnu.build-id"
        })
        .collect();

    let mut best_cave: Option<super::cave::CodeCave> = None;
    let mut best_section_name = String::new();

    for section in &data_sections {
        let section_start = section.sh_offset as usize;
        let section_end = section_start + section.sh_size as usize;

        if section_end > data.len() {
            continue;
        }

        let caves = CaveFinder::find_caves_in_range(data, section_start, section_end, needed_size);
        if let Some(cave) = caves.into_iter().next() {
            if best_cave.is_none() || cave.size > best_cave.as_ref().unwrap().size {
                best_cave = Some(cave);
                best_section_name = elf
                    .shdr_strtab
                    .get_at(section.sh_name)
                    .unwrap_or("")
                    .to_string();
            }
        }
    }

    // If no cave in data sections, search entire file (excluding headers)
    if best_cave.is_none() {
        let header_end = elf.header.e_ehsize as usize;
        best_cave = CaveFinder::find_caves_in_range(data, header_end, data.len(), needed_size)
            .into_iter()
            .next();
    }

    match best_cave {
        Some(cave) => {
            let mut patched = data.to_vec();
            patched[cave.file_offset..cave.file_offset + string_bytes.len()]
                .copy_from_slice(string_bytes);
            patched[cave.file_offset + string_bytes.len()] = 0;

            let va = calculate_va_from_offset(elf, cave.file_offset);

            Ok((
                patched,
                PatchResult {
                    format: if elf.is_64 {
                        BinaryFormat::ELF64
                    } else {
                        BinaryFormat::ELF32
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

fn patch_elf_section(
    data: &[u8],
    elf: &Elf,
    string_bytes: &[u8],
) -> Result<(Vec<u8>, PatchResult), PatchError> {
    // For ELF, adding a new section is complex because we need to update
    // the section header table. Instead, we'll append data and update
    // an existing section or use the note section approach.

    let mut patched = data.to_vec();

    // Find the last loadable segment to determine where to add data
    let last_load_segment = elf
        .program_headers
        .iter()
        .rfind(|ph| ph.p_type == goblin::elf::program_header::PT_LOAD)
        .ok_or(PatchError::PatchFailed("No LOAD segment found".to_string()))?;

    let segment_end = last_load_segment.p_offset + last_load_segment.p_filesz;
    let write_offset = segment_end as usize;

    // Extend file
    let aligned_size = align_up(string_bytes.len() + 1, 16);
    patched.resize(write_offset + aligned_size, 0);

    // Write string
    patched[write_offset..write_offset + string_bytes.len()].copy_from_slice(string_bytes);

    // Update the segment's file size in the program header
    // This is a simplified approach - a full implementation would need to
    // properly update all headers

    let ph_offset = elf.header.e_phoff as usize;
    let ph_size = elf.header.e_phentsize as usize;

    for (i, ph) in elf.program_headers.iter().enumerate() {
        if ph.p_type == goblin::elf::program_header::PT_LOAD
            && ph.p_offset == last_load_segment.p_offset
        {
            let entry_offset = ph_offset + i * ph_size;

            // Update p_filesz and p_memsz
            let new_filesz = last_load_segment.p_filesz + aligned_size as u64;
            let new_memsz = last_load_segment.p_memsz + aligned_size as u64;

            if elf.is_64 {
                // p_filesz at offset 32, p_memsz at offset 40 in 64-bit
                patched[entry_offset + 32..entry_offset + 40]
                    .copy_from_slice(&new_filesz.to_le_bytes());
                patched[entry_offset + 40..entry_offset + 48]
                    .copy_from_slice(&new_memsz.to_le_bytes());
            } else {
                // p_filesz at offset 16, p_memsz at offset 20 in 32-bit
                patched[entry_offset + 16..entry_offset + 20]
                    .copy_from_slice(&(new_filesz as u32).to_le_bytes());
                patched[entry_offset + 20..entry_offset + 24]
                    .copy_from_slice(&(new_memsz as u32).to_le_bytes());
            }
            break;
        }
    }

    let va = last_load_segment.p_vaddr + last_load_segment.p_filesz;

    Ok((
        patched,
        PatchResult {
            format: if elf.is_64 {
                BinaryFormat::ELF64
            } else {
                BinaryFormat::ELF32
            },
            strategy_used: "section (segment extension)".to_string(),
            virtual_address: Some(va),
            file_offset: Some(write_offset as u64),
        },
    ))
}

fn patch_elf_extend(
    data: &[u8],
    elf: &Elf,
    string_bytes: &[u8],
) -> Result<(Vec<u8>, PatchResult), PatchError> {
    // For ELF extend, we append data at the end of the file
    // This is simpler and safer than modifying section headers
    let mut patched = data.to_vec();

    // Find the last LOAD segment to calculate the virtual address
    let last_load = elf
        .program_headers
        .iter()
        .rfind(|ph| ph.p_type == goblin::elf::program_header::PT_LOAD);

    let write_offset = patched.len();

    // Append string at the end of the file
    patched.extend_from_slice(string_bytes);
    patched.push(0); // null terminator

    // Calculate virtual address if we have a LOAD segment reference
    let va = last_load.map(|ph| {
        // Estimate VA based on last load segment
        ph.p_vaddr + ph.p_memsz + (write_offset as u64 - (ph.p_offset + ph.p_filesz))
    });

    Ok((
        patched,
        PatchResult {
            format: if elf.is_64 {
                BinaryFormat::ELF64
            } else {
                BinaryFormat::ELF32
            },
            strategy_used: "extend (file append)".to_string(),
            virtual_address: va,
            file_offset: Some(write_offset as u64),
        },
    ))
}

fn calculate_va_from_offset(elf: &Elf, file_offset: usize) -> Option<u64> {
    for section in &elf.section_headers {
        if section.sh_type == goblin::elf::section_header::SHT_NOBITS {
            continue;
        }

        let section_start = section.sh_offset as usize;
        let section_end = section_start + section.sh_size as usize;

        if file_offset >= section_start && file_offset < section_end {
            let offset_in_section = file_offset - section_start;
            if section.sh_addr > 0 {
                return Some(section.sh_addr + offset_in_section as u64);
            }
        }
    }

    // Try program headers
    for ph in &elf.program_headers {
        if ph.p_type != goblin::elf::program_header::PT_LOAD {
            continue;
        }

        let segment_start = ph.p_offset as usize;
        let segment_end = segment_start + ph.p_filesz as usize;

        if file_offset >= segment_start && file_offset < segment_end {
            let offset_in_segment = file_offset - segment_start;
            return Some(ph.p_vaddr + offset_in_segment as u64);
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
