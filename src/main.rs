use clap::Parser;
use ignore::WalkBuilder;
use std::fs;
use std::path::Path;
use std::process::exit;

#[derive(Parser, Debug)]
#[command(
    name = "markcat",
    version = "0.1.1",
    author = "Ted Pinkerton",
    about = "Converts a project directory to markdown format"
)]
struct Cli {
    #[arg(short = 'p', long, default_value = ".")]
    path: String,

    #[arg(short = 'i', long)]
    ignore_gitignore: bool,

    #[arg(short = 't', long)]
    trim: bool,

    #[arg(short = 'w', long)]
    whitelist: Option<String>,

    #[arg(short = 'b', long)]
    blacklist: Option<String>,
}

fn main() {
    let args = Cli::parse();

    if let Err(err) = process_directory(
        &args.path,
        args.ignore_gitignore,
        args.trim,
        args.whitelist.as_deref(),
        args.blacklist.as_deref(),
    ) {
        eprintln!("Error: {err}");
        exit(1);
    }
}

fn process_directory(
    dir: &str,
    ignore_gitignore: bool,
    trim: bool,
    whitelist: Option<&str>,
    blacklist: Option<&str>,
) -> Result<(), Box<dyn std::error::Error>> {
    let whitelist_exts: Option<Vec<&str>> = whitelist.map(|w| w.split(',').collect());
    let blacklist_exts: Option<Vec<&str>> = blacklist.map(|b| b.split(',').collect());

    let walker = if ignore_gitignore {
        WalkBuilder::new(dir).build()
    } else {
        WalkBuilder::new(dir).standard_filters(true).build()
    };

    for result in walker {
        let entry = result?;
        if entry.file_type().is_some_and(|ft| ft.is_file()) {
            let path = entry.path();

            if let Some(extension) = path.extension().and_then(|ext| ext.to_str()) {
                if let Some(whitelist) = &whitelist_exts {
                    if !whitelist.contains(&extension) {
                        continue;
                    }
                }
                if let Some(blacklist) = &blacklist_exts {
                    if blacklist.contains(&extension) {
                        continue;
                    }
                }
            }

            output_file_to_markdown(path, trim)?;
        }
    }

    Ok(())
}

fn output_file_to_markdown(path: &Path, trim: bool) -> Result<(), Box<dyn std::error::Error>> {
    let content = fs::read_to_string(path)?;
    let content_str = if trim {
        content.trim()
    } else {
        content.as_str()
    };
    let path_str = path.display().to_string();
    if let Some(extension) = path.extension().and_then(|ext| ext.to_str()) {
        println!("`{path_str}`\n```{extension}\n{content_str}\n```");
    } else {
        println!("`{path_str}`\n```\n{content_str}\n```");
    }
    Ok(())
}
