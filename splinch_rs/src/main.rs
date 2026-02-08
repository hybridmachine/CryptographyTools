use std::path::PathBuf;
use std::process;

use anyhow::{Result, bail};
use clap::Parser;
use splinch_rs::{combine_files, secure_delete, split_file, verify_files};

#[derive(Parser)]
#[command(
    name = "splinch",
    version,
    about = "Split a file into two XOR-complementary parts for secure transport, or combine them back"
)]
struct Cli {
    /// Path to the input file to split or a .xor1/.xor2 file to combine
    #[arg(short = 'i', long = "input")]
    input: PathBuf,

    /// Verify the split files against the original after splitting
    #[arg(short = 'v', long = "verify")]
    verify: bool,

    /// Combine two XOR files back into the original
    #[arg(short = 'c', long = "combine")]
    combine: bool,

    /// Securely delete the original file after splitting (overwrite with random data)
    #[arg(short = 's', long = "secure-delete")]
    secure_delete: bool,

    /// Number of overwrite passes for secure delete (default: 1)
    #[arg(short = 'p', long = "passes", default_value_t = 1)]
    passes: u32,
}

fn run_split(cli: &Cli) -> Result<()> {
    let metadata = std::fs::metadata(&cli.input);
    match &metadata {
        Ok(m) if !m.is_file() => bail!("{} is not a regular file", cli.input.display()),
        Err(e) => bail!("cannot access {}: {}", cli.input.display(), e),
        _ => {}
    }

    let file_size = metadata.unwrap().len();
    println!("Splitting {} ({} bytes)...", cli.input.display(), file_size);

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

    if cli.secure_delete {
        println!(
            "Securely deleting {} ({} pass(es))...",
            cli.input.display(),
            cli.passes
        );
        secure_delete(&cli.input, cli.passes)?;
        println!("Deleted.");
    }

    Ok(())
}

fn run_combine(cli: &Cli) -> Result<()> {
    if cli.verify {
        bail!("--verify cannot be used with --combine");
    }
    if cli.secure_delete {
        bail!("--secure-delete cannot be used with --combine");
    }
    if cli.passes != 1 {
        bail!("--passes cannot be used with --combine");
    }

    let metadata = std::fs::metadata(&cli.input);
    match &metadata {
        Ok(m) if !m.is_file() => bail!("{} is not a regular file", cli.input.display()),
        Err(e) => bail!("cannot access {}: {}", cli.input.display(), e),
        _ => {}
    }

    println!("Combining from {}...", cli.input.display());

    let output_path = combine_files(&cli.input)?;
    println!("Restored: {}", output_path.display());

    Ok(())
}

fn run() -> Result<()> {
    let cli = Cli::parse();

    if cli.combine {
        run_combine(&cli)
    } else {
        run_split(&cli)
    }
}

fn main() {
    if let Err(e) = run() {
        eprintln!("Error: {e:#}");
        process::exit(1);
    }
}
