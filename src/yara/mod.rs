use std::fmt::Write;

pub struct YaraGenerator;

#[derive(Default)]
pub struct YaraOptions {
    pub ascii: bool,
    pub wide: bool,
    pub nocase: bool,
    pub fullword: bool,
}

impl YaraGenerator {
    pub fn generate(string: &str, rule_name: Option<&str>, options: &YaraOptions) -> String {
        let sanitized_name = rule_name
            .map(Self::sanitize_rule_name)
            .unwrap_or_else(|| Self::generate_rule_name(string));

        let mut rule = String::new();

        writeln!(rule, "rule {} {{", sanitized_name).unwrap();
        writeln!(rule, "    meta:").unwrap();
        writeln!(
            rule,
            "        description = \"Detects tracking string: {}\"",
            string
        )
        .unwrap();
        writeln!(rule, "        author = \"redteamstrings\"").unwrap();
        writeln!(
            rule,
            "        date = \"{}\"",
            chrono::Utc::now().format("%Y-%m-%d")
        )
        .unwrap();
        writeln!(rule).unwrap();
        writeln!(rule, "    strings:").unwrap();

        let mut modifiers = Vec::new();
        if options.ascii {
            modifiers.push("ascii");
        }
        if options.wide {
            modifiers.push("wide");
        }
        if options.nocase {
            modifiers.push("nocase");
        }
        if options.fullword {
            modifiers.push("fullword");
        }

        let modifier_str = if modifiers.is_empty() {
            String::new()
        } else {
            format!(" {}", modifiers.join(" "))
        };

        writeln!(
            rule,
            "        $tracking_string = \"{}\"{}",
            Self::escape_string(string),
            modifier_str
        )
        .unwrap();

        // Also add hex representation
        writeln!(
            rule,
            "        $tracking_hex = {{ {} }}",
            Self::to_hex_pattern(string)
        )
        .unwrap();

        writeln!(rule).unwrap();
        writeln!(rule, "    condition:").unwrap();
        writeln!(rule, "        any of them").unwrap();
        write!(rule, "}}").unwrap();

        rule
    }

    pub fn generate_hex_only(string: &str, rule_name: Option<&str>) -> String {
        let sanitized_name = rule_name
            .map(Self::sanitize_rule_name)
            .unwrap_or_else(|| Self::generate_rule_name(string));

        let mut rule = String::new();

        writeln!(rule, "rule {} {{", sanitized_name).unwrap();
        writeln!(rule, "    meta:").unwrap();
        writeln!(
            rule,
            "        description = \"Detects tracking string (hex): {}\"",
            string
        )
        .unwrap();
        writeln!(rule, "        author = \"redteamstrings\"").unwrap();
        writeln!(rule).unwrap();
        writeln!(rule, "    strings:").unwrap();
        writeln!(
            rule,
            "        $hex = {{ {} }}",
            Self::to_hex_pattern(string)
        )
        .unwrap();
        writeln!(rule).unwrap();
        writeln!(rule, "    condition:").unwrap();
        writeln!(rule, "        $hex").unwrap();
        write!(rule, "}}").unwrap();

        rule
    }

    fn sanitize_rule_name(name: &str) -> String {
        let sanitized: String = name
            .chars()
            .map(|c| {
                if c.is_alphanumeric() || c == '_' {
                    c
                } else {
                    '_'
                }
            })
            .collect();

        if sanitized
            .chars()
            .next()
            .map(|c| c.is_numeric())
            .unwrap_or(true)
        {
            format!("rule_{}", sanitized)
        } else {
            sanitized
        }
    }

    fn generate_rule_name(string: &str) -> String {
        format!("tracking_string_{}", &string[..string.len().min(8)])
    }

    fn escape_string(s: &str) -> String {
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

    fn to_hex_pattern(s: &str) -> String {
        s.bytes()
            .map(|b| format!("{:02X}", b))
            .collect::<Vec<_>>()
            .join(" ")
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_generate_yara_rule() {
        let options = YaraOptions {
            ascii: true,
            wide: true,
            ..Default::default()
        };
        let rule = YaraGenerator::generate("RT3xK9mPq2Wv", None, &options);
        assert!(rule.contains("RT3xK9mPq2Wv"));
        assert!(rule.contains("ascii"));
        assert!(rule.contains("wide"));
    }

    #[test]
    fn test_hex_pattern() {
        let hex = YaraGenerator::to_hex_pattern("ABC");
        assert_eq!(hex, "41 42 43");
    }
}
