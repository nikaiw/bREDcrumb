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
            writeln!(code, "#include <cstdio>").unwrap();
            writeln!(code).unwrap();
            writeln!(code, "// Tracking string - DO NOT REMOVE").unwrap();
            writeln!(
                code,
                "// This string is used for binary attribution/tracking"
            )
            .unwrap();
            writeln!(code).unwrap();
            writeln!(
                code,
                "static volatile const char TRACKING_STRING[] = \"{}\";",
                escape_c_string(string)
            )
            .unwrap();
            writeln!(code).unwrap();
            writeln!(
                code,
                "// Call this function once at startup to ensure the string is kept"
            )
            .unwrap();
            writeln!(code, "__attribute__((constructor, used))").unwrap();
            writeln!(code, "static void _tracking_init() {{").unwrap();
            writeln!(code, "    volatile const char* p = TRACKING_STRING;").unwrap();
            writeln!(code, "    (void)p;").unwrap();
            writeln!(code, "}}").unwrap();
        } else {
            writeln!(code, "#include <stdio.h>").unwrap();
            writeln!(code).unwrap();
            writeln!(code, "/* Tracking string - DO NOT REMOVE */").unwrap();
            writeln!(
                code,
                "/* This string is used for binary attribution/tracking */"
            )
            .unwrap();
            writeln!(code).unwrap();
            writeln!(
                code,
                "static volatile const char TRACKING_STRING[] = \"{}\";",
                escape_c_string(string)
            )
            .unwrap();
            writeln!(code).unwrap();
            writeln!(
                code,
                "/* Call this function once at startup to ensure the string is kept */"
            )
            .unwrap();
            writeln!(code, "__attribute__((constructor, used))").unwrap();
            writeln!(code, "static void _tracking_init(void) {{").unwrap();
            writeln!(code, "    volatile const char* p = TRACKING_STRING;").unwrap();
            writeln!(code, "    (void)p;").unwrap();
            writeln!(code, "}}").unwrap();
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
