use super::CodeGenerator;
use std::fmt::Write;

pub struct PowerShellCodeGenerator;

impl CodeGenerator for PowerShellCodeGenerator {
    fn generate(&self, string: &str) -> String {
        let mut code = String::new();

        writeln!(code, "# Tracking string: {}", string).unwrap();
        writeln!(code, "$TrackingString = \"{}\"", escape_ps_string(string)).unwrap();
        writeln!(code, "$TrackingStringLen = {}", string.len()).unwrap();

        code
    }
}

fn escape_ps_string(s: &str) -> String {
    s.chars()
        .map(|c| match c {
            '"' => "`\"".to_string(),
            '`' => "``".to_string(),
            '$' => "`$".to_string(),
            '\n' => "`n".to_string(),
            '\r' => "`r".to_string(),
            '\t' => "`t".to_string(),
            '\0' => "`0".to_string(),
            _ => c.to_string(),
        })
        .collect()
}
