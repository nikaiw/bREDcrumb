use super::CodeGenerator;
use std::fmt::Write;

pub struct JavaCodeGenerator;

impl CodeGenerator for JavaCodeGenerator {
    fn generate(&self, string: &str) -> String {
        let mut code = String::new();

        writeln!(code, "// Tracking string - DO NOT REMOVE").unwrap();
        writeln!(
            code,
            "// This string is used for binary attribution/tracking"
        )
        .unwrap();
        writeln!(code).unwrap();
        writeln!(code, "public class TrackingString {{").unwrap();
        writeln!(
            code,
            "    public static final String VALUE = \"{}\";",
            escape_java_string(string)
        )
        .unwrap();
        writeln!(code).unwrap();
        writeln!(
            code,
            "    // Static block ensures the string is kept in the class file"
        )
        .unwrap();
        writeln!(code, "    static {{").unwrap();
        writeln!(code, "        @SuppressWarnings(\"unused\")").unwrap();
        writeln!(code, "        int len = VALUE.length();").unwrap();
        writeln!(code, "    }}").unwrap();
        writeln!(code, "}}").unwrap();

        code
    }
}

fn escape_java_string(s: &str) -> String {
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
