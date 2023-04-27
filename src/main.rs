use std::{
    fs,
    io::{self, Read, Seek, Write},
    path::{Path, PathBuf},
};

use clap::Parser;
use toml_edit::Document;

use crate::dependencies::{fetch_dependencies, DependencyType};

mod dependencies;

/// A command-line tool for assigning minimal dependency versions.
#[derive(Parser)]
struct Args {
    /// Path to the crate root.
    root: PathBuf,
}

const CARGO_TOML: &str = "Cargo.toml";
const CARGO_TOML_BACKUP: &str = "Cargo.toml.old";

fn backup(from: &Path) -> io::Result<()> {
    let mut to = from.to_owned();
    to.pop();
    to.push(CARGO_TOML_BACKUP);
    fs::copy(from, to).map(|_| ())
}

fn revert(to: &Path) -> io::Result<()> {
    let mut from = to.to_owned();
    from.pop();
    from.push("Cargo.toml.old");
    fs::copy(from, to).map(|_| ())
}

fn main() {
    let Args { root } = Args::parse();

    tracing::info!(path = %root.display(), "Starting dependency minimizing...");

    let mut root_toml_path = root;
    root_toml_path.push(CARGO_TOML);

    tracing::info!(toml = %root_toml_path.display(), "Creating a backup...");
    backup(&root_toml_path).expect("TODO");

    let mut root_toml = fs::OpenOptions::new()
        .read(true)
        .write(true)
        .open(&root_toml_path)
        .expect("TODO");
    let mut raw = String::new();
    root_toml.read_to_string(&mut raw).expect("TODO");

    let mut document = raw.parse().expect("TODO");
    let dependencies = fetch_dependencies(&mut document, DependencyType::Standard).expect("TODO");
    for mut dependency in dependencies {
        let version = dependency.version.get_mut();
        if version.major == 0 {
            version.patch = 0;
        } else {
            version.minor = 0;
            version.patch = 0;
        }
    }

    root_toml.set_len(0).expect("TODO");
    root_toml.rewind().expect("TODO");
    root_toml
        .write_all(document.to_string().as_bytes())
        .expect("TODO");
}
