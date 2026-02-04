use super::CodeGenerator;
use std::fmt::Write;

pub struct CSharpCodeGenerator;

impl CodeGenerator for CSharpCodeGenerator {
    fn generate(&self, string: &str) -> String {
        let mut code = String::new();

        writeln!(code, "// Tracking string - DO NOT REMOVE").unwrap();
        writeln!(
            code,
            "// This string is used for binary attribution/tracking"
        )
        .unwrap();
        writeln!(code).unwrap();
        writeln!(code, "using System.Runtime.CompilerServices;").unwrap();
        writeln!(code).unwrap();
        writeln!(code, "public static class TrackingString").unwrap();
        writeln!(code, "{{").unwrap();
        writeln!(
            code,
            "    public static readonly string Value = \"{}\";",
            escape_csharp_string(string)
        )
        .unwrap();
        writeln!(code).unwrap();
        writeln!(code, "    // Static constructor ensures the string is kept").unwrap();
        writeln!(code, "    [MethodImpl(MethodImplOptions.NoInlining)]").unwrap();
        writeln!(code, "    static TrackingString()").unwrap();
        writeln!(code, "    {{").unwrap();
        writeln!(code, "        _ = Value.Length;").unwrap();
        writeln!(code, "    }}").unwrap();
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
