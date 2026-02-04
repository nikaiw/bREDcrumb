use clap::{Parser, Subcommand, ValueEnum};
use std::path::PathBuf;

#[derive(Parser)]
#[command(name = "bredcrumb")]
#[command(about = "Generate tracking strings, YARA rules, code snippets, and patch binaries")]
#[command(version)]
pub struct Cli {
    #[command(subcommand)]
    pub command: Commands,

    /// Enable verbose output
    #[arg(short, long, global = true)]
    pub verbose: bool,
}

#[derive(Subcommand)]
pub enum Commands {
    /// Generate a new unique tracking string
    Generate {
        /// Length of the string to generate (ignored if --custom is used)
        #[arg(short, long, default_value = "12")]
        length: usize,

        /// Tag/name for this string (for tracking purposes)
        #[arg(short, long)]
        tag: Option<String>,

        /// Custom prefix for the string (ignored if --custom is used)
        #[arg(short, long, default_value = "RT")]
        prefix: String,

        /// Use a custom string instead of generating a random one
        #[arg(short, long)]
        custom: Option<String>,
    },

    /// Generate a YARA rule for a tracking string
    Yara {
        /// The tracking string to create a rule for
        string: String,

        /// Include ASCII string matching
        #[arg(long, default_value = "true")]
        ascii: bool,

        /// Include wide (UTF-16) string matching
        #[arg(long)]
        wide: bool,

        /// Rule name (defaults to auto-generated)
        #[arg(short, long)]
        name: Option<String>,

        /// Output file path
        #[arg(short, long)]
        output: Option<PathBuf>,
    },

    /// Generate code snippet for embedding the tracking string
    Code {
        /// The tracking string to embed
        string: String,

        /// Target programming language
        #[arg(short, long, value_enum, default_value = "c")]
        language: Language,

        /// Output file path
        #[arg(short, long)]
        output: Option<PathBuf>,
    },

    /// Patch a binary to inject the tracking string
    Patch {
        /// Path to the binary to patch
        binary: PathBuf,

        /// The tracking string to inject
        string: String,

        /// Output path for the patched binary
        #[arg(short, long)]
        output: Option<PathBuf>,

        /// Patching strategy
        #[arg(short, long, value_enum, default_value = "cave")]
        strategy: PatchStrategy,

        /// Force patching even if it may break the binary
        #[arg(long)]
        force: bool,
    },

    /// List all tracked strings
    List {
        /// Filter by tag
        #[arg(short, long)]
        tag: Option<String>,

        /// Output as JSON
        #[arg(long)]
        json: bool,
    },

    /// Show details about a specific tracked string
    Show {
        /// The tracking string or UUID to show
        identifier: String,
    },
}

#[derive(Clone, Copy, ValueEnum)]
pub enum Language {
    C,
    Cpp,
    Go,
    Rust,
    Csharp,
    Java,
}

impl std::fmt::Display for Language {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Language::C => write!(f, "c"),
            Language::Cpp => write!(f, "cpp"),
            Language::Go => write!(f, "go"),
            Language::Rust => write!(f, "rust"),
            Language::Csharp => write!(f, "csharp"),
            Language::Java => write!(f, "java"),
        }
    }
}

#[derive(Clone, Copy, ValueEnum)]
pub enum PatchStrategy {
    /// Use existing code caves (null byte sequences)
    Cave,
    /// Add a new section to the binary
    Section,
    /// Extend the last section
    Extend,
    /// Append data as overlay (past file end)
    Overlay,
}

impl std::fmt::Display for PatchStrategy {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            PatchStrategy::Cave => write!(f, "cave"),
            PatchStrategy::Section => write!(f, "section"),
            PatchStrategy::Extend => write!(f, "extend"),
            PatchStrategy::Overlay => write!(f, "overlay"),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use clap::CommandFactory;

    #[test]
    fn test_cli_parses() {
        Cli::command().debug_assert();
    }

    #[test]
    fn test_generate_command() {
        let cli = Cli::try_parse_from(["bredcrumb", "generate"]).unwrap();
        match cli.command {
            Commands::Generate { length, prefix, .. } => {
                assert_eq!(length, 12);
                assert_eq!(prefix, "RT");
            }
            _ => panic!("Expected Generate command"),
        }
    }

    #[test]
    fn test_generate_with_custom() {
        let cli = Cli::try_parse_from(["bredcrumb", "generate", "--custom", "MY_TRACKER"]).unwrap();
        match cli.command {
            Commands::Generate { custom, .. } => {
                assert_eq!(custom, Some("MY_TRACKER".to_string()));
            }
            _ => panic!("Expected Generate command"),
        }
    }

    #[test]
    fn test_yara_command() {
        let cli = Cli::try_parse_from(["bredcrumb", "yara", "TEST123"]).unwrap();
        match cli.command {
            Commands::Yara {
                string,
                ascii,
                wide,
                ..
            } => {
                assert_eq!(string, "TEST123");
                assert!(ascii);
                assert!(!wide);
            }
            _ => panic!("Expected Yara command"),
        }
    }

    #[test]
    fn test_yara_with_wide() {
        let cli = Cli::try_parse_from(["bredcrumb", "yara", "TEST123", "--wide"]).unwrap();
        match cli.command {
            Commands::Yara { wide, .. } => {
                assert!(wide);
            }
            _ => panic!("Expected Yara command"),
        }
    }

    #[test]
    fn test_code_command() {
        let cli = Cli::try_parse_from(["bredcrumb", "code", "TEST123"]).unwrap();
        match cli.command {
            Commands::Code {
                string, language, ..
            } => {
                assert_eq!(string, "TEST123");
                assert!(matches!(language, Language::C));
            }
            _ => panic!("Expected Code command"),
        }
    }

    #[test]
    fn test_code_with_language() {
        let cli = Cli::try_parse_from(["bredcrumb", "code", "TEST123", "-l", "rust"]).unwrap();
        match cli.command {
            Commands::Code { language, .. } => {
                assert!(matches!(language, Language::Rust));
            }
            _ => panic!("Expected Code command"),
        }
    }

    #[test]
    fn test_patch_command() {
        let cli = Cli::try_parse_from(["bredcrumb", "patch", "/tmp/test.exe", "TRACKER"]).unwrap();
        match cli.command {
            Commands::Patch {
                binary,
                string,
                strategy,
                ..
            } => {
                assert_eq!(binary.to_str().unwrap(), "/tmp/test.exe");
                assert_eq!(string, "TRACKER");
                assert!(matches!(strategy, PatchStrategy::Cave));
            }
            _ => panic!("Expected Patch command"),
        }
    }

    #[test]
    fn test_patch_with_strategy() {
        let cli = Cli::try_parse_from([
            "bredcrumb",
            "patch",
            "/tmp/test.exe",
            "TRACKER",
            "-s",
            "overlay",
        ])
        .unwrap();
        match cli.command {
            Commands::Patch { strategy, .. } => {
                assert!(matches!(strategy, PatchStrategy::Overlay));
            }
            _ => panic!("Expected Patch command"),
        }
    }

    #[test]
    fn test_list_command() {
        let cli = Cli::try_parse_from(["bredcrumb", "list"]).unwrap();
        match cli.command {
            Commands::List { tag, json } => {
                assert!(tag.is_none());
                assert!(!json);
            }
            _ => panic!("Expected List command"),
        }
    }

    #[test]
    fn test_list_with_json() {
        let cli = Cli::try_parse_from(["bredcrumb", "list", "--json"]).unwrap();
        match cli.command {
            Commands::List { json, .. } => {
                assert!(json);
            }
            _ => panic!("Expected List command"),
        }
    }

    #[test]
    fn test_show_command() {
        let cli = Cli::try_parse_from(["bredcrumb", "show", "abc123"]).unwrap();
        match cli.command {
            Commands::Show { identifier } => {
                assert_eq!(identifier, "abc123");
            }
            _ => panic!("Expected Show command"),
        }
    }

    #[test]
    fn test_verbose_flag() {
        let cli = Cli::try_parse_from(["bredcrumb", "-v", "list"]).unwrap();
        assert!(cli.verbose);
    }

    #[test]
    fn test_all_languages() {
        for lang in ["c", "cpp", "go", "rust", "csharp", "java"] {
            let cli = Cli::try_parse_from(["bredcrumb", "code", "TEST", "-l", lang]).unwrap();
            match cli.command {
                Commands::Code { .. } => {}
                _ => panic!("Expected Code command"),
            }
        }
    }

    #[test]
    fn test_all_strategies() {
        for strategy in ["cave", "section", "extend", "overlay"] {
            let cli =
                Cli::try_parse_from(["bredcrumb", "patch", "/tmp/test", "TRACK", "-s", strategy])
                    .unwrap();
            match cli.command {
                Commands::Patch { .. } => {}
                _ => panic!("Expected Patch command"),
            }
        }
    }

    #[test]
    fn test_language_display() {
        assert_eq!(format!("{}", Language::C), "c");
        assert_eq!(format!("{}", Language::Cpp), "cpp");
        assert_eq!(format!("{}", Language::Rust), "rust");
    }

    #[test]
    fn test_strategy_display() {
        assert_eq!(format!("{}", PatchStrategy::Cave), "cave");
        assert_eq!(format!("{}", PatchStrategy::Overlay), "overlay");
    }
}
