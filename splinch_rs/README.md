# splinch_rs

A CLI tool that splits a file into two XOR-complementary parts for secure transport.

Neither output file alone reveals any information about the original. XOR-ing the two files together reconstructs the original exactly (one-time pad splitting).

## Installation

### From .deb package (Ubuntu/Debian)

```bash
sudo dpkg -i splinch_0.2.0-1_amd64.deb
```

### From source

```bash
cargo build --release
cp target/release/splinch /usr/local/bin/splinch
```

## Usage

Split a file:

```bash
splinch -i secret.pdf
# Creates: secret.pdf.xor1, secret.pdf.xor2
```

Split and verify:

```bash
splinch -i secret.pdf -v
```

Split and securely delete the original (3 overwrite passes):

```bash
splinch -i secret.pdf -s -p 3
# Creates: secret.pdf.xor1, secret.pdf.xor2
# Original file is overwritten with random data and removed
```

Combine two parts back into the original:

```bash
splinch -i secret.pdf.xor1 -c
# Restores: secret.pdf
```

## Security

For secure transport, send the `.xor1` and `.xor2` files over **separate, independent channels**. Sending both over the same channel defeats the security guarantee.

Each output file is statistically indistinguishable from random data. The splitting uses a cryptographically secure random number generator.

### Secure delete caveats

The `-s` flag overwrites the original file with random data before removing it. This is effective on traditional filesystems (ext4/XFS) on HDDs. However, on **copy-on-write filesystems** (btrfs, ZFS) or **SSDs with wear leveling**, old data may persist in remapped blocks. Full-disk encryption is the recommended defense in those environments.

## License

MIT
