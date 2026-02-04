use super::CodeGenerator;
use std::fmt::Write;

pub struct CSharpCodeGenerator;

impl CodeGenerator for CSharpCodeGenerator {
    fn generate(&self, string: &str) -> String {
        let mut code = String::new();

        writeln!(code, "// Tracking string: {}", string).unwrap();
        writeln!(code, "public static class TrackingString").unwrap();
        writeln!(code, "{{").unwrap();
        writeln!(
            code,
            "    public const string Value = \"{}\";",
            escape_csharp_string(string)
        )
        .unwrap();
        writeln!(code, "    public const int Length = {};", string.len()).unwrap();
        writeln!(code, "}}").unwrap();

        code
    }
}

fn escape_csharp_string(s: &str) -> String {
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
