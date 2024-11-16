use chrono::{DateTime, Datelike, Utc};
use clap::{Parser, ValueEnum};
use std::collections::HashMap;
use std::fs::{create_dir, File};
use std::path::Path;
use regex::Regex;
use walkdir::WalkDir;

#[derive(Debug, Parser)]
#[command(version, about)]
struct Seiri {
    target: Target,
    from: String,
    to: String,
    filter: Option<String>,
}

#[derive(Debug, Copy, Clone, ValueEnum)]
enum Target {
    All,
    File,
    Dir,
}

fn main() {
    let Seiri {
        target,
        from,
        to,
        filter
    } = Seiri::parse();
    let from = Path::new(&from);
    let to = Path::new(&to);
    match target {
        Target::All => unimplemented!("All is not implemented yet"),
        Target::File => classify_file(from, to, filter),
        Target::Dir => classify_directory(from),
    }
}

fn classify_file(from: &Path, to: &Path, filter: Option<String>) {
    let files = WalkDir::new(from)
        .into_iter()
        .filter_map(|e| e.ok())
        .filter(|e| e.file_type().is_file())
        .filter(|e| {
            if let Some(filter) = &filter {
                let regex = Regex::new(filter.as_str()).expect("Invalid regex");
                regex.is_match(&e.file_name().to_string_lossy())
            } else {
                true
            }
        })
        .flat_map(|e| {
            if let Ok(file) = File::open(e.path()) {
                let created = file
                    .metadata()
                    .and_then(|n| n.created().map(DateTime::<Utc>::from))
                    .map(|n| (n.year(), n.month()))
                    .ok();
                Some((created, e))
            } else {
                None
            }
        })
        .collect::<Vec<_>>();

    let mut files_map = HashMap::new();

    for (created, file) in files {
        if let Some(created) = created {
            files_map.entry(created).or_insert_with(Vec::new).push(file);
        }
    }

    for ((year, month), entries) in files_map {
        let directory_path = to.join(format!("{year}-{month:02}"));
        let _ = create_dir(&directory_path);

        for entry in entries {
            let file = entry.path();
            if let Some(file_name) = file.file_name() {
                let new_file_path = directory_path.join(file_name);
                let _ = std::fs::rename(file, new_file_path);
            }
        }
    }
}

fn classify_directory(root_path: &Path) {
    let directory_entries = WalkDir::new(root_path)
        .into_iter()
        .filter_map(|e| e.ok())
        .filter(|e| e.file_type().is_dir());
    for entry in directory_entries {
        println!("{}", entry.path().display());
    }
}
