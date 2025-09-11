use clap::Parser;
use ignore::overrides::OverrideBuilder;
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

    #[arg(
        short = 'g',
        long = "glob",
        value_name = "GLOB",
        value_delimiter = ',',
        help = "Gitignore-style glob. Repeat or comma-separate. Prefix with '!' to exclude (e.g., 'src/**,!target/', '!**/*.md')"
    )]
    globs: Vec<String>,

    #[arg(
        long = "glob-file",
        value_name = "PATH",
        help = "Read additional globs from PATH (one per line; '#' starts a comment)"
    )]
    glob_file: Option<String>,
}

#[derive(Default)]
struct Filter {
    exts_lower: HashSet<String>,
    names: HashSet<String>,
    noext: bool,
}

#[derive(Debug)]
struct RunConfig {
    dir: String,
    ignore_gitignore: bool,
    trim: bool,
    whitelist: Option<String>,
    blacklist: Option<String>,
    globs: Vec<String>,
    glob_file: Option<String>,
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
        globs,
        glob_file,
    } = args;

    let dir = path.or(path_pos).unwrap_or_else(|| ".".into());

    let mut boxed_out: Box<dyn Write> = if let Some(p) = output {
        Box::new(BufWriter::new(
            File::create(p).expect("failed to create output file"),
        ))
    } else {
        Box::new(io::stdout())
    };

    let cfg = RunConfig {
        dir,
        ignore_gitignore,
        trim,
        whitelist,
        blacklist,
        globs,
        glob_file,
    };

    if let Err(err) = process_directory(&cfg, &mut boxed_out) {
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

fn build_overrides(
    root: &str,
    globs: &[String],
    glob_file: Option<&str>,
) -> Result<ignore::overrides::Override, Box<dyn std::error::Error>> {
    let mut ob = OverrideBuilder::new(root);

    for g in globs {
        ob.add(g)?;
    }

    if let Some(path) = glob_file {
        let data = fs::read_to_string(path)?;
        for raw in data.lines() {
            let line = raw.trim();
            if line.is_empty() || line.starts_with('#') {
                continue;
            }
            ob.add(line)?;
        }
    }

    Ok(ob.build()?)
}

fn process_directory<W: Write>(
    cfg: &RunConfig,
    out: &mut W,
) -> Result<(), Box<dyn std::error::Error>> {
    let whitelist_f = parse_filter(cfg.whitelist.as_deref());
    let blacklist_f = parse_filter(cfg.blacklist.as_deref());

    let mut builder = WalkBuilder::new(&cfg.dir);
    if cfg.ignore_gitignore {
        builder.standard_filters(false);
    } else {
        builder.require_git(false);
    }

    if !cfg.globs.is_empty() || cfg.glob_file.is_some() {
        let overrides = build_overrides(&cfg.dir, &cfg.globs, cfg.glob_file.as_deref())?;
        builder.overrides(overrides);
    }

    let walker = builder.build();

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

            output_file_to_markdown(path, cfg.trim, out)?;
        }
    }

    Ok(())
}

fn output_file_to_markdown<W: Write>(
    path: &Path,
    trim: bool,
    out: &mut W,
) -> Result<(), Box<dyn std::error::Error>> {
    let content = fs::read_to_string(path)
        .map_err(|e| format!("Failed to read {}: {}", path.display(), e))?;
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
