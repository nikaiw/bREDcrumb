pub mod cave;
pub mod elf;
pub mod macho;
pub mod pe;

use crate::storage::BinaryFormat;
use goblin::Object;
use thiserror::Error;

#[cfg(not(target_arch = "wasm32"))]
use crate::storage::PatchedBinary;
#[cfg(not(target_arch = "wasm32"))]
use chrono::Utc;
#[cfg(not(target_arch = "wasm32"))]
use std::fs;
#[cfg(not(target_arch = "wasm32"))]
use std::path::Path;

#[derive(Error, Debug)]
pub enum PatchError {
    #[error("Failed to read binary: {0}")]
    ReadError(#[from] std::io::Error),

    #[error("Failed to parse binary: {0}")]
    ParseError(#[from] goblin::error::Error),

    #[error("Unsupported binary format")]
    UnsupportedFormat,

    #[error("No suitable code cave found (need {needed} bytes, largest found: {found})")]
    NoCaveFound { needed: usize, found: usize },

    #[error("String too long for available space")]
    StringTooLong,

    #[error("Failed to patch binary: {0}")]
    PatchFailed(String),

    #[error("Binary verification failed")]
    VerificationFailed,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum PatchStrategy {
    Cave,
    Section,
    Extend,
    Overlay,
}

#[cfg(not(target_arch = "wasm32"))]
impl From<crate::cli::PatchStrategy> for PatchStrategy {
    fn from(s: crate::cli::PatchStrategy) -> Self {
        match s {
            crate::cli::PatchStrategy::Cave => PatchStrategy::Cave,
            crate::cli::PatchStrategy::Section => PatchStrategy::Section,
            crate::cli::PatchStrategy::Extend => PatchStrategy::Extend,
            crate::cli::PatchStrategy::Overlay => PatchStrategy::Overlay,
        }
    }
}

#[derive(Debug, Clone)]
pub struct PatchResult {
    pub format: BinaryFormat,
    pub strategy_used: String,
    pub virtual_address: Option<u64>,
    pub file_offset: Option<u64>,
}

pub struct BinaryPatcher;

impl BinaryPatcher {
    /// Detect the binary format from raw bytes
    pub fn detect_format(data: &[u8]) -> Result<BinaryFormat, PatchError> {
        match Object::parse(data)? {
            Object::PE(pe) => {
                if pe.is_64 {
                    Ok(BinaryFormat::PE64)
                } else {
                    Ok(BinaryFormat::PE32)
                }
            }
            Object::Elf(elf) => {
                if elf.is_64 {
                    Ok(BinaryFormat::ELF64)
                } else {
                    Ok(BinaryFormat::ELF32)
                }
            }
            Object::Mach(mach) => match mach {
                goblin::mach::Mach::Binary(macho) => {
                    if macho.is_64 {
                        Ok(BinaryFormat::MachO64)
                    } else {
                        Ok(BinaryFormat::MachO32)
                    }
                }
                goblin::mach::Mach::Fat(_) => Ok(BinaryFormat::MachOFat),
            },
            _ => Ok(BinaryFormat::Unknown),
        }
    }

    /// Patch a binary buffer in memory (WASM-compatible)
    /// Returns the patched binary data and patch result
    pub fn patch_buffer(
        data: &[u8],
        string: &str,
        strategy: PatchStrategy,
    ) -> Result<(Vec<u8>, PatchResult), PatchError> {
        let format = Self::detect_format(data)?;

        // Handle overlay strategy universally (doesn't need format-specific handling)
        if strategy == PatchStrategy::Overlay {
            return Self::patch_overlay(data, string, format);
        }

        let (patched_data, result) = match format {
            BinaryFormat::PE32 | BinaryFormat::PE64 => {
                pe::patch_pe(data, string, strategy)?
            }
            BinaryFormat::ELF32 | BinaryFormat::ELF64 => {
                elf::patch_elf(data, string, strategy)?
            }
            BinaryFormat::MachO32 | BinaryFormat::MachO64 => {
                macho::patch_macho(data, string, strategy)?
            }
            BinaryFormat::MachOFat => {
                return Err(PatchError::PatchFailed(
                    "Fat/Universal binaries not yet supported".to_string(),
                ));
            }
            BinaryFormat::Unknown => return Err(PatchError::UnsupportedFormat),
        };

        // Verify the patch
        if !Self::verify_patch(&patched_data, string) {
            return Err(PatchError::VerificationFailed);
        }

        Ok((patched_data, result))
    }

    /// Patch using overlay strategy (append to end of file)
    fn patch_overlay(
        data: &[u8],
        string: &str,
        format: BinaryFormat,
    ) -> Result<(Vec<u8>, PatchResult), PatchError> {
        let string_bytes = string.as_bytes();
        let file_offset = data.len();

        let mut patched = data.to_vec();
        patched.extend_from_slice(string_bytes);
        patched.push(0); // null terminator

        Ok((
            patched,
            PatchResult {
                format,
                strategy_used: "overlay".to_string(),
                virtual_address: None, // Overlay data isn't mapped to VA
                file_offset: Some(file_offset as u64),
            },
        ))
    }

    /// Verify that the string was successfully injected
    pub fn verify_patch(data: &[u8], string: &str) -> bool {
        let string_bytes = string.as_bytes();
        data.windows(string_bytes.len())
            .any(|window| window == string_bytes)
    }

    /// Patch a binary file on disk (CLI only, not available in WASM)
    #[cfg(not(target_arch = "wasm32"))]
    pub fn patch(
        binary_path: &Path,
        output_path: &Path,
        string: &str,
        strategy: PatchStrategy,
        _force: bool,
    ) -> Result<PatchResult, PatchError> {
        let data = fs::read(binary_path)?;
        let (patched_data, result) = Self::patch_buffer(&data, string, strategy)?;
        fs::write(output_path, &patched_data)?;
        Ok(result)
    }

    /// Create a patched binary record for storage (CLI only)
    #[cfg(not(target_arch = "wasm32"))]
    pub fn create_patched_binary_record(
        original_path: &Path,
        output_path: &Path,
        result: &PatchResult,
    ) -> PatchedBinary {
        PatchedBinary {
            original_path: original_path.to_string_lossy().to_string(),
            output_path: output_path.to_string_lossy().to_string(),
            binary_format: result.format,
            strategy: result.strategy_used.clone(),
            virtual_address: result.virtual_address,
            file_offset: result.file_offset,
            patched_at: Utc::now(),
        }
    }
}
