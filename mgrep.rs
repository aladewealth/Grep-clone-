use std::env;
use std::fs;
use std::io::{self, BufRead, BufReader};
use std::path::{Path, PathBuf};
use std::process;

struct Config {
    pattern: String,
    paths: Vec<String>,
    ignore_case: bool,
    line_numbers: bool,
    recursive: bool,
    invert: bool,
    count_only: bool,
    files_with_matches: bool,
}

fn print_usage() {
    eprintln!(
        "Usage: mgrep [OPTIONS] PATTERN [FILE...]\n\n\
         Search for PATTERN in each FILE, or stdin if no FILE is given.\n\n\
         Options:\n\
         \x20 -i    Ignore case distinctions\n\
         \x20 -n    Print line numbers with output lines\n\
         \x20 -r    Recursively search directories\n\
         \x20 -v    Invert match (print non-matching lines)\n\
         \x20 -c    Print only a count of matching lines per file\n\
         \x20 -l    Print only names of files with a match\n\
         \x20 -h    Show this help message"
    );
}

fn parse_args(args: &[String]) -> Result<Config, String> {
    let mut ignore_case = false;
    let mut line_numbers = false;
    let mut recursive = false;
    let mut invert = false;
    let mut count_only = false;
    let mut files_with_matches = false;

    let mut positional: Vec<String> = Vec::new();
    let mut i = 0;

    while i < args.len() {
        let arg = &args[i];

        if arg == "-h" || arg == "--help" {
            print_usage();
            process::exit(0);
        } else if arg.starts_with('-') && arg.len() > 1 && !arg.starts_with("--") {
            // Support combined short flags like -in
            for c in arg.chars().skip(1) {
                match c {
                    'i' => ignore_case = true,
                    'n' => line_numbers = true,
                    'r' => recursive = true,
                    'v' => invert = true,
                    'c' => count_only = true,
                    'l' => files_with_matches = true,
                    other => return Err(format!("Unknown flag: -{}", other)),
                }
            }
        } else {
            positional.push(arg.clone());
        }
        i += 1;
    }

    if positional.is_empty() {
        return Err("No pattern given".to_string());
    }

    let pattern = positional.remove(0);
    let paths = positional;

    Ok(Config {
        pattern,
        paths,
        ignore_case,
        line_numbers,
        recursive,
        invert,
        count_only,
        files_with_matches,
    })
}

fn matches(line: &str, pattern: &str, ignore_case: bool) -> bool {
    if ignore_case {
        line.to_lowercase().contains(&pattern.to_lowercase())
    } else {
        line.contains(pattern)
    }
}

/// Search a single readable source (anything implementing BufRead).
/// Returns the number of matching lines, and prints output as it goes
/// (unless count_only is set, in which case the caller prints the total).
fn search_reader<R: BufRead>(
    reader: R,
    label: &str,
    config: &Config,
    show_label: bool,
) -> io::Result<usize> {
    let mut match_count = 0;

    for (idx, line_result) in reader.lines().enumerate() {
        let line = match line_result {
            Ok(l) => l,
            Err(_) => continue, // skip non-UTF8 or unreadable lines
        };

        let is_match = matches(&line, &config.pattern, config.ignore_case);
        let should_print = is_match != config.invert;

        if should_print {
            match_count += 1;

            if config.files_with_matches {
                // We only need to know a match happened; stop early.
                return Ok(match_count);
            }

            if !config.count_only {
                let mut prefix = String::new();
                if show_label {
                    prefix.push_str(label);
                    prefix.push(':');
                }
                if config.line_numbers {
                    prefix.push_str(&format!("{}:", idx + 1));
                }
                println!("{}{}", prefix, line);
            }
        }
    }

    Ok(match_count)
}

fn search_file(path: &Path, config: &Config, show_label: bool) -> io::Result<usize> {
    let file = fs::File::open(path)?;
    let reader = BufReader::new(file);
    let label = path.display().to_string();
    search_reader(reader, &label, config, show_label)
}

fn collect_files(path: &Path, recursive: bool, out: &mut Vec<PathBuf>) -> io::Result<()> {
    if path.is_dir() {
        if !recursive {
            eprintln!("mgrep: {}: Is a directory (use -r to search)", path.display());
            return Ok(());
        }
        for entry in fs::read_dir(path)? {
            let entry = entry?;
            collect_files(&entry.path(), recursive, out)?;
        }
    } else {
        out.push(path.to_path_buf());
    }
    Ok(())
}

fn run(config: Config) -> io::Result<i32> {
    let mut match_found = false;

    if config.paths.is_empty() {
        // No files given: read from stdin.
        let stdin = io::stdin();
        let reader = stdin.lock();
        let count = search_reader(reader, "(stdin)", &config, false)?;

        if config.count_only {
            println!("{}", count);
        }
        if count > 0 {
            match_found = true;
        }
        return Ok(if match_found { 0 } else { 1 });
    }

    // Gather the full list of files to search (expanding directories if -r).
    let mut files: Vec<PathBuf> = Vec::new();
    for p in &config.paths {
        let path = Path::new(p);
        if !path.exists() {
            eprintln!("mgrep: {}: No such file or directory", p);
            continue;
        }
        collect_files(path, config.recursive, &mut files)?;
    }

    let show_label = files.len() > 1 || config.recursive;

    for file in &files {
        match search_file(file, &config, show_label) {
            Ok(count) => {
                if count > 0 {
                    match_found = true;

                    if config.files_with_matches {
                        println!("{}", file.display());
                    } else if config.count_only {
                        if show_label {
                            println!("{}:{}", file.display(), count);
                        } else {
                            println!("{}", count);
                        }
                    }
                } else if config.count_only && !show_label {
                    println!("0");
                }
            }
            Err(e) => {
                eprintln!("mgrep: {}: {}", file.display(), e);
            }
        }
    }

    Ok(if match_found { 0 } else { 1 })
}

fn main() {
    let args: Vec<String> = env::args().skip(1).collect();

    let config = match parse_args(&args) {
        Ok(c) => c,
        Err(e) => {
            eprintln!("mgrep: {}", e);
            print_usage();
            process::exit(2);
        }
    };

    match run(config) {
        Ok(code) => process::exit(code),
        Err(e) => {
            eprintln!("mgrep: {}", e);
            process::exit(2);
        }
    }
}
