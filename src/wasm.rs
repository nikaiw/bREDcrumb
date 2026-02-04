//! WebAssembly entry points for redbreadcrumb
//!
//! This module provides WASM-compatible functions for use in browsers and Node.js.
//! All functions work on in-memory data, avoiding filesystem dependencies.

#[cfg(target_arch = "wasm32")]
use wasm_bindgen::prelude::*;

use crate::codegen::{
    CCodeGenerator, CSharpCodeGenerator, CodeGenerator, GoCodeGenerator,
    JavaCodeGenerator, JavaScriptCodeGenerator, PowerShellCodeGenerator, PythonCodeGenerator,
    RustCodeGenerator,
};
use crate::generator::StringGenerator;
use crate::patcher::{BinaryPatcher, PatchStrategy};
use crate::yara::{YaraGenerator, YaraOptions};

/// Generate a random tracking string
#[cfg(target_arch = "wasm32")]
#[wasm_bindgen]
pub fn generate_string(length: usize, prefix: &str) -> String {
    let generator = StringGenerator::new(prefix.to_string());
    generator.generate(length)
}

/// Generate a YARA rule for a tracking string
#[cfg(target_arch = "wasm32")]
#[wasm_bindgen]
pub fn generate_yara(
    tracking_string: &str,
    rule_name: Option<String>,
    ascii: bool,
    wide: bool,
) -> String {
    let options = YaraOptions {
        ascii,
        wide,
        nocase: false,
        fullword: false,
    };
    YaraGenerator::generate(tracking_string, rule_name.as_deref(), &options)
}

/// Generate code snippet for a tracking string
#[cfg(target_arch = "wasm32")]
#[wasm_bindgen]
pub fn generate_code(tracking_string: &str, language: &str) -> Result<String, JsValue> {
    let code = match language.to_lowercase().as_str() {
        "c" => CCodeGenerator::new(false).generate(tracking_string),
        "cpp" | "c++" => CCodeGenerator::new(true).generate(tracking_string),
        "python" | "py" => PythonCodeGenerator.generate(tracking_string),
        "go" | "golang" => GoCodeGenerator.generate(tracking_string),
        "rust" | "rs" => RustCodeGenerator.generate(tracking_string),
        "csharp" | "c#" | "cs" => CSharpCodeGenerator.generate(tracking_string),
        "javascript" | "js" => JavaScriptCodeGenerator.generate(tracking_string),
        "powershell" | "ps1" | "ps" => PowerShellCodeGenerator.generate(tracking_string),
        "java" => JavaCodeGenerator.generate(tracking_string),
        _ => return Err(JsValue::from_str(&format!("Unknown language: {}", language))),
    };

    Ok(code)
}

/// Patch a binary with a tracking string
/// Returns the patched binary data as a Uint8Array
#[cfg(target_arch = "wasm32")]
#[wasm_bindgen]
pub fn patch_binary(
    data: &[u8],
    tracking_string: &str,
    strategy: &str,
) -> Result<Vec<u8>, JsValue> {
    let strategy = match strategy.to_lowercase().as_str() {
        "cave" => PatchStrategy::Cave,
        "section" => PatchStrategy::Section,
        "extend" => PatchStrategy::Extend,
        "overlay" => PatchStrategy::Overlay,
        _ => return Err(JsValue::from_str(&format!("Unknown strategy: {}", strategy))),
    };

    let (patched_data, _result) = BinaryPatcher::patch_buffer(data, tracking_string, strategy)
        .map_err(|e| JsValue::from_str(&e.to_string()))?;

    Ok(patched_data)
}

/// Detect the format of a binary
#[cfg(target_arch = "wasm32")]
#[wasm_bindgen]
pub fn detect_format(data: &[u8]) -> Result<String, JsValue> {
    let format = BinaryPatcher::detect_format(data)
        .map_err(|e| JsValue::from_str(&e.to_string()))?;
    Ok(format.to_string())
}

/// Get list of supported languages
#[cfg(target_arch = "wasm32")]
#[wasm_bindgen]
pub fn get_supported_languages() -> Vec<JsValue> {
    vec![
        JsValue::from_str("c"),
        JsValue::from_str("cpp"),
        JsValue::from_str("python"),
        JsValue::from_str("go"),
        JsValue::from_str("rust"),
        JsValue::from_str("csharp"),
        JsValue::from_str("javascript"),
        JsValue::from_str("powershell"),
        JsValue::from_str("java"),
    ]
}

/// Get list of supported patch strategies
#[cfg(target_arch = "wasm32")]
#[wasm_bindgen]
pub fn get_supported_strategies() -> Vec<JsValue> {
    vec![
        JsValue::from_str("cave"),
        JsValue::from_str("section"),
        JsValue::from_str("extend"),
        JsValue::from_str("overlay"),
    ]
}

// Non-WASM versions (for testing and library use)
#[cfg(not(target_arch = "wasm32"))]
pub fn generate_string_lib(length: usize, prefix: &str) -> String {
    let generator = StringGenerator::new(prefix.to_string());
    generator.generate(length)
}

#[cfg(not(target_arch = "wasm32"))]
pub fn generate_yara_lib(
    tracking_string: &str,
    rule_name: Option<&str>,
    ascii: bool,
    wide: bool,
) -> String {
    let options = YaraOptions {
        ascii,
        wide,
        nocase: false,
        fullword: false,
    };
    YaraGenerator::generate(tracking_string, rule_name, &options)
}
