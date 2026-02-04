use super::CodeGenerator;
use std::fmt::Write;

pub struct JavaScriptCodeGenerator;

impl CodeGenerator for JavaScriptCodeGenerator {
    fn generate(&self, string: &str) -> String {
        let mut code = String::new();

        writeln!(code, "// Tracking string: {}", string).unwrap();
        writeln!(
            code,
            "const TRACKING_STRING = \"{}\";",
            escape_js_string(string)
        )
        .unwrap();
        writeln!(code, "const TRACKING_STRING_LEN = {};", string.len()).unwrap();
        writeln!(code).unwrap();
        writeln!(code, "// For ES6 modules:").unwrap();
        writeln!(
            code,
            "// export {{ TRACKING_STRING, TRACKING_STRING_LEN }};"
        )
        .unwrap();

        code
    }
}

fn escape_js_string(s: &str) -> String {
    s.chars()
        .map(|c| match c {
            '"' => "\\\"".to_string(),
            '\\' => "\\\\".to_string(),
            '\n' => "\\n".to_string(),
            '\r' => "\\r".to_string(),
            '\t' => "\\t".to_string(),
            '\0' => "\\0".to_string(),
            _ => c.to_string(),
        })
        .collect()
}
