use std::collections::BTreeSet;
use std::fs::{File, OpenOptions};
use std::io::{BufReader, BufWriter, Read, Seek, SeekFrom, Write};
use std::path::{Path, PathBuf};

use anyhow::{Context, Result, bail};
use rand::{RngCore, rng};

const CHUNK_SIZE: usize = 64 * 1024;
const VERIFY_FULL_THRESHOLD: u64 = 10 * 1024 * 1024;

/// XOR two equal-length byte slices into the output buffer.
pub fn xor_buffers(a: &[u8], b: &[u8], output: &mut [u8]) {
    assert_eq!(a.len(), b.len(), "input slices must be equal length");
    assert_eq!(a.len(), output.len(), "output must match input length");
    for i in 0..a.len() {
        output[i] = a[i] ^ b[i];
    }
}

/// Split a file into two XOR-complementary parts.
///
/// Given `input_path`, produces `<input_path>.xor1` and `<input_path>.xor2`.
/// Neither file alone reveals any information about the original.
/// XOR-ing the two output files together reconstructs the original.
pub fn split_file(input_path: &Path) -> Result<(PathBuf, PathBuf)> {
    let xor1_path = append_extension(input_path, "xor1");
    let xor2_path = append_extension(input_path, "xor2");

    let input_file = File::open(input_path)
        .with_context(|| format!("failed to open input file: {}", input_path.display()))?;
    let mut reader = BufReader::new(input_file);

    let xor1_file = File::create(&xor1_path)
        .with_context(|| format!("failed to create {}", xor1_path.display()))?;
    let mut writer1 = BufWriter::new(xor1_file);

    let xor2_file = File::create(&xor2_path)
        .with_context(|| format!("failed to create {}", xor2_path.display()))?;
    let mut writer2 = BufWriter::new(xor2_file);

    let mut input_buf = vec![0u8; CHUNK_SIZE];
    let mut rand_buf = vec![0u8; CHUNK_SIZE];
    let mut xor_buf = vec![0u8; CHUNK_SIZE];

    loop {
        let bytes_read = read_exact_or_eof(&mut reader, &mut input_buf)?;
        if bytes_read == 0 {
            break;
        }

        let input_chunk = &input_buf[..bytes_read];
        let rand_chunk = &mut rand_buf[..bytes_read];
        let xor_chunk = &mut xor_buf[..bytes_read];

        rng().fill_bytes(rand_chunk);
        xor_buffers(input_chunk, rand_chunk, xor_chunk);

        writer1
            .write_all(rand_chunk)
            .context("failed to write to xor1 file")?;
        writer2
            .write_all(xor_chunk)
            .context("failed to write to xor2 file")?;
    }

    writer1.flush().context("failed to flush xor1 file")?;
    writer2.flush().context("failed to flush xor2 file")?;

    Ok((xor1_path, xor2_path))
}

/// Verify that XOR-ing the two split files reproduces the original.
pub fn verify_files(original: &Path, xor1: &Path, xor2: &Path) -> Result<bool> {
    let file_size = std::fs::metadata(original)
        .with_context(|| format!("failed to read metadata for {}", original.display()))?
        .len();

    if file_size <= VERIFY_FULL_THRESHOLD {
        verify_full(original, xor1, xor2)
    } else {
        verify_sampled(original, xor1, xor2, file_size)
    }
}

fn verify_full(original: &Path, xor1: &Path, xor2: &Path) -> Result<bool> {
    let mut orig_reader = BufReader::new(
        File::open(original).with_context(|| format!("failed to open {}", original.display()))?,
    );
    let mut xor1_reader = BufReader::new(
        File::open(xor1).with_context(|| format!("failed to open {}", xor1.display()))?,
    );
    let mut xor2_reader = BufReader::new(
        File::open(xor2).with_context(|| format!("failed to open {}", xor2.display()))?,
    );

    let mut orig_buf = vec![0u8; CHUNK_SIZE];
    let mut xor1_buf = vec![0u8; CHUNK_SIZE];
    let mut xor2_buf = vec![0u8; CHUNK_SIZE];
    let mut recombined = vec![0u8; CHUNK_SIZE];

    loop {
        let orig_n = read_exact_or_eof(&mut orig_reader, &mut orig_buf)?;
        let xor1_n = read_exact_or_eof(&mut xor1_reader, &mut xor1_buf)?;
        let xor2_n = read_exact_or_eof(&mut xor2_reader, &mut xor2_buf)?;

        if orig_n != xor1_n || orig_n != xor2_n {
            return Ok(false);
        }
        if orig_n == 0 {
            break;
        }

        xor_buffers(
            &xor1_buf[..orig_n],
            &xor2_buf[..orig_n],
            &mut recombined[..orig_n],
        );

        if recombined[..orig_n] != orig_buf[..orig_n] {
            return Ok(false);
        }
    }

    Ok(true)
}

fn verify_sampled(original: &Path, xor1: &Path, xor2: &Path, file_size: u64) -> Result<bool> {
    let xor1_size = std::fs::metadata(xor1)
        .with_context(|| format!("failed to read metadata for {}", xor1.display()))?
        .len();
    let xor2_size = std::fs::metadata(xor2)
        .with_context(|| format!("failed to read metadata for {}", xor2.display()))?
        .len();

    if file_size != xor1_size || file_size != xor2_size {
        return Ok(false);
    }

    let chunk = CHUNK_SIZE as u64;
    let last_offset = if file_size >= chunk {
        file_size - chunk
    } else {
        0
    };

    let mut offsets = BTreeSet::new();
    offsets.insert(0u64);
    offsets.insert(last_offset);

    // Generate 8 random interior offsets
    let interior_range = if file_size > chunk {
        file_size - chunk
    } else {
        0
    };
    if interior_range > 0 {
        let mut r = rng();
        while offsets.len() < 10 {
            let offset = r.next_u64() % interior_range;
            offsets.insert(offset);
        }
    }

    let mut orig_file = BufReader::new(
        File::open(original).with_context(|| format!("failed to open {}", original.display()))?,
    );
    let mut xor1_file = BufReader::new(
        File::open(xor1).with_context(|| format!("failed to open {}", xor1.display()))?,
    );
    let mut xor2_file = BufReader::new(
        File::open(xor2).with_context(|| format!("failed to open {}", xor2.display()))?,
    );

    let mut orig_buf = vec![0u8; CHUNK_SIZE];
    let mut xor1_buf = vec![0u8; CHUNK_SIZE];
    let mut xor2_buf = vec![0u8; CHUNK_SIZE];
    let mut recombined = vec![0u8; CHUNK_SIZE];

    for &offset in &offsets {
        orig_file.seek(SeekFrom::Start(offset))?;
        xor1_file.seek(SeekFrom::Start(offset))?;
        xor2_file.seek(SeekFrom::Start(offset))?;

        let orig_n = read_exact_or_eof(&mut orig_file, &mut orig_buf)?;
        let xor1_n = read_exact_or_eof(&mut xor1_file, &mut xor1_buf)?;
        let xor2_n = read_exact_or_eof(&mut xor2_file, &mut xor2_buf)?;

        if orig_n != xor1_n || orig_n != xor2_n {
            return Ok(false);
        }
        if orig_n == 0 {
            continue;
        }

        xor_buffers(
            &xor1_buf[..orig_n],
            &xor2_buf[..orig_n],
            &mut recombined[..orig_n],
        );

        if recombined[..orig_n] != orig_buf[..orig_n] {
            return Ok(false);
        }
    }

    Ok(true)
}

fn read_exact_or_eof(reader: &mut impl Read, buf: &mut [u8]) -> Result<usize> {
    let mut total = 0;
    while total < buf.len() {
        match reader.read(&mut buf[total..])? {
            0 => break,
            n => total += n,
        }
    }
    Ok(total)
}

fn append_extension(path: &Path, ext: &str) -> PathBuf {
    let mut new_path = path.as_os_str().to_owned();
    new_path.push(".");
    new_path.push(ext);
    PathBuf::from(new_path)
}

/// Combine two XOR-complementary files back into the original.
///
/// Given either the `.xor1` or `.xor2` file, auto-discovers the partner
/// and XORs them together to reconstruct the original file.
/// Returns the path of the output file.
pub fn combine_files(input_path: &Path) -> Result<PathBuf> {
    let (xor1_path, xor2_path) = resolve_xor_pair(input_path)?;

    let xor1_size = std::fs::metadata(&xor1_path)
        .with_context(|| format!("failed to read metadata for {}", xor1_path.display()))?
        .len();
    let xor2_size = std::fs::metadata(&xor2_path)
        .with_context(|| format!("failed to read metadata for {}", xor2_path.display()))?
        .len();

    if xor1_size != xor2_size {
        bail!(
            "file sizes differ: {} is {} bytes, {} is {} bytes",
            xor1_path.display(),
            xor1_size,
            xor2_path.display(),
            xor2_size
        );
    }

    let base_path = strip_xor_extension(&xor1_path)?;
    let output_path = resolve_output_path(&base_path);

    let mut xor1_reader = BufReader::new(
        File::open(&xor1_path)
            .with_context(|| format!("failed to open {}", xor1_path.display()))?,
    );
    let mut xor2_reader = BufReader::new(
        File::open(&xor2_path)
            .with_context(|| format!("failed to open {}", xor2_path.display()))?,
    );

    let out_file = File::create(&output_path)
        .with_context(|| format!("failed to create {}", output_path.display()))?;
    let mut writer = BufWriter::new(out_file);

    let mut xor1_buf = vec![0u8; CHUNK_SIZE];
    let mut xor2_buf = vec![0u8; CHUNK_SIZE];
    let mut out_buf = vec![0u8; CHUNK_SIZE];

    loop {
        let n1 = read_exact_or_eof(&mut xor1_reader, &mut xor1_buf)?;
        let n2 = read_exact_or_eof(&mut xor2_reader, &mut xor2_buf)?;

        if n1 != n2 {
            bail!("unexpected read size mismatch during combine");
        }
        if n1 == 0 {
            break;
        }

        xor_buffers(&xor1_buf[..n1], &xor2_buf[..n1], &mut out_buf[..n1]);

        writer
            .write_all(&out_buf[..n1])
            .context("failed to write to output file")?;
    }

    writer.flush().context("failed to flush output file")?;

    Ok(output_path)
}

fn resolve_xor_pair(input_path: &Path) -> Result<(PathBuf, PathBuf)> {
    let ext = input_path
        .extension()
        .and_then(|e| e.to_str())
        .unwrap_or("");

    let (xor1_path, xor2_path) = match ext {
        "xor1" => {
            let xor2 = input_path.with_extension("xor2");
            (input_path.to_path_buf(), xor2)
        }
        "xor2" => {
            let xor1 = input_path.with_extension("xor1");
            (xor1, input_path.to_path_buf())
        }
        _ => bail!(
            "input file must have .xor1 or .xor2 extension, got: {}",
            input_path.display()
        ),
    };

    if !xor1_path.exists() {
        bail!("partner file not found: {}", xor1_path.display());
    }
    if !xor2_path.exists() {
        bail!("partner file not found: {}", xor2_path.display());
    }

    Ok((xor1_path, xor2_path))
}

fn strip_xor_extension(path: &Path) -> Result<PathBuf> {
    let ext = path.extension().and_then(|e| e.to_str()).unwrap_or("");

    match ext {
        "xor1" | "xor2" => Ok(path.with_extension("")),
        _ => bail!("expected .xor1 or .xor2 extension, got: {}", path.display()),
    }
}

fn resolve_output_path(base_path: &Path) -> PathBuf {
    if !base_path.exists() {
        return base_path.to_path_buf();
    }

    let stem = base_path.file_stem().and_then(|s| s.to_str()).unwrap_or("");
    let extension = base_path.extension().and_then(|e| e.to_str());
    let parent = base_path.parent().unwrap_or(Path::new(""));

    let mut n = 1u32;
    loop {
        let filename = match extension {
            Some(ext) => format!("{}.{}.{}", stem, n, ext),
            None => format!("{}.{}", stem, n),
        };
        let candidate = parent.join(filename);
        if !candidate.exists() {
            return candidate;
        }
        n += 1;
    }
}

/// Securely delete a file by overwriting it with random bytes, then removing it.
///
/// Each pass overwrites the entire file with cryptographically random data
/// and flushes to physical storage with `sync_all()`. After all passes,
/// the file is removed from the filesystem.
pub fn secure_delete(path: &Path, passes: u32) -> Result<()> {
    let file_size = std::fs::metadata(path)
        .with_context(|| format!("failed to read metadata for {}", path.display()))?
        .len();

    let mut file = OpenOptions::new()
        .write(true)
        .open(path)
        .with_context(|| format!("failed to open {} for writing", path.display()))?;

    let mut rand_buf = vec![0u8; CHUNK_SIZE];

    for pass in 1..=passes {
        file.seek(SeekFrom::Start(0))
            .with_context(|| format!("failed to seek in {} (pass {})", path.display(), pass))?;

        let mut remaining = file_size;
        while remaining > 0 {
            let to_write = remaining.min(CHUNK_SIZE as u64) as usize;
            rng().fill_bytes(&mut rand_buf[..to_write]);
            file.write_all(&rand_buf[..to_write]).with_context(|| {
                format!("failed to overwrite {} (pass {})", path.display(), pass)
            })?;
            remaining -= to_write as u64;
        }

        file.sync_all()
            .with_context(|| format!("failed to sync {} (pass {})", path.display(), pass))?;
    }

    drop(file);
    std::fs::remove_file(path).with_context(|| format!("failed to remove {}", path.display()))?;

    Ok(())
}
