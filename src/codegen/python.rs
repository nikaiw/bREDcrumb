use super::CodeGenerator;
use std::fmt::Write;

pub struct PythonCodeGenerator;

impl CodeGenerator for PythonCodeGenerator {
    fn generate(&self, string: &str) -> String {
        let mut code = String::new();

        writeln!(code, "# Tracking string: {}", string).unwrap();
        writeln!(
            code,
            "TRACKING_STRING = \"{}\"",
            escape_python_string(string)
        )
        .unwrap();
        writeln!(code, "TRACKING_STRING_LEN = {}", string.len()).unwrap();

        code
    }
}

fn escape_python_string(s: &str) -> String {
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
