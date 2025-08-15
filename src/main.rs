use clap::Parser;
use ignore::WalkBuilder;
use std::collections::HashSet;
use std::fs::{self, File};
use std::io::{self, BufWriter, Write};
use std::path::Path;
use std::process::exit;

#[derive(Parser, Debug)]
#[command(
    version,
    name = "markcat",
    author = "Ted Pinkerton",
    about = "Converts a project directory to markdown format"
)]
struct Cli {
    #[arg(
        value_name = "DIR",
        help = "Positional directory argument (alternative to -p/--path)",
        conflicts_with = "path"
    )]
    path_pos: Option<String>,

    #[arg(
        short = 'p',
        long,
        value_name = "DIR",
        help = "Directory to convert. Positional <DIR> is also supported",
        conflicts_with = "path_pos"
    )]
    path: Option<String>,

    #[arg(
        short = 'i',
        long,
        help = "Do not apply .gitignore or standard ignore filters"
    )]
    ignore_gitignore: bool,

    #[arg(
        short = 't',
        long,
        help = "Trim leading and trailing whitespace in file contents"
    )]
    trim: bool,

    #[arg(
        short = 'w',
        long,
        value_name = "ITEMS",
        help = "Comma-separated allow-list of extensions, exact filenames, and/or 'noext' (e.g., 'rs,md,LICENSE,noext')"
    )]
    whitelist: Option<String>,

    #[arg(
        short = 'b',
        long,
        value_name = "ITEMS",
        help = "Comma-separated deny-list of extensions, exact filenames, and/or 'noext'"
    )]
    blacklist: Option<String>,

    #[arg(
        short = 'o',
        long,
        value_name = "FILE",
        help = "Write output to FILE instead of stdout (creates or truncates)"
    )]
    output: Option<String>,
}

#[derive(Default)]
struct Filter {
    exts_lower: HashSet<String>,
    names: HashSet<String>,
    noext: bool,
}

fn main() {
    let args = Cli::parse();
    let Cli {
        path,
        path_pos,
        ignore_gitignore,
        trim,
        whitelist,
        blacklist,
        output,
    } = args;

    let dir = path.or(path_pos).unwrap_or_else(|| ".".into());

    let mut boxed_out: Box<dyn Write> = if let Some(p) = output {
        Box::new(BufWriter::new(
            File::create(p).expect("failed to create output file"),
        ))
    } else {
        Box::new(io::stdout())
    };

    if let Err(err) = process_directory(
        &dir,
        ignore_gitignore,
        trim,
        whitelist.as_deref(),
        blacklist.as_deref(),
        &mut boxed_out,
    ) {
        eprintln!("Error: {err}");
        exit(1);
    }
}

fn parse_filter(list: Option<&str>) -> Option<Filter> {
    let mut f = Filter::default();
    let s = list?;
    for token in s.split(',').map(|t| t.trim()).filter(|t| !t.is_empty()) {
        if token.eq_ignore_ascii_case("noext") {
            f.noext = true;
        } else if let Some(stripped) = token.strip_prefix('.') {
            f.exts_lower.insert(stripped.to_ascii_lowercase());
        } else {
            f.exts_lower.insert(token.to_ascii_lowercase());
            f.names.insert(token.to_string());
        }
    }
    Some(f)
}

fn matches_filter(path: &Path, f: &Filter) -> bool {
    let name = path.file_name().and_then(|n| n.to_str()).unwrap_or("");
    let ext_lower = path
        .extension()
        .and_then(|e| e.to_str())
        .map(|e| e.to_ascii_lowercase());

    let has_ext = ext_lower.is_some();
    let ext_ok = ext_lower.as_ref().is_some_and(|e| f.exts_lower.contains(e));
    let name_ok = f.names.contains(name);
    let noext_ok = f.noext && !has_ext;

    ext_ok || name_ok || noext_ok
}

fn process_directory<W: Write>(
    dir: &str,
    ignore_gitignore: bool,
    trim: bool,
    whitelist: Option<&str>,
    blacklist: Option<&str>,
    out: &mut W,
) -> Result<(), Box<dyn std::error::Error>> {
    let whitelist_f = parse_filter(whitelist);
    let blacklist_f = parse_filter(blacklist);

    let walker = if ignore_gitignore {
        WalkBuilder::new(dir).standard_filters(false).build()
    } else {
        WalkBuilder::new(dir).require_git(false).build()
    };

    for result in walker {
        let entry = result?;
        if entry.file_type().is_some_and(|ft| ft.is_file()) {
            let path = entry.path();

            if let Some(f) = &whitelist_f {
                if !matches_filter(path, f) {
                    continue;
                }
            }
            if let Some(f) = &blacklist_f {
                if matches_filter(path, f) {
                    continue;
                }
            }

            output_file_to_markdown(path, trim, out)?;
        }
    }

    Ok(())
}

fn output_file_to_markdown<W: Write>(
    path: &Path,
    trim: bool,
    out: &mut W,
) -> Result<(), Box<dyn std::error::Error>> {
    let content = fs::read_to_string(path)?;
    let content_str = if trim {
        content.trim()
    } else {
        content.as_str()
    };
    let path_str = path.display().to_string();

    if let Some(extension) = path.extension().and_then(|ext| ext.to_str()) {
        writeln!(out, "`{path_str}`")?;
        writeln!(out, "```{extension}\n{content_str}\n```")?;
    } else {
        writeln!(out, "`{path_str}`")?;
        writeln!(out, "```\n{content_str}\n```")?;
    }
    Ok(())
}
