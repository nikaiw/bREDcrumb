use clap::Parser;
use redbreadcrumb::{
    cli::{Cli, Commands, Language},
    codegen::{
        CCodeGenerator, CSharpCodeGenerator, CodeGenerator, GoCodeGenerator,
        JavaCodeGenerator, JavaScriptCodeGenerator, PowerShellCodeGenerator, PythonCodeGenerator,
        RustCodeGenerator,
    },
    generator::StringGenerator,
    patcher::BinaryPatcher,
    storage::{Storage, TrackedString},
    yara::{YaraGenerator, YaraOptions},
};
use std::fs;
use std::path::PathBuf;

fn main() {
    if let Err(e) = run() {
        eprintln!("Error: {}", e);
        std::process::exit(1);
    }
}

fn run() -> Result<(), Box<dyn std::error::Error>> {
    let cli = Cli::parse();

    match cli.command {
        Commands::Generate { length, tag, prefix, custom } => {
            cmd_generate(length, tag, prefix, custom, cli.verbose)?;
        }
        Commands::Yara {
            string,
            ascii,
            wide,
            name,
            output,
        } => {
            cmd_yara(&string, ascii, wide, name.as_deref(), output, cli.verbose)?;
        }
        Commands::Code {
            string,
            language,
            output,
        } => {
            cmd_code(&string, language, output, cli.verbose)?;
        }
        Commands::Patch {
            binary,
            string,
            output,
            strategy,
            force,
        } => {
            cmd_patch(binary, &string, output, strategy, force, cli.verbose)?;
        }
        Commands::List { tag, json } => {
            cmd_list(tag.as_deref(), json)?;
        }
        Commands::Show { identifier } => {
            cmd_show(&identifier)?;
        }
    }

    Ok(())
}

fn cmd_generate(
    length: usize,
    tag: Option<String>,
    prefix: String,
    custom: Option<String>,
    verbose: bool,
) -> Result<(), Box<dyn std::error::Error>> {
    let value = if let Some(custom_str) = custom {
        if verbose {
            eprintln!("Using custom string: {}", custom_str);
        }
        custom_str
    } else {
        let generator = StringGenerator::new(prefix);
        let generated = generator.generate(length);
        if verbose {
            eprintln!("Generated string of length {}", length);
        }
        generated
    };

    // Store in database
    let storage = Storage::new()?;
    let tags = tag.map(|t| vec![t]).unwrap_or_default();
    let tracked = TrackedString::new(value.clone(), None, tags);

    if verbose {
        eprintln!("Storing with ID: {}", tracked.id);
    }

    storage.add_string(tracked)?;

    println!("{}", value);

    Ok(())
}

fn cmd_yara(
    string: &str,
    ascii: bool,
    wide: bool,
    name: Option<&str>,
    output: Option<PathBuf>,
    verbose: bool,
) -> Result<(), Box<dyn std::error::Error>> {
    let options = YaraOptions {
        ascii,
        wide,
        nocase: false,
        fullword: false,
    };

    let rule = YaraGenerator::generate(string, name, &options);

    if verbose {
        eprintln!("Generated YARA rule for: {}", string);
    }

    if let Some(path) = output {
        fs::write(&path, &rule)?;
        if verbose {
            eprintln!("Written to: {}", path.display());
        }
    } else {
        println!("{}", rule);
    }

    Ok(())
}

fn cmd_code(
    string: &str,
    language: Language,
    output: Option<PathBuf>,
    verbose: bool,
) -> Result<(), Box<dyn std::error::Error>> {
    let code = generate_code(string, language);

    if verbose {
        eprintln!("Generated {} code snippet", language);
    }

    if let Some(path) = output {
        fs::write(&path, &code)?;
        if verbose {
            eprintln!("Written to: {}", path.display());
        }
    } else {
        println!("{}", code);
    }

    Ok(())
}

fn generate_code(string: &str, language: Language) -> String {
    match language {
        Language::C => CCodeGenerator::new(false).generate(string),
        Language::Cpp => CCodeGenerator::new(true).generate(string),
        Language::Python => PythonCodeGenerator.generate(string),
        Language::Go => GoCodeGenerator.generate(string),
        Language::Rust => RustCodeGenerator.generate(string),
        Language::Csharp => CSharpCodeGenerator.generate(string),
        Language::Javascript => JavaScriptCodeGenerator.generate(string),
        Language::Powershell => PowerShellCodeGenerator.generate(string),
        Language::Java => JavaCodeGenerator.generate(string),
    }
}

fn cmd_patch(
    binary: PathBuf,
    string: &str,
    output: Option<PathBuf>,
    strategy: redbreadcrumb::cli::PatchStrategy,
    force: bool,
    verbose: bool,
) -> Result<(), Box<dyn std::error::Error>> {
    let output_path = output.unwrap_or_else(|| {
        let stem = binary.file_stem().unwrap_or_default().to_string_lossy();
        let ext = binary.extension().map(|e| e.to_string_lossy()).unwrap_or_default();
        let new_name = if ext.is_empty() {
            format!("{}_patched", stem)
        } else {
            format!("{}_patched.{}", stem, ext)
        };
        binary.with_file_name(new_name)
    });

    if verbose {
        eprintln!("Patching: {}", binary.display());
        eprintln!("Output: {}", output_path.display());
        eprintln!("String: {}", string);
        eprintln!("Strategy: {}", strategy);
    }

    let result = BinaryPatcher::patch(
        &binary,
        &output_path,
        string,
        strategy.into(),
        force,
    )?;

    println!("Successfully patched binary!");
    println!("  Format: {}", result.format);
    println!("  Strategy: {}", result.strategy_used);
    if let Some(va) = result.virtual_address {
        println!("  Virtual Address: 0x{:X}", va);
    }
    if let Some(offset) = result.file_offset {
        println!("  File Offset: 0x{:X}", offset);
    }
    println!("  Output: {}", output_path.display());

    // Update database
    let storage = Storage::new()?;
    if let Some(mut tracked) = storage.find_by_value(string)? {
        let record = BinaryPatcher::create_patched_binary_record(&binary, &output_path, &result);
        tracked.patched_binaries.push(record);
        storage.update_string(tracked)?;
        if verbose {
            eprintln!("Updated database record");
        }
    } else if verbose {
        eprintln!("Note: String not found in database, patch not recorded");
    }

    Ok(())
}

fn cmd_list(tag: Option<&str>, json: bool) -> Result<(), Box<dyn std::error::Error>> {
    let storage = Storage::new()?;
    let strings = if let Some(tag) = tag {
        storage.list_by_tag(tag)?
    } else {
        storage.list_all()?
    };

    if json {
        let output = serde_json::to_string_pretty(&strings)?;
        println!("{}", output);
    } else {
        if strings.is_empty() {
            println!("No tracked strings found.");
            return Ok(());
        }

        println!("{:<36}  {:<16}  {:<20}  {}", "ID", "Value", "Created", "Tags");
        println!("{}", "-".repeat(90));

        for s in strings {
            let tags = s.tags.join(", ");
            let created = s.created_at.format("%Y-%m-%d %H:%M");
            println!(
                "{:<36}  {:<16}  {:<20}  {}",
                s.id, s.value, created, tags
            );
        }
    }

    Ok(())
}

fn cmd_show(identifier: &str) -> Result<(), Box<dyn std::error::Error>> {
    let storage = Storage::new()?;
    let tracked = storage.find_by_id(identifier)?;

    match tracked {
        Some(s) => {
            println!("ID:      {}", s.id);
            println!("Value:   {}", s.value);
            if let Some(name) = &s.name {
                println!("Name:    {}", name);
            }
            println!("Tags:    {}", s.tags.join(", "));
            println!("Created: {}", s.created_at.format("%Y-%m-%d %H:%M:%S UTC"));

            if !s.patched_binaries.is_empty() {
                println!("\nPatched Binaries:");
                for pb in &s.patched_binaries {
                    println!("  - {} -> {}", pb.original_path, pb.output_path);
                    println!("    Format: {}, Strategy: {}", pb.binary_format, pb.strategy);
                    if let Some(va) = pb.virtual_address {
                        println!("    VA: 0x{:X}", va);
                    }
                }
            }
        }
        None => {
            println!("String not found: {}", identifier);
        }
    }

    Ok(())
}
