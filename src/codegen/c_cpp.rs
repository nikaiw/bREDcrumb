use super::CodeGenerator;
use std::fmt::Write;

pub struct CCodeGenerator {
    pub use_cpp: bool,
}

impl CCodeGenerator {
    pub fn new(use_cpp: bool) -> Self {
        Self { use_cpp }
    }
}

impl CodeGenerator for CCodeGenerator {
    fn generate(&self, string: &str) -> String {
        let mut code = String::new();

        if self.use_cpp {
            writeln!(code, "#include <cstring>").unwrap();
            writeln!(code).unwrap();
            writeln!(code, "// Tracking string: {}", string).unwrap();
            writeln!(
                code,
                "constexpr char TRACKING_STRING[] = \"{}\";",
                escape_c_string(string)
            )
            .unwrap();
            writeln!(code, "constexpr size_t TRACKING_STRING_LEN = {};", string.len()).unwrap();
        } else {
            writeln!(code, "#include <string.h>").unwrap();
            writeln!(code).unwrap();
            writeln!(code, "/* Tracking string: {} */", string).unwrap();
            writeln!(
                code,
                "static const char tracking_string[] = \"{}\";",
                escape_c_string(string)
            )
            .unwrap();
            writeln!(
                code,
                "#define TRACKING_STRING_LEN {}",
                string.len()
            )
            .unwrap();
        }

        code
    }
}

fn escape_c_string(s: &str) -> String {
    s.chars()
        .map(|c| match c {
            '"' => "\\\"".to_string(),
            '\\' => "\\\\".to_string(),
            '\n' => "\\n".to_string(),
            '\r' => "\\r".to_string(),
            '\t' => "\\t".to_string(),
            '\0' => "\\0".to_string(),
            _ if c.is_ascii_graphic() || c == ' ' => c.to_string(),
            _ => format!("\\x{:02X}", c as u8),
        })
        .collect()
}
