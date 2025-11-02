use anyhow::{Context, Result};
use clap::Parser;
use colored::*;
use std::collections::BTreeSet;
use std::fs;
use std::io::{Write};
use std::path::{Path, PathBuf};


#[derive(Parser, Debug)]
#[command(name = "treegen",version = "0.1.0",author = "JoeChala", about = "Generate directory and file structures easily")]


struct Args {
    //File and directory structure
    paths: Vec<String>,

    //Output directory
    #[arg(short, long,default_value = ".", help = "Base output directory")]
    output: PathBuf,
    
    //preview the tree before creating
    #[arg(long)]
    dry : bool,
    
    //load tree from text file
    #[arg(long)]
    from: Option<PathBuf>,
    
    //load tree from a saved template
    #[arg(long)]
    template: Option<String>,

    //create default structre for a language
    #[arg(long)]
    default: Option<String>,

}
fn main() -> Result<()> {
    let args = Args::parse();
    if args.paths.is_empty() && args.from.is_none() && args.template.is_none() && args.default.is_none() {
        eprintln!("{} No input provided. Use arguements, --from, --template, or --default.","Error:".red());
        std::process::exit(1);
    }
    let mut all_paths = BTreeSet::new();

    //args priority, template > from > default > args
    if let Some(template_name) = args.template {
        let template_path = get_template_path(&template_name);

        if !template_path.exists() {
            eprintln!("{} template not found {}", "Error:".red(),template_path.display());
            std::process::exit(1);
        }
        let lines = parse_structure_file(&template_path)
            .with_context(|| format!("Failed to read template file: {}", template_path.display()))?;
        collect_groups(&args.output, &[lines], &mut all_paths)?;
    } else if let Some(file) = args.from {
        let lines = parse_structure_file(&file)
            .with_context(|| format!("Failed to read structure file : {}",file.display()))?;
        collect_groups(&args.output, &[lines], &mut all_paths)?;
    } else if let Some(lang) = args.default {
        let structure = get_default(&lang);
        if structure.is_empty() {
            eprintln!("{} unknown default template '{}'", "Error:".red(), lang);
            std::process::exit(1);
        }
        collect_groups(&args.output, &[structure], &mut all_paths)?;
    } else {
        let groups = parse_groups(args.paths);
        collect_groups(&args.output, &groups, &mut all_paths)?;
    }

    if all_paths.is_empty() {
        eprintln!("{} No valid paths to generate.", "Error:".yellow());
        std::process::exit(1);
    }

    if args.dry {
        println!("\nProject structure preview:\n");
        print_tree(&args.output, &all_paths);
        println!("\n(No files created yet)\n");

        // Ask for user confirmation
        print!("Would you like to create this structure? (y/n): ");
        std::io::stdout().flush()?;
        let mut input = String::new();
        std::io::stdin().read_line(&mut input)?;
        let choice = input.trim().to_lowercase();

        if choice != "y" && choice != "yes" {
            println!("{} Structure not created.", "Error".red());
            return Ok(());
        }

        println!("Proceeding to create directories and files...\n");
    } 

    // Try to create files and dirs
    for path in &all_paths {
        if let Err(e) = create_path(path) {
            eprintln!("{} {},failed to create {}", "Error:".red(),e, path.display());
        }
    }

    println!("Structure created successfully!!");
    Ok(())
}


fn get_template_path(name: &str) -> PathBuf {
    let home = dirs::home_dir().unwrap_or_else(|| PathBuf::from("."));
    home.join(".config/treegen/templates").join(format!("{}.txt",name))
}


fn parse_structure_file(path: &Path) -> Result<Vec<String>> {
    let content = fs::read_to_string(path)
        .with_context(|| format!("Cannot read file '{}'",path.display()))?;
    let lines: Vec<String> = content
        .lines()
        .map(|l| l.trim())
        .filter(|l| !l.is_empty())
        .map(String::from)
        .collect();

    if lines.is_empty() {
        eprintln!("{} template or structure file '{}' is empty.","Error".yellow(),path.display());
        std::process::exit(1);
    }
    Ok(lines)
}


fn get_default(lang: &str) -> Vec<String> {
    match lang {
        "py" | "python" => vec![
            "src/__init__.py".into(),
            "src/main.py".into(),
            ".gitignore".into(),
            "requirements.txt".into(),
            "README.md".into(),
        ],
        "rs" | "rust" => vec![
            "src/main.rs".into(),
            "Cargo.toml".into(),
            ".gitignore".into(),
            "README.md".into(),
        ],
        "web" | "js" | "ts" => vec![
            "src/index.js".into(),
            "src/style.css".into(),
            "public/index.html".into(),
            ".gitignore".into(),
            "package.json".into(),
            "README.md".into(),
        ],
        _ => vec![],
    }
}


fn parse_groups(tokens: Vec<String>) -> Vec<Vec<String>> {
    let mut groups = Vec::new();
    let mut current = Vec::new();

    for token in tokens {
        if token == ":" {
            if !current.is_empty() {
                groups.push(current);
                current = Vec::new();
            }
        } else {
            current.push(token);
        }
    }
    if !current.is_empty() {
        groups.push(current);
    }
    groups
}


fn collect_groups(base: &Path, groups: &[Vec<String>], all_paths: &mut BTreeSet<PathBuf>) -> Result<()> {
    for group in groups {
        let mut current_dir = base.to_path_buf();

        for token in group {
            if token == ".." {
                current_dir = current_dir.parent().unwrap_or(base).to_path_buf();
                continue;
            }

            let path = current_dir.join(token);
            all_paths.insert(path.clone());

            if let Some(parent) = path.parent() {
                all_paths.insert(parent.to_path_buf());
            }

            // determine if token should be treated as a directory
            let name = path.file_name().and_then(|n| n.to_str()).unwrap_or("");
            let is_dir_like = path.extension().is_none() && !name.starts_with('.');

            if is_dir_like {
                current_dir = path;
            }
        }
    }
    Ok(())
}



fn print_tree(base: &Path, paths: &BTreeSet<PathBuf>) {
    println!("{}", "📦 Project Structure:".bold().cyan());
    for path in paths {
        let rel = match path.strip_prefix(base) {
            Ok(p) if !p.as_os_str().is_empty() => p,
            _ => continue,
        };
        let depth = rel.components().count();
        let indent = "  ".repeat(depth - 1);
        let name = rel.file_name().unwrap_or_default().to_string_lossy();
        
        let is_dotfile = name.starts_with('.');
        let has_extension = path.extension().is_some();
        let is_special_file = ["Dockerfile", "Makefile"].contains(&name.as_ref());

        if path.extension().is_none() {
            // Folder
            println!("{}📁 {}", indent, name.blue().bold());
        } else {
            // File
            let emoji = match path.extension().and_then(|e| e.to_str()) {
                Some("rs") => "🦀",
                Some("py") => "🐍",
                Some("js") | Some("ts") => "🧩",
                Some("toml") => "📝",
                Some("md") => "📘",
                Some("html") => "🌐",
                Some("css") => "🎨",
                _ => "📄",
            };
            println!("{}{} {}", indent, emoji, name.green());
        }
    }
}


fn create_path(path: &Path) -> Result<()> {
    if let Some(parent) = path.parent() {
        fs::create_dir_all(parent)
            .with_context(|| format!("Failed to create directory '{}'", parent.display()))?;
    }

    let file_like = path
        .file_name()
        .and_then(|n| n.to_str())
        .map(|n| n.contains('.'))  // for dot files
        .unwrap_or(false);

    if path.extension().is_some() || file_like {
        fs::File::create(path)
            .with_context(|| format!("Failed to create file '{}'", path.display()))?;
    } else {
        fs::create_dir_all(path)
            .with_context(|| format!("Failed to create directory '{}'", path.display()))?;
    }

    Ok(())
}

