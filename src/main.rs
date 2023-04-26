use std::path::PathBuf;

use clap::Parser;

mod dependencies;

/// A command-line tool for assigning minimal dependency versions.
#[derive(Parser)]
struct Args {
    /// Path to the crate root.
    root: PathBuf,
}

fn main() {
    let args = Args::parse();
}
