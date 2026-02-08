use std::path::PathBuf;
use std::process;

use anyhow::{bail, Result};
use clap::Parser;
use splinch_rs::{split_file, verify_files};

#[derive(Parser)]
#[command(about = "Split a file into two XOR-complementary parts for secure transport")]
struct Cli {
    /// Path to the input file to split
    #[arg(short = 'i', long = "input")]
    input: PathBuf,

    /// Verify the split files against the original after splitting
    #[arg(short = 'v', long = "verify")]
    verify: bool,
}

fn run() -> Result<()> {
    let cli = Cli::parse();

    let metadata = std::fs::metadata(&cli.input);
    match &metadata {
        Ok(m) if !m.is_file() => bail!("{} is not a regular file", cli.input.display()),
        Err(e) => bail!("cannot access {}: {}", cli.input.display(), e),
        _ => {}
    }

    let file_size = metadata.unwrap().len();
    println!(
        "Splitting {} ({} bytes)...",
        cli.input.display(),
        file_size
    );

    let (xor1, xor2) = split_file(&cli.input)?;
    println!("Created: {}", xor1.display());
    println!("Created: {}", xor2.display());

    if cli.verify {
        print!("Verifying... ");
        let ok = verify_files(&cli.input, &xor1, &xor2)?;
        if ok {
            println!("OK");
        } else {
            println!("FAILED");
            process::exit(1);
        }
    }

    Ok(())
}

fn main() {
    if let Err(e) = run() {
        eprintln!("Error: {e:#}");
        process::exit(1);
    }
}
