use super::CodeGenerator;
use std::fmt::Write;

pub struct GoCodeGenerator;

impl CodeGenerator for GoCodeGenerator {
    fn generate(&self, string: &str) -> String {
        let mut code = String::new();

        writeln!(code, "package main").unwrap();
        writeln!(code).unwrap();
        writeln!(code, "// Tracking string - DO NOT REMOVE").unwrap();
        writeln!(
            code,
            "// This string is used for binary attribution/tracking"
        )
        .unwrap();
        writeln!(code).unwrap();
        writeln!(code, "//go:noinline").unwrap();
        writeln!(
            code,
            "var trackingString = \"{}\"",
            escape_go_string(string)
        )
        .unwrap();
        writeln!(code).unwrap();
        writeln!(code, "// init() ensures the string is kept in the binary").unwrap();
        writeln!(code, "func init() {{").unwrap();
        writeln!(code, "\t_ = trackingString").unwrap();
        writeln!(code, "}}").unwrap();

        code
    }
}

fn escape_go_string(s: &str) -> String {
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
