pub mod codegen;
pub mod generator;
pub mod patcher;
pub mod storage;
pub mod yara;

#[cfg(not(target_arch = "wasm32"))]
pub mod cli;

#[cfg(target_arch = "wasm32")]
pub mod wasm;

#[cfg(not(target_arch = "wasm32"))]
pub use cli::{Cli, Commands, Language, PatchStrategy};

pub use codegen::{
    CCodeGenerator, CSharpCodeGenerator, CodeGenerator, GoCodeGenerator,
    JavaCodeGenerator, JavaScriptCodeGenerator, PowerShellCodeGenerator, PythonCodeGenerator,
    RustCodeGenerator,
};
pub use generator::StringGenerator;
pub use patcher::BinaryPatcher;
pub use storage::{BinaryFormat, TrackedString};
pub use yara::{YaraGenerator, YaraOptions};

#[cfg(not(target_arch = "wasm32"))]
pub use storage::Storage;

#[cfg(target_arch = "wasm32")]
pub use wasm::*;
