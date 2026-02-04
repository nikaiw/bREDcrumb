use super::CodeGenerator;
use std::fmt::Write;

pub struct RustCodeGenerator;

impl CodeGenerator for RustCodeGenerator {
    fn generate(&self, string: &str) -> String {
        let mut code = String::new();

        writeln!(code, "// Tracking string: {}", string).unwrap();
        writeln!(
            code,
            "const TRACKING_STRING: &str = \"{}\";",
            escape_rust_string(string)
        )
        .unwrap();
        writeln!(code, "const TRACKING_STRING_LEN: usize = {};", string.len()).unwrap();

        code
    }
}

fn escape_rust_string(s: &str) -> String {
    s.chars()
        .map(|c| match c {
            '"' => "\\\"".to_string(),
            '\\' => "\\\\".to_string(),
            '\n' => "\\n".to_string(),
            '\r' => "\\r".to_string(),
            '\t' => "\\t".to_string(),
            _ => c.to_string(),
        })
        .collect()
}
