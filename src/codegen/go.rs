use super::CodeGenerator;
use std::fmt::Write;

pub struct GoCodeGenerator;

impl CodeGenerator for GoCodeGenerator {
    fn generate(&self, string: &str) -> String {
        let mut code = String::new();

        writeln!(code, "package main").unwrap();
        writeln!(code).unwrap();
        writeln!(code, "// Tracking string: {}", string).unwrap();
        writeln!(
            code,
            "const TrackingString = \"{}\"",
            escape_go_string(string)
        )
        .unwrap();
        writeln!(code, "const TrackingStringLen = {}", string.len()).unwrap();

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
