use super::CodeGenerator;
use std::fmt::Write;

pub struct RustCodeGenerator;

impl CodeGenerator for RustCodeGenerator {
    fn generate(&self, string: &str) -> String {
        let mut code = String::new();

        writeln!(code, "// Tracking string - DO NOT REMOVE").unwrap();
        writeln!(
            code,
            "// This string is used for binary attribution/tracking"
        )
        .unwrap();
        writeln!(code).unwrap();
        writeln!(code, "#[used]").unwrap();
        writeln!(code, "#[no_mangle]").unwrap();
        writeln!(
            code,
            "static TRACKING_STRING: &[u8] = b\"{}\";",
            escape_rust_bytes(string)
        )
        .unwrap();
        writeln!(code).unwrap();
        writeln!(code, "// Ensure the string is not optimized away").unwrap();
        writeln!(code, "#[inline(never)]").unwrap();
        writeln!(code, "fn _tracking_init() {{").unwrap();
        writeln!(
            code,
            "    let _ = unsafe {{ std::ptr::read_volatile(&TRACKING_STRING) }};"
        )
        .unwrap();
        writeln!(code, "}}").unwrap();

        code
    }
}

fn escape_rust_bytes(s: &str) -> String {
    s.chars()
        .map(|c| match c {
            '"' => "\\\"".to_string(),
            '\\' => "\\\\".to_string(),
            '\n' => "\\n".to_string(),
            '\r' => "\\r".to_string(),
            '\t' => "\\t".to_string(),
            _ if c.is_ascii_graphic() || c == ' ' => c.to_string(),
            _ => format!("\\x{:02X}", c as u8),
        })
        .collect()
}
