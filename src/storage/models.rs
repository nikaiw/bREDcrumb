use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TrackedString {
    pub id: Uuid,
    pub value: String,
    pub name: Option<String>,
    pub tags: Vec<String>,
    pub created_at: DateTime<Utc>,
    #[serde(default)]
    pub patched_binaries: Vec<PatchedBinary>,
}

impl TrackedString {
    pub fn new(value: String, name: Option<String>, tags: Vec<String>) -> Self {
        Self {
            id: Uuid::new_v4(),
            value,
            name,
            tags,
            created_at: Utc::now(),
            patched_binaries: Vec::new(),
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PatchedBinary {
    pub original_path: String,
    pub output_path: String,
    pub binary_format: BinaryFormat,
    pub strategy: String,
    pub virtual_address: Option<u64>,
    pub file_offset: Option<u64>,
    pub patched_at: DateTime<Utc>,
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq)]
pub enum BinaryFormat {
    PE32,
    PE64,
    ELF32,
    ELF64,
    MachO32,
    MachO64,
    MachOFat,
    Unknown,
}

impl std::fmt::Display for BinaryFormat {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            BinaryFormat::PE32 => write!(f, "PE32"),
            BinaryFormat::PE64 => write!(f, "PE64"),
            BinaryFormat::ELF32 => write!(f, "ELF32"),
            BinaryFormat::ELF64 => write!(f, "ELF64"),
            BinaryFormat::MachO32 => write!(f, "Mach-O 32-bit"),
            BinaryFormat::MachO64 => write!(f, "Mach-O 64-bit"),
            BinaryFormat::MachOFat => write!(f, "Mach-O Fat/Universal"),
            BinaryFormat::Unknown => write!(f, "Unknown"),
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Database {
    pub version: u32,
    pub strings: Vec<TrackedString>,
}

impl Default for Database {
    fn default() -> Self {
        Self {
            version: 1,
            strings: Vec::new(),
        }
    }
}
